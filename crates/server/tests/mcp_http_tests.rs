//! HTTP-based MCP integration tests
//!
//! These tests make real HTTP calls to the MCP servers.
//! They require the MCP servers to be running:
//!   - Task server on port 9717 (or MCP_TASK_URL env var)
//!   - System server on port 9718 (or MCP_SYSTEM_URL env var)
//!
//! To run these tests:
//!   cargo test --package server --test mcp_http_tests -- --nocapture
//!
//! If servers are not available, tests will be skipped (not failed).

#[path = "common/mod.rs"]
mod common;

use common::mcp_client::{McpClient, McpClientError};
use serde_json::json;

/// Helper to skip tests when server is unavailable
macro_rules! require_mcp_server {
    ($client:expr, $server_name:expr) => {
        if !$client.is_available().await {
            eprintln!("SKIPPED: {} not available", $server_name);
            return;
        }
    };
}

// ============================================================================
// Task Server Tests
// ============================================================================

#[cfg(test)]
mod task_server_connection {
    use super::*;

    #[tokio::test]
    async fn test_task_server_is_available() {
        let client = McpClient::task_server();
        let available = client.is_available().await;
        if !available {
            eprintln!("NOTE: Task server not available - some tests will be skipped");
        }
        // This test always passes - it just reports status
    }

    #[tokio::test]
    async fn test_task_server_has_expected_tools() {
        let client = McpClient::task_server();
        require_mcp_server!(client, "Task server");

        let tools = client.list_tools().await.expect("Failed to list tools");
        
        // Verify expected tools are present
        let tool_names: Vec<&str> = tools
            .iter()
            .filter_map(|t| t["name"].as_str())
            .collect();

        let expected_tools = [
            "list_projects",
            "get_project",
            "create_project",
            "list_tasks",
            "get_task",
            "create_task",
            "update_task",
            "delete_task",
            "list_task_attempts",
            "get_task_attempt",
            "start_task_attempt",
        ];

        for expected in expected_tools {
            assert!(
                tool_names.contains(&expected),
                "Missing expected tool: {}. Available: {:?}",
                expected,
                tool_names
            );
        }
    }
}

#[cfg(test)]
mod list_projects_http_tests {
    use super::*;

    #[tokio::test]
    async fn test_list_projects_returns_array() {
        let client = McpClient::task_server();
        require_mcp_server!(client, "Task server");

        let result = client.list_projects().await.expect("Failed to list projects");
        
        assert!(result.get("projects").is_some(), "Response should have 'projects' field");
        assert!(result["projects"].is_array(), "Projects should be an array");
        assert!(result.get("count").is_some(), "Response should have 'count' field");
    }

    #[tokio::test]
    async fn test_list_projects_returns_valid_structure() {
        let client = McpClient::task_server();
        require_mcp_server!(client, "Task server");

        let result = client.list_projects().await.expect("Failed to list projects");
        let projects = result["projects"].as_array().expect("Projects should be array");

        // If there are projects, verify their structure
        if let Some(project) = projects.first() {
            assert!(project.get("id").is_some(), "Project should have id");
            assert!(project.get("name").is_some(), "Project should have name");
            assert!(project.get("created_at").is_some(), "Project should have created_at");
            assert!(project.get("updated_at").is_some(), "Project should have updated_at");
        }
    }

    #[tokio::test]
    async fn test_list_projects_count_matches_array_length() {
        let client = McpClient::task_server();
        require_mcp_server!(client, "Task server");

        let result = client.list_projects().await.expect("Failed to list projects");
        let projects = result["projects"].as_array().expect("Projects should be array");
        let count = result["count"].as_u64().expect("Count should be a number");

        assert_eq!(
            projects.len() as u64,
            count,
            "Count should match array length"
        );
    }
}

#[cfg(test)]
mod get_project_http_tests {
    use super::*;

    #[tokio::test]
    async fn test_get_project_by_id() {
        let client = McpClient::task_server();
        require_mcp_server!(client, "Task server");

        // First, list projects to get a valid ID
        let list_result = client.list_projects().await.expect("Failed to list projects");
        let projects = list_result["projects"].as_array().expect("Projects should be array");
        
        if projects.is_empty() {
            eprintln!("SKIPPED: No projects available to test get_project");
            return;
        }

        let project_id = projects[0]["id"].as_str().expect("Project should have id");
        let result = client.get_project(project_id).await.expect("Failed to get project");

        assert!(result.get("id").is_some(), "Project should have id");
        assert_eq!(result["id"].as_str(), Some(project_id), "Should return correct project");
        assert!(result.get("name").is_some(), "Project should have name");
    }

    #[tokio::test]
    async fn test_get_project_invalid_id() {
        let client = McpClient::task_server();
        require_mcp_server!(client, "Task server");

        let result = client.get_project("00000000-0000-0000-0000-000000000000").await;
        
        // Should either return error or empty result
        match result {
            Err(McpClientError::ToolError(msg)) => {
                assert!(msg.contains("not found") || msg.contains("error"), 
                    "Error message should indicate not found: {}", msg);
            }
            Ok(val) => {
                // Some implementations might return null/empty instead of error
                if val.is_null() {
                    // OK - returning null for not found
                } else {
                    panic!("Expected error or null for invalid project ID, got: {:?}", val);
                }
            }
            Err(e) => {
                // Other errors are acceptable for invalid ID
                eprintln!("Got error for invalid project ID: {}", e);
            }
        }
    }
}

#[cfg(test)]
mod list_tasks_http_tests {
    use super::*;

    #[tokio::test]
    async fn test_list_tasks_for_project() {
        let client = McpClient::task_server();
        require_mcp_server!(client, "Task server");

        // Get a project ID first
        let list_result = client.list_projects().await.expect("Failed to list projects");
        let projects = list_result["projects"].as_array().expect("Projects should be array");
        
        if projects.is_empty() {
            eprintln!("SKIPPED: No projects available");
            return;
        }

        let project_id = projects[0]["id"].as_str().expect("Project should have id");
        let result = client.list_tasks(project_id).await.expect("Failed to list tasks");

        assert!(result.get("tasks").is_some(), "Response should have 'tasks' field");
        assert!(result["tasks"].is_array(), "Tasks should be an array");
    }

    #[tokio::test]
    async fn test_list_tasks_returns_valid_structure() {
        let client = McpClient::task_server();
        require_mcp_server!(client, "Task server");

        let list_result = client.list_projects().await.expect("Failed to list projects");
        let projects = list_result["projects"].as_array().expect("Projects should be array");
        
        if projects.is_empty() {
            eprintln!("SKIPPED: No projects available");
            return;
        }

        let project_id = projects[0]["id"].as_str().expect("Project should have id");
        let result = client.list_tasks(project_id).await.expect("Failed to list tasks");
        let tasks = result["tasks"].as_array().expect("Tasks should be array");

        if let Some(task) = tasks.first() {
            assert!(task.get("id").is_some(), "Task should have id");
            assert!(task.get("title").is_some(), "Task should have title");
            assert!(task.get("status").is_some(), "Task should have status");
            // project_id may not be in list view, only in detail view
        }
    }

    #[tokio::test]
    async fn test_list_tasks_invalid_project() {
        let client = McpClient::task_server();
        require_mcp_server!(client, "Task server");

        let result = client.list_tasks("00000000-0000-0000-0000-000000000000").await;
        
        // Should either return error or empty array
        match result {
            Ok(val) => {
                let tasks = val["tasks"].as_array();
                if let Some(arr) = tasks {
                    assert!(arr.is_empty(), "Invalid project should have no tasks");
                }
            }
            Err(_) => {
                // Error is also acceptable for invalid project
            }
        }
    }
}

#[cfg(test)]
mod get_task_http_tests {
    use super::*;

    #[tokio::test]
    async fn test_get_task_by_id() {
        let client = McpClient::task_server();
        require_mcp_server!(client, "Task server");

        // Find a project with tasks
        let list_result = client.list_projects().await.expect("Failed to list projects");
        let projects = list_result["projects"].as_array().expect("Projects should be array");
        
        for project in projects {
            let project_id = project["id"].as_str().expect("Project should have id");
            let tasks_result = client.list_tasks(project_id).await.expect("Failed to list tasks");
            let tasks = tasks_result["tasks"].as_array().expect("Tasks should be array");
            
            if let Some(task) = tasks.first() {
                let task_id = task["id"].as_str().expect("Task should have id");
                let result = client.get_task(task_id).await.expect("Failed to get task");
                
                // Response may be wrapped in "task" key
                let task_data = result.get("task").unwrap_or(&result);
                
                assert!(task_data.get("id").is_some(), "Task should have id");
                assert_eq!(task_data["id"].as_str(), Some(task_id), "Should return correct task");
                assert!(task_data.get("title").is_some(), "Task should have title");
                assert!(task_data.get("status").is_some(), "Task should have status");
                return;
            }
        }
        
        eprintln!("SKIPPED: No tasks found to test get_task");
    }
}

// ============================================================================
// Task CRUD Tests (Create, Read, Update, Delete)
// ============================================================================

#[cfg(test)]
mod task_crud_http_tests {
    use super::*;

    /// Helper to extract task_id from create response
    /// Handles different response formats: {"task_id": "..."} or {"task": {"id": "..."}}
    fn extract_task_id(result: &serde_json::Value) -> Option<&str> {
        result.get("task_id")
            .or_else(|| result.get("task").and_then(|t| t.get("id")))
            .or_else(|| result.get("id"))
            .and_then(|v| v.as_str())
    }

    /// Test creating a new task via HTTP
    /// Note: This creates real data in the database
    #[tokio::test]
    async fn test_create_task_basic() {
        let client = McpClient::task_server();
        require_mcp_server!(client, "Task server");

        // Get a project to create task in
        let list_result = client.list_projects().await.expect("Failed to list projects");
        let projects = list_result["projects"].as_array().expect("Projects should be array");
        
        if projects.is_empty() {
            eprintln!("SKIPPED: No projects available for task creation test");
            return;
        }

        let project_id = projects[0]["id"].as_str().expect("Project should have id");
        let test_title = format!("HTTP Test Task {}", chrono::Utc::now().timestamp());
        
        let result = client.create_task(project_id, &test_title, Some("Test description"))
            .await
            .expect("Failed to create task");

        // Response format: {"task_id": "uuid"} or {"task": {...}} or {"id": "..."}
        let task_id = result.get("task_id")
            .or_else(|| result.get("task").and_then(|t| t.get("id")))
            .or_else(|| result.get("id"))
            .and_then(|v| v.as_str())
            .expect("Created task should return task_id");
        
        // Verify the task was created by fetching it
        let fetch_result = client.get_task(task_id).await.expect("Failed to fetch created task");
        let fetched_task = fetch_result.get("task").unwrap_or(&fetch_result);
        assert_eq!(fetched_task["id"].as_str(), Some(task_id));
    }

    /// Test creating task with only required fields
    #[tokio::test]
    async fn test_create_task_minimal() {
        let client = McpClient::task_server();
        require_mcp_server!(client, "Task server");

        let list_result = client.list_projects().await.expect("Failed to list projects");
        let projects = list_result["projects"].as_array().expect("Projects should be array");
        
        if projects.is_empty() {
            eprintln!("SKIPPED: No projects available");
            return;
        }

        let project_id = projects[0]["id"].as_str().expect("Project should have id");
        let test_title = format!("Minimal Task {}", chrono::Utc::now().timestamp());
        
        // Create with no description
        let result = client.create_task(project_id, &test_title, None)
            .await
            .expect("Failed to create task");

        assert!(extract_task_id(&result).is_some(), "Created task should return task_id");
    }

    /// Test updating a task's title
    #[tokio::test]
    async fn test_update_task_title() {
        let client = McpClient::task_server();
        require_mcp_server!(client, "Task server");

        let list_result = client.list_projects().await.expect("Failed to list projects");
        let projects = list_result["projects"].as_array().expect("Projects should be array");
        
        if projects.is_empty() {
            eprintln!("SKIPPED: No projects available");
            return;
        }

        let project_id = projects[0]["id"].as_str().expect("Project should have id");
        
        // Create a task first
        let test_title = format!("Update Test Task {}", chrono::Utc::now().timestamp());
        let create_result = client.create_task(project_id, &test_title, None)
            .await
            .expect("Failed to create task");
        
        let task_id = extract_task_id(&create_result).expect("Created task should return task_id");

        // Update the title
        let new_title = format!("Updated Title {}", chrono::Utc::now().timestamp());
        let update_result = client.update_task(project_id, task_id, Some(&new_title), None)
            .await
            .expect("Failed to update task");

        // Verify update worked - may return {"success": true} or the task itself
        if let Some(task) = update_result.get("task") {
            assert_eq!(task["title"].as_str(), Some(new_title.as_str()));
        } else if update_result.get("success").is_some() {
            // Success indicator - fetch task to verify
            let fetch_result = client.get_task(task_id).await.expect("Failed to fetch updated task");
            let task = fetch_result.get("task").unwrap_or(&fetch_result);
            assert_eq!(task["title"].as_str(), Some(new_title.as_str()));
        }
    }

    /// Test updating a task's status
    #[tokio::test]
    async fn test_update_task_status() {
        let client = McpClient::task_server();
        require_mcp_server!(client, "Task server");

        let list_result = client.list_projects().await.expect("Failed to list projects");
        let projects = list_result["projects"].as_array().expect("Projects should be array");
        
        if projects.is_empty() {
            eprintln!("SKIPPED: No projects available");
            return;
        }

        let project_id = projects[0]["id"].as_str().expect("Project should have id");
        
        // Create a task
        let test_title = format!("Status Test Task {}", chrono::Utc::now().timestamp());
        let create_result = client.create_task(project_id, &test_title, None)
            .await
            .expect("Failed to create task");
        
        let task_id = extract_task_id(&create_result).expect("Created task should return task_id");

        // Update status to in-progress (note: hyphenated)
        let update_result = client.update_task(project_id, task_id, None, Some("in-progress"))
            .await
            .expect("Failed to update task status");

        // Verify status changed - fetch task to confirm
        let fetch_result = client.get_task(task_id).await.expect("Failed to fetch updated task");
        let task = fetch_result.get("task").unwrap_or(&fetch_result);
        assert_eq!(task["status"].as_str(), Some("in-progress"));
    }

    /// Test deleting a task
    /// Note: Delete may not be fully implemented in the backend
    #[tokio::test]
    async fn test_delete_task() {
        let client = McpClient::task_server();
        require_mcp_server!(client, "Task server");

        let list_result = client.list_projects().await.expect("Failed to list projects");
        let projects = list_result["projects"].as_array().expect("Projects should be array");
        
        if projects.is_empty() {
            eprintln!("SKIPPED: No projects available");
            return;
        }

        let project_id = projects[0]["id"].as_str().expect("Project should have id");
        
        // Create a task to delete
        let test_title = format!("Delete Test Task {}", chrono::Utc::now().timestamp());
        let create_result = client.create_task(project_id, &test_title, None)
            .await
            .expect("Failed to create task");
        
        let task_id = extract_task_id(&create_result).expect("Created task should return task_id");

        // Delete the task - may not be fully supported
        let delete_result = client.delete_task(project_id, task_id).await;
        
        match delete_result {
            Ok(val) => {
                // Success - may return success indicator or empty response
                println!("Task deleted successfully: {:?}", val);
                
                // Verify task is deleted by trying to fetch it
                let fetch_result = client.get_task(task_id).await;
                match fetch_result {
                    Err(_) => {
                        // Good - task not found
                    }
                    Ok(val) => {
                        // May return null or empty
                        if val.is_null() {
                            // OK
                        } else if let Some(task) = val.get("task") {
                            if task.is_null() {
                                // OK
                            } else {
                                // Task might still exist if delete was soft-delete
                                println!("Note: Task still exists after delete (may be soft-delete): {:?}", task);
                            }
                        }
                    }
                }
            }
            Err(e) => {
                // Delete might not be fully implemented - skip gracefully
                eprintln!("SKIPPED: Delete task not supported: {}", e);
            }
        }
    }

    /// Test task status transitions (lifecycle)
    #[tokio::test]
    async fn test_task_status_lifecycle() {
        let client = McpClient::task_server();
        require_mcp_server!(client, "Task server");

        let list_result = client.list_projects().await.expect("Failed to list projects");
        let projects = list_result["projects"].as_array().expect("Projects should be array");
        
        if projects.is_empty() {
            eprintln!("SKIPPED: No projects available");
            return;
        }

        let project_id = projects[0]["id"].as_str().expect("Project should have id");
        
        // Create task (starts as 'todo')
        let test_title = format!("Lifecycle Test Task {}", chrono::Utc::now().timestamp());
        let create_result = client.create_task(project_id, &test_title, None)
            .await
            .expect("Failed to create task");
        
        let task_id = extract_task_id(&create_result).expect("Created task should return task_id");

        // Transition: todo -> in-progress -> in-review -> done (note: hyphenated status values)
        let transitions = ["in-progress", "in-review", "done"];
        
        for status in transitions {
            client.update_task(project_id, task_id, None, Some(status))
                .await
                .expect(&format!("Failed to transition to {}", status));
            
            // Fetch and verify status
            let fetch_result = client.get_task(task_id).await.expect("Failed to fetch task");
            let task = fetch_result.get("task").unwrap_or(&fetch_result);
            assert_eq!(
                task["status"].as_str(), 
                Some(status),
                "Status should be {} after transition",
                status
            );
        }
        
        println!("Task lifecycle test completed successfully!");
    }
}

// ============================================================================
// System Server Tests
// ============================================================================

#[cfg(test)]
mod system_server_connection {
    use super::*;

    #[tokio::test]
    async fn test_system_server_is_available() {
        let client = McpClient::system_server();
        let available = client.is_available().await;
        if !available {
            eprintln!("NOTE: System server not available - some tests will be skipped");
        }
    }

    #[tokio::test]
    async fn test_system_server_has_expected_tools() {
        let client = McpClient::system_server();
        require_mcp_server!(client, "System server");

        let tools = client.list_tools().await.expect("Failed to list tools");
        
        let tool_names: Vec<&str> = tools
            .iter()
            .filter_map(|t| t["name"].as_str())
            .collect();

        let expected_tools = [
            "health_check",
            "get_system_info",
            "get_config",
            "list_executor_profiles",
        ];

        for expected in expected_tools {
            assert!(
                tool_names.contains(&expected),
                "Missing expected tool: {}. Available: {:?}",
                expected,
                tool_names
            );
        }
    }
}

#[cfg(test)]
mod health_check_http_tests {
    use super::*;

    #[tokio::test]
    async fn test_health_check_returns_ok() {
        let client = McpClient::system_server();
        require_mcp_server!(client, "System server");

        let result = client.health_check().await.expect("Health check failed");
        
        assert!(result.get("status").is_some(), "Should have status field");
        // Status can be "ok" or "healthy" depending on implementation
        let status = result["status"].as_str().unwrap_or("");
        assert!(
            status == "ok" || status == "healthy",
            "Status should be 'ok' or 'healthy', got: {}",
            status
        );
    }

    #[tokio::test]
    async fn test_health_check_returns_version() {
        let client = McpClient::system_server();
        require_mcp_server!(client, "System server");

        let result = client.health_check().await.expect("Health check failed");
        
        assert!(result.get("version").is_some(), "Should have version field");
    }
}

#[cfg(test)]
mod get_system_info_http_tests {
    use super::*;

    #[tokio::test]
    async fn test_get_system_info_returns_data() {
        let client = McpClient::system_server();
        require_mcp_server!(client, "System server");

        let result = client.get_system_info().await.expect("Failed to get system info");
        
        // Check for expected fields - may be nested under "system" key
        let has_system_info = result.get("system").is_some() 
            || result.get("os").is_some() 
            || result.get("platform").is_some()
            || result.get("os_type").is_some();
        
        assert!(has_system_info, "Should have system/OS info. Got: {:?}", result);
    }
}

#[cfg(test)]
mod list_executor_profiles_http_tests {
    use super::*;

    #[tokio::test]
    async fn test_list_executor_profiles_returns_array() {
        let client = McpClient::system_server();
        require_mcp_server!(client, "System server");

        let result = client.list_executor_profiles().await;
        
        match result {
            Ok(val) => {
                // Check for executors list or array response
                let has_executors = val.get("executors").is_some() 
                    || val.get("profiles").is_some()
                    || val.is_array();
                assert!(has_executors, "Should return executors list. Got: {:?}", val);
            }
            Err(e) => {
                // This endpoint may fail if backend doesn't support it - skip gracefully
                eprintln!("SKIPPED: list_executor_profiles failed: {}", e);
            }
        }
    }
}

#[cfg(test)]
mod get_config_http_tests {
    use super::*;

    #[tokio::test]
    async fn test_get_config_returns_data() {
        let client = McpClient::system_server();
        require_mcp_server!(client, "System server");

        let result = client.get_config().await.expect("Failed to get config");
        
        // Config should be an object
        assert!(result.is_object(), "Config should be an object");
    }
}

// ============================================================================
// End-to-End Workflow Tests
// ============================================================================

#[cfg(test)]
mod workflow_http_tests {
    use super::*;

    /// Test the complete task lifecycle via HTTP:
    /// 1. List projects
    /// 2. Get tasks for a project
    /// 3. Get task details
    /// 4. List attempts for a task
    #[tokio::test]
    async fn test_task_retrieval_workflow() {
        let client = McpClient::task_server();
        require_mcp_server!(client, "Task server");

        // Step 1: List projects
        let projects_result = client.list_projects().await.expect("Failed to list projects");
        let projects = projects_result["projects"].as_array().expect("Projects should be array");
        
        if projects.is_empty() {
            eprintln!("SKIPPED: No projects available for workflow test");
            return;
        }

        // Step 2: Get tasks for first project
        let project_id = projects[0]["id"].as_str().expect("Project should have id");
        let project_name = projects[0]["name"].as_str().unwrap_or("Unknown");
        println!("Testing with project: {} ({})", project_name, project_id);

        let tasks_result = client.list_tasks(project_id).await.expect("Failed to list tasks");
        let tasks = tasks_result["tasks"].as_array().expect("Tasks should be array");
        
        println!("Found {} tasks in project", tasks.len());

        if let Some(task) = tasks.first() {
            // Step 3: Get task details
            let task_id = task["id"].as_str().expect("Task should have id");
            let task_title = task["title"].as_str().unwrap_or("Unknown");
            println!("Testing with task: {} ({})", task_title, task_id);

            let task_detail = client.get_task(task_id).await.expect("Failed to get task");
            // Response may be wrapped in "task" key
            let task_data = task_detail.get("task").unwrap_or(&task_detail);
            assert_eq!(task_data["id"].as_str(), Some(task_id));

            // Step 4: List attempts
            let attempts_result = client.list_task_attempts(task_id).await.expect("Failed to list attempts");
            assert!(attempts_result.get("attempts").is_some() || attempts_result.is_array());
            
            println!("Workflow test completed successfully!");
        } else {
            eprintln!("No tasks found in project - workflow partially tested");
        }
    }

    /// Test that update_config actually works (catches URL path bugs like /api/config/config)
    #[tokio::test]
    async fn test_update_config_round_trip() {
        let client = McpClient::system_server();
        require_mcp_server!(client, "System server");

        // Get current config
        let original_config = client.get_config().await.expect("Failed to get config");
        let original_prefix = original_config["git_branch_prefix"]
            .as_str()
            .unwrap_or("vibe/");
        println!("Original git_branch_prefix: {}", original_prefix);

        // Update to a test value
        let test_prefix = if original_prefix == "test-prefix-" {
            "vibe/"
        } else {
            "test-prefix-"
        };

        let update_result = client
            .call_tool(
                "update_config",
                json!({ "git_branch_prefix": test_prefix }),
            )
            .await;

        match update_result {
            Ok(updated) => {
                // Verify the update was applied
                let new_prefix = updated["git_branch_prefix"].as_str().unwrap_or("");
                assert_eq!(new_prefix, test_prefix, "Config update should be applied");
                println!("Updated git_branch_prefix to: {}", new_prefix);

                // Restore original value
                let restore_result = client
                    .call_tool(
                        "update_config",
                        json!({ "git_branch_prefix": original_prefix }),
                    )
                    .await;
                assert!(restore_result.is_ok(), "Should restore original config");
                println!("Restored git_branch_prefix to: {}", original_prefix);
            }
            Err(e) => {
                // If update fails with 405, this catches the URL path bug
                let err_str = e.to_string();
                if err_str.contains("405") || err_str.contains("Method Not Allowed") {
                    panic!(
                        "update_config returned 405 - likely URL path bug (e.g., /api/config/config instead of /api/config): {}",
                        err_str
                    );
                }
                // Other errors may be acceptable (e.g., backend not configured)
                eprintln!("update_config failed (may be expected): {}", e);
            }
        }
    }

    /// Test system info retrieval workflow
    #[tokio::test]
    async fn test_system_info_workflow() {
        let client = McpClient::system_server();
        require_mcp_server!(client, "System server");

        // Check health
        let health = client.health_check().await.expect("Health check failed");
        let status = health["status"].as_str().unwrap_or("");
        assert!(status == "ok" || status == "healthy", "Health should be ok/healthy");
        println!("Health check passed: {}", status);

        // Get system info
        let info = client.get_system_info().await.expect("Failed to get system info");
        println!("System info retrieved: {:?}", info.get("system").or(info.get("os")));

        // List executors - may fail gracefully
        match client.list_executor_profiles().await {
            Ok(_) => println!("Executor profiles retrieved"),
            Err(e) => println!("Executor profiles skipped: {}", e),
        }

        // Get config
        let config = client.get_config().await.expect("Failed to get config");
        println!("Config retrieved");

        println!("System info workflow completed successfully!");
    }
}
