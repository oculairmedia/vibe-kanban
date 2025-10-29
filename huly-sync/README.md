# Huly → Vibe Kanban Sync Service

Syncs issues/tasks from Huly projects to Vibe Kanban projects.

## Features

- ✅ Fetches projects from Huly via MCP
- ✅ Fetches issues from Huly projects
- ✅ Syncs tasks to matching Vibe Kanban projects
- ✅ Maps Huly issue statuses to Vibe task statuses
- ✅ Tracks Huly issue IDs in Vibe task descriptions
- ✅ Configurable sync interval
- ✅ Dry-run mode for testing

## Limitations

**Vibe Kanban projects cannot be created programmatically via MCP.**

The Vibe Kanban MCP server only provides task management tools:
- `list_projects`
- `list_tasks`
- `create_task`
- `update_task`
- `delete_task`
- `get_task`
- `start_task_attempt`

There is no `create_project` tool. Projects must be created manually through the Vibe Kanban web UI or REST API.

## How It Works

1. **Fetch Huly Projects**: Queries Huly MCP for all projects
2. **Parse Text Responses**: Huly MCP returns formatted text, which is parsed to extract project/issue data
3. **Match Projects**: Finds Vibe Kanban projects with matching names
4. **Sync Tasks**: For each matched project:
   - Fetches issues from Huly
   - Creates corresponding tasks in Vibe Kanban
   - Maps status (todo, inprogress, inreview, done, cancelled)
   - Adds Huly issue ID to task description for tracking

## Configuration

Environment variables:

```bash
# MCP Server URLs
HULY_MCP_URL=http://192.168.50.90:3457/mcp      # Huly MCP server
VIBE_MCP_URL=http://192.168.50.90:9717/mcp      # Vibe Kanban MCP server

# Sync Settings
SYNC_INTERVAL=300000    # Sync every 5 minutes (in milliseconds)
                        # Set to 0 for one-time sync
DRY_RUN=false           # Set to true to test without making changes

# Paths
STACKS_DIR=/opt/stacks  # Base directory for project repositories
```

## Usage

### One-Time Sync (Dry Run)
```bash
DRY_RUN=true SYNC_INTERVAL=0 node index.js
```

### One-Time Sync (Real)
```bash
SYNC_INTERVAL=0 node index.js
```

### Continuous Sync (Every 5 Minutes)
```bash
SYNC_INTERVAL=300000 node index.js
```

### Run as Systemd Service

The service is installed at `/etc/systemd/system/huly-vibe-sync.service`

```bash
# Start the service
sudo systemctl start huly-vibe-sync

# Check status
sudo systemctl status huly-vibe-sync

# View logs
sudo journalctl -u huly-vibe-sync -f

# Stop the service
sudo systemctl stop huly-vibe-sync

# Restart after config changes
sudo systemctl restart huly-vibe-sync

# Disable auto-start on boot
sudo systemctl disable huly-vibe-sync
```

The service is already enabled to start on boot.

## Status Mapping

Huly statuses are mapped to Vibe Kanban statuses:

| Huly Status | Vibe Status |
|-------------|-------------|
| Backlog, Todo | `todo` |
| In Progress | `inprogress` |
| In Review | `inreview` |
| Done, Completed | `done` |
| Cancelled | `cancelled` |
| (other) | `todo` |

## File Structure

```
huly-sync/
├── index.js          # Main sync service
├── sync-projects.js  # One-time script to sync /opt/stacks repos to Huly
├── package.json      # Dependencies
└── README.md         # This file
```

## Prerequisites

- Node.js 18+
- Access to Huly MCP server (port 3457)
- Access to Vibe Kanban MCP server (port 9717)
- Matching project names in both Huly and Vibe Kanban

## Setup Steps

1. **Create Huly Projects**: Run `sync-projects.js` to create Huly projects for /opt/stacks repos
2. **Create Vibe Projects**: Manually create matching projects in Vibe Kanban web UI
3. **Run Sync**: Start the sync service to sync tasks

## Example Workflow

```bash
# Step 1: Sync /opt/stacks repos to Huly
node sync-projects.js

# Step 2: Create matching projects in Vibe Kanban web UI
#   (Navigate to http://192.168.50.90:3105 and create projects)

# Step 3: Test sync in dry-run mode
DRY_RUN=true SYNC_INTERVAL=0 node index.js

# Step 4: Run actual sync
SYNC_INTERVAL=0 node index.js

# Step 5: Deploy as service
sudo systemctl start huly-vibe-sync
```

## Troubleshooting

### "Project not found in Vibe Kanban"
Projects must exist in both systems with matching names. Create the project in Vibe Kanban web UI first.

### "MCP Error: tool not found"
The Vibe Kanban MCP server doesn't have a `create_project` tool. This is expected.

### "Session expired" errors
MCP sessions can timeout. The service will automatically reconnect on the next sync interval.

### No tasks syncing
- Check that project names match exactly (case-sensitive)
- Verify issues exist in the Huly project
- Check sync logs for errors

## Future Enhancements

- [ ] Add bidirectional sync (Vibe → Huly)
- [ ] Support for components and milestones
- [ ] Incremental sync (only sync changes since last run)
- [ ] Project creation via Vibe Kanban REST API
- [ ] Webhook-based real-time sync
