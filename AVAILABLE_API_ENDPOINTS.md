# Vibe Kanban API Endpoints - Current Availability

**Generated:** 2025-10-29  
**Backend Version:** NPM Package (running on port 3105)  
**Purpose:** Document which API endpoints are available vs. require custom deployment

---

## Status Legend

- ✅ **AVAILABLE** - Endpoint exists in current NPM package, MCP tool can use it
- ⚠️ **PARTIALLY AVAILABLE** - Endpoint exists but may have limitations
- ❌ **NOT AVAILABLE** - Endpoint implemented in codebase but not in NPM package
- 🔄 **PENDING** - Waiting for next NPM package release

---

## Projects API

**Base Path:** `/api/projects`

### Available Endpoints

| Method | Path | Status | Description |
|--------|------|--------|-------------|
| GET | `/api/projects` | ✅ | List all projects |
| POST | `/api/projects` | ✅ | Create a new project |
| GET | `/api/projects/{id}` | ✅ | Get project details |
| PUT | `/api/projects/{id}` | ✅ | Update project |
| DELETE | `/api/projects/{id}` | ✅ | Delete project |
| GET | `/api/projects/{id}/branches` | ✅ | List git branches in project |
| GET | `/api/projects/{id}/search` | ✅ | Search files in project |
| POST | `/api/projects/{id}/open-editor` | ✅ | Open project in editor |

**MCP Tools Using This:**
- `list_projects` - ✅ Working

---

## Tasks API

**Base Path:** `/api/tasks`

### Available Endpoints

| Method | Path | Status | Description |
|--------|------|--------|-------------|
| GET | `/api/tasks` | ✅ | List tasks (requires `project_id` query param) |
| POST | `/api/tasks` | ✅ | Create a new task |
| GET | `/api/tasks/{id}` | ✅ | Get task details |
| PUT | `/api/tasks/{id}` | ✅ | Update task |
| DELETE | `/api/tasks/{id}` | ✅ | Delete task |
| GET | `/api/tasks/stream/ws` | ✅ | WebSocket stream of tasks |
| POST | `/api/tasks/create-and-start` | ✅ | Create task and start attempt in one call |

**MCP Tools Using This:**
- `list_tasks` - ✅ Working
- `create_task` - ✅ Working
- `get_task` - ✅ Working
- `update_task` - ✅ Working
- `delete_task` - ✅ Working

---

## Task Attempts API

**Base Path:** `/api/task-attempts`

### Available Endpoints

| Method | Path | Status | Description |
|--------|------|--------|-------------|
| GET | `/api/task-attempts` | ✅ | List task attempts (requires `task_id` query param) |
| POST | `/api/task-attempts` | ✅ | Create a new task attempt |
| GET | `/api/task-attempts/{id}` | ✅ | Get task attempt details |
| GET | `/api/task-attempts/{id}/details` | ✅ | Get detailed attempt info with processes |
| POST | `/api/task-attempts/followup` | ✅ | Create follow-up attempt from previous |
| POST | `/api/task-attempts/{id}/follow-up` | ✅ | Add follow-up message to attempt |
| POST | `/api/task-attempts/{id}/merge` | ✅ | Merge attempt into target branch |
| POST | `/api/task-attempts/{id}/stop` | ✅ | Stop running execution |
| POST | `/api/task-attempts/{id}/start-dev-server` | ✅ | Start dev server for attempt |
| GET | `/api/task-attempts/{id}/branch-status` | ✅ | Get git branch status |
| GET | `/api/task-attempts/{id}/commits` | ✅ | Get all commits with metadata |
| GET | `/api/task-attempts/{id}/commit-info` | ✅ | Get commit information |
| GET | `/api/task-attempts/{id}/commit-compare` | ✅ | Compare commits |
| GET | `/api/task-attempts/{id}/diff/ws` | ✅ | WebSocket stream of git diff |
| POST | `/api/task-attempts/{id}/replace-process` | ✅ | Replace execution process |
| POST | `/api/task-attempts/{id}/open-editor` | ✅ | Open attempt in editor |
| POST | `/api/task-attempts/{id}/delete-file` | ✅ | Delete file from attempt |
| POST | `/api/task-attempts/{id}/change-target-branch` | ✅ | Change target branch |
| POST | `/api/task-attempts/{id}/conflicts/abort` | ✅ | Abort merge conflicts |
| POST | `/api/task-attempts/{id}/pr/attach` | ✅ | Attach existing PR to attempt |
| GET | `/api/task-attempts/{id}/children` | ✅ | Get child task relationships |

### Draft Management

| Method | Path | Status | Description |
|--------|------|--------|-------------|
| GET | `/api/task-attempts/{id}/draft` | ✅ | Get draft message |
| PUT | `/api/task-attempts/{id}/draft` | ✅ | Save draft message |
| DELETE | `/api/task-attempts/{id}/draft` | ✅ | Delete draft message |
| POST | `/api/task-attempts/{id}/draft/queue` | ✅ | Set draft queue |

### 🔄 Pending Deployment (Not Yet Available in NPM)

| Method | Path | Status | Description | Commit |
|--------|------|--------|-------------|--------|
| GET | `/api/task-attempts/{id}/artifacts` | ❌ | Get execution artifacts | 246958b |
| POST | `/api/task-attempts/{id}/pr` | ❌ | Create GitHub PR | fd98a7d |
| POST | `/api/task-attempts/{id}/push` | ❌ | Push branch to remote | 4958cac |
| POST | `/api/task-attempts/{id}/rebase` | ❌ | Rebase onto target branch | 10b13fe |

**MCP Tools Using This:**
- `list_task_attempts` - ✅ Working
- `get_task_attempt` - ✅ Working
- `create_followup_attempt` - ✅ Working
- `merge_task_attempt` - ✅ Working
- `start_dev_server` - ✅ Working
- `get_branch_status` - ✅ Working (VIBEK-34) ⭐ NEW
- `get_attempt_commits` - ✅ Working (VIBEK-28) ⭐ NEW
- `compare_commit_to_head` - ✅ Working (VIBEK-29) ⭐ NEW
- `abort_conflicts` - ✅ Working (VIBEK-27) ⭐ NEW
- `get_attempt_artifacts` - ❌ DISABLED (awaiting deployment)
- `create_github_pr` - ❌ DISABLED (awaiting deployment)
- `push_attempt_branch` - ❌ DISABLED (awaiting deployment)
- `rebase_task_attempt` - ❌ DISABLED (awaiting deployment)

---

## Execution Processes API

**Base Path:** `/api/execution-processes`

### Available Endpoints

| Method | Path | Status | Description |
|--------|------|--------|-------------|
| GET | `/api/execution-processes` | ✅ | List execution processes (requires `task_attempt_id` query param) |
| GET | `/api/execution-processes/{id}` | ✅ | Get process details |
| POST | `/api/execution-processes/{id}/stop` | ✅ | Stop running process |
| GET | `/api/execution-processes/{id}/logs` | ✅ | Get raw logs |
| GET | `/api/execution-processes/{id}/logs/normalized` | ✅ | Get normalized logs |
| GET | `/api/execution-processes/{id}/raw-logs/ws` | ✅ | WebSocket stream of raw logs |
| GET | `/api/execution-processes/{id}/normalized-logs/ws` | ✅ | WebSocket stream of normalized logs |
| GET | `/api/execution-processes/stream/ws` | ✅ | WebSocket stream of all processes |

**MCP Tools Using This:**
- `list_execution_processes` - ✅ Working
- `get_execution_process` - ✅ Working
- `stop_execution_process` - ✅ Working
- `get_process_raw_logs` - ✅ Working

---

## Configuration API

**Base Path:** `/api/config`

### Available Endpoints

| Method | Path | Status | Description |
|--------|------|--------|-------------|
| GET | `/api/config` | ✅ | Get current configuration |
| PUT | `/api/config` | ✅ | Update configuration |

---

## Authentication API

**Base Path:** `/api/auth`

### Available Endpoints

| Method | Path | Status | Description |
|--------|------|--------|-------------|
| GET | `/api/auth/github/url` | ✅ | Get GitHub OAuth URL |
| GET | `/api/auth/github/callback` | ✅ | GitHub OAuth callback |
| POST | `/api/auth/github/disconnect` | ✅ | Disconnect GitHub |
| GET | `/api/auth/github/status` | ✅ | Check GitHub connection status |

---

## Tags API

**Base Path:** `/api/tags`

### Available Endpoints

| Method | Path | Status | Description |
|--------|------|--------|-------------|
| GET | `/api/tags` | ✅ | List all tags |
| POST | `/api/tags` | ✅ | Create a tag |
| PUT | `/api/tags/{id}` | ✅ | Update tag |
| DELETE | `/api/tags/{id}` | ✅ | Delete tag |

---

## Images API

**Base Path:** `/api/images`

### Available Endpoints

| Method | Path | Status | Description |
|--------|------|--------|-------------|
| POST | `/api/images/upload` | ✅ | Upload image |
| GET | `/api/images/{id}` | ✅ | Get image |
| DELETE | `/api/images/{id}` | ✅ | Delete image |

---

## Events API (Server-Sent Events)

**Base Path:** `/api/events`

### Available Endpoints

| Method | Path | Status | Description |
|--------|------|--------|-------------|
| GET | `/api/events` | ✅ | SSE stream of all events |

---

## Approvals API

**Base Path:** `/api/approvals`

### Available Endpoints

| Method | Path | Status | Description |
|--------|------|--------|-------------|
| GET | `/api/approvals/pending` | ✅ | Get pending approvals |
| POST | `/api/approvals/{id}/approve` | ✅ | Approve an action |
| POST | `/api/approvals/{id}/reject` | ✅ | Reject an action |

---

## Filesystem API

**Base Path:** `/api/filesystem`

### Available Endpoints

| Method | Path | Status | Description |
|--------|------|--------|-------------|
| GET | `/api/filesystem/read` | ✅ | Read file content |
| POST | `/api/filesystem/write` | ✅ | Write file content |

---

## Containers API

**Base Path:** `/api/containers`

### Available Endpoints

| Method | Path | Status | Description |
|--------|------|--------|-------------|
| GET | `/api/containers/info` | ✅ | Get container information |

---

## Drafts API (Project-level)

**Base Path:** `/api/drafts`

### Available Endpoints

| Method | Path | Status | Description |
|--------|------|--------|-------------|
| GET | `/api/drafts/stream/ws` | ✅ | WebSocket stream of project drafts |

---

## Health Check

**Base Path:** `/api`

### Available Endpoints

| Method | Path | Status | Description |
|--------|------|--------|-------------|
| GET | `/health` | ✅ | Health check endpoint |

---

## Summary for MCP Tool Planning

### ✅ Currently Viable Features (21 Active MCP Tools)

**Projects:**
- List, create, update, delete projects ✓
- Browse project files ✓
- Open in editor ✓

**Tasks:**
- CRUD operations on tasks ✓
- Filter by status ✓
- Create and start in one step ✓

**Task Attempts:**
- Create and manage attempts ✓
- Create follow-up attempts ✓
- Merge into main branch ✓
- Stop running executions ✓
- Start dev servers ✓
- Monitor branch status ✓ (NEW: get_branch_status)
- View commit history with metadata ✓ (NEW: get_attempt_commits)
- Compare commits to HEAD ✓ (NEW: compare_commit_to_head)
- Abort merge/rebase conflicts ✓ (NEW: abort_conflicts)
- Manage conflicts ✓

**Execution Processes:**
- List and monitor processes ✓
- View logs (raw and normalized) ✓
- Stop processes ✓
- Stream logs via WebSocket ✓

### ❌ Features Requiring Custom Deployment (4 Disabled MCP Tools)

**GitHub Integration:**
- Create pull requests ✗
- Push branches to remote ✗

**Git Operations:**
- Rebase branches ✗

**Artifacts:**
- Get execution artifacts (diffs, commits) ✗

### 🎯 Recommendation

For maximum compatibility with the published NPM package:
1. **Use the 21 active MCP tools** - These work with current deployment
2. **Avoid the 4 disabled tools** - They require endpoints not yet published
3. **For PR workflows** - Use GitHub CLI (`gh`) or API directly as workaround
4. **For git operations** - Use git commands directly in task attempts
5. **For artifacts** - Query execution processes and use git commands

### 🆕 Recent Additions (2025-10-29)

**Batch 1 - Quick Wins (4 tools completed):**
- ✅ `get_branch_status` (VIBEK-34) - Monitor branch sync status, commits ahead/behind, conflicts
- ✅ `get_attempt_commits` (VIBEK-28) - View commit history with author, timestamp, diff stats
- ✅ `compare_commit_to_head` (VIBEK-29) - Compare commits and check if history is linear
- ✅ `abort_conflicts` (VIBEK-27) - Abort merge/rebase operations and restore clean state

When the project maintainer publishes the next NPM release, uncomment the 4 disabled tools in `crates/server/src/mcp/task_server.rs` to activate them.

---

## Testing Endpoints

To verify an endpoint is available:

```bash
# Check if endpoint returns success
curl -s http://localhost:3105/api/projects | jq '.success'

# Test with parameters
curl -s "http://localhost:3105/api/tasks?project_id={UUID}" | jq '.success'

# Test POST endpoint
curl -s -X POST http://localhost:3105/api/tasks \
  -H "Content-Type: application/json" \
  -d '{"project_id":"...","title":"..."}' | jq '.success'
```

Expected response for available endpoints:
```json
{
  "success": true,
  "data": { ... }
}
```

404 response indicates endpoint not available in current deployment.
