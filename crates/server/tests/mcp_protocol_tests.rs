//! MCP Protocol Compliance Tests
//!
//! These tests verify that the MCP servers implement the Model Context Protocol
//! correctly, including JSON-RPC format, error codes, and protocol-specific methods.
//!
//! Reference: https://spec.modelcontextprotocol.io/specification/2024-11-05/
//!
//! To run these tests:
//!   cargo test --package server --test mcp_protocol_tests -- --nocapture

use reqwest::Client;
use serde_json::{json, Value};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use std::collections::HashSet;

static REQUEST_ID: AtomicU64 = AtomicU64::new(1);

fn next_id() -> u64 {
    REQUEST_ID.fetch_add(1, Ordering::SeqCst)
}

fn task_server_url() -> String {
    std::env::var("MCP_TASK_URL").unwrap_or_else(|_| "http://localhost:9717".to_string())
}

fn system_server_url() -> String {
    std::env::var("MCP_SYSTEM_URL").unwrap_or_else(|_| "http://localhost:9718".to_string())
}

async fn mcp_request(base_url: &str, method: &str, params: Value) -> Result<Value, String> {
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| e.to_string())?;

    let request = json!({
        "jsonrpc": "2.0",
        "id": next_id(),
        "method": method,
        "params": params
    });

    let response = client
        .post(&format!("{}/mcp", base_url))
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    response.json().await.map_err(|e| e.to_string())
}

async fn is_server_available(base_url: &str) -> bool {
    mcp_request(base_url, "tools/list", json!({})).await.is_ok()
}

macro_rules! require_server {
    ($url:expr, $name:expr) => {
        if !is_server_available($url).await {
            eprintln!("SKIPPED: {} not available at {}", $name, $url);
            return;
        }
    };
}

// ============================================================================
// JSON-RPC 2.0 Format Tests
// ============================================================================

#[cfg(test)]
mod jsonrpc_format_tests {
    use super::*;

    #[tokio::test]
    async fn test_response_has_jsonrpc_field() {
        let url = task_server_url();
        require_server!(&url, "Task server");

        let response = mcp_request(&url, "tools/list", json!({}))
            .await
            .expect("Request failed");

        assert_eq!(
            response["jsonrpc"].as_str(),
            Some("2.0"),
            "Response must have jsonrpc: '2.0'"
        );
    }

    #[tokio::test]
    async fn test_response_has_matching_id() {
        let url = task_server_url();
        require_server!(&url, "Task server");

        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap();

        let request_id = 12345u64;
        let request = json!({
            "jsonrpc": "2.0",
            "id": request_id,
            "method": "tools/list",
            "params": {}
        });

        let response: Value = client
            .post(&format!("{}/mcp", url))
            .json(&request)
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();

        assert_eq!(
            response["id"].as_u64(),
            Some(request_id),
            "Response id must match request id"
        );
    }

    #[tokio::test]
    async fn test_successful_response_has_result() {
        let url = task_server_url();
        require_server!(&url, "Task server");

        let response = mcp_request(&url, "tools/list", json!({}))
            .await
            .expect("Request failed");

        assert!(
            response.get("result").is_some(),
            "Successful response must have 'result' field"
        );
        assert!(
            response.get("error").is_none(),
            "Successful response must not have 'error' field"
        );
    }

    #[tokio::test]
    async fn test_error_response_has_error() {
        let url = task_server_url();
        require_server!(&url, "Task server");

        let response = mcp_request(&url, "nonexistent_method", json!({}))
            .await
            .expect("Request failed");

        assert!(
            response.get("error").is_some(),
            "Error response must have 'error' field"
        );
        assert!(
            response.get("result").is_none(),
            "Error response must not have 'result' field"
        );
    }

    #[tokio::test]
    async fn test_error_has_code_and_message() {
        let url = task_server_url();
        require_server!(&url, "Task server");

        let response = mcp_request(&url, "nonexistent_method", json!({}))
            .await
            .expect("Request failed");

        let error = response.get("error").expect("Should have error");
        assert!(
            error.get("code").is_some(),
            "Error must have 'code' field"
        );
        assert!(
            error.get("message").is_some(),
            "Error must have 'message' field"
        );
        assert!(
            error["code"].is_i64(),
            "Error code must be an integer"
        );
        assert!(
            error["message"].is_string(),
            "Error message must be a string"
        );
    }
}

// ============================================================================
// MCP Initialize Protocol Tests
// ============================================================================

#[cfg(test)]
mod initialize_tests {
    use super::*;

    #[tokio::test]
    async fn test_initialize_returns_protocol_version() {
        let url = task_server_url();
        require_server!(&url, "Task server");

        let response = mcp_request(
            &url,
            "initialize",
            json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {
                    "name": "test-client",
                    "version": "1.0.0"
                }
            }),
        )
        .await
        .expect("Initialize failed");

        let result = response.get("result").expect("Should have result");
        assert!(
            result.get("protocolVersion").is_some(),
            "Initialize result must have protocolVersion"
        );
    }

    #[tokio::test]
    async fn test_initialize_returns_server_info() {
        let url = task_server_url();
        require_server!(&url, "Task server");

        let response = mcp_request(
            &url,
            "initialize",
            json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {
                    "name": "test-client",
                    "version": "1.0.0"
                }
            }),
        )
        .await
        .expect("Initialize failed");

        let result = response.get("result").expect("Should have result");
        let server_info = result.get("serverInfo").expect("Should have serverInfo");
        
        assert!(
            server_info.get("name").is_some(),
            "serverInfo must have name"
        );
        assert!(
            server_info.get("version").is_some(),
            "serverInfo must have version"
        );
    }

    #[tokio::test]
    async fn test_initialize_returns_capabilities() {
        let url = task_server_url();
        require_server!(&url, "Task server");

        let response = mcp_request(
            &url,
            "initialize",
            json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {
                    "name": "test-client",
                    "version": "1.0.0"
                }
            }),
        )
        .await
        .expect("Initialize failed");

        let result = response.get("result").expect("Should have result");
        assert!(
            result.get("capabilities").is_some(),
            "Initialize result must have capabilities"
        );
    }

    #[tokio::test]
    async fn test_system_server_initialize() {
        let url = system_server_url();
        require_server!(&url, "System server");

        let response = mcp_request(
            &url,
            "initialize",
            json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {
                    "name": "test-client",
                    "version": "1.0.0"
                }
            }),
        )
        .await
        .expect("Initialize failed");

        assert!(
            response.get("result").is_some(),
            "System server should support initialize"
        );
    }
}

// ============================================================================
// Tool Discovery Tests (tools/list)
// ============================================================================

#[cfg(test)]
mod tool_discovery_tests {
    use super::*;

    #[tokio::test]
    async fn test_tools_list_returns_array() {
        let url = task_server_url();
        require_server!(&url, "Task server");

        let response = mcp_request(&url, "tools/list", json!({}))
            .await
            .expect("Request failed");

        let result = response.get("result").expect("Should have result");
        let tools = result.get("tools").expect("Should have tools field");
        
        assert!(tools.is_array(), "tools must be an array");
    }

    #[tokio::test]
    async fn test_tool_has_required_fields() {
        let url = task_server_url();
        require_server!(&url, "Task server");

        let response = mcp_request(&url, "tools/list", json!({}))
            .await
            .expect("Request failed");

        let tools = response["result"]["tools"]
            .as_array()
            .expect("tools should be array");

        assert!(!tools.is_empty(), "Should have at least one tool");

        for tool in tools {
            assert!(
                tool.get("name").is_some(),
                "Tool must have 'name' field: {:?}",
                tool
            );
            assert!(
                tool["name"].is_string(),
                "Tool name must be a string"
            );
            
            // Description is required per MCP spec
            assert!(
                tool.get("description").is_some(),
                "Tool must have 'description' field: {:?}",
                tool
            );
            
            // inputSchema is required per MCP spec
            assert!(
                tool.get("inputSchema").is_some(),
                "Tool must have 'inputSchema' field: {:?}",
                tool
            );
        }
    }

    #[tokio::test]
    async fn test_input_schema_is_valid_json_schema() {
        let url = task_server_url();
        require_server!(&url, "Task server");

        let response = mcp_request(&url, "tools/list", json!({}))
            .await
            .expect("Request failed");

        let tools = response["result"]["tools"]
            .as_array()
            .expect("tools should be array");

        for tool in tools {
            let schema = tool.get("inputSchema").expect("Should have inputSchema");
            
            // JSON Schema must have "type" field
            assert!(
                schema.get("type").is_some(),
                "inputSchema must have 'type' field for tool: {}",
                tool["name"]
            );
            
            // For object types, should have "properties"
            if schema["type"].as_str() == Some("object") {
                assert!(
                    schema.get("properties").is_some(),
                    "Object schema should have 'properties' for tool: {}",
                    tool["name"]
                );
            }
        }
    }

    #[tokio::test]
    async fn test_task_server_has_expected_tools() {
        let url = task_server_url();
        require_server!(&url, "Task server");

        let response = mcp_request(&url, "tools/list", json!({}))
            .await
            .expect("Request failed");

        let tools = response["result"]["tools"]
            .as_array()
            .expect("tools should be array");

        let tool_names: Vec<&str> = tools
            .iter()
            .filter_map(|t| t["name"].as_str())
            .collect();

        // Core task management tools
        let expected = [
            "list_projects",
            "list_tasks",
            "get_task",
            "create_task",
            "update_task",
        ];

        for name in expected {
            assert!(
                tool_names.contains(&name),
                "Task server should have '{}' tool. Found: {:?}",
                name,
                tool_names
            );
        }
    }

    #[tokio::test]
    async fn test_system_server_has_expected_tools() {
        let url = system_server_url();
        require_server!(&url, "System server");

        let response = mcp_request(&url, "tools/list", json!({}))
            .await
            .expect("Request failed");

        let tools = response["result"]["tools"]
            .as_array()
            .expect("tools should be array");

        let tool_names: Vec<&str> = tools
            .iter()
            .filter_map(|t| t["name"].as_str())
            .collect();

        let expected = ["health_check", "get_system_info", "get_config"];

        for name in expected {
            assert!(
                tool_names.contains(&name),
                "System server should have '{}' tool. Found: {:?}",
                name,
                tool_names
            );
        }
    }
}

// ============================================================================
// Tool Invocation Tests (tools/call)
// ============================================================================

#[cfg(test)]
mod tool_invocation_tests {
    use super::*;

    #[tokio::test]
    async fn test_tools_call_returns_content_array() {
        let url = task_server_url();
        require_server!(&url, "Task server");

        let response = mcp_request(
            &url,
            "tools/call",
            json!({
                "name": "list_projects",
                "arguments": {}
            }),
        )
        .await
        .expect("Request failed");

        let result = response.get("result").expect("Should have result");
        let content = result.get("content").expect("Should have content");
        
        assert!(content.is_array(), "content must be an array");
    }

    #[tokio::test]
    async fn test_content_item_has_type_and_text() {
        let url = task_server_url();
        require_server!(&url, "Task server");

        let response = mcp_request(
            &url,
            "tools/call",
            json!({
                "name": "list_projects",
                "arguments": {}
            }),
        )
        .await
        .expect("Request failed");

        let content = response["result"]["content"]
            .as_array()
            .expect("content should be array");

        assert!(!content.is_empty(), "Should have at least one content item");

        for item in content {
            assert!(
                item.get("type").is_some(),
                "Content item must have 'type' field"
            );
            assert!(
                item.get("text").is_some(),
                "Content item must have 'text' field (for text type)"
            );
        }
    }

    #[tokio::test]
    async fn test_tools_call_has_is_error_field() {
        let url = task_server_url();
        require_server!(&url, "Task server");

        let response = mcp_request(
            &url,
            "tools/call",
            json!({
                "name": "list_projects",
                "arguments": {}
            }),
        )
        .await
        .expect("Request failed");

        let result = response.get("result").expect("Should have result");
        
        // isError should be present (MCP spec)
        assert!(
            result.get("isError").is_some(),
            "Result should have 'isError' field"
        );
    }

    #[tokio::test]
    async fn test_successful_call_has_is_error_false() {
        let url = task_server_url();
        require_server!(&url, "Task server");

        let response = mcp_request(
            &url,
            "tools/call",
            json!({
                "name": "list_projects",
                "arguments": {}
            }),
        )
        .await
        .expect("Request failed");

        assert_eq!(
            response["result"]["isError"].as_bool(),
            Some(false),
            "Successful tool call should have isError: false"
        );
    }

    #[tokio::test]
    async fn test_content_text_is_valid_json() {
        let url = task_server_url();
        require_server!(&url, "Task server");

        let response = mcp_request(
            &url,
            "tools/call",
            json!({
                "name": "list_projects",
                "arguments": {}
            }),
        )
        .await
        .expect("Request failed");

        let text = response["result"]["content"][0]["text"]
            .as_str()
            .expect("Should have text");

        let parsed: Result<Value, _> = serde_json::from_str(text);
        assert!(
            parsed.is_ok(),
            "Content text should be valid JSON: {}",
            text
        );
    }
}

// ============================================================================
// MCP Error Code Tests
// ============================================================================

#[cfg(test)]
mod error_code_tests {
    use super::*;

    // Standard JSON-RPC error codes
    const METHOD_NOT_FOUND: i64 = -32601;
    const INVALID_PARAMS: i64 = -32602;
    
    // MCP-specific error codes (per spec)
    const RESOURCE_NOT_FOUND: i64 = -32004;

    #[tokio::test]
    async fn test_method_not_found_error_code() {
        let url = task_server_url();
        require_server!(&url, "Task server");

        let response = mcp_request(&url, "nonexistent/method", json!({}))
            .await
            .expect("Request failed");

        let error = response.get("error").expect("Should have error");
        assert_eq!(
            error["code"].as_i64(),
            Some(METHOD_NOT_FOUND),
            "Unknown method should return -32601 (Method not found)"
        );
    }

    #[tokio::test]
    async fn test_tool_not_found_error_code() {
        let url = task_server_url();
        require_server!(&url, "Task server");

        let response = mcp_request(
            &url,
            "tools/call",
            json!({
                "name": "nonexistent_tool",
                "arguments": {}
            }),
        )
        .await
        .expect("Request failed");

        let error = response.get("error").expect("Should have error");
        
        // Should be resource not found (-32004) or similar
        let code = error["code"].as_i64().expect("Should have code");
        assert!(
            code == RESOURCE_NOT_FOUND || code < 0,
            "Unknown tool should return a negative error code, got: {}",
            code
        );
    }

    #[tokio::test]
    async fn test_invalid_uuid_error() {
        let url = task_server_url();
        require_server!(&url, "Task server");

        let response = mcp_request(
            &url,
            "tools/call",
            json!({
                "name": "get_task",
                "arguments": {
                    "task_id": "not-a-valid-uuid"
                }
            }),
        )
        .await
        .expect("Request failed");

        // Should return error (either in error field or isError: true)
        let has_error = response.get("error").is_some()
            || response["result"]["isError"].as_bool() == Some(true);
        
        assert!(has_error, "Invalid UUID should return an error");
    }

    #[tokio::test]
    async fn test_error_message_is_descriptive() {
        let url = task_server_url();
        require_server!(&url, "Task server");

        let response = mcp_request(
            &url,
            "tools/call",
            json!({
                "name": "nonexistent_tool",
                "arguments": {}
            }),
        )
        .await
        .expect("Request failed");

        let error = response.get("error").expect("Should have error");
        let message = error["message"].as_str().expect("Should have message");
        
        assert!(
            message.len() > 10,
            "Error message should be descriptive, got: {}",
            message
        );
        assert!(
            message.to_lowercase().contains("not found") || message.to_lowercase().contains("nonexistent"),
            "Error message should indicate the problem: {}",
            message
        );
    }
}

// ============================================================================
// Concurrent Request Tests
// ============================================================================

#[cfg(test)]
mod concurrent_tests {
    use super::*;

    #[tokio::test]
    async fn test_concurrent_tools_list() {
        let url = task_server_url();
        require_server!(&url, "Task server");

        let mut handles = Vec::new();
        for i in 0..10 {
            let url_clone = url.clone();
            handles.push(tokio::spawn(async move {
                (i, mcp_request(&url_clone, "tools/list", json!({})).await)
            }));
        }

        for handle in handles {
            let (i, result) = handle.await.expect("Task panicked");
            assert!(
                result.is_ok(),
                "Concurrent request {} failed: {:?}",
                i,
                result
            );
            let response = result.unwrap();
            assert!(
                response.get("result").is_some(),
                "Request {} should have result",
                i
            );
        }
    }

    #[tokio::test]
    async fn test_concurrent_tool_calls() {
        let url = task_server_url();
        require_server!(&url, "Task server");

        let mut handles = Vec::new();
        for i in 0..5 {
            let url_clone = url.clone();
            handles.push(tokio::spawn(async move {
                (i, mcp_request(
                    &url_clone,
                    "tools/call",
                    json!({
                        "name": "list_projects",
                        "arguments": {}
                    }),
                ).await)
            }));
        }

        for handle in handles {
            let (i, result) = handle.await.expect("Task panicked");
            assert!(
                result.is_ok(),
                "Concurrent tool call {} failed: {:?}",
                i,
                result
            );
        }
    }

    #[tokio::test]
    async fn test_mixed_concurrent_requests() {
        let url = task_server_url();
        require_server!(&url, "Task server");

        let url1 = url.clone();
        let url2 = url.clone();
        let url3 = url.clone();
        let url4 = url.clone();

        let handles = vec![
            tokio::spawn(async move {
                mcp_request(&url1, "tools/list", json!({})).await
            }),
            tokio::spawn(async move {
                mcp_request(&url2, "initialize", json!({
                    "protocolVersion": "2024-11-05",
                    "capabilities": {},
                    "clientInfo": {"name": "test", "version": "1.0"}
                })).await
            }),
            tokio::spawn(async move {
                mcp_request(&url3, "tools/call", json!({
                    "name": "list_projects",
                    "arguments": {}
                })).await
            }),
            tokio::spawn(async move {
                mcp_request(&url4, "tools/list", json!({})).await
            }),
        ];

        for (i, handle) in handles.into_iter().enumerate() {
            let result = handle.await.expect("Task panicked");
            assert!(
                result.is_ok(),
                "Mixed concurrent request {} failed: {:?}",
                i,
                result
            );
        }
    }
}

// ============================================================================
// Both Servers Comparison Tests
// ============================================================================

#[cfg(test)]
mod cross_server_tests {
    use super::*;

    #[tokio::test]
    async fn test_both_servers_use_same_protocol_version() {
        let task_url = task_server_url();
        let system_url = system_server_url();
        
        if !is_server_available(&task_url).await || !is_server_available(&system_url).await {
            eprintln!("SKIPPED: Both servers not available");
            return;
        }

        let init_params = json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {"name": "test", "version": "1.0"}
        });

        let task_response = mcp_request(&task_url, "initialize", init_params.clone())
            .await
            .expect("Task server init failed");
        let system_response = mcp_request(&system_url, "initialize", init_params)
            .await
            .expect("System server init failed");

        let task_version = task_response["result"]["protocolVersion"].as_str();
        let system_version = system_response["result"]["protocolVersion"].as_str();

        assert_eq!(
            task_version, system_version,
            "Both servers should use same protocol version"
        );
    }

    #[tokio::test]
    async fn test_both_servers_return_consistent_jsonrpc() {
        let task_url = task_server_url();
        let system_url = system_server_url();
        
        if !is_server_available(&task_url).await || !is_server_available(&system_url).await {
            eprintln!("SKIPPED: Both servers not available");
            return;
        }

        let task_response = mcp_request(&task_url, "tools/list", json!({}))
            .await
            .expect("Task server failed");
        let system_response = mcp_request(&system_url, "tools/list", json!({}))
            .await
            .expect("System server failed");

        assert_eq!(
            task_response["jsonrpc"].as_str(),
            Some("2.0"),
            "Task server should return jsonrpc 2.0"
        );
        assert_eq!(
            system_response["jsonrpc"].as_str(),
            Some("2.0"),
            "System server should return jsonrpc 2.0"
        );
    }

    #[tokio::test]
    async fn test_servers_have_no_tool_name_overlap() {
        let task_url = task_server_url();
        let system_url = system_server_url();
        
        if !is_server_available(&task_url).await || !is_server_available(&system_url).await {
            eprintln!("SKIPPED: Both servers not available");
            return;
        }

        let task_response = mcp_request(&task_url, "tools/list", json!({}))
            .await
            .expect("Task server failed");
        let system_response = mcp_request(&system_url, "tools/list", json!({}))
            .await
            .expect("System server failed");

        let empty_vec = vec![];
        let task_tools_array = task_response["result"]["tools"]
            .as_array()
            .unwrap_or(&empty_vec);
        let system_tools_array = system_response["result"]["tools"]
            .as_array()
            .unwrap_or(&empty_vec);

        let task_tools: HashSet<&str> = task_tools_array
            .iter()
            .filter_map(|t| t["name"].as_str())
            .collect();

        let system_tools: HashSet<&str> = system_tools_array
            .iter()
            .filter_map(|t| t["name"].as_str())
            .collect();

        let overlap: Vec<&&str> = task_tools.intersection(&system_tools).collect();
        
        assert!(
            overlap.is_empty(),
            "Servers should not have overlapping tool names: {:?}",
            overlap
        );
    }
}
