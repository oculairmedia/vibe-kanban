# Vibe Kanban MCP Tools Reference

Complete API reference for all Model Context Protocol tools provided by Vibe Kanban.

## Table of Contents

- [Overview](#overview)
- [Task Server (vibe-kanban)](#task-server-vibe-kanban)
- [System Server (vibe-kanban-system)](#system-server-vibe-kanban-system)
- [Usage Examples](#usage-examples)
- [Error Handling](#error-handling)

---

## Overview

Vibe Kanban provides two MCP servers:

1. **Task Server** (`vibe-kanban`) - 11 tools for task and project management
2. **System Server** (`vibe-kanban-system`) - 8 tools for system configuration

### Connection Info

```json
{
  "task_server": {
    "name": "vibe-kanban",
    "url": "http://127.0.0.1:3456/mcp",
    "transport": "http",
    "version": "1.0.0"
  },
  "system_server": {
    "name": "vibe-kanban-system",
    "url": "http://127.0.0.1:3457/mcp",
    "transport": "http",
    "version": "1.0.0"
  }
}
```

---

## Task Server (vibe-kanban)

Server for managing projects, tasks, and execution attempts.

### create_task

Create a new task/ticket in a project.

**Parameters:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `project_id` | UUID | Yes | The ID of the project to create the task in |
| `title` | String | Yes | The title of the task |
| `description` | String | No | Optional description of the task |

**Response:**

```json
{
  "task_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

**Example:**

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "create_task",
    "arguments": {
      "project_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
      "title": "Implement user authentication",
      "description": "Add JWT-based authentication to the API"
    }
  }
}
```

---

### list_projects

List all available projects.

**Parameters:** None

**Response:**

```json
{
  "projects": [
    {
      "id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
      "name": "Vibe Kanban",
      "git_repo_path": "/path/to/repo",
      "setup_script": "npm install",
      "cleanup_script": null,
      "dev_script": "npm run dev",
      "created_at": "2024-10-01T12:00:00Z",
      "updated_at": "2024-10-28T14:30:00Z"
    }
  ],
  "count": 1
}
```

**Example:**

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "list_projects",
    "arguments": {}
  }
}
```

---

### list_tasks

List all tasks in a project with optional filtering and execution status.

**Parameters:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `project_id` | UUID | Yes | The ID of the project to list tasks from |
| `status` | String | No | Optional status filter: `'todo'`, `'inprogress'`, `'inreview'`, `'done'`, `'cancelled'` |
| `limit` | Integer | No | Maximum number of tasks to return (default: 50) |

**Response:**

```json
{
  "tasks": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "title": "Implement user authentication",
      "status": "inprogress",
      "created_at": "2024-10-27T10:00:00Z",
      "updated_at": "2024-10-28T14:00:00Z",
      "has_in_progress_attempt": true,
      "has_merged_attempt": false,
      "last_attempt_failed": false
    }
  ],
  "count": 1,
  "project_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
  "applied_filters": {
    "status": "inprogress",
    "limit": 50
  }
}
```

**Example:**

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "list_tasks",
    "arguments": {
      "project_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
      "status": "inprogress",
      "limit": 10
    }
  }
}
```

---

### get_task

Get detailed information about a specific task/ticket.

**Parameters:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `task_id` | UUID | Yes | The ID of the task to retrieve |

**Response:**

```json
{
  "task": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "title": "Implement user authentication",
    "description": "Add JWT-based authentication to the API",
    "status": "inprogress",
    "created_at": "2024-10-27T10:00:00Z",
    "updated_at": "2024-10-28T14:00:00Z",
    "has_in_progress_attempt": true,
    "has_merged_attempt": false,
    "last_attempt_failed": false
  }
}
```

**Example:**

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "get_task",
    "arguments": {
      "task_id": "550e8400-e29b-41d4-a716-446655440000"
    }
  }
}
```

---

### update_task

Update an existing task's title, description, or status.

**Parameters:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `task_id` | UUID | Yes | The ID of the task to update |
| `title` | String | No | New title for the task |
| `description` | String | No | New description for the task |
| `status` | String | No | New status: `'todo'`, `'inprogress'`, `'inreview'`, `'done'`, `'cancelled'` |

**Response:**

```json
{
  "task": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "title": "Implement user authentication (updated)",
    "description": "Add JWT-based authentication to the API",
    "status": "inreview",
    "created_at": "2024-10-27T10:00:00Z",
    "updated_at": "2024-10-28T15:00:00Z"
  }
}
```

**Example:**

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "update_task",
    "arguments": {
      "task_id": "550e8400-e29b-41d4-a716-446655440000",
      "status": "inreview"
    }
  }
}
```

---

### delete_task

Delete a task from a project.

**Parameters:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `task_id` | UUID | Yes | The ID of the task to delete |

**Response:**

```json
{
  "deleted_task_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

**Example:**

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "delete_task",
    "arguments": {
      "task_id": "550e8400-e29b-41d4-a716-446655440000"
    }
  }
}
```

---

### start_task_attempt

Start working on a task by creating and launching a new task attempt.

**Parameters:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `task_id` | UUID | Yes | The ID of the task to start |
| `executor` | String | Yes | The coding agent executor: `'CLAUDE_CODE'`, `'CODEX'`, `'GEMINI'`, `'CURSOR'`, `'OPENCODE'`, `'AMP'`, `'QWEN_CODE'`, `'COPILOT'` |
| `variant` | String | No | Optional executor variant, if needed |
| `base_branch` | String | Yes | The base branch to use for the attempt (e.g., `'main'`, `'develop'`) |

**Response:**

```json
{
  "task_id": "550e8400-e29b-41d4-a716-446655440000",
  "attempt_id": "660f9511-f3ac-52e5-b827-557766551111"
}
```

**Example:**

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "start_task_attempt",
    "arguments": {
      "task_id": "550e8400-e29b-41d4-a716-446655440000",
      "executor": "CLAUDE_CODE",
      "base_branch": "main"
    }
  }
}
```

**Notes:**
- Executor names are case-insensitive and hyphens are converted to underscores
- `'claude-code'` → `'CLAUDE_CODE'`
- Starting an attempt triggers the execution process automatically

---

### list_task_attempts

List all execution attempts for a specific task.

**Parameters:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `task_id` | UUID | Yes | The ID of the task to list attempts for |

**Response:**

```json
{
  "attempts": [
    {
      "id": "660f9511-f3ac-52e5-b827-557766551111",
      "task_id": "550e8400-e29b-41d4-a716-446655440000",
      "branch": "vk/task-550e8400",
      "target_branch": "main",
      "executor": "CLAUDE_CODE",
      "container_ref": "/path/to/worktree",
      "worktree_deleted": false,
      "setup_completed_at": "2024-10-28T10:05:00Z",
      "created_at": "2024-10-28T10:00:00Z",
      "updated_at": "2024-10-28T14:00:00Z"
    }
  ],
  "count": 1,
  "task_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

**Example:**

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "list_task_attempts",
    "arguments": {
      "task_id": "550e8400-e29b-41d4-a716-446655440000"
    }
  }
}
```

---

### get_task_attempt

Get detailed information about a specific task attempt.

**Parameters:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `attempt_id` | UUID | Yes | The ID of the attempt to retrieve |

**Response:**

```json
{
  "attempt": {
    "id": "660f9511-f3ac-52e5-b827-557766551111",
    "task_id": "550e8400-e29b-41d4-a716-446655440000",
    "branch": "vk/task-550e8400",
    "target_branch": "main",
    "executor": "CLAUDE_CODE",
    "container_ref": "/path/to/worktree",
    "worktree_deleted": false,
    "setup_completed_at": "2024-10-28T10:05:00Z",
    "created_at": "2024-10-28T10:00:00Z",
    "updated_at": "2024-10-28T14:00:00Z"
  }
}
```

**Example:**

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "get_task_attempt",
    "arguments": {
      "attempt_id": "660f9511-f3ac-52e5-b827-557766551111"
    }
  }
}
```

---

### create_followup_attempt

Create a follow-up attempt based on a previous attempt.

**Parameters:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `previous_attempt_id` | UUID | Yes | The ID of the previous attempt to base this followup on |
| `feedback` | String | No | Optional feedback or instructions for the followup attempt |
| `variant` | String | No | Optional executor variant to use |

**Response:**

```json
{
  "task_id": "550e8400-e29b-41d4-a716-446655440000",
  "attempt_id": "770fa622-g4bd-63f6-c938-668877662222",
  "based_on_attempt_id": "660f9511-f3ac-52e5-b827-557766551111"
}
```

**Example:**

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "create_followup_attempt",
    "arguments": {
      "previous_attempt_id": "660f9511-f3ac-52e5-b827-557766551111",
      "feedback": "Please add error handling to the authentication logic"
    }
  }
}
```

**Notes:**
- Follow-up attempts reuse the same executor and target branch as the previous attempt
- Useful for addressing review feedback or retrying after fixes

---

### merge_task_attempt

Merge a completed task attempt into its target branch.

**Parameters:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `attempt_id` | UUID | Yes | The ID of the task attempt to merge |

**Response:**

```json
{
  "success": true,
  "message": "Task attempt merged successfully",
  "task_id": "550e8400-e29b-41d4-a716-446655440000",
  "attempt_id": "660f9511-f3ac-52e5-b827-557766551111"
}
```

**Example:**

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "merge_task_attempt",
    "arguments": {
      "attempt_id": "660f9511-f3ac-52e5-b827-557766551111"
    }
  }
}
```

**Notes:**
- This performs a git merge operation
- The attempt must be complete with no conflicts
- Task status is automatically updated to "done"

---

## System Server (vibe-kanban-system)

Server for system configuration, discovery, and monitoring.

### get_system_info

Get system information including OS details and key directories.

**Parameters:** None

**Response:**

```json
{
  "system": {
    "os_type": "Linux",
    "os_version": "Ubuntu 22.04.3 LTS",
    "os_architecture": "x86_64",
    "bitness": "64",
    "home_directory": "/home/user",
    "current_directory": "/home/user/projects/vibe-kanban"
  }
}
```

**Example:**

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "get_system_info",
    "arguments": {}
  }
}
```

---

### get_config

Get the current Vibe Kanban configuration.

**Parameters:** None

**Response:**

```json
{
  "config": {
    "git_branch_prefix": "vk",
    "executor_profile": {
      "executor": "CLAUDE_CODE",
      "variant": null
    },
    "analytics_enabled": true,
    "telemetry_acknowledged": true,
    "editor": {
      "type": "vscode",
      "path": "/usr/bin/code"
    }
  }
}
```

**Example:**

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "get_config",
    "arguments": {}
  }
}
```

---

### update_config

Update Vibe Kanban configuration settings.

**Parameters:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `git_branch_prefix` | String | No | Git branch prefix for task branches |
| `executor_profile` | String | No | Default executor profile (currently disabled - use web UI) |
| `analytics_enabled` | Boolean | No | Enable analytics |
| `telemetry_enabled` | Boolean | No | Enable telemetry |
| `editor` | String | No | Preferred editor (not yet supported) |

**Response:**

```json
{
  "config": {
    "git_branch_prefix": "feature",
    "analytics_enabled": false
  },
  "message": "Configuration updated successfully"
}
```

**Example:**

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "update_config",
    "arguments": {
      "git_branch_prefix": "feature",
      "analytics_enabled": false
    }
  }
}
```

**Notes:**
- Only provided fields will be updated
- `executor_profile` update is temporarily disabled - use Vibe Kanban web UI instead

---

### list_mcp_servers

List configured MCP servers for a specific executor.

**Parameters:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `executor` | String | Yes | The executor to list MCP servers for (e.g., `'CLAUDE_CODE'`) |

**Status:** Temporarily disabled due to build dependencies

**Example:**

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "list_mcp_servers",
    "arguments": {
      "executor": "CLAUDE_CODE"
    }
  }
}
```

**Notes:**
- This functionality requires access to executor configs which have compilation issues
- Please access MCP config files directly from `~/.config/claude-code/` or use the web UI

---

### update_mcp_servers

Update MCP server configuration for a specific executor.

**Parameters:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `executor` | String | Yes | The executor to update MCP servers for |
| `servers` | Object | Yes | The MCP servers configuration as a JSON object |

**Response:**

```json
{
  "message": "MCP servers updated successfully",
  "servers_count": 3
}
```

**Example:**

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "update_mcp_servers",
    "arguments": {
      "executor": "CLAUDE_CODE",
      "servers": {
        "vibe-kanban": {
          "transport": "http",
          "url": "http://127.0.0.1:3456/mcp"
        }
      }
    }
  }
}
```

---

### list_executor_profiles

List all available executor profiles with their capabilities and availability status.

**Parameters:** None

**Response:**

```json
{
  "profiles": [
    {
      "id": {
        "executor": "CLAUDE_CODE",
        "variant": null
      },
      "name": "Claude Code",
      "description": "Anthropic's Claude 3.7 Sonnet coding agent",
      "available": true,
      "capabilities": ["coding", "debugging", "refactoring"]
    },
    {
      "id": {
        "executor": "GEMINI",
        "variant": "pro"
      },
      "name": "Gemini Pro",
      "description": "Google's Gemini Pro coding model",
      "available": true,
      "capabilities": ["coding", "analysis"]
    }
  ],
  "count": 2
}
```

**Example:**

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "list_executor_profiles",
    "arguments": {}
  }
}
```

**Notes:**
- Critical for discovering available coding agents
- Availability depends on API keys and configuration

---

### list_git_repos

List git repositories on the system.

**Parameters:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `path` | String | No | Optional path to search for git repositories |
| `timeout_ms` | Integer | No | Timeout in milliseconds (default: 5000) |
| `max_depth` | Integer | No | Maximum depth to search (default: 5) |

**Response:**

```json
{
  "repositories": [
    {
      "name": "vibe-kanban",
      "path": "/home/user/projects/vibe-kanban",
      "is_directory": true,
      "is_git_repo": true,
      "size": null,
      "modified": "2024-10-28T14:00:00Z"
    }
  ],
  "count": 1,
  "search_path": "/home/user/projects"
}
```

**Example:**

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "list_git_repos",
    "arguments": {
      "path": "/home/user/projects",
      "timeout_ms": 10000,
      "max_depth": 3
    }
  }
}
```

**Notes:**
- Searches common directories by default if no path specified
- Useful for discovering project repositories

---

### list_directory

List files and directories in a path.

**Parameters:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `path` | String | No | Path to list (defaults to home directory) |

**Response:**

```json
{
  "entries": [
    {
      "name": "projects",
      "path": "/home/user/projects",
      "is_directory": true,
      "is_git_repo": false,
      "size": null,
      "modified": "2024-10-28T12:00:00Z"
    },
    {
      "name": "README.md",
      "path": "/home/user/README.md",
      "is_directory": false,
      "is_git_repo": false,
      "size": 1024,
      "modified": "2024-10-28T10:00:00Z"
    }
  ],
  "current_path": "/home/user",
  "count": 2
}
```

**Example:**

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "list_directory",
    "arguments": {
      "path": "/home/user/projects"
    }
  }
}
```

---

### health_check

Check if Vibe Kanban is healthy and get version information.

**Parameters:** None

**Response:**

```json
{
  "status": "healthy",
  "version": "1.0.0",
  "uptime_seconds": 3600
}
```

**Example:**

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "health_check",
    "arguments": {}
  }
}
```

---

## Usage Examples

### Complete Task Lifecycle

```bash
# 1. List projects
{"method": "tools/call", "params": {"name": "list_projects"}}

# 2. Create a task
{"method": "tools/call", "params": {
  "name": "create_task",
  "arguments": {
    "project_id": "a1b2c3d4-...",
    "title": "Add authentication",
    "description": "Implement JWT auth"
  }
}}

# 3. Start execution
{"method": "tools/call", "params": {
  "name": "start_task_attempt",
  "arguments": {
    "task_id": "550e8400-...",
    "executor": "CLAUDE_CODE",
    "base_branch": "main"
  }
}}

# 4. Monitor attempts
{"method": "tools/call", "params": {
  "name": "list_task_attempts",
  "arguments": {"task_id": "550e8400-..."}
}}

# 5. Merge completed work
{"method": "tools/call", "params": {
  "name": "merge_task_attempt",
  "arguments": {"attempt_id": "660f9511-..."}
}}
```

### Discovering Executors

```bash
# 1. List available executors
{"method": "tools/call", "params": {"name": "list_executor_profiles"}}

# Response shows available coding agents:
{
  "profiles": [
    {"id": {"executor": "CLAUDE_CODE"}, "available": true},
    {"id": {"executor": "GEMINI", "variant": "pro"}, "available": true}
  ]
}

# 2. Use executor in task attempt
{"method": "tools/call", "params": {
  "name": "start_task_attempt",
  "arguments": {
    "task_id": "...",
    "executor": "GEMINI",
    "variant": "pro",
    "base_branch": "main"
  }
}}
```

### Follow-up Workflow

```bash
# 1. Get initial attempt details
{"method": "tools/call", "params": {
  "name": "get_task_attempt",
  "arguments": {"attempt_id": "660f9511-..."}
}}

# 2. Create follow-up with feedback
{"method": "tools/call", "params": {
  "name": "create_followup_attempt",
  "arguments": {
    "previous_attempt_id": "660f9511-...",
    "feedback": "Add unit tests for the new authentication endpoints"
  }
}}

# 3. Monitor new attempt
{"method": "tools/call", "params": {
  "name": "list_task_attempts",
  "arguments": {"task_id": "550e8400-..."}
}}
```

---

## Error Handling

### Error Response Format

All errors follow the JSON-RPC 2.0 error format:

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": -32000,
    "message": "Invalid request: Task ID is required"
  }
}
```

### Common Error Codes

| Code | Type | Description |
|------|------|-------------|
| `-32600` | Invalid Request | The JSON sent is not a valid Request object |
| `-32601` | Method not found | The method does not exist / is not available |
| `-32602` | Invalid params | Invalid method parameter(s) |
| `-32603` | Internal error | Internal JSON-RPC error |
| `-32000` | Server error | Generic server error |

### Common Error Scenarios

#### Invalid Executor

```json
{
  "error": {
    "code": -32602,
    "message": "Invalid request: Unknown executor 'INVALID'. Valid executors are: CLAUDE_CODE, AMP, GEMINI, CODEX, OPENCODE, CURSOR, QWEN_CODE, COPILOT"
  }
}
```

#### Invalid Status

```json
{
  "error": {
    "code": -32602,
    "message": "Invalid request: Invalid status 'invalid-status'. Valid values: 'todo', 'in-progress', 'in-review', 'done', 'cancelled'"
  }
}
```

#### API Connection Error

```json
{
  "error": {
    "code": -32000,
    "message": "Internal error: Failed to connect to VK API: connection refused"
  }
}
```

#### Missing Required Field

```json
{
  "error": {
    "code": -32602,
    "message": "Invalid request: Base branch must not be empty"
  }
}
```

### Best Practices

1. **Always check for errors** before processing responses
2. **Parse error messages** for actionable information (valid values, requirements)
3. **Retry on connection errors** with exponential backoff
4. **Validate inputs client-side** to reduce round-trips
5. **Log errors** with full context for debugging

---

## Version History

- **1.0.0** (2024-10-28) - Initial TurboMCP implementation
  - Task Server: 11 tools
  - System Server: 8 tools
  - Migration from rmcp complete

---

**Last Updated**: 2025-10-28
**API Version**: 1.0.0
