//! HTTP client for MCP integration tests
//!
//! Provides utilities for making real HTTP calls to MCP servers

use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

static REQUEST_ID: AtomicU64 = AtomicU64::new(1);

/// MCP JSON-RPC request structure
#[derive(Debug, Serialize)]
pub struct McpRequest {
    pub jsonrpc: &'static str,
    pub id: u64,
    pub method: &'static str,
    pub params: Value,
}

/// MCP JSON-RPC response structure
#[derive(Debug, Deserialize)]
pub struct McpResponse {
    pub jsonrpc: String,
    pub id: u64,
    pub result: Option<McpResult>,
    pub error: Option<McpError>,
}

#[derive(Debug, Deserialize)]
pub struct McpResult {
    pub content: Vec<McpContent>,
    #[serde(rename = "isError")]
    pub is_error: bool,
}

#[derive(Debug, Deserialize)]
pub struct McpContent {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: String,
}

#[derive(Debug, Deserialize)]
pub struct McpError {
    pub code: i32,
    pub message: String,
}

/// HTTP client for calling MCP servers
pub struct McpClient {
    client: Client,
    base_url: String,
}

impl McpClient {
    /// Create a new MCP client for the task server
    pub fn task_server() -> Self {
        Self::new(&std::env::var("MCP_TASK_URL").unwrap_or_else(|_| "http://localhost:9717".to_string()))
    }

    /// Create a new MCP client for the system server
    pub fn system_server() -> Self {
        Self::new(&std::env::var("MCP_SYSTEM_URL").unwrap_or_else(|_| "http://localhost:9718".to_string()))
    }

    /// Create a new MCP client with a custom URL
    pub fn new(base_url: &str) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            base_url: base_url.to_string(),
        }
    }

    /// Check if the MCP server is available
    pub async fn is_available(&self) -> bool {
        self.list_tools().await.is_ok()
    }

    /// List available tools from the MCP server
    pub async fn list_tools(&self) -> Result<Vec<Value>, McpClientError> {
        let request = McpRequest {
            jsonrpc: "2.0",
            id: REQUEST_ID.fetch_add(1, Ordering::SeqCst),
            method: "tools/list",
            params: json!({}),
        };

        let response = self
            .client
            .post(&format!("{}/mcp", self.base_url))
            .json(&request)
            .send()
            .await
            .map_err(|e| McpClientError::HttpError(e.to_string()))?;

        let mcp_response: Value = response
            .json()
            .await
            .map_err(|e| McpClientError::ParseError(e.to_string()))?;

        mcp_response["result"]["tools"]
            .as_array()
            .cloned()
            .ok_or_else(|| McpClientError::InvalidResponse("Missing tools array".to_string()))
    }

    /// Call an MCP tool with the given name and arguments
    pub async fn call_tool(&self, name: &str, arguments: Value) -> Result<Value, McpClientError> {
        let request = McpRequest {
            jsonrpc: "2.0",
            id: REQUEST_ID.fetch_add(1, Ordering::SeqCst),
            method: "tools/call",
            params: json!({
                "name": name,
                "arguments": arguments
            }),
        };

        let response = self
            .client
            .post(&format!("{}/mcp", self.base_url))
            .json(&request)
            .send()
            .await
            .map_err(|e| McpClientError::HttpError(e.to_string()))?;

        let mcp_response: McpResponse = response
            .json()
            .await
            .map_err(|e| McpClientError::ParseError(e.to_string()))?;

        if let Some(error) = mcp_response.error {
            return Err(McpClientError::McpError {
                code: error.code,
                message: error.message,
            });
        }

        let result = mcp_response
            .result
            .ok_or_else(|| McpClientError::InvalidResponse("Missing result".to_string()))?;

        if result.is_error {
            let error_text = result
                .content
                .first()
                .map(|c| c.text.clone())
                .unwrap_or_else(|| "Unknown error".to_string());
            return Err(McpClientError::ToolError(error_text));
        }

        let text = result
            .content
            .first()
            .map(|c| c.text.clone())
            .ok_or_else(|| McpClientError::InvalidResponse("Empty content".to_string()))?;

        serde_json::from_str(&text)
            .map_err(|e| McpClientError::ParseError(format!("Failed to parse tool response: {}", e)))
    }

    // ==================== Task Server Tools ====================

    /// List all projects
    pub async fn list_projects(&self) -> Result<Value, McpClientError> {
        self.call_tool("list_projects", json!({})).await
    }

    /// Get a project by ID
    pub async fn get_project(&self, project_id: &str) -> Result<Value, McpClientError> {
        self.call_tool("get_project", json!({ "project_id": project_id })).await
    }

    /// Create a new project
    pub async fn create_project(&self, name: &str, git_repo_path: &str) -> Result<Value, McpClientError> {
        self.call_tool("create_project", json!({
            "name": name,
            "git_repo_path": git_repo_path,
            "use_existing_repo": true
        })).await
    }

    /// Update a project
    pub async fn update_project(&self, project_id: &str, name: Option<&str>) -> Result<Value, McpClientError> {
        let mut args = json!({ "project_id": project_id });
        if let Some(n) = name {
            args["name"] = json!(n);
        }
        self.call_tool("update_project", args).await
    }

    /// Delete a project
    pub async fn delete_project(&self, project_id: &str) -> Result<Value, McpClientError> {
        self.call_tool("delete_project", json!({ "project_id": project_id })).await
    }

    /// List tasks for a project
    pub async fn list_tasks(&self, project_id: &str) -> Result<Value, McpClientError> {
        self.call_tool("list_tasks", json!({ "project_id": project_id })).await
    }

    /// Get a task by ID
    pub async fn get_task(&self, task_id: &str) -> Result<Value, McpClientError> {
        self.call_tool("get_task", json!({ "task_id": task_id })).await
    }

    /// Create a new task
    pub async fn create_task(&self, project_id: &str, title: &str, description: Option<&str>) -> Result<Value, McpClientError> {
        let mut args = json!({
            "project_id": project_id,
            "title": title
        });
        if let Some(desc) = description {
            args["description"] = json!(desc);
        }
        self.call_tool("create_task", args).await
    }

    /// Update a task
    pub async fn update_task(&self, project_id: &str, task_id: &str, title: Option<&str>, status: Option<&str>) -> Result<Value, McpClientError> {
        let mut args = json!({
            "project_id": project_id,
            "task_id": task_id
        });
        if let Some(t) = title {
            args["title"] = json!(t);
        }
        if let Some(s) = status {
            args["status"] = json!(s);
        }
        self.call_tool("update_task", args).await
    }

    /// Delete a task
    pub async fn delete_task(&self, project_id: &str, task_id: &str) -> Result<Value, McpClientError> {
        self.call_tool("delete_task", json!({
            "project_id": project_id,
            "task_id": task_id
        })).await
    }

    /// List task attempts
    pub async fn list_task_attempts(&self, task_id: &str) -> Result<Value, McpClientError> {
        self.call_tool("list_task_attempts", json!({ "task_id": task_id })).await
    }

    /// Get a task attempt
    pub async fn get_task_attempt(&self, attempt_id: &str) -> Result<Value, McpClientError> {
        self.call_tool("get_task_attempt", json!({ "attempt_id": attempt_id })).await
    }

    /// Start a task attempt
    pub async fn start_task_attempt(&self, task_id: &str, executor: Option<&str>) -> Result<Value, McpClientError> {
        let mut args = json!({ "task_id": task_id });
        if let Some(e) = executor {
            args["executor"] = json!(e);
        }
        self.call_tool("start_task_attempt", args).await
    }

    // ==================== System Server Tools ====================

    /// Get system health
    pub async fn health_check(&self) -> Result<Value, McpClientError> {
        self.call_tool("health_check", json!({})).await
    }

    /// Get system info
    pub async fn get_system_info(&self) -> Result<Value, McpClientError> {
        self.call_tool("get_system_info", json!({})).await
    }

    /// List executor profiles
    pub async fn list_executor_profiles(&self) -> Result<Value, McpClientError> {
        self.call_tool("list_executor_profiles", json!({})).await
    }

    /// Get config
    pub async fn get_config(&self) -> Result<Value, McpClientError> {
        self.call_tool("get_config", json!({})).await
    }
}

/// Errors that can occur when calling MCP endpoints
#[derive(Debug)]
pub enum McpClientError {
    /// HTTP request failed
    HttpError(String),
    /// Failed to parse response
    ParseError(String),
    /// MCP returned an error
    McpError { code: i32, message: String },
    /// Tool execution failed
    ToolError(String),
    /// Invalid response structure
    InvalidResponse(String),
    /// Server not available
    ServerUnavailable,
}

impl std::fmt::Display for McpClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::HttpError(e) => write!(f, "HTTP error: {}", e),
            Self::ParseError(e) => write!(f, "Parse error: {}", e),
            Self::McpError { code, message } => write!(f, "MCP error {}: {}", code, message),
            Self::ToolError(e) => write!(f, "Tool error: {}", e),
            Self::InvalidResponse(e) => write!(f, "Invalid response: {}", e),
            Self::ServerUnavailable => write!(f, "MCP server unavailable"),
        }
    }
}

impl std::error::Error for McpClientError {}

/// Helper macro to skip test if MCP server is not available
#[macro_export]
macro_rules! skip_if_mcp_unavailable {
    ($client:expr) => {
        if !$client.is_available().await {
            eprintln!("SKIPPED: MCP server not available at {}", stringify!($client));
            return;
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_client_creation() {
        let client = McpClient::task_server();
        assert!(!client.base_url.is_empty());
    }

    #[tokio::test]
    async fn test_task_server_available() {
        let client = McpClient::task_server();
        if client.is_available().await {
            let tools = client.list_tools().await.expect("Failed to list tools");
            assert!(!tools.is_empty(), "Should have at least one tool");
        } else {
            eprintln!("SKIPPED: Task server not available");
        }
    }

    #[tokio::test]
    async fn test_system_server_available() {
        let client = McpClient::system_server();
        if client.is_available().await {
            let tools = client.list_tools().await.expect("Failed to list tools");
            assert!(!tools.is_empty(), "Should have at least one tool");
        } else {
            eprintln!("SKIPPED: System server not available");
        }
    }
}
