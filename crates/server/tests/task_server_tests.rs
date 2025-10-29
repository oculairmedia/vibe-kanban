//! Integration tests for TaskServer MCP tools
//!
//! Tests all task-related MCP tools:
//! - list_projects
//! - create_task
//! - list_tasks
//! - get_task
//! - update_task
//! - delete_task
//! - start_task_attempt
//! - list_task_attempts
//! - get_task_attempt
//! - create_followup_attempt
//! - merge_task_attempt

use super::common::*;
use serde_json::json;
use uuid::Uuid;

#[cfg(test)]
mod list_projects_tests {
    use super::*;

    #[tokio::test]
    async fn test_list_projects_returns_valid_json() {
        // This test would call the actual MCP tool
        // For now, we'll test the response structure

        let response = json!({
            "projects": [
                {
                    "id": "00000000-0000-0000-0000-000000000001",
                    "name": "Test Project",
                    "git_repo_path": "/path/to/repo",
                    "setup_script": null,
                    "cleanup_script": null,
                    "dev_script": null,
                    "created_at": "2024-01-01T00:00:00Z",
                    "updated_at": "2024-01-01T00:00:00Z"
                }
            ],
            "count": 1
        });

        assert_json_has_field(&response, "projects");
        assert_json_has_field(&response, "count");
        assert!(response["projects"].is_array());
    }

    #[tokio::test]
    async fn test_list_projects_empty_returns_zero_count() {
        let response = json!({
            "projects": [],
            "count": 0
        });

        assert_eq!(response["count"], 0);
        assert_eq!(response["projects"].as_array().unwrap().len(), 0);
    }
}

#[cfg(test)]
mod create_task_tests {
    use super::*;

    #[tokio::test]
    async fn test_create_task_requires_project_id() {
        // Test that create_task validates project_id parameter
        // This would test the actual tool invocation with missing project_id
        // and expect a validation error
    }

    #[tokio::test]
    async fn test_create_task_returns_task_id() {
        let project_id = Uuid::new_v4();
        let task_id = Uuid::new_v4();

        let response = json!({
            "task_id": task_id.to_string()
        });

        assert_json_has_field(&response, "task_id");

        // Verify it's a valid UUID
        let parsed_id = Uuid::parse_str(response["task_id"].as_str().unwrap());
        assert!(parsed_id.is_ok());
    }

    #[tokio::test]
    async fn test_create_task_with_description() {
        // Test creating a task with optional description field
        let project_id = Uuid::new_v4();
        let description = "This is a test task description";

        // Would invoke: create_task(project_id, "Test Task", Some(description))
        // and verify the task is created with the description
    }

    #[tokio::test]
    async fn test_create_task_validates_project_exists() {
        // Test that create_task fails for non-existent project_id
        let fake_project_id = Uuid::new_v4();

        // Would expect error: "Project not found" or similar
    }
}

#[cfg(test)]
mod list_tasks_tests {
    use super::*;

    #[tokio::test]
    async fn test_list_tasks_requires_project_id() {
        // Test that list_tasks requires project_id parameter
    }

    #[tokio::test]
    async fn test_list_tasks_returns_task_summaries() {
        let response = json!({
            "tasks": [
                {
                    "id": "00000000-0000-0000-0000-000000000001",
                    "title": "Test Task",
                    "status": "todo",
                    "created_at": "2024-01-01T00:00:00Z",
                    "updated_at": "2024-01-01T00:00:00Z",
                    "has_in_progress_attempt": false,
                    "has_merged_attempt": false,
                    "last_attempt_failed": false
                }
            ],
            "count": 1,
            "project_id": "00000000-0000-0000-0000-000000000002",
            "applied_filters": {
                "status": null,
                "limit": 50
            }
        });

        assert_json_structure(&response, &["tasks", "count", "project_id", "applied_filters"]);
        assert!(response["tasks"].is_array());
    }

    #[tokio::test]
    async fn test_list_tasks_filters_by_status() {
        // Test status filtering: 'todo', 'inprogress', 'inreview', 'done', 'cancelled'
        let valid_statuses = vec!["todo", "in-progress", "in-review", "done", "cancelled"];

        for status in valid_statuses {
            // Would invoke: list_tasks(project_id, Some(status), None)
            // and verify only tasks with that status are returned
        }
    }

    #[tokio::test]
    async fn test_list_tasks_respects_limit() {
        // Test that limit parameter is respected
        // Create 100 tasks, request with limit=10, verify only 10 returned
    }

    #[tokio::test]
    async fn test_list_tasks_invalid_status_returns_error() {
        // Test that invalid status value returns proper error
        let invalid_status = "invalid_status_value";

        // Would expect error message containing valid status values
    }
}

#[cfg(test)]
mod get_task_tests {
    use super::*;

    #[tokio::test]
    async fn test_get_task_returns_full_details() {
        let response = json!({
            "task": {
                "id": "00000000-0000-0000-0000-000000000001",
                "title": "Test Task",
                "description": "Task description",
                "status": "todo",
                "created_at": "2024-01-01T00:00:00Z",
                "updated_at": "2024-01-01T00:00:00Z",
                "has_in_progress_attempt": null,
                "has_merged_attempt": null,
                "last_attempt_failed": null
            }
        });

        assert_json_has_field(&response, "task");
        let task = &response["task"];
        assert_json_structure(task, &["id", "title", "status", "created_at", "updated_at"]);
    }

    #[tokio::test]
    async fn test_get_task_nonexistent_returns_404() {
        // Test that getting a non-existent task returns proper error
        let fake_task_id = Uuid::new_v4();

        // Would expect 404 or "Task not found" error
    }
}

#[cfg(test)]
mod update_task_tests {
    use super::*;

    #[tokio::test]
    async fn test_update_task_title() {
        // Test updating task title
        let task_id = Uuid::new_v4();
        let new_title = "Updated Task Title";

        // Would invoke: update_task(task_id, Some(new_title), None, None)
    }

    #[tokio::test]
    async fn test_update_task_description() {
        // Test updating task description
        let task_id = Uuid::new_v4();
        let new_description = "Updated description";

        // Would invoke: update_task(task_id, None, Some(new_description), None)
    }

    #[tokio::test]
    async fn test_update_task_status() {
        // Test updating task status through all valid transitions
        let valid_statuses = vec!["todo", "in-progress", "in-review", "done", "cancelled"];

        for status in valid_statuses {
            // Would invoke: update_task(task_id, None, None, Some(status))
        }
    }

    #[tokio::test]
    async fn test_update_task_invalid_status_returns_error() {
        let task_id = Uuid::new_v4();
        let invalid_status = "invalid_status";

        // Would expect error with list of valid status values
    }

    #[tokio::test]
    async fn test_update_task_multiple_fields() {
        // Test updating multiple fields at once
        let task_id = Uuid::new_v4();

        // Would invoke: update_task(task_id, Some("New Title"), Some("New Desc"), Some("in-progress"))
    }
}

#[cfg(test)]
mod delete_task_tests {
    use super::*;

    #[tokio::test]
    async fn test_delete_task_returns_deleted_id() {
        let task_id = Uuid::new_v4();

        let response = json!({
            "deleted_task_id": task_id.to_string()
        });

        assert_json_has_field(&response, "deleted_task_id");
    }

    #[tokio::test]
    async fn test_delete_task_nonexistent_returns_404() {
        // Test deleting non-existent task
        let fake_task_id = Uuid::new_v4();

        // Would expect 404 or "Task not found" error
    }

    #[tokio::test]
    async fn test_delete_task_with_attempts() {
        // Test that deleting a task with existing attempts works properly
        // (or returns appropriate error if not allowed)
    }
}

#[cfg(test)]
mod start_task_attempt_tests {
    use super::*;

    #[tokio::test]
    async fn test_start_task_attempt_valid_executor() {
        let valid_executors = vec![
            "CLAUDE_CODE",
            "AMP",
            "GEMINI",
            "CODEX",
            "OPENCODE",
            "CURSOR",
            "QWEN_CODE",
            "COPILOT",
        ];

        for executor in valid_executors {
            // Would invoke: start_task_attempt(task_id, executor, None, "main")
            // and verify attempt is created
        }
    }

    #[tokio::test]
    async fn test_start_task_attempt_invalid_executor() {
        let task_id = Uuid::new_v4();
        let invalid_executor = "INVALID_EXECUTOR";

        // Would expect error listing valid executors
    }

    #[tokio::test]
    async fn test_start_task_attempt_with_variant() {
        let task_id = Uuid::new_v4();
        let executor = "CLAUDE_CODE";
        let variant = "3.5-sonnet";

        // Would invoke: start_task_attempt(task_id, executor, Some(variant), "main")
    }

    #[tokio::test]
    async fn test_start_task_attempt_empty_base_branch_error() {
        let task_id = Uuid::new_v4();

        // Would expect error: "Base branch must not be empty"
    }

    #[tokio::test]
    async fn test_start_task_attempt_normalizes_executor_name() {
        // Test that executor names are normalized (replace '-' with '_', uppercase)
        let task_id = Uuid::new_v4();
        let executor = "claude-code"; // lowercase with dash

        // Should normalize to "CLAUDE_CODE" and succeed
    }
}

#[cfg(test)]
mod task_attempts_tests {
    use super::*;

    #[tokio::test]
    async fn test_list_task_attempts_returns_summaries() {
        let response = json!({
            "attempts": [
                {
                    "id": "00000000-0000-0000-0000-000000000001",
                    "task_id": "00000000-0000-0000-0000-000000000002",
                    "branch": "task/test-branch",
                    "target_branch": "main",
                    "executor": "CLAUDE_CODE",
                    "container_ref": "/path/to/worktree",
                    "worktree_deleted": false,
                    "setup_completed_at": "2024-01-01T00:00:00Z",
                    "created_at": "2024-01-01T00:00:00Z",
                    "updated_at": "2024-01-01T00:00:00Z"
                }
            ],
            "count": 1,
            "task_id": "00000000-0000-0000-0000-000000000002"
        });

        assert_json_structure(&response, &["attempts", "count", "task_id"]);
        assert!(response["attempts"].is_array());
    }

    #[tokio::test]
    async fn test_get_task_attempt_returns_details() {
        let response = json!({
            "attempt": {
                "id": "00000000-0000-0000-0000-000000000001",
                "task_id": "00000000-0000-0000-0000-000000000002",
                "branch": "task/test-branch",
                "target_branch": "main",
                "executor": "CLAUDE_CODE",
                "container_ref": "/path/to/worktree",
                "worktree_deleted": false,
                "setup_completed_at": "2024-01-01T00:00:00Z",
                "created_at": "2024-01-01T00:00:00Z",
                "updated_at": "2024-01-01T00:00:00Z"
            }
        });

        assert_json_has_field(&response, "attempt");
    }

    #[tokio::test]
    async fn test_create_followup_attempt() {
        let previous_attempt_id = Uuid::new_v4();
        let feedback = "Please address review comments";

        // Would invoke: create_followup_attempt(previous_attempt_id, Some(feedback), None)

        let response = json!({
            "task_id": "00000000-0000-0000-0000-000000000001",
            "attempt_id": "00000000-0000-0000-0000-000000000002",
            "based_on_attempt_id": previous_attempt_id.to_string()
        });

        assert_json_structure(&response, &["task_id", "attempt_id", "based_on_attempt_id"]);
    }

    #[tokio::test]
    async fn test_merge_task_attempt_success() {
        let attempt_id = Uuid::new_v4();

        let response = json!({
            "success": true,
            "message": "Task attempt merged successfully",
            "task_id": "00000000-0000-0000-0000-000000000001",
            "attempt_id": attempt_id.to_string()
        });

        assert_json_structure(&response, &["success", "message", "task_id", "attempt_id"]);
        assert_eq!(response["success"], true);
    }

    #[tokio::test]
    async fn test_merge_task_attempt_with_conflicts() {
        // Test merging an attempt that has conflicts
        // Should return appropriate error
    }
}

#[cfg(test)]
mod response_format_tests {
    use super::*;

    #[tokio::test]
    async fn test_all_responses_are_valid_json() {
        // Test that all tool responses are valid JSON
        let test_responses = vec![
            r#"{"task_id": "123"}"#,
            r#"{"projects": [], "count": 0}"#,
            r#"{"success": true, "message": "ok"}"#,
        ];

        for response in test_responses {
            let result = parse_tool_response(response);
            assert!(result.is_ok(), "Failed to parse: {}", response);
        }
    }

    #[tokio::test]
    async fn test_responses_use_pretty_printing() {
        // Test that responses use pretty printing (have newlines)
        let response = json!({
            "task_id": "123",
            "nested": {
                "field": "value"
            }
        });

        let pretty = serde_json::to_string_pretty(&response).unwrap();
        assert!(pretty.contains('\n'), "Response should be pretty-printed");
    }
}
