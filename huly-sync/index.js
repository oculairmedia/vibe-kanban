#!/usr/bin/env node

/**
 * Huly → Vibe Kanban Sync Service
 *
 * Syncs projects and issues from Huly to Vibe Kanban
 * Uses both Huly MCP and Vibe Kanban MCP servers
 */

import fetch from 'node-fetch';
import { execSync } from 'child_process';
import fs from 'fs';
import path from 'path';

// Configuration
const config = {
  huly: {
    mcpUrl: process.env.HULY_MCP_URL || 'http://192.168.50.90:3457/mcp',
  },
  vibeKanban: {
    mcpUrl: process.env.VIBE_MCP_URL || 'http://192.168.50.90:9717/mcp',
    apiUrl: process.env.VIBE_API_URL || 'http://192.168.50.90:3106/api',
  },
  sync: {
    interval: parseInt(process.env.SYNC_INTERVAL || '300000'), // 5 minutes default
    dryRun: process.env.DRY_RUN === 'true',
  },
  stacks: {
    baseDir: process.env.STACKS_DIR || '/opt/stacks',
  },
};

console.log('Huly → Vibe Kanban Sync Service');
console.log('Configuration:', {
  hulyMcp: config.huly.mcpUrl,
  vibeMcp: config.vibeKanban.mcpUrl,
  stacksDir: config.stacks.baseDir,
  syncInterval: `${config.sync.interval / 1000}s`,
  dryRun: config.sync.dryRun,
});

/**
 * Timeout wrapper for async operations
 */
async function withTimeout(promise, timeoutMs, operation) {
  return Promise.race([
    promise,
    new Promise((_, reject) =>
      setTimeout(() => reject(new Error(`Timeout after ${timeoutMs}ms: ${operation}`)), timeoutMs)
    )
  ]);
}

// Simple MCP client with session support
class MCPClient {
  constructor(url, name) {
    this.url = url;
    this.name = name;
    this.requestId = 1;
    this.sessionId = null;
  }

  async initialize() {
    console.log(`[${this.name}] Initializing MCP session...`);

    // Initialize session
    const initResult = await this.call('initialize', {
      protocolVersion: '2024-11-05',
      capabilities: {},
      clientInfo: {
        name: 'huly-vibe-sync',
        version: '1.0.0',
      },
    });

    console.log(`[${this.name}] ✓ Session initialized`);
    return initResult;
  }

  async call(method, params = {}) {
    const headers = {
      'Content-Type': 'application/json',
      'Accept': 'application/json, text/event-stream',
    };

    // Add session ID to headers if we have one (use lowercase for compatibility)
    if (this.sessionId) {
      headers['mcp-session-id'] = this.sessionId;
    }

    const response = await fetch(this.url, {
      method: 'POST',
      headers,
      body: JSON.stringify({
        jsonrpc: '2.0',
        id: this.requestId++,
        method,
        params,
      }),
    });

    if (!response.ok) {
      throw new Error(`HTTP ${response.status}: ${response.statusText}`);
    }

    // Check for session ID in response headers (try multiple header names)
    const newSessionId = response.headers.get('mcp-session-id') ||
                        response.headers.get('Mcp-Session-Id') ||
                        response.headers.get('X-Session-ID');
    if (newSessionId && !this.sessionId) {
      this.sessionId = newSessionId;
      console.log(`[${this.name}] Session ID: ${newSessionId}`);
    }

    // Check if response is SSE or JSON
    const contentType = response.headers.get('content-type');
    if (contentType && contentType.includes('text/event-stream')) {
      // Parse SSE response
      const text = await response.text();
      const lines = text.split('\n');
      let jsonData = null;

      for (const line of lines) {
        if (line.startsWith('data: ')) {
          const dataStr = line.substring(6);
          try {
            jsonData = JSON.parse(dataStr);
          } catch (e) {
            // Skip invalid JSON lines
          }
        }
      }

      if (!jsonData) {
        throw new Error('No valid JSON data in SSE response');
      }

      if (jsonData.error) {
        throw new Error(`MCP Error: ${jsonData.error.message}`);
      }

      return jsonData.result;
    } else {
      // Parse JSON response
      const data = await response.json();

      if (data.error) {
        throw new Error(`MCP Error: ${data.error.message}`);
      }

      return data.result;
    }
  }

  async callTool(name, args) {
    // Wrap MCP call with 60-second timeout
    const result = await withTimeout(
      this.call('tools/call', { name, arguments: args }),
      60000,
      `MCP ${this.name} callTool(${name})`
    );

    if (result && result.content && result.content[0]) {
      const content = result.content[0];
      if (content.type === 'text') {
        try {
          return JSON.parse(content.text);
        } catch (e) {
          return content.text;
        }
      }
    }

    return result;
  }
}

/**
 * Parse projects from Huly MCP text response
 */
function parseProjectsFromText(text) {
  const projects = [];
  const lines = text.split('\n');

  let currentProject = null;

  for (const line of lines) {
    const trimmed = line.trim();

    // Project header: 📁 Project Name (CODE)
    if (trimmed.startsWith('📁 ') && trimmed.includes('(') && trimmed.endsWith(')')) {
      if (currentProject) {
        projects.push(currentProject);
      }

      // Extract name and identifier
      const content = trimmed.substring(2); // Remove "📁 "
      const lastParen = content.lastIndexOf('(');
      const name = content.substring(0, lastParen).trim();
      const identifier = content.substring(lastParen + 1, content.length - 1).trim();

      currentProject = {
        name,
        identifier,
        description: '',
        issues: 0,
        status: 'active',
      };
    }
    // Description line
    else if (trimmed.startsWith('Description: ') && currentProject) {
      currentProject.description = trimmed.substring(13).trim();
    }
    // Issues count
    else if (trimmed.startsWith('Issues: ') && currentProject) {
      try {
        currentProject.issues = parseInt(trimmed.substring(8).split()[0], 10);
      } catch (e) {
        currentProject.issues = 0;
      }
    }
    // Status
    else if (trimmed.startsWith('Status: ') && currentProject) {
      currentProject.status = trimmed.substring(8).trim().toLowerCase();
    }
    // Filesystem path (special handling for our synced projects)
    else if (trimmed.startsWith('Filesystem: ') && currentProject) {
      if (!currentProject.description.includes('Filesystem:')) {
        currentProject.description += `\n\n---\n${trimmed}`;
      }
    }
    else if (trimmed.includes('Filesystem:') && !trimmed.startsWith('Description:') && currentProject) {
      // Sometimes filesystem path appears on its own line
      if (!currentProject.description.includes('Filesystem:')) {
        currentProject.description += `\n\n---\n${trimmed}`;
      }
    }
  }

  // Add the last project
  if (currentProject) {
    projects.push(currentProject);
  }

  return projects;
}

/**
 * Extract filesystem path from Huly project description
 */
function extractFilesystemPath(description) {
  if (!description) return null;

  const match = description.match(/Filesystem:\s*(.+?)$/m);
  return match ? match[1].trim() : null;
}

/**
 * Get git URL from local repository
 */
function getGitUrl(repoPath) {
  try {
    if (!fs.existsSync(path.join(repoPath, '.git'))) {
      return null;
    }

    const url = execSync('git remote get-url origin', {
      cwd: repoPath,
      encoding: 'utf8',
    }).trim();

    return url || null;
  } catch (error) {
    return null;
  }
}

/**
 * Fetch projects from Huly using MCP
 */
async function fetchHulyProjects(hulyClient) {
  console.log('\n[Huly] Fetching projects...');

  try {
    const result = await hulyClient.callTool('huly_query', {
      entity_type: 'project',
      mode: 'list',
    });

    // Huly MCP returns formatted text, not JSON
    const text = typeof result === 'string' ? result : result.toString();

    // Parse projects from text
    const projects = parseProjectsFromText(text);

    console.log(`[Huly] Found ${projects.length} projects`);

    // Debug: show first project structure
    if (projects.length > 0 && config.sync.dryRun) {
      console.log('[Huly] Sample project:', JSON.stringify(projects[0], null, 2));
    }

    return projects;
  } catch (error) {
    console.error('[Huly] Error fetching projects:', error.message);
    return [];
  }
}

/**
 * Extract full description from Huly issue detail response
 * The detail response has a ## Description section with full multi-line content
 * The description ends at specific top-level sections like "Recent Comments"
 */
function extractFullDescription(detailText) {
  const lines = detailText.split('\n');
  let inDescription = false;
  let description = [];

  // Top-level sections that mark the end of description
  const endSections = ['## Recent Comments', '## Sub-issues', '## Attachments'];

  for (const line of lines) {
    // Start capturing after ## Description header
    if (line.trim() === '## Description') {
      inDescription = true;
      continue;
    }

    // Stop at known end sections (not subsections within description)
    if (inDescription) {
      const trimmedLine = line.trim();
      if (endSections.some(section => trimmedLine === section)) {
        break;
      }
    }

    // Capture all description lines (including subsections like ## Summary, etc.)
    if (inDescription) {
      description.push(line);
    }
  }

  // Join and trim the description
  return description.join('\n').trim();
}

/**
 * Parse issues from Huly MCP text response
 */
function parseIssuesFromText(text, projectId) {
  const issues = [];
  const lines = text.split('\n');

  let currentIssue = null;

  for (const line of lines) {
    const trimmed = line.trim();

    // Issue header: 📋 **PROJ-123**: Issue Title
    if (trimmed.startsWith('📋 **') && trimmed.includes('**:')) {
      if (currentIssue) {
        issues.push(currentIssue);
      }

      // Extract identifier and title
      const parts = trimmed.split('**:', 1);
      const identifier = parts[0].substring(5).trim(); // Remove "📋 **"
      const title = trimmed.substring(trimmed.indexOf('**:') + 3).trim();

      currentIssue = {
        identifier,
        title,
        description: '',
        status: 'unknown',
        priority: 'medium',
        component: null,
        milestone: null,
      };
    }
    // Status line
    else if (trimmed.startsWith('Status: ') && currentIssue) {
      currentIssue.status = trimmed.substring(8).trim().toLowerCase();
    }
    // Priority line
    else if (trimmed.startsWith('Priority: ') && currentIssue) {
      currentIssue.priority = trimmed.substring(10).trim().toLowerCase();
    }
    // Description line
    else if (trimmed.startsWith('Description: ') && currentIssue) {
      currentIssue.description = trimmed.substring(13).trim();
    }
  }

  // Add the last issue
  if (currentIssue) {
    issues.push(currentIssue);
  }

  return issues;
}

/**
 * Fetch issues for a specific Huly project
 */
async function fetchHulyIssues(hulyClient, projectIdentifier) {
  console.log(`[Huly] Fetching issues for project ${projectIdentifier}...`);

  try {
    // First, get the list of issues (summary only)
    const listResult = await hulyClient.callTool('huly_query', {
      entity_type: 'issue',
      mode: 'list',
      project_identifier: projectIdentifier,
      options: {
        limit: 100,
      },
    });

    // Huly MCP returns formatted text, not JSON
    const text = typeof listResult === 'string' ? listResult : listResult.toString();

    // Parse issues from text to get identifiers
    const issues = parseIssuesFromText(text, projectIdentifier);

    console.log(`[Huly] Found ${issues.length} issues in ${projectIdentifier}`);

    // Fetch full details for each issue to get complete descriptions
    console.log(`[Huly] Fetching full details for ${issues.length} issues...`);
    const detailedIssues = [];

    for (const issue of issues) {
      try {
        const detailResult = await hulyClient.callTool('huly_query', {
          entity_type: 'issue',
          mode: 'get',
          issue_identifier: issue.identifier,
        });

        const detailText = typeof detailResult === 'string' ? detailResult : detailResult.toString();

        // Extract full description from the detailed response
        const fullDescription = extractFullDescription(detailText);

        detailedIssues.push({
          ...issue,
          description: fullDescription || issue.description,
        });

        console.log(`[Huly] ✓ Fetched details for ${issue.identifier}`);
      } catch (error) {
        console.error(`[Huly] ✗ Error fetching details for ${issue.identifier}:`, error.message);
        // Fallback to the summary description
        detailedIssues.push(issue);
      }
    }

    return detailedIssues;
  } catch (error) {
    console.error(`[Huly] Error fetching issues for ${projectIdentifier}:`, error.message);
    return [];
  }
}

/**
 * List existing Vibe Kanban projects (using HTTP API for reliability)
 */
async function listVibeProjects(vibeClient) {
  console.log('\n[Vibe] Listing existing projects...');

  try {
    const response = await fetch(`${config.vibeKanban.apiUrl}/projects`);

    if (!response.ok) {
      throw new Error(`HTTP ${response.status}: ${response.statusText}`);
    }

    const result = await response.json();

    if (!result.success) {
      throw new Error(result.message || 'Failed to list projects');
    }

    const projects = result.data || [];
    console.log(`[Vibe] Found ${projects.length} existing projects`);
    return projects;
  } catch (error) {
    console.error('[Vibe] Error listing projects:', error.message);
    return [];
  }
}

/**
 * Determine git repo path for Vibe Kanban project
 */
function determineGitRepoPath(hulyProject) {
  // Priority 1: Extract filesystem path from Huly description
  const filesystemPath = extractFilesystemPath(hulyProject.description);
  if (filesystemPath && fs.existsSync(filesystemPath)) {
    console.log(`[Vibe] Using filesystem path from Huly: ${filesystemPath}`);
    return filesystemPath;
  }

  // Priority 2: Use placeholder in /opt/stacks (mounted in Docker)
  const placeholder = `/opt/stacks/huly-sync-placeholders/${hulyProject.identifier}`;
  console.log(`[Vibe] Using placeholder path: ${placeholder}`);
  return placeholder;
}

/**
 * Create a project in Vibe Kanban via HTTP API
 */
async function createVibeProject(hulyProject) {
  if (config.sync.dryRun) {
    console.log(`[Vibe] [DRY RUN] Would create project: ${hulyProject.name}`);
    return null;
  }

  console.log(`[Vibe] Creating project: ${hulyProject.name}`);

  try {
    const gitRepoPath = determineGitRepoPath(hulyProject);

    const response = await fetch(`${config.vibeKanban.apiUrl}/projects`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        name: hulyProject.name,
        git_repo_path: gitRepoPath,
        use_existing_repo: fs.existsSync(gitRepoPath),
      }),
    });

    if (!response.ok) {
      const errorText = await response.text();
      throw new Error(`HTTP ${response.status}: ${errorText}`);
    }

    const result = await response.json();

    if (!result.success) {
      throw new Error(result.message || 'Project creation failed');
    }

    console.log(`[Vibe] ✓ Created project: ${hulyProject.name}`);
    return result.data;
  } catch (error) {
    console.error(`[Vibe] ✗ Error creating project ${hulyProject.name}:`, error.message);
    return null;
  }
}

/**
 * Map Huly issue status to Vibe Kanban task status
 */
function mapHulyStatusToVibe(hulyStatus) {
  if (!hulyStatus) return 'todo';

  const status = hulyStatus.toLowerCase();

  if (status.includes('backlog') || status.includes('todo')) return 'todo';
  if (status.includes('progress')) return 'inprogress';
  if (status.includes('review')) return 'inreview';
  if (status.includes('done') || status.includes('completed')) return 'done';
  if (status.includes('cancel')) return 'cancelled';

  return 'todo';
}

/**
 * Create a task in Vibe Kanban
 */
async function createVibeTask(vibeClient, vibeProjectId, hulyIssue) {
  if (config.sync.dryRun) {
    console.log(`[Vibe] [DRY RUN] Would create task: ${hulyIssue.title}`);
    return null;
  }

  console.log(`[Vibe] Creating task: ${hulyIssue.title}`);

  try {
    // Add Huly issue ID to description for tracking
    const description = hulyIssue.description
      ? `${hulyIssue.description}\n\n---\nHuly Issue: ${hulyIssue.identifier}`
      : `Synced from Huly: ${hulyIssue.identifier}`;

    const result = await vibeClient.callTool('create_task', {
      project_id: vibeProjectId,
      title: hulyIssue.title,
      description: description,
    });

    console.log(`[Vibe] ✓ Created task: ${hulyIssue.title}`);

    // Update status if not todo
    const vibeStatus = mapHulyStatusToVibe(hulyIssue.status);
    if (vibeStatus !== 'todo' && result.id) {
      await updateVibeTaskStatus(vibeClient, result.id, vibeStatus);
    }

    return result;
  } catch (error) {
    console.error(`[Vibe] ✗ Error creating task ${hulyIssue.title}:`, error.message);
    return null;
  }
}

/**
 * Update task status in Vibe Kanban
 */
async function updateVibeTaskStatus(vibeClient, taskId, status) {
  if (config.sync.dryRun) {
    console.log(`[Vibe] [DRY RUN] Would update task ${taskId} status to: ${status}`);
    return;
  }

  try {
    await vibeClient.callTool('update_task', {
      task_id: taskId,
      status: status,
    });

    console.log(`[Vibe] ✓ Updated task ${taskId} status to: ${status}`);
  } catch (error) {
    console.error(`[Vibe] Error updating task ${taskId} status:`, error.message);
  }
}

/**
 * Update Huly issue status
 */
async function updateHulyIssueStatus(hulyClient, issueIdentifier, status) {
  if (config.sync.dryRun) {
    console.log(`[Huly] [DRY RUN] Would update issue ${issueIdentifier} status to: ${status}`);
    return true;
  }

  try {
    await hulyClient.callTool('huly_issue_ops', {
      operation: 'update',
      issue_identifier: issueIdentifier,
      update: {
        field: 'status',
        value: status
      }
    });

    console.log(`[Huly] ✓ Updated issue ${issueIdentifier} status to: ${status}`);
    return true;
  } catch (error) {
    console.error(`[Huly] Error updating issue ${issueIdentifier} status:`, error.message);
    return false;
  }
}

/**
 * Extract Huly issue identifier from Vibe task description
 */
function extractHulyIdentifier(description) {
  if (!description) return null;

  const match = description.match(/Huly Issue: ([A-Z]+-\d+)/);
  return match ? match[1] : null;
}

/**
 * Map Vibe status to Huly status
 */
function mapVibeStatusToHuly(vibeStatus) {
  const statusMap = {
    'todo': 'Backlog',
    'inprogress': 'In Progress',
    'inreview': 'In Review',
    'done': 'Done',
    'cancelled': 'Cancelled'
  };

  return statusMap[vibeStatus] || 'Backlog';
}

/**
 * Sync task status changes from Vibe back to Huly (bidirectional)
 */
async function syncVibeTaskToHuly(hulyClient, vibeTask, hulyIssues) {
  // Extract Huly identifier from task description
  const hulyIdentifier = extractHulyIdentifier(vibeTask.description);

  if (!hulyIdentifier) {
    return; // Not synced from Huly, skip
  }

  // Find corresponding Huly issue
  const hulyIssue = hulyIssues.find(issue => issue.identifier === hulyIdentifier);

  if (!hulyIssue) {
    console.log(`[Skip] Huly issue ${hulyIdentifier} not found`);
    return;
  }

  // Map Vibe status to Huly status
  const vibeStatusMapped = mapVibeStatusToHuly(vibeTask.status);
  const hulyStatusNormalized = hulyIssue.status || 'Backlog';

  // Check if status needs updating
  if (vibeStatusMapped !== hulyStatusNormalized) {
    console.log(`[Bidirectional] Task "${vibeTask.title}" status changed: ${hulyStatusNormalized} → ${vibeStatusMapped}`);
    await updateHulyIssueStatus(hulyClient, hulyIdentifier, vibeStatusMapped);
  }
}

/**
 * Sync all projects and issues (bidirectional)
 */
async function syncHulyToVibe(hulyClient, vibeClient) {
  console.log('\n='.repeat(60));
  console.log(`Starting bidirectional sync at ${new Date().toISOString()}`);
  console.log('='.repeat(60));

  // Setup heartbeat logging
  const heartbeatInterval = setInterval(() => {
    console.log(`[HEARTBEAT] Sync still running... ${new Date().toISOString()}`);
  }, 30000); // Log every 30 seconds

  try {
    // Fetch Huly projects
    const hulyProjects = await fetchHulyProjects(hulyClient);
    if (hulyProjects.length === 0) {
      console.log('No Huly projects found. Skipping sync.');
      clearInterval(heartbeatInterval);
      return;
    }

    console.log(`[Huly] Found ${hulyProjects.length} projects\n`);

    // Get existing Vibe projects
    const vibeProjects = await listVibeProjects(vibeClient);
    console.log(`[Vibe] Found ${vibeProjects.length} existing projects\n`);
    // Use lowercase names for case-insensitive matching
    const vibeProjectsByName = new Map(vibeProjects.map(p => [p.name.toLowerCase(), p]));

    // Sync each Huly project
    let projectsProcessed = 0;
    for (const hulyProject of hulyProjects) {
      try {
        console.log(`\n--- Processing Huly project: ${hulyProject.name} ---`);

        // Check if project exists in Vibe (case-insensitive)
        let vibeProject = vibeProjectsByName.get(hulyProject.name.toLowerCase());

        if (!vibeProject) {
          // Try to create the project via HTTP API
          console.log(`[Vibe] Project not found, attempting to create: ${hulyProject.name}`);
          const createdProject = await createVibeProject(hulyProject);

          if (createdProject) {
            vibeProject = createdProject;
            // Add to map for subsequent iterations
            vibeProjectsByName.set(hulyProject.name.toLowerCase(), vibeProject);
          } else {
            console.log(`[Skip] Could not create project: ${hulyProject.name}`);
            continue;
          }
        } else {
          console.log(`[Vibe] ✓ Found existing project: ${hulyProject.name}`);
        }


        // Fetch issues from both systems
        const projectIdentifier = hulyProject.identifier || hulyProject.name;
        const hulyIssues = await fetchHulyIssues(hulyClient, projectIdentifier);
        const vibeTasks = await listVibeTasks(vibeProject.id);

        console.log(`\n[Sync] Huly: ${hulyIssues.length} issues, Vibe: ${vibeTasks.length} tasks`);

        // Phase 1: Sync Huly → Vibe (create missing tasks)
        console.log('[Phase 1] Syncing Huly → Vibe...');
        const vibeTasksByTitle = new Map(vibeTasks.map(t => [t.title.toLowerCase(), t]));

        for (const hulyIssue of hulyIssues) {
          const existingTask = vibeTasksByTitle.get(hulyIssue.title.toLowerCase());

          if (!existingTask) {
            await createVibeTask(vibeClient, vibeProject.id, hulyIssue);
          } else {
            // Task exists, check if description needs updating
            const hulyIdentifier = extractHulyIdentifier(existingTask.description);
            if (!hulyIdentifier || hulyIdentifier !== hulyIssue.identifier) {
              // Update description to include Huly identifier
              console.log(`[Vibe] Updating task "${existingTask.title}" with Huly identifier`);
              // We could add update logic here if needed
            }

            // Update status if it changed in Huly
            const vibeStatus = mapHulyStatusToVibe(hulyIssue.status);
            if (vibeStatus !== existingTask.status) {
              console.log(`[Vibe] Updating task "${existingTask.title}" status: ${existingTask.status} → ${vibeStatus}`);
              await updateVibeTaskStatus(vibeClient, existingTask.id, vibeStatus);
            }
          }

          // Small delay to avoid overwhelming the API
          await new Promise(resolve => setTimeout(resolve, 50));
        }

        // Phase 2: Sync Vibe → Huly (update statuses)
        console.log('[Phase 2] Syncing Vibe → Huly...');
        for (const vibeTask of vibeTasks) {
          await syncVibeTaskToHuly(hulyClient, vibeTask, hulyIssues);

          // Small delay (reduced from 100ms to 50ms for consistency)
          await new Promise(resolve => setTimeout(resolve, 50));
        }
      } catch (error) {
        console.error(`\n[ERROR] Failed to process project ${hulyProject.name}:`, error.message);
        console.log('[INFO] Continuing with next project...');
        // Continue with next project instead of crashing
      }
      projectsProcessed++;
    }

    console.log('\n' + '='.repeat(60));
    console.log(`Bidirectional sync completed at ${new Date().toISOString()}`);
    console.log(`Processed ${projectsProcessed}/${hulyProjects.length} projects`);
    console.log('='.repeat(60));
  } catch (error) {
    console.error('\n[ERROR] Sync failed:', error);
  } finally {
    clearInterval(heartbeatInterval);
  }
}

/**
 * List tasks for a Vibe project (using HTTP API)
 */
async function listVibeTasks(projectId) {
  try {
    const response = await fetch(`${config.vibeKanban.apiUrl}/tasks?project_id=${projectId}`);

    if (!response.ok) {
      throw new Error(`HTTP ${response.status}: ${response.statusText}`);
    }

    const result = await response.json();

    if (!result.success) {
      throw new Error(result.message || 'Failed to list tasks');
    }

    return result.data || [];
  } catch (error) {
    console.error(`[Vibe] Error listing tasks for project ${projectId}:`, error.message);
    return [];
  }
}

/**
 * Start the sync service
 */
async function main() {
  console.log('\nStarting sync service...\n');

  // Initialize MCP clients
  const hulyClient = new MCPClient(config.huly.mcpUrl, 'Huly');
  const vibeClient = new MCPClient(config.vibeKanban.mcpUrl, 'Vibe Kanban');

  console.log('[MCP] Connecting to servers:');
  console.log(`  - Huly: ${config.huly.mcpUrl}`);
  console.log(`  - Vibe: ${config.vibeKanban.mcpUrl}\n`);

  // Initialize MCP sessions
  try {
    await hulyClient.initialize();
    await vibeClient.initialize();
  } catch (error) {
    console.error('\n[ERROR] Failed to initialize MCP sessions:', error);
    process.exit(1);
  }

  console.log('\n[MCP] All sessions initialized successfully\n');

  // Wrapper function to run sync with timeout
  const runSyncWithTimeout = async () => {
    try {
      await withTimeout(
        syncHulyToVibe(hulyClient, vibeClient),
        900000, // 15-minute timeout for entire sync
        'Full sync cycle'
      );
    } catch (error) {
      console.error('\n[TIMEOUT] Sync exceeded 15-minute timeout:', error.message);
      console.log('[INFO] Will retry in next cycle...\n');
    }
  };

  // Run initial sync
  await runSyncWithTimeout();

  // Schedule periodic syncs
  if (config.sync.interval > 0) {
    console.log(`\nScheduling syncs every ${config.sync.interval / 1000} seconds...`);
    setInterval(runSyncWithTimeout, config.sync.interval);
  } else {
    console.log('\nOne-time sync completed. Exiting...');
    process.exit(0);
  }
}

// Run the service
main().catch(error => {
  console.error('Fatal error:', error);
  process.exit(1);
});
