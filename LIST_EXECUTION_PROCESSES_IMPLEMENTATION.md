# Implementation: list_execution_processes Tool

## Summary
Added the `list_execution_processes` MCP tool to enable monitoring and debugging of execution processes for task attempts.

## Changes Made

### 1. Updated Imports (`task_server.rs:3-7`)
Added execution process model imports:
```rust
use db::models::{
    execution_process::{ExecutionProcess, ExecutionProcessRunReason, ExecutionProcessStatus},
    project::Project,
    task::{CreateTask, Task, TaskStatus, TaskWithAttemptStatus, UpdateTask},
    task_attempt::TaskAttempt,
};
```

### 2. Added Request/Response Types (`task_server.rs:~363-447`)

**Request Type:**
```rust
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListExecutionProcessesRequest {
    #[schemars(description = "The ID of the task attempt to list processes for")]
    pub task_attempt_id: Uuid,
    #[schemars(description = "Whether to include soft-deleted (dropped) processes")]
    pub show_soft_deleted: Option<bool>,
}
```

**Response Type:**
```rust
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct ListExecutionProcessesResponse {
    pub processes: Vec<ExecutionProcessSummary>,
    pub count: usize,
    pub task_attempt_id: String,
}
```

**Summary Type:**
```rust
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct ExecutionProcessSummary {
    pub id: String,
    pub task_attempt_id: String,
    pub run_reason: String,  // 'setupscript', 'cleanupscript', 'codingagent', 'devserver'
    pub status: String,       // 'running', 'completed', 'failed', 'killed'
    pub exit_code: Option<i64>,
    pub dropped: bool,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub before_head_commit: Option<String>,
    pub after_head_commit: Option<String>,
}
```

### 3. Implemented Tool (`task_server.rs:~806-831`)

```rust
#[tool(
    description = "List all execution processes for a specific task attempt. Shows setup scripts, cleanup scripts, coding agent runs, and dev servers. Useful for monitoring what's currently running and debugging process history. `task_attempt_id` is required!"
)]
async fn list_execution_processes(&self, request: ListExecutionProcessesRequest) -> McpResult<String> {
    let show_deleted = request.show_soft_deleted.unwrap_or(false);
    let url = self.url(&format!(
        "/api/processes?task_attempt_id={}&show_soft_deleted={}",
        request.task_attempt_id,
        show_deleted
    ));

    let processes: Vec<ExecutionProcess> = self.send_json(self.client.get(&url)).await?;

    let process_summaries: Vec<ExecutionProcessSummary> = processes
        .into_iter()
        .map(ExecutionProcessSummary::from_execution_process)
        .collect();

    let response = ListExecutionProcessesResponse {
        count: process_summaries.len(),
        processes: process_summaries,
        task_attempt_id: request.task_attempt_id.to_string(),
    };

    Ok(serde_json::to_string_pretty(&response).unwrap())
}
```

## API Integration

**Backend Endpoint:** `GET /api/processes?task_attempt_id={uuid}&show_soft_deleted={bool}`

**Backend Handler:** `crates/server/src/routes/execution_processes.rs::get_execution_processes()`

## Tool Schema

```json
{
  "name": "list_execution_processes",
  "description": "List all execution processes for a specific task attempt...",
  "inputSchema": {
    "type": "object",
    "properties": {
      "show_soft_deleted": {
        "description": "Whether to include soft-deleted (dropped) processes",
        "type": ["boolean", "null"]
      },
      "task_attempt_id": {
        "description": "The ID of the task attempt to list processes for",
        "type": "string",
        "format": "uuid"
      }
    },
    "required": ["task_attempt_id"]
  }
}
```

## Usage Example

### Via MCP Client
```javascript
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "list_execution_processes",
    "arguments": {
      "task_attempt_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
      "show_soft_deleted": false
    }
  },
  "id": 1
}
```

### Response
```json
{
  "processes": [
    {
      "id": "proc-uuid-1",
      "task_attempt_id": "attempt-uuid",
      "run_reason": "setupscript",
      "status": "completed",
      "exit_code": 0,
      "dropped": false,
      "started_at": "2025-10-29T05:00:00Z",
      "completed_at": "2025-10-29T05:00:30Z",
      "before_head_commit": "abc123",
      "after_head_commit": "abc123"
    },
    {
      "id": "proc-uuid-2",
      "task_attempt_id": "attempt-uuid",
      "run_reason": "codingagent",
      "status": "running",
      "exit_code": null,
      "dropped": false,
      "started_at": "2025-10-29T05:01:00Z",
      "completed_at": null,
      "before_head_commit": "abc123",
      "after_head_commit": null
    }
  ],
  "count": 2,
  "task_attempt_id": "attempt-uuid"
}
```

## Use Cases

1. **Monitoring Active Processes**
   - Check what's currently running for a task attempt
   - Identify long-running processes

2. **Debugging Failed Attempts**
   - Review all processes that ran
   - Check exit codes and status
   - Identify which process failed

3. **Understanding Execution History**
   - See the sequence of setup → coding → cleanup
   - Track git commits before/after each process

4. **Process Management**
   - List processes before stopping them
   - Identify processes to get logs from

## Testing

### Build
```bash
cd /opt/stacks/vibe-kanban
cargo build --bin mcp_task_server
```

### Start Server
```bash
TRANSPORT=http MCP_PORT=9717 ./target/debug/mcp_task_server
```

### Verify Tool Registration
```bash
curl -s -X POST http://192.168.50.90:9717/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc": "2.0", "method": "tools/list", "id": 1}' \
  | jq '.result.tools[] | select(.name == "list_execution_processes")'
```

## Status

✅ **COMPLETED**

- Tool implementation: ✅
- Schema generation: ✅ (2 properties with descriptions)
- Backend integration: ✅ (uses existing `/api/processes` endpoint)
- Testing: ✅ (verified tool registration and schema)
- Documentation: ✅

## Related Files

- `crates/server/src/mcp/task_server.rs` - Tool implementation
- `crates/server/src/routes/execution_processes.rs` - Backend API
- `crates/db/src/models/execution_process.rs` - Data models

## Task
- Huly Issue: VIBEK-38
- Task ID: bf63880f-49d3-460e-94f2-0990ef384054
- Title: Implement list_execution_processes tool
