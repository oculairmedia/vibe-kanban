# Vibe Kanban MCP Server

Comprehensive Model Context Protocol (MCP) server implementation for Vibe Kanban, built with TurboMCP.

## Table of Contents

- [Overview](#overview)
- [Architecture](#architecture)
- [Migration from rmcp](#migration-from-rmcp)
- [Tool Implementation Patterns](#tool-implementation-patterns)
- [Error Handling](#error-handling)
- [Testing Guide](#testing-guide)
- [Deployment](#deployment)
- [API Reference](#api-reference)

---

## Overview

The Vibe Kanban MCP Server exposes AI coding task management capabilities via the Model Context Protocol. It enables AI agents (Claude Code, OpenCode, etc.) to:

- Manage projects, tasks, and execution attempts
- Start and monitor coding agent executions
- Access system configuration and executor profiles
- Discover git repositories and filesystem resources

### Key Features

- **Production-Ready**: TurboMCP provides enterprise-grade performance and security
- **Type-Safe**: Comprehensive Rust types with automatic schema generation via `schemars`
- **HTTP Transport**: Streamable HTTP/2 with CORS, rate limiting, and OAuth 2.1 support
- **100% MCP Compliant**: Implements MCP 2025-06-18 specification
- **RESTful Integration**: Wraps Vibe Kanban REST API with MCP-friendly interfaces

### Current Tools

The server provides two primary MCP servers:

#### 1. Task Server (`vibe-kanban`)
**11 tools** for task and project management:
- `create_task` - Create new tasks
- `list_projects` - List all projects
- `list_tasks` - List tasks with filtering
- `start_task_attempt` - Launch coding agent execution
- `update_task` - Update task metadata
- `delete_task` - Remove tasks
- `get_task` - Get task details
- `list_task_attempts` - View execution history
- `get_task_attempt` - Get attempt details
- `create_followup_attempt` - Create follow-up executions
- `merge_task_attempt` - Merge completed work

#### 2. System Server (`vibe-kanban-system`)
**8 tools** for system management:
- `get_system_info` - System environment details
- `get_config` - Get Vibe Kanban configuration
- `update_config` - Update settings
- `list_mcp_servers` - List MCP server configs
- `update_mcp_servers` - Update MCP configs
- `list_executor_profiles` - List available executors
- `list_git_repos` - Discover git repositories
- `list_directory` - Browse filesystem
- `health_check` - Server health status

---

## Architecture

### Project Structure

```
crates/server/src/mcp/
├── mod.rs                  # Module exports
├── task_server.rs          # Task management tools (11 tools)
├── system_server.rs        # System management tools (8 tools)
└── README.md              # This file
```

### Design Patterns

#### 1. Server Structure

Each MCP server is implemented as a Rust struct with:
- HTTP client for API communication
- Base URL configuration
- Shared utilities (error handling, URL construction)

```rust
#[derive(Clone)]
pub struct TaskServer {
    client: Arc<reqwest::Client>,
    base_url: Arc<String>,
}
```

#### 2. Tool Registration

Tools are registered using the `#[turbomcp::server]` and `#[tool]` macros:

```rust
#[turbomcp::server(
    name = "vibe-kanban",
    version = "1.0.0",
    description = "Task and project management server"
)]
impl TaskServer {
    #[tool(description = "Create a new task in a project")]
    async fn create_task(&self, request: CreateTaskRequest) -> McpResult<String> {
        // Implementation
    }
}
```

#### 3. Request/Response Types

All types use `schemars::JsonSchema` for automatic schema generation:

```rust
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreateTaskRequest {
    #[schemars(description = "The ID of the project")]
    pub project_id: Uuid,
    #[schemars(description = "The title of the task")]
    pub title: String,
    #[schemars(description = "Optional description")]
    pub description: Option<String>,
}
```

#### 4. API Integration Pattern

Standard pattern for calling Vibe Kanban REST API:

```rust
async fn send_json<T: DeserializeOwned>(
    &self,
    rb: reqwest::RequestBuilder,
) -> Result<T, McpError> {
    // 1. Send request
    let resp = rb.send().await
        .map_err(|e| Self::err_str("Failed to connect", Some(&e.to_string())))?;

    // 2. Check HTTP status
    if !resp.status().is_success() {
        return Err(Self::err_str(&format!("API error: {}", resp.status()), None));
    }

    // 3. Parse API envelope
    let api_response = resp.json::<ApiResponseEnvelope<T>>().await
        .map_err(|e| Self::err_str("Parse error", Some(&e.to_string())))?;

    // 4. Check API success flag
    if !api_response.success {
        let msg = api_response.message.as_deref().unwrap_or("Unknown error");
        return Err(Self::err_str("API returned error", Some(msg)));
    }

    // 5. Extract data
    api_response.data
        .ok_or_else(|| Self::err_str("Response missing data", None))
}
```

---

## Migration from rmcp

### Overview

Vibe Kanban successfully migrated from `rmcp` to `turbomcp` in Phase 2 of the MCP migration project. This guide documents the migration process and patterns.

### Why TurboMCP?

| Feature | rmcp | TurboMCP |
|---------|------|----------|
| **Performance** | Standard | SIMD-accelerated JSON (2-3x faster) |
| **Security** | Basic | OAuth 2.1, CORS, TLS, rate limiting |
| **Transport** | HTTP | STDIO, HTTP, WebSocket, TCP, Unix sockets |
| **Production Features** | Limited | Connection pooling, health monitoring, auto-retry |
| **MCP Compliance** | Partial | 100% (MCP 2025-06-18 spec) |
| **Type Safety** | Yes | Comprehensive with schemars |

### Migration Steps

#### Step 1: Update Dependencies

**Before** (`Cargo.toml`):
```toml
[dependencies]
rmcp = "0.1"
```

**After**:
```toml
[dependencies]
turbomcp = { version = "2.0.0-rc.3", features = ["http", "schemars"] }
turbomcp-macros = "2.0.0-rc.3"
turbomcp-protocol = "2.0.0-rc.3"
turbomcp-server = "2.0.0-rc.3"
turbomcp-transport = { version = "2.0.0-rc.3", features = ["http"] }
schemars = { version = "1.0.4", features = ["derive", "chrono04", "uuid1"] }

# Use patched fork with $defs support
[patch.crates-io]
turbomcp = { git = "https://github.com/oculairmedia/turbomcp.git", branch = "feature/flatten-structs" }
turbomcp-macros = { git = "https://github.com/oculairmedia/turbomcp.git", branch = "feature/flatten-structs" }
# ... (repeat for all turbomcp crates)
```

#### Step 2: Update Server Registration

**Before** (rmcp):
```rust
use rmcp::*;

#[server(name = "vibe-kanban")]
pub struct TaskServer {
    client: reqwest::Client,
    base_url: String,
}
```

**After** (TurboMCP):
```rust
use turbomcp::prelude::*;
use std::sync::Arc;

#[derive(Clone)]
pub struct TaskServer {
    client: Arc<reqwest::Client>,
    base_url: Arc<String>,
}

#[turbomcp::server(
    name = "vibe-kanban",
    version = "1.0.0",
    description = "Task and project management server"
)]
impl TaskServer {
    // Tools defined here
}
```

**Key Changes**:
- Import `turbomcp::prelude::*` instead of `rmcp::*`
- Add `#[derive(Clone)]` to server struct
- Wrap fields in `Arc<>` for thread safety
- Move `#[turbomcp::server]` to `impl` block
- Add version and description

#### Step 3: Update Request/Response Types

**Before** (rmcp):
```rust
#[derive(Deserialize)]
pub struct CreateTaskRequest {
    pub project_id: Uuid,
    pub title: String,
}

#[derive(Serialize)]
pub struct CreateTaskResponse {
    pub task_id: String,
}
```

**After** (TurboMCP):
```rust
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreateTaskRequest {
    #[schemars(description = "The ID of the project to create the task in")]
    pub project_id: Uuid,
    #[schemars(description = "The title of the task")]
    pub title: String,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct CreateTaskResponse {
    #[schemars(description = "The ID of the created task")]
    pub task_id: String,
}
```

**Key Changes**:
- Add `schemars::JsonSchema` derive
- Add `#[schemars(description = "...")]` to all fields
- Add `Debug` derive for better error messages

#### Step 4: Update Tool Signatures

**Before** (rmcp):
```rust
#[tool(description = "Create a new task")]
async fn create_task(&self, params: CreateTaskRequest) -> Result<CallToolResult, ErrorData> {
    let url = self.url("/api/tasks");
    let task: Task = self.send_json(self.client.post(&url).json(&params)).await?;

    TaskServer::success(&CreateTaskResponse {
        task_id: task.id.to_string(),
    })
}
```

**After** (TurboMCP):
```rust
#[tool(description = "Create a new task in a project")]
async fn create_task(&self, request: CreateTaskRequest) -> McpResult<String> {
    let url = self.url("/api/tasks");
    let task: Task = self.send_json(
        self.client.post(&url).json(&CreateTask::from_title_description(
            request.project_id,
            request.title,
            request.description,
        ))
    ).await?;

    let response = CreateTaskResponse {
        task_id: task.id.to_string(),
    };
    Ok(serde_json::to_string_pretty(&response).unwrap())
}
```

**Key Changes**:
- Return type: `Result<CallToolResult, ErrorData>` → `McpResult<String>`
- Parameter name: `params` → `request` (convention)
- Return format: `TaskServer::success(&data)` → `Ok(serde_json::to_string_pretty(&data).unwrap())`

#### Step 5: Update Error Handling

**Before** (rmcp):
```rust
async fn send_json<T>(&self, rb: RequestBuilder) -> Result<T, ErrorData> {
    let resp = rb.send().await
        .map_err(|e| ErrorData::internal(&e.to_string()))?;

    // ...
}
```

**After** (TurboMCP):
```rust
fn err_str(msg: &str, details: Option<&str>) -> McpError {
    let mut error_msg = msg.to_string();
    if let Some(d) = details {
        error_msg.push_str(&format!(": {}", d));
    }
    McpError::internal(error_msg)
}

async fn send_json<T: DeserializeOwned>(
    &self,
    rb: reqwest::RequestBuilder,
) -> Result<T, McpError> {
    let resp = rb.send().await
        .map_err(|e| Self::err_str("Failed to connect to VK API", Some(&e.to_string())))?;

    // ...
}
```

**Key Changes**:
- Error type: `ErrorData` → `McpError`
- Use helper function `err_str()` for consistent error formatting
- More descriptive error messages

#### Step 6: Update HTTP Server Initialization

**Before** (rmcp - automatic):
```rust
// rmcp handled server initialization internally
```

**After** (TurboMCP - explicit):
```rust
#[cfg(feature = "http")]
impl TaskServer {
    /// Run HTTP server with custom security configuration
    pub async fn run_http_custom(&self, addr: &str) -> Result<(), Box<dyn std::error::Error>> {
        use turbomcp_transport::streamable_http_v2::{StreamableHttpConfigBuilder, run_server};
        use std::time::Duration;

        let config = StreamableHttpConfigBuilder::new()
            .with_bind_address(addr)
            .allow_any_origin(true) // Development mode
            .allow_localhost(true)
            .with_rate_limit(1_000_000, Duration::from_secs(60))
            .build();

        run_server(config, Arc::new(self.clone())).await?;
        Ok(())
    }
}
```

**Key Changes**:
- Explicit HTTP configuration with `StreamableHttpConfigBuilder`
- Fine-grained control over CORS, rate limiting, security
- Separation of development vs production configs

### Common Migration Pitfalls

#### 1. Forgetting `schemars::JsonSchema`

**Problem**: MCP schema generation fails
```rust
// ❌ WRONG
#[derive(Deserialize)]
pub struct Request { ... }
```

**Solution**: Always add `schemars::JsonSchema`
```rust
// ✅ CORRECT
#[derive(Deserialize, schemars::JsonSchema)]
pub struct Request { ... }
```

#### 2. Using non-Clone server struct

**Problem**: TurboMCP requires `Clone` for Arc wrapping
```rust
// ❌ WRONG
pub struct TaskServer { ... }
```

**Solution**: Add `#[derive(Clone)]`
```rust
// ✅ CORRECT
#[derive(Clone)]
pub struct TaskServer { ... }
```

#### 3. Missing Arc wrappers

**Problem**: Shared state not thread-safe
```rust
// ❌ WRONG
pub struct TaskServer {
    client: reqwest::Client,
}
```

**Solution**: Wrap in `Arc<>`
```rust
// ✅ CORRECT
pub struct TaskServer {
    client: Arc<reqwest::Client>,
}
```

#### 4. Returning wrong type from tools

**Problem**: Type mismatch errors
```rust
// ❌ WRONG
async fn tool(&self) -> Result<Response, McpError> { ... }
```

**Solution**: Return `McpResult<String>`
```rust
// ✅ CORRECT
async fn tool(&self) -> McpResult<String> {
    Ok(serde_json::to_string_pretty(&response).unwrap())
}
```

---

## Tool Implementation Patterns

### Pattern 1: Simple Query Tool

For read-only operations that fetch and return data:

```rust
#[tool(description = "List all projects")]
async fn list_projects(&self) -> McpResult<String> {
    // 1. Build URL
    let url = self.url("/api/projects");

    // 2. Call API
    let projects: Vec<Project> = self.send_json(self.client.get(&url)).await?;

    // 3. Transform to response type
    let project_summaries: Vec<ProjectSummary> = projects
        .into_iter()
        .map(ProjectSummary::from_project)
        .collect();

    // 4. Build response
    let response = ListProjectsResponse {
        count: project_summaries.len(),
        projects: project_summaries,
    };

    // 5. Serialize and return
    Ok(serde_json::to_string_pretty(&response).unwrap())
}
```

### Pattern 2: Filtered Query Tool

For queries with optional filters and pagination:

```rust
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListTasksRequest {
    #[schemars(description = "The ID of the project")]
    pub project_id: Uuid,
    #[schemars(description = "Optional status filter")]
    pub status: Option<String>,
    #[schemars(description = "Maximum results (default: 50)")]
    pub limit: Option<i32>,
}

#[tool(description = "List tasks with optional filtering")]
async fn list_tasks(&self, request: ListTasksRequest) -> McpResult<String> {
    // 1. Validate and parse filter
    let status_filter = if let Some(ref status_str) = request.status {
        match TaskStatus::from_str(status_str) {
            Ok(s) => Some(s),
            Err(_) => {
                return Err(McpError::invalid_request(format!(
                    "Invalid status '{}'. Valid: 'todo', 'in-progress', 'done'",
                    status_str
                )));
            }
        }
    } else {
        None
    };

    // 2. Fetch all data
    let url = self.url(&format!("/api/tasks?project_id={}", request.project_id));
    let all_tasks: Vec<Task> = self.send_json(self.client.get(&url)).await?;

    // 3. Apply filters
    let task_limit = request.limit.unwrap_or(50).max(0) as usize;
    let filtered: Vec<Task> = all_tasks.into_iter()
        .filter(|t| status_filter.as_ref().map_or(true, |s| &t.status == s))
        .take(task_limit)
        .collect();

    // 4. Build response
    let response = ListTasksResponse {
        count: filtered.len(),
        tasks: filtered,
        applied_filters: ListTasksFilters {
            status: request.status,
            limit: task_limit as i32,
        },
    };

    Ok(serde_json::to_string_pretty(&response).unwrap())
}
```

### Pattern 3: Create Tool with Validation

For tools that create new resources:

```rust
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreateTaskRequest {
    #[schemars(description = "Project ID (required)")]
    pub project_id: Uuid,
    #[schemars(description = "Task title")]
    pub title: String,
    #[schemars(description = "Optional description")]
    pub description: Option<String>,
}

#[tool(description = "Create a new task")]
async fn create_task(&self, request: CreateTaskRequest) -> McpResult<String> {
    // 1. Input validation
    if request.title.trim().is_empty() {
        return Err(McpError::invalid_request("Title cannot be empty"));
    }

    // 2. Build API request payload
    let create_payload = CreateTask::from_title_description(
        request.project_id,
        request.title,
        request.description,
    );

    // 3. Make API call
    let url = self.url("/api/tasks");
    let task: Task = self.send_json(
        self.client.post(&url).json(&create_payload)
    ).await?;

    // 4. Return created resource
    let response = CreateTaskResponse {
        task_id: task.id.to_string(),
    };

    Ok(serde_json::to_string_pretty(&response).unwrap())
}
```

### Pattern 4: Update Tool with Partial Fields

For tools that update existing resources:

```rust
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct UpdateTaskRequest {
    #[schemars(description = "Task ID to update")]
    pub task_id: Uuid,
    #[schemars(description = "New title (optional)")]
    pub title: Option<String>,
    #[schemars(description = "New status (optional)")]
    pub status: Option<String>,
}

#[tool(description = "Update a task")]
async fn update_task(&self, request: UpdateTaskRequest) -> McpResult<String> {
    // 1. Validate status if provided
    let status = if let Some(ref status_str) = request.status {
        match TaskStatus::from_str(status_str) {
            Ok(s) => Some(s),
            Err(_) => {
                return Err(McpError::invalid_request(format!(
                    "Invalid status '{}'", status_str
                )));
            }
        }
    } else {
        None
    };

    // 2. Build partial update payload
    let payload = UpdateTask {
        title: request.title,
        status,
        ..Default::default()
    };

    // 3. Make API call
    let url = self.url(&format!("/api/tasks/{}", request.task_id));
    let updated_task: Task = self.send_json(
        self.client.put(&url).json(&payload)
    ).await?;

    // 4. Return updated resource
    let response = UpdateTaskResponse {
        task: TaskDetails::from_task(updated_task),
    };

    Ok(serde_json::to_string_pretty(&response).unwrap())
}
```

### Pattern 5: Complex Operation with Multiple Steps

For tools that orchestrate multiple API calls:

```rust
#[tool(description = "Merge a task attempt into target branch")]
async fn merge_task_attempt(&self, request: MergeTaskAttemptRequest) -> McpResult<String> {
    // 1. Trigger merge operation
    let url = self.url(&format!("/api/task-attempts/{}/merge", request.attempt_id));
    self.send_json::<serde_json::Value>(self.client.post(&url)).await?;

    // 2. Fetch updated attempt details for response
    let attempt_url = self.url(&format!("/api/task-attempts/{}", request.attempt_id));
    let attempt: TaskAttempt = self.send_json(self.client.get(&attempt_url)).await?;

    // 3. Build comprehensive response
    let response = MergeTaskAttemptResponse {
        success: true,
        message: "Task attempt merged successfully".to_string(),
        task_id: attempt.task_id.to_string(),
        attempt_id: request.attempt_id.to_string(),
    };

    Ok(serde_json::to_string_pretty(&response).unwrap())
}
```

### Pattern 6: Enum-Based Validation

For tools with restricted value sets:

```rust
// Define valid values as constants
const VALID_EXECUTORS: &[&str] = &[
    "CLAUDE_CODE",
    "AMP",
    "GEMINI",
    "CODEX",
    "OPENCODE",
    "CURSOR",
];

fn validate_executor(executor: &str) -> Result<(), String> {
    if VALID_EXECUTORS.contains(&executor) {
        Ok(())
    } else {
        Err(format!(
            "Unknown executor '{}'. Valid: {}",
            executor,
            VALID_EXECUTORS.join(", ")
        ))
    }
}

#[tool(description = "Start a task execution")]
async fn start_task_attempt(&self, request: StartTaskAttemptRequest) -> McpResult<String> {
    // Validate executor
    let normalized = request.executor.trim().replace('-', "_").to_ascii_uppercase();
    if let Err(err_msg) = validate_executor(&normalized) {
        return Err(McpError::invalid_request(err_msg));
    }

    // ... rest of implementation
}
```

---

## Error Handling

### Error Types

TurboMCP provides several error constructors:

```rust
// For invalid client requests
McpError::invalid_request("Task ID is required")

// For internal server errors
McpError::internal("Database connection failed")

// For resource not found
McpError::internal("Task not found") // Note: no dedicated NotFound variant

// For method not supported
McpError::method_not_found("This operation is not supported")
```

### Error Handling Utilities

Standard error helper for consistent formatting:

```rust
fn err_str(msg: &str, details: Option<&str>) -> McpError {
    let mut error_msg = msg.to_string();
    if let Some(d) = details {
        error_msg.push_str(&format!(": {}", d));
    }
    McpError::internal(error_msg)
}

// Usage:
.map_err(|e| Self::err_str("Failed to connect", Some(&e.to_string())))?
```

### API Response Error Handling

Pattern for handling Vibe Kanban API responses:

```rust
async fn send_json<T: DeserializeOwned>(
    &self,
    rb: reqwest::RequestBuilder,
) -> Result<T, McpError> {
    // 1. Network errors
    let resp = rb.send().await
        .map_err(|e| Self::err_str("Failed to connect to VK API", Some(&e.to_string())))?;

    // 2. HTTP status errors
    if !resp.status().is_success() {
        let status = resp.status();
        return Err(Self::err_str(
            &format!("VK API returned error status: {}", status),
            None,
        ));
    }

    // 3. Parse API envelope
    let api_response = resp.json::<ApiResponseEnvelope<T>>().await
        .map_err(|e| Self::err_str("Failed to parse VK API response", Some(&e.to_string())))?;

    // 4. API-level errors
    if !api_response.success {
        let msg = api_response.message.as_deref().unwrap_or("Unknown error");
        return Err(Self::err_str("VK API returned error", Some(msg)));
    }

    // 5. Missing data errors
    api_response.data
        .ok_or_else(|| Self::err_str("VK API response missing data field", None))
}
```

### Input Validation Errors

Best practices for validation:

```rust
#[tool(description = "Start a task execution")]
async fn start_task_attempt(&self, request: StartTaskAttemptRequest) -> McpResult<String> {
    // Validate required fields
    let base_branch = request.base_branch.trim().to_string();
    if base_branch.is_empty() {
        return Err(McpError::invalid_request("Base branch must not be empty"));
    }

    // Validate enum values
    let executor_trimmed = request.executor.trim();
    if executor_trimmed.is_empty() {
        return Err(McpError::invalid_request("Executor must not be empty"));
    }

    let normalized_executor = executor_trimmed.replace('-', "_").to_ascii_uppercase();
    if let Err(err_msg) = validate_executor(&normalized_executor) {
        return Err(McpError::invalid_request(err_msg));
    }

    // Validate format
    if let Some(status_str) = &request.status {
        match TaskStatus::from_str(status_str) {
            Ok(_) => {},
            Err(_) => {
                return Err(McpError::invalid_request(format!(
                    "Invalid status '{}'. Valid: 'todo', 'in-progress', 'done', 'cancelled'",
                    status_str
                )));
            }
        }
    }

    // ... proceed with validated inputs
}
```

### Error Context

Add context to errors for debugging:

```rust
// ❌ Bad - no context
.map_err(|e| McpError::internal(e.to_string()))?

// ✅ Good - descriptive context
.map_err(|e| Self::err_str("Failed to list git repositories", Some(&e.to_string())))?
```

---

## Testing Guide

### Unit Testing Tools

Test individual tool methods:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_list_projects() {
        // Setup mock server
        let mock_server = MockServer::start().await;
        let server = TaskServer::new(&mock_server.uri());

        // Mock API response
        Mock::given(method("GET"))
            .and(path("/api/projects"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "success": true,
                "data": [
                    {"id": "uuid-1", "name": "Project 1"},
                    {"id": "uuid-2", "name": "Project 2"}
                ]
            })))
            .mount(&mock_server)
            .await;

        // Test tool
        let result = server.list_projects().await;
        assert!(result.is_ok());

        let response: ListProjectsResponse = serde_json::from_str(&result.unwrap()).unwrap();
        assert_eq!(response.count, 2);
    }
}
```

### Integration Testing

Test against real Vibe Kanban instance:

```bash
# Start Vibe Kanban in test mode
npm run backend:dev

# Run integration tests
cargo test --package server --lib mcp::tests::integration -- --nocapture
```

Example integration test:

```rust
#[tokio::test]
#[ignore] // Run with --ignored flag
async fn test_create_and_list_tasks_integration() {
    let server = TaskServer::new("http://127.0.0.1:8080");

    // Create task
    let create_req = CreateTaskRequest {
        project_id: test_project_id(),
        title: "Test Task".to_string(),
        description: Some("Integration test".to_string()),
    };

    let create_result = server.create_task(create_req).await.unwrap();
    let create_response: CreateTaskResponse = serde_json::from_str(&create_result).unwrap();

    // List tasks
    let list_req = ListTasksRequest {
        project_id: test_project_id(),
        status: None,
        limit: None,
    };

    let list_result = server.list_tasks(list_req).await.unwrap();
    let list_response: ListTasksResponse = serde_json::from_str(&list_result).unwrap();

    // Verify task appears in list
    assert!(list_response.tasks.iter().any(|t| t.id == create_response.task_id));
}
```

### Testing with MCP Client

Test using Claude Code or other MCP clients:

```bash
# 1. Build and run the server
cargo build --release
./target/release/vibe-kanban-mcp

# 2. Configure Claude Code to connect
# Add to ~/.config/claude-code/config.json:
{
  "mcpServers": {
    "vibe-kanban": {
      "transport": "http",
      "url": "http://127.0.0.1:3456/mcp"
    }
  }
}

# 3. Test in Claude Code
# > /mcp
# Should show vibe-kanban server with all tools

# 4. Test a tool
# > Use vibe-kanban to list all projects
```

### Validation Testing

Test input validation:

```rust
#[tokio::test]
async fn test_invalid_status_rejected() {
    let server = TaskServer::new("http://127.0.0.1:8080");

    let request = UpdateTaskRequest {
        task_id: Uuid::new_v4(),
        title: None,
        status: Some("invalid-status".to_string()),
    };

    let result = server.update_task(request).await;
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert!(err.to_string().contains("Invalid status"));
}
```

### Error Handling Tests

Test error propagation:

```rust
#[tokio::test]
async fn test_api_error_propagation() {
    let mock_server = MockServer::start().await;
    let server = TaskServer::new(&mock_server.uri());

    // Mock API error
    Mock::given(method("GET"))
        .and(path("/api/projects"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&mock_server)
        .await;

    let result = server.list_projects().await;
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert!(err.to_string().contains("API returned error status: 500"));
}
```

---

## Deployment

### Building the Server

```bash
# Development build
cargo build --package server --lib

# Production build (optimized)
cargo build --release --package server --lib

# Build with specific features
cargo build --features http --package server
```

### Running as HTTP Server

The MCP servers are embedded in the Vibe Kanban main server and start automatically:

```bash
# Start Vibe Kanban (includes MCP servers)
npm run backend:dev

# MCP endpoints:
# - Task Server: http://127.0.0.1:3456/mcp
# - System Server: http://127.0.0.1:3457/mcp
```

### Standalone Mode (Future)

For standalone deployment:

```rust
// main.rs (future implementation)
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let task_server = TaskServer::new("http://127.0.0.1:8080");
    let system_server = SystemServer::new("http://127.0.0.1:8080");

    tokio::try_join!(
        task_server.run_http_custom("127.0.0.1:3456"),
        system_server.run_http_custom("127.0.0.1:3457"),
    )?;

    Ok(())
}
```

### Configuration

Environment variables:

```bash
# Vibe Kanban API base URL
export VIBE_KANBAN_API_URL="http://127.0.0.1:8080"

# MCP server ports
export VIBE_KANBAN_MCP_TASK_PORT="3456"
export VIBE_KANBAN_MCP_SYSTEM_PORT="3457"

# CORS settings (development)
export MCP_ALLOW_ANY_ORIGIN="true"
export MCP_ALLOW_LOCALHOST="true"

# Rate limiting
export MCP_RATE_LIMIT_REQUESTS="1000000"
export MCP_RATE_LIMIT_WINDOW_SECS="60"
```

### Docker Deployment (Future)

```dockerfile
# Dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release --package server

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/vibe-kanban-mcp /usr/local/bin/
EXPOSE 3456 3457
CMD ["vibe-kanban-mcp"]
```

### Health Monitoring

Check server health:

```bash
# Task server health (via system server tool)
curl -X POST http://127.0.0.1:3457/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"health_check","arguments":{}}}'

# Expected response:
{
  "status": "healthy",
  "version": "1.0.0",
  "uptime_seconds": 3600
}
```

---

## API Reference

### Task Server Tools

See [MCP_TOOLS.md](../../../../docs/MCP_TOOLS.md) for complete API reference.

Quick reference:

| Tool | Description |
|------|-------------|
| `create_task` | Create new task |
| `list_projects` | List all projects |
| `list_tasks` | List tasks with filtering |
| `start_task_attempt` | Start coding execution |
| `update_task` | Update task metadata |
| `delete_task` | Delete task |
| `get_task` | Get task details |
| `list_task_attempts` | List execution attempts |
| `get_task_attempt` | Get attempt details |
| `create_followup_attempt` | Create follow-up |
| `merge_task_attempt` | Merge completed work |

### System Server Tools

| Tool | Description |
|------|-------------|
| `get_system_info` | System environment info |
| `get_config` | Get configuration |
| `update_config` | Update settings |
| `list_executor_profiles` | List executors |
| `list_git_repos` | Find git repos |
| `list_directory` | Browse filesystem |
| `health_check` | Server health |

---

## Reference Documentation

- **MCP Specification**: https://spec.modelcontextprotocol.io/
- **TurboMCP**: https://github.com/QuantumEntangledAndy/turbomcp
- **Vibe Kanban API**: See `crates/server/src/routes/`
- **Letta MCP Reference**: `/opt/stacks/letta-MCP-server/`

---

## Changelog

See [CHANGELOG.md](../../../../CHANGELOG.md) for version history and migration notes.

---

**Version**: 2.0.0
**Last Updated**: 2025-10-28
**Migration Status**: Phase 2 Complete (rmcp → TurboMCP)
