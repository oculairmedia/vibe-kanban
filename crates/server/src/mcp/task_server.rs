use std::{path::PathBuf, str::FromStr, sync::Arc};

use db::models::{
    execution_process::{ExecutionProcess, ExecutionProcessRunReason, ExecutionProcessStatus},
    project::Project,
    task::{CreateTask, Task, TaskStatus, TaskWithAttemptStatus, UpdateTask},
    task_attempt::TaskAttempt,
};
use turbomcp::prelude::*;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json;
use uuid::Uuid;

use crate::routes::task_attempts::CreateTaskAttemptBody;

// Minimal copy of ExecutorProfileId to avoid depending on executors crate
// which has codex-protocol compilation issues
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
struct McpExecutorProfileId {
    executor: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    variant: Option<String>,
}

// Valid executor names (from executors::executors::BaseCodingAgent enum)
const VALID_EXECUTORS: &[&str] = &[
    "CLAUDE_CODE",
    "AMP",
    "GEMINI",
    "CODEX",
    "OPENCODE",
    "CURSOR",
    "QWEN_CODE",
    "COPILOT",
];

fn validate_executor(executor: &str) -> Result<(), String> {
    if VALID_EXECUTORS.contains(&executor) {
        Ok(())
    } else {
        Err(format!(
            "Unknown executor '{}'. Valid executors are: {}",
            executor,
            VALID_EXECUTORS.join(", ")
        ))
    }
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreateTaskRequest {
    #[schemars(description = "The ID of the project to create the task in. This is required!")]
    pub project_id: Uuid,
    #[schemars(description = "The title of the task")]
    pub title: String,
    #[schemars(description = "Optional description of the task")]
    pub description: Option<String>,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct CreateTaskResponse {
    pub task_id: String,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct ProjectSummary {
    #[schemars(description = "The unique identifier of the project")]
    pub id: String,
    #[schemars(description = "The name of the project")]
    pub name: String,
    #[schemars(description = "The path to the git repository")]
    pub git_repo_path: PathBuf,
    #[schemars(description = "Optional setup script for the project")]
    pub setup_script: Option<String>,
    #[schemars(description = "Optional cleanup script for the project")]
    pub cleanup_script: Option<String>,
    #[schemars(description = "Optional development script for the project")]
    pub dev_script: Option<String>,
    #[schemars(description = "When the project was created")]
    pub created_at: String,
    #[schemars(description = "When the project was last updated")]
    pub updated_at: String,
}

impl ProjectSummary {
    fn from_project(project: Project) -> Self {
        Self {
            id: project.id.to_string(),
            name: project.name,
            git_repo_path: project.git_repo_path,
            setup_script: project.setup_script,
            cleanup_script: project.cleanup_script,
            dev_script: project.dev_script,
            created_at: project.created_at.to_rfc3339(),
            updated_at: project.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct ListProjectsResponse {
    pub projects: Vec<ProjectSummary>,
    pub count: usize,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListTasksRequest {
    #[schemars(description = "The ID of the project to list tasks from")]
    pub project_id: Uuid,
    #[schemars(
        description = "Optional status filter: 'todo', 'inprogress', 'inreview', 'done', 'cancelled'"
    )]
    pub status: Option<String>,
    #[schemars(description = "Maximum number of tasks to return (default: 50)")]
    pub limit: Option<i32>,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct TaskSummary {
    #[schemars(description = "The unique identifier of the task")]
    pub id: String,
    #[schemars(description = "The title of the task")]
    pub title: String,
    #[schemars(description = "Current status of the task")]
    pub status: String,
    #[schemars(description = "When the task was created")]
    pub created_at: String,
    #[schemars(description = "When the task was last updated")]
    pub updated_at: String,
    #[schemars(description = "Whether the task has an in-progress execution attempt")]
    pub has_in_progress_attempt: Option<bool>,
    #[schemars(description = "Whether the task has a merged execution attempt")]
    pub has_merged_attempt: Option<bool>,
    #[schemars(description = "Whether the last execution attempt failed")]
    pub last_attempt_failed: Option<bool>,
}

impl TaskSummary {
    fn from_task_with_status(task: TaskWithAttemptStatus) -> Self {
        Self {
            id: task.id.to_string(),
            title: task.title.to_string(),
            status: task.status.to_string(),
            created_at: task.created_at.to_rfc3339(),
            updated_at: task.updated_at.to_rfc3339(),
            has_in_progress_attempt: Some(task.has_in_progress_attempt),
            has_merged_attempt: Some(task.has_merged_attempt),
            last_attempt_failed: Some(task.last_attempt_failed),
        }
    }
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct TaskDetails {
    #[schemars(description = "The unique identifier of the task")]
    pub id: String,
    #[schemars(description = "The title of the task")]
    pub title: String,
    #[schemars(description = "Optional description of the task")]
    pub description: Option<String>,
    #[schemars(description = "Current status of the task")]
    pub status: String,
    #[schemars(description = "When the task was created")]
    pub created_at: String,
    #[schemars(description = "When the task was last updated")]
    pub updated_at: String,
    #[schemars(description = "Whether the task has an in-progress execution attempt")]
    pub has_in_progress_attempt: Option<bool>,
    #[schemars(description = "Whether the task has a merged execution attempt")]
    pub has_merged_attempt: Option<bool>,
    #[schemars(description = "Whether the last execution attempt failed")]
    pub last_attempt_failed: Option<bool>,
}

impl TaskDetails {
    fn from_task(task: Task) -> Self {
        Self {
            id: task.id.to_string(),
            title: task.title,
            description: task.description,
            status: task.status.to_string(),
            created_at: task.created_at.to_rfc3339(),
            updated_at: task.updated_at.to_rfc3339(),
            has_in_progress_attempt: None,
            has_merged_attempt: None,
            last_attempt_failed: None,
        }
    }
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct ListTasksResponse {
    pub tasks: Vec<TaskSummary>,
    pub count: usize,
    pub project_id: String,
    pub applied_filters: ListTasksFilters,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct ListTasksFilters {
    pub status: Option<String>,
    pub limit: i32,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct UpdateTaskRequest {
    #[schemars(description = "The ID of the task to update")]
    pub task_id: Uuid,
    #[schemars(description = "New title for the task")]
    pub title: Option<String>,
    #[schemars(description = "New description for the task")]
    pub description: Option<String>,
    #[schemars(description = "New status: 'todo', 'inprogress', 'inreview', 'done', 'cancelled'")]
    pub status: Option<String>,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct UpdateTaskResponse {
    pub task: TaskDetails,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DeleteTaskRequest {
    #[schemars(description = "The ID of the task to delete")]
    pub task_id: Uuid,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct StartTaskAttemptRequest {
    #[schemars(description = "The ID of the task to start")]
    pub task_id: Uuid,
    #[schemars(
        description = "The coding agent executor to run ('CLAUDE_CODE', 'CODEX', 'GEMINI', 'CURSOR', 'OPENCODE', 'AMP', 'QWEN_CODE', 'COPILOT')"
    )]
    pub executor: String,
    #[schemars(description = "Optional executor variant, if needed")]
    pub variant: Option<String>,
    #[schemars(description = "The base branch to use for the attempt")]
    pub base_branch: String,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct StartTaskAttemptResponse {
    pub task_id: String,
    pub attempt_id: String,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct DeleteTaskResponse {
    pub deleted_task_id: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetTaskRequest {
    #[schemars(description = "The ID of the task to retrieve")]
    pub task_id: Uuid,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct GetTaskResponse {
    pub task: TaskDetails,
}

// ============================================================================
// Task Attempts Types
// ============================================================================

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListTaskAttemptsRequest {
    #[schemars(description = "The ID of the task to list attempts for")]
    pub task_id: Uuid,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct TaskAttemptSummary {
    #[schemars(description = "The unique identifier of the attempt")]
    pub id: String,
    #[schemars(description = "The task ID this attempt belongs to")]
    pub task_id: String,
    #[schemars(description = "Git branch name for this attempt")]
    pub branch: String,
    #[schemars(description = "Target branch for this attempt (PR destination)")]
    pub target_branch: String,
    #[schemars(description = "The executor used for this attempt (e.g., CLAUDE_CODE, GEMINI)")]
    pub executor: String,
    #[schemars(description = "Path to worktree or container reference")]
    pub container_ref: Option<String>,
    #[schemars(description = "Whether the worktree has been deleted")]
    pub worktree_deleted: bool,
    #[schemars(description = "When setup script was completed")]
    pub setup_completed_at: Option<String>,
    #[schemars(description = "When the attempt was created")]
    pub created_at: String,
    #[schemars(description = "When the attempt was last updated")]
    pub updated_at: String,
}

impl TaskAttemptSummary {
    fn from_task_attempt(attempt: TaskAttempt) -> Self {
        Self {
            id: attempt.id.to_string(),
            task_id: attempt.task_id.to_string(),
            branch: attempt.branch,
            target_branch: attempt.target_branch,
            executor: attempt.executor,
            container_ref: attempt.container_ref,
            worktree_deleted: attempt.worktree_deleted,
            setup_completed_at: attempt.setup_completed_at.map(|dt| dt.to_rfc3339()),
            created_at: attempt.created_at.to_rfc3339(),
            updated_at: attempt.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct ListTaskAttemptsResponse {
    pub attempts: Vec<TaskAttemptSummary>,
    pub count: usize,
    pub task_id: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetTaskAttemptRequest {
    #[schemars(description = "The ID of the attempt to retrieve")]
    pub attempt_id: Uuid,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct GetTaskAttemptResponse {
    pub attempt: TaskAttemptSummary,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreateFollowupAttemptRequest {
    #[schemars(description = "The ID of the previous attempt to base this followup on")]
    pub previous_attempt_id: Uuid,
    #[schemars(description = "Optional feedback or instructions for the followup attempt")]
    pub feedback: Option<String>,
    #[schemars(description = "Optional executor variant to use")]
    pub variant: Option<String>,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct CreateFollowupAttemptResponse {
    pub task_id: String,
    pub attempt_id: String,
    pub based_on_attempt_id: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct MergeTaskAttemptRequest {
    #[schemars(description = "The ID of the task attempt to merge")]
    pub attempt_id: Uuid,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct MergeTaskAttemptResponse {
    pub success: bool,
    pub message: String,
    pub task_id: String,
    pub attempt_id: String,
}

// ============================================================================
// Execution Process Types
// ============================================================================

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetExecutionProcessRequest {
    #[schemars(description = "The ID of the execution process to retrieve")]
    pub process_id: Uuid,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct ExecutionProcessSummary {
    #[schemars(description = "The unique identifier of the execution process")]
    pub id: String,
    #[schemars(description = "The task attempt ID this process belongs to")]
    pub task_attempt_id: String,
    #[schemars(description = "Why this process was run (e.g., SetupScript, CodingAgent, DevServer)")]
    pub run_reason: ExecutionProcessRunReason,
    #[schemars(description = "Current execution status (Running, Completed, Failed, Killed)")]
    pub status: ExecutionProcessStatus,
    #[schemars(description = "Exit code if the process has completed")]
    pub exit_code: Option<i64>,
    #[schemars(description = "Git commit hash before execution started")]
    pub before_head_commit: Option<String>,
    #[schemars(description = "Git commit hash after execution completed")]
    pub after_head_commit: Option<String>,
    #[schemars(description = "Whether this process has been soft-deleted from history")]
    pub dropped: bool,
    #[schemars(description = "When the process started executing")]
    pub started_at: String,
    #[schemars(description = "When the process completed (if finished)")]
    pub completed_at: Option<String>,
    #[schemars(description = "Total runtime in seconds (if completed)")]
    pub runtime_seconds: Option<f64>,
}

impl ExecutionProcessSummary {
    fn from_execution_process(process: ExecutionProcess) -> Self {
        let runtime_seconds = process.completed_at.map(|completed| {
            (completed - process.started_at).num_milliseconds() as f64 / 1000.0
        });

        Self {
            id: process.id.to_string(),
            task_attempt_id: process.task_attempt_id.to_string(),
            run_reason: process.run_reason,
            status: process.status,
            exit_code: process.exit_code,
            before_head_commit: process.before_head_commit,
            after_head_commit: process.after_head_commit,
            dropped: process.dropped,
            started_at: process.started_at.to_rfc3339(),
            completed_at: process.completed_at.map(|dt| dt.to_rfc3339()),
            runtime_seconds,
        }
    }
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct GetExecutionProcessResponse {
    pub process: ExecutionProcessSummary,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListExecutionProcessesRequest {
    #[schemars(description = "The ID of the task attempt to list processes for")]
    pub task_attempt_id: Uuid,
    #[schemars(description = "Whether to include soft-deleted (dropped) processes")]
    pub show_soft_deleted: Option<bool>,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct ListExecutionProcessesResponse {
    pub processes: Vec<ExecutionProcessSummary>,
    pub count: usize,
    pub task_attempt_id: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct StopExecutionProcessRequest {
    #[schemars(description = "The ID of the execution process to stop")]
    pub process_id: Uuid,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct StopExecutionProcessResponse {
    pub success: bool,
    pub message: String,
    pub process_id: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetProcessRawLogsRequest {
    #[schemars(description = "The ID of the execution process to retrieve logs for")]
    pub process_id: Uuid,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct LogMessage {
    #[schemars(description = "Type of log message (Stdout, Stderr, JsonPatch, SessionId, Finished, Unknown, Raw)")]
    pub msg_type: String,
    #[schemars(description = "Content of the log message")]
    pub content: serde_json::Value,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct GetProcessRawLogsResponse {
    pub process_id: String,
    pub logs: Vec<LogMessage>,
    pub byte_size: i64,
    pub log_count: usize,
    pub inserted_at: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetProcessNormalizedLogsRequest {
    #[schemars(description = "The ID of the execution process to retrieve normalized logs for")]
    pub process_id: Uuid,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct ProcessLogEntry {
    #[schemars(description = "Sequential index of the log entry")]
    pub index: usize,
    #[schemars(description = "Log level (stdout, stderr, info)")]
    pub level: String,
    #[schemars(description = "The log message content")]
    pub message: String,
    #[schemars(description = "ISO 8601 timestamp of the log entry")]
    pub timestamp: Option<String>,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct GetProcessNormalizedLogsResponse {
    pub execution_id: String,
    pub total_entries: usize,
    pub logs: Vec<ProcessLogEntry>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct StartDevServerRequest {
    #[schemars(description = "The ID of the task attempt to start the dev server for")]
    pub attempt_id: Uuid,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct StartDevServerResponse {
    pub success: bool,
    pub message: String,
    pub attempt_id: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetAttemptArtifactsRequest {
    #[schemars(description = "The ID of the task attempt to get artifacts for")]
    pub attempt_id: Uuid,
    #[schemars(description = "Filter by artifact type (GIT_DIFF, GIT_COMMIT, EXECUTION_LOG)")]
    pub artifact_type: Option<String>,
    #[schemars(description = "Maximum number of artifacts to return")]
    pub limit: Option<usize>,
    #[schemars(description = "Offset for pagination")]
    pub offset: Option<usize>,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct ArtifactSummary {
    #[schemars(description = "Type of artifact (GIT_DIFF, GIT_COMMIT, EXECUTION_LOG)")]
    pub artifact_type: String,
    #[schemars(description = "Execution process ID this artifact came from")]
    pub process_id: String,
    #[schemars(description = "Content of the artifact (may be truncated for display)")]
    pub content: Option<String>,
    #[schemars(description = "Size in bytes")]
    pub size_bytes: usize,
    #[schemars(description = "Git commit SHA (for commit artifacts)")]
    pub commit_sha: Option<String>,
    #[schemars(description = "Git commit subject/message (for commit artifacts)")]
    pub commit_subject: Option<String>,
    #[schemars(description = "Before commit SHA (for diff artifacts)")]
    pub before_commit: Option<String>,
    #[schemars(description = "After commit SHA (for diff artifacts)")]
    pub after_commit: Option<String>,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct GetAttemptArtifactsResponse {
    pub attempt_id: String,
    pub artifacts: Vec<ArtifactSummary>,
    pub total_count: usize,
}

/// Main Vibe Kanban Task MCP Server
#[derive(Clone)]
pub struct TaskServer {
    client: Arc<reqwest::Client>,
    base_url: Arc<String>,
}

#[derive(Debug, Deserialize)]
struct ApiResponseEnvelope<T> {
    success: bool,
    data: Option<T>,
    message: Option<String>,
}

impl TaskServer {
    pub fn new(base_url: &str) -> Self {
        Self {
            client: Arc::new(reqwest::Client::new()),
            base_url: Arc::new(base_url.to_string()),
        }
    }

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
        let resp = rb
            .send()
            .await
            .map_err(|e| Self::err_str("Failed to connect to VK API", Some(&e.to_string())))?;

        if !resp.status().is_success() {
            let status = resp.status();
            return Err(Self::err_str(
                &format!("VK API returned error status: {}", status),
                None,
            ));
        }

        let api_response = resp
            .json::<ApiResponseEnvelope<T>>()
            .await
            .map_err(|e| Self::err_str("Failed to parse VK API response", Some(&e.to_string())))?;

        if !api_response.success {
            let msg = api_response.message.as_deref().unwrap_or("Unknown error");
            return Err(Self::err_str("VK API returned error", Some(msg)));
        }

        api_response
            .data
            .ok_or_else(|| Self::err_str("VK API response missing data field", None))
    }

    fn url(&self, path: &str) -> String {
        format!(
            "{}/{}",
            self.base_url.trim_end_matches('/'),
            path.trim_start_matches('/')
        )
    }
}

#[turbomcp::server(
    name = "vibe-kanban",
    version = "1.0.0",
    description = "A task and project management server. If you need to create or update tickets or tasks then use these tools. Most of them absolutely require that you pass the `project_id` of the project that you are currently working on. This should be provided to you. Call `list_tasks` to fetch the `task_ids` of all the tasks in a project. TOOLS: 'list_projects', 'list_tasks', 'create_task', 'start_task_attempt', 'get_task', 'update_task', 'delete_task', 'list_task_attempts', 'get_task_attempt', 'get_attempt_artifacts', 'create_followup_attempt', 'merge_task_attempt', 'list_execution_processes', 'get_execution_process', 'stop_execution_process', 'get_process_raw_logs', 'get_process_normalized_logs', 'start_dev_server'. Make sure to pass `project_id` or `task_id` where required. You can use list tools to get the available ids."
)]
impl TaskServer {
    #[tool(
        description = "Create a new task/ticket in a project. Always pass the `project_id` of the project you want to create the task in - it is required!"
    )]
    async fn create_task(&self, request: CreateTaskRequest) -> McpResult<String> {
        let url = self.url("/api/tasks");
        let task: Task = self
            .send_json(
                self.client
                    .post(&url)
                    .json(&CreateTask::from_title_description(
                        request.project_id,
                        request.title,
                        request.description,
                    )),
            )
            .await?;

        let response = CreateTaskResponse {
            task_id: task.id.to_string(),
        };
        Ok(serde_json::to_string_pretty(&response).unwrap())
    }

    #[tool(description = "List all the available projects")]
    async fn list_projects(&self) -> McpResult<String> {
        let url = self.url("/api/projects");
        let projects: Vec<Project> = self.send_json(self.client.get(&url)).await?;

        let project_summaries: Vec<ProjectSummary> = projects
            .into_iter()
            .map(ProjectSummary::from_project)
            .collect();

        let response = ListProjectsResponse {
            count: project_summaries.len(),
            projects: project_summaries,
        };

        Ok(serde_json::to_string_pretty(&response).unwrap())
    }

    #[tool(
        description = "List all the task/tickets in a project with optional filtering and execution status. `project_id` is required!"
    )]
    async fn list_tasks(&self, request: ListTasksRequest) -> McpResult<String> {
        let status_filter = if let Some(ref status_str) = request.status {
            match TaskStatus::from_str(status_str) {
                Ok(s) => Some(s),
                Err(_) => {
                    return Err(McpError::invalid_request(format!(
                        "Invalid status filter '{}'. Valid values: 'todo', 'in-progress', 'in-review', 'done', 'cancelled'",
                        status_str
                    )));
                }
            }
        } else {
            None
        };

        let url = self.url(&format!("/api/tasks?project_id={}", request.project_id));
        let all_tasks: Vec<TaskWithAttemptStatus> =
            self.send_json(self.client.get(&url)).await?;

        let task_limit = request.limit.unwrap_or(50).max(0) as usize;
        let filtered = all_tasks.into_iter().filter(|t| {
            if let Some(ref want) = status_filter {
                &t.status == want
            } else {
                true
            }
        });
        let limited: Vec<TaskWithAttemptStatus> = filtered.take(task_limit).collect();

        let task_summaries: Vec<TaskSummary> = limited
            .into_iter()
            .map(TaskSummary::from_task_with_status)
            .collect();

        let response = ListTasksResponse {
            count: task_summaries.len(),
            tasks: task_summaries,
            project_id: request.project_id.to_string(),
            applied_filters: ListTasksFilters {
                status: request.status.clone(),
                limit: task_limit as i32,
            },
        };

        Ok(serde_json::to_string_pretty(&response).unwrap())
    }

    #[tool(description = "Start working on a task by creating and launching a new task attempt.")]
    async fn start_task_attempt(&self, request: StartTaskAttemptRequest) -> McpResult<String> {
        let base_branch = request.base_branch.trim().to_string();
        if base_branch.is_empty() {
            return Err(McpError::invalid_request("Base branch must not be empty."));
        }

        let executor_trimmed = request.executor.trim();
        if executor_trimmed.is_empty() {
            return Err(McpError::invalid_request("Executor must not be empty."));
        }

        let normalized_executor = executor_trimmed.replace('-', "_").to_ascii_uppercase();
        if let Err(err_msg) = validate_executor(&normalized_executor) {
            return Err(McpError::invalid_request(err_msg));
        }

        let variant = request.variant.and_then(|v| {
            let trimmed = v.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        });

        let executor_profile_id = McpExecutorProfileId {
            executor: normalized_executor,
            variant,
        };

        // Convert McpExecutorProfileId to JSON and then parse as the backend's ExecutorProfileId
        // This works because they have the same structure - we just can't depend on executors crate
        let executor_json = serde_json::to_value(&executor_profile_id)
            .map_err(|e| McpError::internal(format!("Failed to serialize executor: {}", e)))?;
        let backend_executor_profile_id = serde_json::from_value(executor_json)
            .map_err(|e| McpError::internal(format!("Failed to deserialize executor: {}", e)))?;

        let payload = CreateTaskAttemptBody {
            task_id: request.task_id,
            executor_profile_id: backend_executor_profile_id,
            base_branch,
        };

        let url = self.url("/api/task-attempts");
        let attempt: TaskAttempt = self.send_json(self.client.post(&url).json(&payload)).await?;

        let response = StartTaskAttemptResponse {
            task_id: attempt.task_id.to_string(),
            attempt_id: attempt.id.to_string(),
        };

        Ok(serde_json::to_string_pretty(&response).unwrap())
    }

    #[tool(
        description = "Update an existing task/ticket's title, description, or status. `project_id` and `task_id` are required! `title`, `description`, and `status` are optional."
    )]
    async fn update_task(&self, request: UpdateTaskRequest) -> McpResult<String> {
        let status = if let Some(ref status_str) = request.status {
            match TaskStatus::from_str(status_str) {
                Ok(s) => Some(s),
                Err(_) => {
                    return Err(McpError::invalid_request(format!(
                        "Invalid status '{}'. Valid values: 'todo', 'in-progress', 'in-review', 'done', 'cancelled'",
                        status_str
                    )));
                }
            }
        } else {
            None
        };

        let payload = UpdateTask {
            title: request.title,
            description: request.description,
            status,
            parent_task_attempt: None,
            image_ids: None,
        };
        let url = self.url(&format!("/api/tasks/{}", request.task_id));
        let updated_task: Task = self.send_json(self.client.put(&url).json(&payload)).await?;

        let details = TaskDetails::from_task(updated_task);
        let response = UpdateTaskResponse { task: details };
        Ok(serde_json::to_string_pretty(&response).unwrap())
    }

    #[tool(
        description = "Delete a task/ticket from a project. `project_id` and `task_id` are required!"
    )]
    async fn delete_task(&self, request: DeleteTaskRequest) -> McpResult<String> {
        let url = self.url(&format!("/api/tasks/{}", request.task_id));
        self.send_json::<serde_json::Value>(self.client.delete(&url))
            .await?;

        let response = DeleteTaskResponse {
            deleted_task_id: Some(request.task_id.to_string()),
        };

        Ok(serde_json::to_string_pretty(&response).unwrap())
    }

    #[tool(
        description = "Get detailed information (like task description) about a specific task/ticket. You can use `list_tasks` to find the `task_ids` of all tasks in a project. `project_id` and `task_id` are required!"
    )]
    async fn get_task(&self, request: GetTaskRequest) -> McpResult<String> {
        let url = self.url(&format!("/api/tasks/{}", request.task_id));
        let task: Task = self.send_json(self.client.get(&url)).await?;

        let details = TaskDetails::from_task(task);
        let response = GetTaskResponse { task: details };

        Ok(serde_json::to_string_pretty(&response).unwrap())
    }

    #[tool(
        description = "List all execution attempts for a specific task. Shows what was tried, branch names, executors used, and timestamps. Useful for understanding task history and debugging failed attempts. `task_id` is required!"
    )]
    async fn list_task_attempts(&self, request: ListTaskAttemptsRequest) -> McpResult<String> {
        let url = self.url(&format!("/api/task-attempts?task_id={}", request.task_id));
        let attempts: Vec<TaskAttempt> = self.send_json(self.client.get(&url)).await?;

        let attempt_summaries: Vec<TaskAttemptSummary> = attempts
            .into_iter()
            .map(TaskAttemptSummary::from_task_attempt)
            .collect();

        let response = ListTaskAttemptsResponse {
            count: attempt_summaries.len(),
            attempts: attempt_summaries,
            task_id: request.task_id.to_string(),
        };

        Ok(serde_json::to_string_pretty(&response).unwrap())
    }

    #[tool(
        description = "Get detailed information about a specific task attempt including branch, executor, timestamps, and worktree status. `attempt_id` is required!"
    )]
    async fn get_task_attempt(&self, request: GetTaskAttemptRequest) -> McpResult<String> {
        let url = self.url(&format!("/api/task-attempts/{}", request.attempt_id));
        let attempt: TaskAttempt = self.send_json(self.client.get(&url)).await?;

        let attempt_summary = TaskAttemptSummary::from_task_attempt(attempt);
        let response = GetTaskAttemptResponse {
            attempt: attempt_summary,
        };

        Ok(serde_json::to_string_pretty(&response).unwrap())
    }

    #[tool(
        description = "Create a follow-up attempt based on a previous attempt. Useful for addressing review feedback or retrying after fixes. `previous_attempt_id` is required!"
    )]
    async fn create_followup_attempt(&self, request: CreateFollowupAttemptRequest) -> McpResult<String> {
        let url = self.url("/api/task-attempts/followup");

        #[derive(Serialize)]
        struct FollowupPayload {
            previous_attempt_id: Uuid,
            feedback: Option<String>,
            variant: Option<String>,
        }

        let payload = FollowupPayload {
            previous_attempt_id: request.previous_attempt_id,
            feedback: request.feedback,
            variant: request.variant,
        };

        let attempt: TaskAttempt = self.send_json(self.client.post(&url).json(&payload)).await?;

        let response = CreateFollowupAttemptResponse {
            task_id: attempt.task_id.to_string(),
            attempt_id: attempt.id.to_string(),
            based_on_attempt_id: request.previous_attempt_id.to_string(),
        };

        Ok(serde_json::to_string_pretty(&response).unwrap())
    }

    #[tool(
        description = "Merge a completed task attempt into its target branch. This performs a git merge operation and marks the task as done. The attempt must be complete with no conflicts. `attempt_id` is required!"
    )]
    async fn merge_task_attempt(&self, request: MergeTaskAttemptRequest) -> McpResult<String> {
        let url = self.url(&format!("/api/task-attempts/{}/merge", request.attempt_id));

        // POST to merge endpoint returns ApiResponse<()>
        self.send_json::<serde_json::Value>(self.client.post(&url)).await?;

        // Fetch the task attempt to get task_id for response
        let attempt_url = self.url(&format!("/api/task-attempts/{}", request.attempt_id));
        let attempt: TaskAttempt = self.send_json(self.client.get(&attempt_url)).await?;

        let response = MergeTaskAttemptResponse {
            success: true,
            message: "Task attempt merged successfully".to_string(),
            task_id: attempt.task_id.to_string(),
            attempt_id: request.attempt_id.to_string(),
        };

        Ok(serde_json::to_string_pretty(&response).unwrap())
    }

    #[tool(
        description = "Get detailed information about a specific execution process including status, exit code, runtime metrics, and git commit information. `process_id` is required!"
    )]
    async fn get_execution_process(&self, request: GetExecutionProcessRequest) -> McpResult<String> {
        let url = self.url(&format!("/api/execution-processes/{}", request.process_id));
        let process: ExecutionProcess = self.send_json(self.client.get(&url)).await?;

        let process_summary = ExecutionProcessSummary::from_execution_process(process);
        let response = GetExecutionProcessResponse {
            process: process_summary,
        };

        Ok(serde_json::to_string_pretty(&response).unwrap())
    }

    #[tool(
        description = "Stop a running execution process. This kills the process, updates its status to 'Killed', and sets the parent task to 'InReview' status. `process_id` is required!"
    )]
    async fn stop_execution_process(&self, request: StopExecutionProcessRequest) -> McpResult<String> {
        let url = self.url(&format!("/api/execution-processes/{}/stop", request.process_id));

        // POST to stop endpoint returns ApiResponse<()>
        self.send_json::<serde_json::Value>(self.client.post(&url)).await?;

        let response = StopExecutionProcessResponse {
            success: true,
            message: "Execution process stopped successfully".to_string(),
            process_id: request.process_id.to_string(),
        };

        Ok(serde_json::to_string_pretty(&response).unwrap())
    }

    #[tool(
        description = "List all execution processes for a task attempt. Returns process history with status, runtime metrics, and git commits. Optionally include soft-deleted processes. `task_attempt_id` is required!"
    )]
    async fn list_execution_processes(&self, request: ListExecutionProcessesRequest) -> McpResult<String> {
        let mut url = self.url("/api/processes");
        let params = format!("?task_attempt_id={}", request.task_attempt_id);
        url.push_str(&params);
        
        if let Some(show_deleted) = request.show_soft_deleted {
            url.push_str(&format!("&show_soft_deleted={}", show_deleted));
        }

        let processes: Vec<ExecutionProcess> = self.send_json(self.client.get(&url)).await?;

        let process_summaries: Vec<ExecutionProcessSummary> = processes
            .into_iter()
            .map(ExecutionProcessSummary::from_execution_process)
            .collect();

        let response = ListExecutionProcessesResponse {
            count: process_summaries.len(),
            task_attempt_id: request.task_attempt_id.to_string(),
            processes: process_summaries,
        };

        Ok(serde_json::to_string_pretty(&response).unwrap())
    }

    #[tool(
        description = "Get the raw stdout/stderr logs for an execution process. Returns all log messages including stdout, stderr, and process state. Useful for debugging task execution and understanding what happened during a run. `process_id` is required!"
    )]
    async fn get_process_raw_logs(&self, request: GetProcessRawLogsRequest) -> McpResult<String> {
        let url = self.url(&format!("/api/execution-processes/{}/logs", request.process_id));

        // Define a minimal response structure matching the API endpoint
        #[derive(Debug, Deserialize)]
        struct RawLogsApiResponse {
            execution_id: Uuid,
            logs: Vec<serde_json::Value>, // LogMsg deserialized as raw JSON
            byte_size: i64,
            inserted_at: String,
        }

        let api_response: RawLogsApiResponse = self.send_json(self.client.get(&url)).await?;

        // Convert raw JSON log messages to structured LogMessage format
        let mut log_messages = Vec::new();
        for log_value in &api_response.logs {
            let log_msg = match log_value {
                serde_json::Value::Object(map) => {
                    if let Some(stdout) = map.get("Stdout") {
                        LogMessage {
                            msg_type: "Stdout".to_string(),
                            content: stdout.clone(),
                        }
                    } else if let Some(stderr) = map.get("Stderr") {
                        LogMessage {
                            msg_type: "Stderr".to_string(),
                            content: stderr.clone(),
                        }
                    } else if let Some(json_patch) = map.get("JsonPatch") {
                        LogMessage {
                            msg_type: "JsonPatch".to_string(),
                            content: json_patch.clone(),
                        }
                    } else if let Some(session_id) = map.get("SessionId") {
                        LogMessage {
                            msg_type: "SessionId".to_string(),
                            content: session_id.clone(),
                        }
                    } else if map.contains_key("Finished") {
                        LogMessage {
                            msg_type: "Finished".to_string(),
                            content: serde_json::Value::Null,
                        }
                    } else {
                        // Unknown log type - include as-is
                        LogMessage {
                            msg_type: "Unknown".to_string(),
                            content: log_value.clone(),
                        }
                    }
                }
                _ => {
                    // Non-object log entry
                    LogMessage {
                        msg_type: "Raw".to_string(),
                        content: log_value.clone(),
                    }
                }
            };
            log_messages.push(log_msg);
        }

        let response = GetProcessRawLogsResponse {
            process_id: api_response.execution_id.to_string(),
            logs: log_messages,
            byte_size: api_response.byte_size,
            log_count: api_response.logs.len(),
            inserted_at: api_response.inserted_at,
        };

        Ok(serde_json::to_string_pretty(&response).unwrap())
    }

    #[tool(
        description = "Get parsed and normalized logs for an execution process. Returns structured log entries with timestamps, levels (stdout/stderr/info), and messages. Useful for debugging task execution. `process_id` is required!"
    )]
    async fn get_process_normalized_logs(
        &self,
        request: GetProcessNormalizedLogsRequest,
    ) -> McpResult<String> {
        let url = self.url(&format!(
            "/api/execution-processes/{}/logs/normalized",
            request.process_id
        ));

        // Define a local response type that matches the API response
        #[derive(Debug, Deserialize)]
        struct ApiNormalizedLogEntry {
            index: usize,
            level: String,
            message: String,
            timestamp: Option<String>,
        }

        #[derive(Debug, Deserialize)]
        struct ApiNormalizedLogsResponse {
            execution_id: String,
            logs: Vec<ApiNormalizedLogEntry>,
            total_entries: usize,
        }

        let api_response: ApiNormalizedLogsResponse =
            self.send_json(self.client.get(&url)).await?;

        // Convert to MCP response format
        let logs: Vec<ProcessLogEntry> = api_response
            .logs
            .into_iter()
            .map(|entry| ProcessLogEntry {
                index: entry.index,
                level: entry.level,
                message: entry.message,
                timestamp: entry.timestamp,
            })
            .collect();

        let response = GetProcessNormalizedLogsResponse {
            execution_id: api_response.execution_id,
            total_entries: api_response.total_entries,
            logs,
        };

        Ok(serde_json::to_string_pretty(&response).unwrap())
    }

    #[tool(
        description = "Start a development server for a task attempt. This will execute the project's dev script (e.g., 'npm run dev') in the attempt's worktree. Only one dev server can run per project at a time - starting a new one will stop any existing dev server for the project. `attempt_id` is required!"
    )]
    async fn start_dev_server(&self, request: StartDevServerRequest) -> McpResult<String> {
        let url = self.url(&format!("/api/task-attempts/{}/start-dev-server", request.attempt_id));

        // POST to start-dev-server endpoint returns ApiResponse<()>
        self.send_json::<serde_json::Value>(self.client.post(&url)).await?;

        let response = StartDevServerResponse {
            success: true,
            message: "Development server started successfully".to_string(),
            attempt_id: request.attempt_id.to_string(),
        };

        Ok(serde_json::to_string_pretty(&response).unwrap())
    }

    #[tool(
        description = "Get all artifacts (git diffs, commits, execution logs) for a task attempt. Returns work products from execution processes including code changes, commit messages, and process outputs. Useful for reviewing what work was done during an attempt. `attempt_id` is required!"
    )]
    async fn get_attempt_artifacts(&self, request: GetAttemptArtifactsRequest) -> McpResult<String> {
        let mut url = self.url(&format!("/api/task-attempts/{}/artifacts", request.attempt_id));

        // Add query parameters
        let mut params = vec![];
        if let Some(artifact_type) = &request.artifact_type {
            params.push(format!("artifact_type={}", artifact_type));
        }
        if let Some(limit) = request.limit {
            params.push(format!("limit={}", limit));
        }
        if let Some(offset) = request.offset {
            params.push(format!("offset={}", offset));
        }

        if !params.is_empty() {
            url.push('?');
            url.push_str(&params.join("&"));
        }

        // Define local response type matching the API
        #[derive(Debug, Deserialize)]
        struct ApiArtifact {
            artifact_type: String,
            process_id: String,
            content: Option<String>,
            size_bytes: usize,
            commit_sha: Option<String>,
            commit_subject: Option<String>,
            before_commit: Option<String>,
            after_commit: Option<String>,
        }

        #[derive(Debug, Deserialize)]
        struct ApiArtifactsResponse {
            attempt_id: String,
            artifacts: Vec<ApiArtifact>,
            total_count: usize,
        }

        let api_response: ApiArtifactsResponse = self.send_json(self.client.get(&url)).await?;

        // Convert to MCP response format
        let artifacts: Vec<ArtifactSummary> = api_response
            .artifacts
            .into_iter()
            .map(|artifact| ArtifactSummary {
                artifact_type: artifact.artifact_type,
                process_id: artifact.process_id,
                content: artifact.content,
                size_bytes: artifact.size_bytes,
                commit_sha: artifact.commit_sha,
                commit_subject: artifact.commit_subject,
                before_commit: artifact.before_commit,
                after_commit: artifact.after_commit,
            })
            .collect();

        let response = GetAttemptArtifactsResponse {
            attempt_id: api_response.attempt_id,
            artifacts,
            total_count: api_response.total_count,
        };

        Ok(serde_json::to_string_pretty(&response).unwrap())
    }
}

// Custom HTTP runner implementation with permissive security for development
#[cfg(feature = "http")]
impl TaskServer {
    /// Run HTTP server with custom security configuration
    pub async fn run_http_custom(&self, addr: &str) -> Result<(), Box<dyn std::error::Error>> {
        use turbomcp_transport::streamable_http_v2::{StreamableHttpConfigBuilder, run_server};
        use std::time::Duration;

        // Create permissive HTTP config for development
        let config = StreamableHttpConfigBuilder::new()
            .with_bind_address(addr)
            .allow_any_origin(true) // Allow any origin in development mode
            .allow_localhost(true)
            .with_rate_limit(1_000_000, Duration::from_secs(60)) // Very high limit for development
            .build();

        // Run the HTTP server with custom config
        run_server(config, Arc::new(self.clone())).await?;
        Ok(())
    }
}
