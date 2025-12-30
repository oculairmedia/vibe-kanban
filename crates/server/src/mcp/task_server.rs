use std::{str::FromStr, sync::Arc};

use db::models::{
    execution_process::{ExecutionProcess, ExecutionProcessRunReason, ExecutionProcessStatus},
    task::{CreateTask, Task, TaskStatus, TaskWithAttemptStatus, UpdateTask},
    workspace::Workspace,
};
use turbomcp::prelude::*;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json;
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// API response struct for Project that matches the NPX backend schema
/// This is separate from db::models::Project to handle schema differences
#[derive(Debug, Clone, Deserialize)]
pub struct ApiProject {
    pub id: Uuid,
    pub name: String,
    pub dev_script: Option<String>,
    pub dev_script_working_dir: Option<String>,
    pub default_agent_working_dir: Option<String>,
    pub remote_project_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

use crate::routes::task_attempts::{
    CreateTaskAttemptBody,
    WorkspaceRepoInput,
    RebaseTaskAttemptRequest as ApiRebaseRequest,
    GitOperationError,
};

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
    #[schemars(description = "Optional development script for the project")]
    pub dev_script: Option<String>,
    #[schemars(description = "Working directory for the development script")]
    pub dev_script_working_dir: Option<String>,
    #[schemars(description = "Default working directory for agents")]
    pub default_agent_working_dir: Option<String>,
    #[schemars(description = "Remote project ID if synced")]
    pub remote_project_id: Option<String>,
    #[schemars(description = "When the project was created")]
    pub created_at: String,
    #[schemars(description = "When the project was last updated")]
    pub updated_at: String,
}

impl ProjectSummary {
    fn from_api_project(project: ApiProject) -> Self {
        Self {
            id: project.id.to_string(),
            name: project.name,
            dev_script: project.dev_script,
            dev_script_working_dir: project.dev_script_working_dir,
            default_agent_working_dir: project.default_agent_working_dir,
            remote_project_id: project.remote_project_id.map(|id| id.to_string()),
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
pub struct GetProjectRequest {
    #[schemars(description = "The unique identifier of the project to retrieve")]
    pub project_id: Uuid,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreateProjectRequest {
    #[schemars(description = "The name of the project")]
    pub name: String,
    #[schemars(description = "Path to the git repository for this project")]
    pub git_repo_path: String,
    #[schemars(description = "If true, use an existing git repository at the path. If false, create a new one.")]
    pub use_existing_repo: bool,
    #[schemars(description = "Optional setup script to run when starting work on the project")]
    pub setup_script: Option<String>,
    #[schemars(description = "Optional development server script (e.g., 'npm run dev')")]
    pub dev_script: Option<String>,
    #[schemars(description = "Optional cleanup script to run after work is complete")]
    pub cleanup_script: Option<String>,
    #[schemars(description = "Optional comma-separated list of files to copy to attempt branches")]
    pub copy_files: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct UpdateProjectRequest {
    #[schemars(description = "The unique identifier of the project to update")]
    pub project_id: Uuid,
    #[schemars(description = "New name for the project")]
    pub name: Option<String>,
    #[schemars(description = "New git repository path")]
    pub git_repo_path: Option<String>,
    #[schemars(description = "New setup script")]
    pub setup_script: Option<String>,
    #[schemars(description = "New development server script")]
    pub dev_script: Option<String>,
    #[schemars(description = "New cleanup script")]
    pub cleanup_script: Option<String>,
    #[schemars(description = "New comma-separated list of files to copy")]
    pub copy_files: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DeleteProjectRequest {
    #[schemars(description = "The unique identifier of the project to delete")]
    pub project_id: Uuid,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetProjectBranchesRequest {
    #[schemars(description = "The unique identifier of the project")]
    pub project_id: Uuid,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct GitBranchInfo {
    #[schemars(description = "Name of the branch")]
    pub name: String,
    #[schemars(description = "Whether this is the currently checked out branch")]
    pub is_current: bool,
    #[schemars(description = "Whether this is a remote branch")]
    pub is_remote: bool,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct GetProjectBranchesResponse {
    #[schemars(description = "List of branches in the project")]
    pub branches: Vec<GitBranchInfo>,
    #[schemars(description = "Total number of branches")]
    pub count: usize,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SearchProjectFilesRequest {
    #[schemars(description = "The unique identifier of the project")]
    pub project_id: Uuid,
    #[schemars(description = "Search query string to match against file names and paths")]
    pub query: String,
    #[schemars(description = "Search mode: 'settings' (includes ignored files) or 'task_form' (respects .gitignore). Default: 'task_form'")]
    pub mode: Option<String>,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct FileSearchResult {
    #[schemars(description = "Relative path to the file or directory")]
    pub path: String,
    #[schemars(description = "Whether this is a file (true) or directory (false)")]
    pub is_file: bool,
    #[schemars(description = "How the match was found: 'FileName', 'DirectoryName', or 'FullPath'")]
    pub match_type: String,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct SearchProjectFilesResponse {
    #[schemars(description = "List of matching files and directories")]
    pub results: Vec<FileSearchResult>,
    #[schemars(description = "Total number of matches")]
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
    #[schemars(description = "Optional search query to filter tasks by title (case-insensitive substring match)")]
    pub search: Option<String>,
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
    #[schemars(description = "Whether the last execution attempt failed")]
    pub last_attempt_failed: Option<bool>,
    #[schemars(description = "The executor used for the task")]
    pub executor: Option<String>,
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
            last_attempt_failed: Some(task.last_attempt_failed),
            executor: Some(task.executor),
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
    #[schemars(description = "Whether the last execution attempt failed")]
    pub last_attempt_failed: Option<bool>,
    #[schemars(description = "The executor used for the task")]
    pub executor: Option<String>,
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
            last_attempt_failed: None,
            executor: None,
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

/// Input for specifying a repository and target branch for a workspace
#[derive(Debug, Clone, Deserialize, schemars::JsonSchema)]
pub struct McpWorkspaceRepoInput {
    #[schemars(description = "The UUID of the repository")]
    pub repo_id: Uuid,
    #[schemars(description = "The target branch for this repository")]
    pub target_branch: String,
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
    #[schemars(description = "List of repositories with target branches for this workspace. Each entry requires repo_id (UUID) and target_branch.")]
    pub repos: Vec<McpWorkspaceRepoInput>,
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
    #[schemars(description = "Include recent attempts in the response (default: false)")]
    pub include_attempts: Option<bool>,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct GetTaskResponse {
    pub task: TaskDetails,
    #[schemars(description = "Recent attempts for this task (if requested)")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attempts: Option<Vec<TaskAttemptSummary>>,
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
    #[schemars(description = "The unique identifier of the attempt (workspace)")]
    pub id: String,
    #[schemars(description = "The task ID this attempt belongs to")]
    pub task_id: String,
    #[schemars(description = "Git branch name for this attempt")]
    pub branch: String,
    #[schemars(description = "Path to worktree or container reference")]
    pub container_ref: Option<String>,
    #[schemars(description = "Working directory for the agent within the worktree")]
    pub agent_working_dir: Option<String>,
    #[schemars(description = "When setup script was completed")]
    pub setup_completed_at: Option<String>,
    #[schemars(description = "When the attempt was created")]
    pub created_at: String,
    #[schemars(description = "When the attempt was last updated")]
    pub updated_at: String,
}

impl TaskAttemptSummary {
    fn from_workspace(workspace: Workspace) -> Self {
        Self {
            id: workspace.id.to_string(),
            task_id: workspace.task_id.to_string(),
            branch: workspace.branch,
            container_ref: workspace.container_ref,
            agent_working_dir: workspace.agent_working_dir,
            setup_completed_at: workspace.setup_completed_at.map(|dt| dt.to_rfc3339()),
            created_at: workspace.created_at.to_rfc3339(),
            updated_at: workspace.updated_at.to_rfc3339(),
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
    #[schemars(description = "Include execution processes in the response (default: false)")]
    pub include_processes: Option<bool>,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct GetTaskAttemptResponse {
    pub attempt: TaskAttemptSummary,
    #[schemars(description = "Execution processes for this attempt (if requested)")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub processes: Option<Vec<ExecutionProcessSummary>>,
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

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct RebaseTaskAttemptRequest {
    #[schemars(description = "The ID of the task attempt to rebase")]
    pub attempt_id: Uuid,
    #[schemars(description = "The ID of the repo to rebase")]
    pub repo_id: Uuid,
    #[schemars(description = "Optional old base branch (defaults to attempt's target branch)")]
    pub old_base_branch: Option<String>,
    #[schemars(description = "Optional new base branch (defaults to attempt's target branch)")]
    pub new_base_branch: Option<String>,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct RebaseTaskAttemptResponse {
    pub success: bool,
    pub message: String,
    pub task_id: String,
    pub attempt_id: String,
    #[schemars(description = "True if there are merge conflicts that need to be resolved")]
    pub has_conflicts: bool,
    #[schemars(description = "Conflict details if present")]
    pub conflict_info: Option<ConflictInfo>,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct ConflictInfo {
    #[schemars(description = "Type of conflict operation (Rebase, Merge, CherryPick, Revert)")]
    pub operation: String,
    #[schemars(description = "Human-readable conflict message")]
    pub message: String,
    #[schemars(description = "List of files with conflicts")]
    pub conflicted_files: Vec<String>,
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
    #[schemars(description = "The session ID this process belongs to")]
    pub session_id: String,
    #[schemars(description = "Why this process was run (e.g., SetupScript, CodingAgent, DevServer)")]
    pub run_reason: String,
    #[schemars(description = "Current execution status (Running, Completed, Failed, Killed)")]
    pub status: String,
    #[schemars(description = "Exit code if the process has completed")]
    pub exit_code: Option<i64>,
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
            session_id: process.session_id.to_string(),
            run_reason: format!("{:?}", process.run_reason),
            status: format!("{:?}", process.status),
            exit_code: process.exit_code,
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
pub struct ReplaceExecutionProcessRequest {
    #[schemars(description = "The ID of the task attempt containing the process")]
    pub attempt_id: Uuid,
    #[schemars(description = "The ID of the execution process to replace (this and all later processes will be deleted)")]
    pub process_id: Uuid,
    #[schemars(description = "The new prompt to use for the replacement execution")]
    pub prompt: String,
    #[schemars(description = "Optional executor variant override")]
    pub variant: Option<String>,
    #[schemars(description = "If true, allow resetting Git even when uncommitted changes exist (default: false)")]
    pub force_when_dirty: Option<bool>,
    #[schemars(description = "If false, skip performing the Git reset step but still drop process history (default: true)")]
    pub perform_git_reset: Option<bool>,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct ReplaceExecutionProcessResponse {
    #[schemars(description = "Whether the operation succeeded")]
    pub success: bool,
    #[schemars(description = "Status message")]
    pub message: String,
    #[schemars(description = "The task attempt ID")]
    pub attempt_id: String,
    #[schemars(description = "Number of execution processes that were deleted (soft-dropped)")]
    pub deleted_count: i64,
    #[schemars(description = "Whether a Git reset was needed to restore the worktree state")]
    pub git_reset_needed: bool,
    #[schemars(description = "Whether the Git reset was actually applied")]
    pub git_reset_applied: bool,
    #[schemars(description = "The commit SHA the worktree was reset to (before the replaced process)")]
    pub target_before_oid: Option<String>,
    #[schemars(description = "The ID of the newly started execution process")]
    pub new_execution_id: Option<String>,
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

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreateGitHubPrRequest {
    #[schemars(description = "The ID of the task attempt to create a PR for")]
    pub attempt_id: Uuid,
    #[schemars(description = "The title of the pull request")]
    pub title: String,
    #[schemars(description = "Optional description/body for the pull request")]
    pub body: Option<String>,
    #[schemars(description = "Optional target branch (defaults to attempt's target branch)")]
    pub target_branch: Option<String>,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct CreateGitHubPrResponse {
    pub success: bool,
    pub pr_url: String,
    pub message: String,
    pub attempt_id: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct PushAttemptBranchRequest {
    #[schemars(description = "The ID of the task attempt to push to remote")]
    pub attempt_id: Uuid,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct PushAttemptBranchResponse {
    pub success: bool,
    pub message: String,
    pub attempt_id: String,
    pub branch: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetBranchStatusRequest {
    #[schemars(description = "The ID of the task attempt to get branch status for")]
    pub attempt_id: Uuid,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct GetBranchStatusResponse {
    #[schemars(description = "The task attempt ID")]
    pub attempt_id: String,
    #[schemars(description = "The target branch this attempt will merge into")]
    pub target_branch: String,
    #[schemars(description = "Number of commits this branch is ahead of target")]
    pub commits_ahead: Option<usize>,
    #[schemars(description = "Number of commits this branch is behind target")]
    pub commits_behind: Option<usize>,
    #[schemars(description = "Overall sync status summary (UpToDate, Ahead, Behind, Diverged, HasConflicts, etc.)")]
    pub sync_status: String,
    #[schemars(description = "Whether there are uncommitted changes in the worktree")]
    pub has_uncommitted_changes: Option<bool>,
    #[schemars(description = "Number of uncommitted file changes")]
    pub uncommitted_count: Option<usize>,
    #[schemars(description = "Number of untracked files")]
    pub untracked_count: Option<usize>,
    #[schemars(description = "Current HEAD commit SHA")]
    pub head_commit: Option<String>,
    #[schemars(description = "Commits ahead of remote (only if PR is open)")]
    pub remote_commits_ahead: Option<usize>,
    #[schemars(description = "Commits behind remote (only if PR is open)")]
    pub remote_commits_behind: Option<usize>,
    #[schemars(description = "Whether a rebase operation is in progress")]
    pub is_rebase_in_progress: bool,
    #[schemars(description = "Whether there are merge conflicts")]
    pub has_conflicts: bool,
    #[schemars(description = "Type of operation that caused conflicts (if any)")]
    pub conflict_operation: Option<String>,
    #[schemars(description = "List of files with conflicts (if any)")]
    pub conflicted_files: Option<Vec<String>>,
    #[schemars(description = "Suggested actions based on current status")]
    pub suggested_actions: Vec<String>,
}

/// Determine the overall sync status based on branch state
fn determine_sync_status(
    commits_ahead: Option<usize>,
    commits_behind: Option<usize>,
    has_uncommitted: Option<bool>,
    is_rebasing: bool,
    has_conflicts: bool,
) -> String {
    if has_conflicts {
        return "HasConflicts".to_string();
    }

    if is_rebasing {
        return "RebaseInProgress".to_string();
    }

    let ahead = commits_ahead.unwrap_or(0);
    let behind = commits_behind.unwrap_or(0);
    let dirty = has_uncommitted.unwrap_or(false);

    match (ahead, behind, dirty) {
        (0, 0, false) => "UpToDate".to_string(),
        (0, 0, true) => "UpToDateWithUncommittedChanges".to_string(),
        (a, 0, false) if a > 0 => "Ahead".to_string(),
        (a, 0, true) if a > 0 => "AheadWithUncommittedChanges".to_string(),
        (0, b, false) if b > 0 => "Behind".to_string(),
        (0, b, true) if b > 0 => "BehindWithUncommittedChanges".to_string(),
        (a, b, false) if a > 0 && b > 0 => "Diverged".to_string(),
        (a, b, true) if a > 0 && b > 0 => "DivergedWithUncommittedChanges".to_string(),
        _ => "Unknown".to_string(),
    }
}

/// Suggest actions based on branch status
fn suggest_actions(
    commits_ahead: Option<usize>,
    commits_behind: Option<usize>,
    has_uncommitted: Option<bool>,
    is_rebasing: bool,
    has_conflicts: bool,
    remote_behind: Option<usize>,
) -> Vec<String> {
    let mut actions = Vec::new();

    if has_conflicts {
        actions.push("Resolve conflicts in the conflicted files".to_string());
        actions.push("Use 'abort_conflicts_task_attempt' to abort the operation if needed".to_string());
        return actions;
    }

    if is_rebasing {
        actions.push("Complete or abort the rebase operation in progress".to_string());
        return actions;
    }

    let ahead = commits_ahead.unwrap_or(0);
    let behind = commits_behind.unwrap_or(0);
    let dirty = has_uncommitted.unwrap_or(false);

    if dirty {
        actions.push("Commit or stash uncommitted changes".to_string());
    }

    if behind > 0 {
        actions.push(format!("Rebase onto target branch to sync {} commit(s)", behind));
        actions.push("Use 'rebase_task_attempt' tool to update your branch".to_string());
    }

    if ahead > 0 && behind == 0 && !dirty {
        if remote_behind.unwrap_or(0) > 0 {
            actions.push("Push changes to remote using 'push_attempt_branch'".to_string());
        }
        actions.push("Branch is ready to merge or create a PR".to_string());
        actions.push("Use 'create_github_pr' to create a pull request".to_string());
    }

    if ahead == 0 && behind == 0 && !dirty {
        actions.push("Branch is up to date with target".to_string());
    }

    if actions.is_empty() {
        actions.push("No immediate actions needed".to_string());
    }

    actions
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetAttemptCommitsRequest {
    #[schemars(description = "The ID of the task attempt to get commits for")]
    pub attempt_id: Uuid,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct CommitDetails {
    #[schemars(description = "Full commit SHA")]
    pub sha: String,
    #[schemars(description = "Commit message (first line)")]
    pub message: String,
    #[schemars(description = "Author name")]
    pub author_name: Option<String>,
    #[schemars(description = "Author email")]
    pub author_email: Option<String>,
    #[schemars(description = "Commit timestamp (ISO 8601)")]
    pub timestamp: Option<String>,
    #[schemars(description = "Number of files changed")]
    pub files_changed: Option<usize>,
    #[schemars(description = "Number of lines added")]
    pub additions: Option<usize>,
    #[schemars(description = "Number of lines deleted")]
    pub deletions: Option<usize>,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct GetAttemptCommitsResponse {
    pub attempt_id: String,
    pub commits: Vec<CommitDetails>,
    pub total_count: usize,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CompareCommitToHeadRequest {
    #[schemars(description = "The ID of the task attempt")]
    pub attempt_id: Uuid,
    #[schemars(description = "The commit SHA to compare against")]
    pub commit_sha: String,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct CompareCommitToHeadResponse {
    #[schemars(description = "Current HEAD commit SHA")]
    pub head_oid: String,
    #[schemars(description = "Target commit SHA being compared")]
    pub target_oid: String,
    #[schemars(description = "Number of commits HEAD is ahead of target")]
    pub ahead_from_head: usize,
    #[schemars(description = "Number of commits HEAD is behind target")]
    pub behind_from_head: usize,
    #[schemars(description = "Whether the history is linear (can fast-forward)")]
    pub is_linear: bool,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct AbortConflictsRequest {
    #[schemars(description = "The ID of the task attempt with conflicts to abort")]
    pub attempt_id: Uuid,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct AbortConflictsResponse {
    #[schemars(description = "Whether the operation succeeded")]
    pub success: bool,
    #[schemars(description = "Status message")]
    pub message: String,
    #[schemars(description = "The attempt ID")]
    pub attempt_id: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ChangeTargetBranchRequest {
    #[schemars(description = "The ID of the task attempt to update")]
    pub attempt_id: Uuid,
    #[schemars(description = "The new target branch name (must exist in the repository)")]
    pub new_target_branch: String,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct ChangeTargetBranchResponse {
    #[schemars(description = "Whether the operation succeeded")]
    pub success: bool,
    #[schemars(description = "Status message")]
    pub message: String,
    #[schemars(description = "The attempt ID")]
    pub attempt_id: String,
    #[schemars(description = "The new target branch")]
    pub new_target_branch: String,
    #[schemars(description = "Number of commits ahead of target")]
    pub commits_ahead: usize,
    #[schemars(description = "Number of commits behind target")]
    pub commits_behind: usize,
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

    /// Send a request that doesn't expect data in the response (e.g., DELETE operations)
    /// Returns Ok(()) on success, Err on failure
    async fn send_no_data(&self, rb: reqwest::RequestBuilder) -> Result<(), McpError> {
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

        // Parse response to check success field, but ignore data
        let api_response = resp
            .json::<ApiResponseEnvelope<serde_json::Value>>()
            .await
            .map_err(|e| Self::err_str("Failed to parse VK API response", Some(&e.to_string())))?;

        if !api_response.success {
            let msg = api_response.message.as_deref().unwrap_or("Unknown error");
            return Err(Self::err_str("VK API returned error", Some(msg)));
        }

        Ok(())
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
    description = "A task and project management server. If you need to create or update tickets or tasks then use these tools. Most of them absolutely require that you pass the `project_id` of the project that you are currently working on. This should be provided to you. Call `list_tasks` to fetch the `task_ids` of all the tasks in a project. TOOLS: 'list_projects', 'get_project', 'create_project', 'update_project', 'delete_project', 'get_project_branches', 'search_project_files', 'list_tasks', 'create_task', 'start_task_attempt', 'get_task', 'update_task', 'delete_task', 'list_task_attempts', 'get_task_attempt', 'create_followup_attempt', 'merge_task_attempt', 'get_branch_status', 'get_attempt_commits', 'compare_commit_to_head', 'abort_conflicts', 'list_execution_processes', 'get_execution_process', 'stop_execution_process', 'replace_execution_process', 'get_process_raw_logs', 'get_process_normalized_logs', 'start_dev_server', 'create_github_pr', 'push_attempt_branch', 'rebase_task_attempt', 'get_attempt_artifacts', 'change_target_branch'. Make sure to pass `project_id` or `task_id` where required. You can use list tools to get the available ids."
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
        let projects: Vec<ApiProject> = self.send_json(self.client.get(&url)).await?;

        let project_summaries: Vec<ProjectSummary> = projects
            .into_iter()
            .map(ProjectSummary::from_api_project)
            .collect();

        let response = ListProjectsResponse {
            count: project_summaries.len(),
            projects: project_summaries,
        };

        Ok(serde_json::to_string_pretty(&response).unwrap())
    }

    #[tool(description = "Get details of a specific project by its ID")]
    async fn get_project(&self, request: GetProjectRequest) -> McpResult<String> {
        let url = self.url(&format!("/api/projects/{}", request.project_id));
        let project: ApiProject = self.send_json(self.client.get(&url)).await?;

        let project_summary = ProjectSummary::from_api_project(project);
        Ok(serde_json::to_string_pretty(&project_summary).unwrap())
    }

    #[tool(description = "Create a new project. Requires a name and git repository path. Set use_existing_repo=true to use an existing git repo, or false to initialize a new one.")]
    async fn create_project(&self, request: CreateProjectRequest) -> McpResult<String> {
        let url = self.url("/api/projects");
        
        let body = serde_json::json!({
            "name": request.name,
            "git_repo_path": request.git_repo_path,
            "use_existing_repo": request.use_existing_repo,
            "setup_script": request.setup_script,
            "dev_script": request.dev_script,
            "cleanup_script": request.cleanup_script,
            "copy_files": request.copy_files,
        });

        let project: ApiProject = self.send_json(
            self.client.post(&url).json(&body)
        ).await?;

        let project_summary = ProjectSummary::from_api_project(project);
        Ok(serde_json::to_string_pretty(&project_summary).unwrap())
    }

    #[tool(description = "Update an existing project's configuration. Only provided fields will be updated.")]
    async fn update_project(&self, request: UpdateProjectRequest) -> McpResult<String> {
        let url = self.url(&format!("/api/projects/{}", request.project_id));
        
        let body = serde_json::json!({
            "name": request.name,
            "git_repo_path": request.git_repo_path,
            "setup_script": request.setup_script,
            "dev_script": request.dev_script,
            "cleanup_script": request.cleanup_script,
            "copy_files": request.copy_files,
        });

        let project: ApiProject = self.send_json(
            self.client.put(&url).json(&body)
        ).await?;

        let project_summary = ProjectSummary::from_api_project(project);
        Ok(serde_json::to_string_pretty(&project_summary).unwrap())
    }

    #[tool(description = "Delete a project. This removes the project from tracking but does not delete the git repository.")]
    async fn delete_project(&self, request: DeleteProjectRequest) -> McpResult<String> {
        let url = self.url(&format!("/api/projects/{}", request.project_id));
        
        let response = self.client.delete(&url).send().await
            .map_err(|e| McpError::internal(format!("Request failed: {}", e)))?;

        if response.status().is_success() {
            Ok(serde_json::json!({
                "success": true,
                "message": format!("Project {} deleted successfully", request.project_id)
            }).to_string())
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            Err(McpError::internal(format!("Delete failed with status {}: {}", status, body)))
        }
    }

    #[tool(description = "List all git branches in a project's repository")]
    async fn get_project_branches(&self, request: GetProjectBranchesRequest) -> McpResult<String> {
        let url = self.url(&format!("/api/projects/{}/branches", request.project_id));
        
        #[derive(Debug, Deserialize)]
        struct ApiBranch {
            name: String,
            is_current: bool,
            is_remote: bool,
        }

        let branches: Vec<ApiBranch> = self.send_json(self.client.get(&url)).await?;

        let branch_infos: Vec<GitBranchInfo> = branches
            .into_iter()
            .map(|b| GitBranchInfo {
                name: b.name,
                is_current: b.is_current,
                is_remote: b.is_remote,
            })
            .collect();

        let response = GetProjectBranchesResponse {
            count: branch_infos.len(),
            branches: branch_infos,
        };

        Ok(serde_json::to_string_pretty(&response).unwrap())
    }

    #[tool(description = "Search for files in a project's repository. Returns matching file and directory paths ranked by relevance.")]
    async fn search_project_files(&self, request: SearchProjectFilesRequest) -> McpResult<String> {
        let mode = request.mode.as_deref().unwrap_or("task_form");
        let url = self.url(&format!(
            "/api/projects/{}/search?q={}&mode={}",
            request.project_id,
            urlencoding::encode(&request.query),
            mode
        ));

        #[derive(Debug, Deserialize)]
        struct ApiSearchResult {
            path: String,
            is_file: bool,
            match_type: String,
        }

        let results: Vec<ApiSearchResult> = self.send_json(self.client.get(&url)).await?;

        let file_results: Vec<FileSearchResult> = results
            .into_iter()
            .map(|r| FileSearchResult {
                path: r.path,
                is_file: r.is_file,
                match_type: r.match_type,
            })
            .collect();

        let response = SearchProjectFilesResponse {
            count: file_results.len(),
            results: file_results,
        };

        Ok(serde_json::to_string_pretty(&response).unwrap())
    }

    #[tool(
        description = "List all the task/tickets in a project with optional filtering by status and search. `project_id` is required!"
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

        // Normalize search query for case-insensitive matching
        let search_lower = request.search.as_ref().map(|s| s.to_lowercase());

        let url = self.url(&format!("/api/tasks?project_id={}", request.project_id));
        let all_tasks: Vec<TaskWithAttemptStatus> =
            self.send_json(self.client.get(&url)).await?;

        let task_limit = request.limit.unwrap_or(50).max(0) as usize;
        let filtered = all_tasks.into_iter().filter(|t| {
            // Apply status filter
            let status_matches = if let Some(ref want) = status_filter {
                &t.status == want
            } else {
                true
            };
            // Apply search filter (case-insensitive substring match on title)
            let search_matches = if let Some(ref query) = search_lower {
                t.title.to_lowercase().contains(query)
            } else {
                true
            };
            status_matches && search_matches
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
        // Validate repos array
        if request.repos.is_empty() {
            return Err(McpError::invalid_request("At least one repository must be specified in the 'repos' array."));
        }

        // Validate each repo entry
        for (i, repo) in request.repos.iter().enumerate() {
            let target_branch = repo.target_branch.trim();
            if target_branch.is_empty() {
                return Err(McpError::invalid_request(format!(
                    "target_branch must not be empty for repo at index {}",
                    i
                )));
            }
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

        // Convert MCP repo inputs to backend WorkspaceRepoInput
        let repos: Vec<WorkspaceRepoInput> = request.repos.iter().map(|r| WorkspaceRepoInput {
            repo_id: r.repo_id,
            target_branch: r.target_branch.trim().to_string(),
        }).collect();

        let payload = CreateTaskAttemptBody {
            task_id: request.task_id,
            executor_profile_id: backend_executor_profile_id,
            repos,
        };

        let url = self.url("/api/task-attempts");
        let workspace: Workspace = self.send_json(self.client.post(&url).json(&payload)).await?;

        let response = StartTaskAttemptResponse {
            task_id: workspace.task_id.to_string(),
            attempt_id: workspace.id.to_string(),
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
            parent_workspace_id: None,
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
        self.send_no_data(self.client.delete(&url)).await?;

        let response = DeleteTaskResponse {
            deleted_task_id: Some(request.task_id.to_string()),
        };

        Ok(serde_json::to_string_pretty(&response).unwrap())
    }

    #[tool(
        description = "Get detailed information (like task description) about a specific task/ticket. Optionally include recent attempts. `task_id` is required!"
    )]
    async fn get_task(&self, request: GetTaskRequest) -> McpResult<String> {
        let url = self.url(&format!("/api/tasks/{}", request.task_id));
        let task: Task = self.send_json(self.client.get(&url)).await?;

        let details = TaskDetails::from_task(task);
        
        // Optionally fetch attempts
        let attempts = if request.include_attempts.unwrap_or(false) {
            let attempts_url = self.url(&format!("/api/task-attempts?task_id={}", request.task_id));
            let workspaces: Vec<Workspace> = self.send_json(self.client.get(&attempts_url)).await?;
            Some(workspaces.into_iter().map(TaskAttemptSummary::from_workspace).collect())
        } else {
            None
        };
        
        let response = GetTaskResponse { task: details, attempts };

        Ok(serde_json::to_string_pretty(&response).unwrap())
    }

    #[tool(
        description = "List all execution attempts for a specific task. Shows what was tried, branch names, executors used, and timestamps. Useful for understanding task history and debugging failed attempts. `task_id` is required!"
    )]
    async fn list_task_attempts(&self, request: ListTaskAttemptsRequest) -> McpResult<String> {
        let url = self.url(&format!("/api/task-attempts?task_id={}", request.task_id));
        let workspaces: Vec<Workspace> = self.send_json(self.client.get(&url)).await?;

        let attempt_summaries: Vec<TaskAttemptSummary> = workspaces
            .into_iter()
            .map(TaskAttemptSummary::from_workspace)
            .collect();

        let response = ListTaskAttemptsResponse {
            count: attempt_summaries.len(),
            attempts: attempt_summaries,
            task_id: request.task_id.to_string(),
        };

        Ok(serde_json::to_string_pretty(&response).unwrap())
    }

    #[tool(
        description = "Get detailed information about a specific task attempt including branch, executor, timestamps, and worktree status. Optionally include execution processes. `attempt_id` is required!"
    )]
    async fn get_task_attempt(&self, request: GetTaskAttemptRequest) -> McpResult<String> {
        let url = self.url(&format!("/api/task-attempts/{}", request.attempt_id));
        let workspace: Workspace = self.send_json(self.client.get(&url)).await?;

        let attempt_summary = TaskAttemptSummary::from_workspace(workspace);
        
        // Optionally fetch execution processes
        let processes = if request.include_processes.unwrap_or(false) {
            let processes_url = self.url(&format!(
                "/api/execution-processes?task_attempt_id={}",
                request.attempt_id
            ));
            let procs: Vec<ExecutionProcess> = self.send_json(self.client.get(&processes_url)).await?;
            Some(procs.into_iter().map(ExecutionProcessSummary::from_execution_process).collect())
        } else {
            None
        };
        
        let response = GetTaskAttemptResponse {
            attempt: attempt_summary,
            processes,
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

        let workspace: Workspace = self.send_json(self.client.post(&url).json(&payload)).await?;

        let response = CreateFollowupAttemptResponse {
            task_id: workspace.task_id.to_string(),
            attempt_id: workspace.id.to_string(),
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
        let attempt: Workspace = self.send_json(self.client.get(&attempt_url)).await?;

        let response = MergeTaskAttemptResponse {
            success: true,
            message: "Task attempt merged successfully".to_string(),
            task_id: attempt.task_id.to_string(),
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

    #[tool(
        description = "Create a GitHub pull request for a completed task attempt. The PR will be created from the attempt's branch to the target branch, with the task details included in the PR description. Returns the PR URL on success. `attempt_id` and `title` are required!"
    )]
    async fn create_github_pr(&self, request: CreateGitHubPrRequest) -> McpResult<String> {
        let url = self.url(&format!("/api/task-attempts/{}/pr", request.attempt_id));

        #[derive(Serialize)]
        struct PrPayload {
            title: String,
            body: Option<String>,
            target_branch: Option<String>,
        }

        let payload = PrPayload {
            title: request.title.clone(),
            body: request.body.clone(),
            target_branch: request.target_branch.clone(),
        };

        // POST to PR endpoint returns ApiResponse<String> where String is the PR URL
        let pr_url: String = self.send_json(self.client.post(&url).json(&payload)).await?;

        let response = CreateGitHubPrResponse {
            success: true,
            pr_url: pr_url.clone(),
            message: format!("GitHub PR created successfully: {}", pr_url),
            attempt_id: request.attempt_id.to_string(),
        };

        Ok(serde_json::to_string_pretty(&response).unwrap())
    }

    #[tool(
        description = "Rebase a task attempt branch onto the latest target branch. This updates the attempt with the latest changes from the target branch. Detects and reports any merge conflicts that need manual resolution. Use the `old_base_branch` and `new_base_branch` parameters to rebase onto a different branch. `attempt_id` is required!"
    )]
    async fn rebase_task_attempt(&self, request: RebaseTaskAttemptRequest) -> McpResult<String> {
        // Prepare the rebase payload
        let payload = ApiRebaseRequest {
            repo_id: request.repo_id,
            old_base_branch: request.old_base_branch,
            new_base_branch: request.new_base_branch,
        };

        let url = self.url(&format!("/api/task-attempts/{}/rebase", request.attempt_id));

        // Define response structure that matches the API's error-with-data pattern
        #[derive(Debug, Deserialize)]
        struct ApiRebaseResponse {
            success: bool,
            data: Option<GitOperationError>,
            message: Option<String>,
        }

        // Make the rebase request
        let resp = self.client.post(&url).json(&payload).send().await
            .map_err(|e| Self::err_str("Failed to connect to VK API", Some(&e.to_string())))?;

        let status_code = resp.status();
        let api_response = resp.json::<ApiRebaseResponse>().await
            .map_err(|e| Self::err_str("Failed to parse VK API response", Some(&e.to_string())))?;

        // Fetch the task attempt to get task_id for response
        let attempt_url = self.url(&format!("/api/task-attempts/{}", request.attempt_id));
        let attempt: Workspace = self.send_json(self.client.get(&attempt_url)).await?;

        // Check if rebase succeeded or encountered conflicts
        let (success, has_conflicts, conflict_info, message) = if status_code.is_success() && api_response.success {
            // Rebase succeeded
            (true, false, None, "Task attempt rebased successfully".to_string())
        } else if let Some(git_error) = api_response.data {
            // Rebase encountered conflicts or other git errors
            match git_error {
                GitOperationError::MergeConflicts { message: conflict_msg, op } => {
                    let operation = format!("{:?}", op);
                    let conflict_info = ConflictInfo {
                        operation,
                        message: conflict_msg.clone(),
                        conflicted_files: vec![], // API doesn't return files directly in rebase response
                    };
                    (false, true, Some(conflict_info), conflict_msg)
                }
                GitOperationError::RebaseInProgress => {
                    (false, true, Some(ConflictInfo {
                        operation: "Rebase".to_string(),
                        message: "A rebase is already in progress. Please complete or abort the current rebase first.".to_string(),
                        conflicted_files: vec![],
                    }), "Rebase already in progress".to_string())
                }
            }
        } else {
            // Unknown error
            let msg = api_response.message.unwrap_or_else(|| "Unknown error during rebase".to_string());
            return Err(Self::err_str("Rebase failed", Some(&msg)));
        };

        let response = RebaseTaskAttemptResponse {
            success,
            message,
            task_id: attempt.task_id.to_string(),
            attempt_id: request.attempt_id.to_string(),
            has_conflicts,
            conflict_info,
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
        description = "Replace an execution process by deleting it and all later processes, resetting the Git worktree to the state before that process, and starting a new execution with the given prompt. Useful for retrying a failed execution from a clean state or trying a different approach. `attempt_id`, `process_id`, and `prompt` are required!"
    )]
    async fn replace_execution_process(&self, request: ReplaceExecutionProcessRequest) -> McpResult<String> {
        let url = self.url(&format!("/api/task-attempts/{}/replace-process", request.attempt_id));

        #[derive(Serialize)]
        struct ReplacePayload {
            process_id: Uuid,
            prompt: String,
            variant: Option<String>,
            force_when_dirty: Option<bool>,
            perform_git_reset: Option<bool>,
        }

        let payload = ReplacePayload {
            process_id: request.process_id,
            prompt: request.prompt.clone(),
            variant: request.variant.clone(),
            force_when_dirty: request.force_when_dirty,
            perform_git_reset: request.perform_git_reset,
        };

        // Define response structure matching the API
        #[derive(Debug, Deserialize)]
        struct ApiReplaceResult {
            deleted_count: i64,
            git_reset_needed: bool,
            git_reset_applied: bool,
            target_before_oid: Option<String>,
            new_execution_id: Option<Uuid>,
        }

        let api_response: ApiReplaceResult = self
            .send_json(self.client.post(&url).json(&payload))
            .await?;

        let response = ReplaceExecutionProcessResponse {
            success: true,
            message: format!(
                "Replaced execution process. Deleted {} process(es), started new execution.",
                api_response.deleted_count
            ),
            attempt_id: request.attempt_id.to_string(),
            deleted_count: api_response.deleted_count,
            git_reset_needed: api_response.git_reset_needed,
            git_reset_applied: api_response.git_reset_applied,
            target_before_oid: api_response.target_before_oid,
            new_execution_id: api_response.new_execution_id.map(|id| id.to_string()),
        };

        Ok(serde_json::to_string_pretty(&response).unwrap())
    }

    #[tool(
        description = "List all execution processes for a task attempt. Returns process history with status, runtime metrics, and git commits. Optionally include soft-deleted processes. `task_attempt_id` is required!"
    )]
    async fn list_execution_processes(&self, request: ListExecutionProcessesRequest) -> McpResult<String> {
        let mut url = self.url("/api/execution-processes");
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
        description = "Get the git branch synchronization status for a task attempt. Shows how many commits the attempt branch is ahead/behind the target branch, uncommitted changes, conflict status, and remote sync information if a PR is open. Useful for understanding if a branch needs rebasing or is ready to merge. `attempt_id` is required!"
    )]
    async fn get_branch_status(&self, request: GetBranchStatusRequest) -> McpResult<String> {
        let url = self.url(&format!("/api/task-attempts/{}/branch-status", request.attempt_id));

        // Define local response type matching the API
        #[derive(Debug, Deserialize)]
        struct ApiBranchStatus {
            commits_behind: Option<usize>,
            commits_ahead: Option<usize>,
            has_uncommitted_changes: Option<bool>,
            head_oid: Option<String>,
            uncommitted_count: Option<usize>,
            untracked_count: Option<usize>,
            target_branch_name: String,
            remote_commits_behind: Option<usize>,
            remote_commits_ahead: Option<usize>,
            is_rebase_in_progress: bool,
            conflict_op: Option<String>,
            conflicted_files: Vec<String>,
        }

        let api_response: ApiBranchStatus = self.send_json(self.client.get(&url)).await?;

        // Check conflict status before moving conflicted_files
        let has_conflicts = !api_response.conflicted_files.is_empty();

        // Convert to MCP response format
        let response = GetBranchStatusResponse {
            attempt_id: request.attempt_id.to_string(),
            target_branch: api_response.target_branch_name,
            commits_ahead: api_response.commits_ahead,
            commits_behind: api_response.commits_behind,
            sync_status: determine_sync_status(
                api_response.commits_ahead,
                api_response.commits_behind,
                api_response.has_uncommitted_changes,
                api_response.is_rebase_in_progress,
                has_conflicts,
            ),
            has_uncommitted_changes: api_response.has_uncommitted_changes,
            uncommitted_count: api_response.uncommitted_count,
            untracked_count: api_response.untracked_count,
            head_commit: api_response.head_oid,
            remote_commits_ahead: api_response.remote_commits_ahead,
            remote_commits_behind: api_response.remote_commits_behind,
            is_rebase_in_progress: api_response.is_rebase_in_progress,
            has_conflicts,
            conflict_operation: api_response.conflict_op,
            conflicted_files: if api_response.conflicted_files.is_empty() {
                None
            } else {
                Some(api_response.conflicted_files)
            },
            suggested_actions: suggest_actions(
                api_response.commits_ahead,
                api_response.commits_behind,
                api_response.has_uncommitted_changes,
                api_response.is_rebase_in_progress,
                has_conflicts,
                api_response.remote_commits_behind,
            ),
        };

        Ok(serde_json::to_string_pretty(&response).unwrap())
    }

    #[tool(
        description = "Get all commits for a task attempt with detailed metadata and diff statistics. Returns commit messages, authors, timestamps, and change statistics (files changed, additions, deletions). Useful for reviewing what changes were made during an attempt. `attempt_id` is required!"
    )]
    async fn get_attempt_commits(&self, request: GetAttemptCommitsRequest) -> McpResult<String> {
        let url = self.url(&format!("/api/task-attempts/{}/commits", request.attempt_id));

        // Define local response type matching the API
        #[derive(Debug, Deserialize)]
        struct ApiCommitDetails {
            sha: String,
            message: String,
            author_name: Option<String>,
            author_email: Option<String>,
            timestamp: Option<String>,
            files_changed: Option<usize>,
            additions: Option<usize>,
            deletions: Option<usize>,
        }

        #[derive(Debug, Deserialize)]
        struct ApiCommitsResponse {
            attempt_id: String,
            commits: Vec<ApiCommitDetails>,
            total_count: usize,
        }

        let api_response: ApiCommitsResponse = self.send_json(self.client.get(&url)).await?;

        // Convert to MCP response format
        let commits: Vec<CommitDetails> = api_response
            .commits
            .into_iter()
            .map(|commit| CommitDetails {
                sha: commit.sha,
                message: commit.message,
                author_name: commit.author_name,
                author_email: commit.author_email,
                timestamp: commit.timestamp,
                files_changed: commit.files_changed,
                additions: commit.additions,
                deletions: commit.deletions,
            })
            .collect();

        let response = GetAttemptCommitsResponse {
            attempt_id: api_response.attempt_id,
            commits,
            total_count: api_response.total_count,
        };

        Ok(serde_json::to_string_pretty(&response).unwrap())
    }

    #[tool(
        description = "Compare a commit SHA to the current HEAD of an attempt branch. Returns how many commits ahead and behind, and whether the history is linear. Useful for understanding if a commit can be fast-forwarded or needs rebasing. `attempt_id` and `commit_sha` are required!"
    )]
    async fn compare_commit_to_head(&self, request: CompareCommitToHeadRequest) -> McpResult<String> {
        let url = self.url(&format!(
            "/api/task-attempts/{}/commit-compare?sha={}",
            request.attempt_id, request.commit_sha
        ));

        // Define local response type matching the API
        #[derive(Debug, Deserialize)]
        struct ApiCompareResult {
            head_oid: String,
            target_oid: String,
            ahead_from_head: usize,
            behind_from_head: usize,
            is_linear: bool,
        }

        let api_response: ApiCompareResult = self.send_json(self.client.get(&url)).await?;

        let response = CompareCommitToHeadResponse {
            head_oid: api_response.head_oid,
            target_oid: api_response.target_oid,
            ahead_from_head: api_response.ahead_from_head,
            behind_from_head: api_response.behind_from_head,
            is_linear: api_response.is_linear,
        };

        Ok(serde_json::to_string_pretty(&response).unwrap())
    }

    #[tool(
        description = "Abort an ongoing merge or rebase operation on an attempt branch. This restores the worktree to a clean state by aborting any conflicts. Use this when you want to cancel a conflicted merge/rebase operation. `attempt_id` is required!"
    )]
    async fn abort_conflicts(&self, request: AbortConflictsRequest) -> McpResult<String> {
        let url = self.url(&format!("/api/task-attempts/{}/conflicts/abort", request.attempt_id));

        // POST to abort endpoint returns ApiResponse<()>
        self.send_json::<serde_json::Value>(self.client.post(&url)).await?;

        let response = AbortConflictsResponse {
            success: true,
            message: "Successfully aborted conflicts and restored clean state".to_string(),
            attempt_id: request.attempt_id.to_string(),
        };

        Ok(serde_json::to_string_pretty(&response).unwrap())
    }

    #[tool(
        description = "Push a task attempt's branch to the remote GitHub repository. This validates GitHub authentication, ensures the worktree is clean, and pushes all commits to remote. Use this before creating a pull request. `attempt_id` is required!"
    )]
    async fn push_attempt_branch(&self, request: PushAttemptBranchRequest) -> McpResult<String> {
        let url = self.url(&format!("/api/task-attempts/{}/push", request.attempt_id));

        // POST to push endpoint returns ApiResponse<()>
        self.send_json::<serde_json::Value>(self.client.post(&url)).await?;

        // Fetch the task attempt to get branch name for response
        let attempt_url = self.url(&format!("/api/task-attempts/{}", request.attempt_id));
        let attempt: Workspace = self.send_json(self.client.get(&attempt_url)).await?;

        let response = PushAttemptBranchResponse {
            success: true,
            message: "Branch pushed to remote successfully".to_string(),
            attempt_id: request.attempt_id.to_string(),
            branch: attempt.branch,
        };

        Ok(serde_json::to_string_pretty(&response).unwrap())
    }

    #[tool(
        description = "Change the target branch for a task attempt. This updates which branch the attempt will be merged into. The new target branch must exist in the repository. Returns the new branch status (commits ahead/behind). `attempt_id` and `new_target_branch` are required!"
    )]
    async fn change_target_branch(&self, request: ChangeTargetBranchRequest) -> McpResult<String> {
        let url = self.url(&format!("/api/task-attempts/{}/change-target-branch", request.attempt_id));

        #[derive(Serialize)]
        struct Payload {
            new_target_branch: String,
        }

        let payload = Payload {
            new_target_branch: request.new_target_branch.clone(),
        };

        // Define response structure matching the API
        #[derive(Debug, Deserialize)]
        struct ApiChangeTargetBranchResponse {
            new_target_branch: String,
            status: (usize, usize),
        }

        let api_response: ApiChangeTargetBranchResponse = self
            .send_json(self.client.post(&url).json(&payload))
            .await?;

        let response = ChangeTargetBranchResponse {
            success: true,
            message: format!("Target branch changed to '{}'", api_response.new_target_branch),
            attempt_id: request.attempt_id.to_string(),
            new_target_branch: api_response.new_target_branch,
            commits_ahead: api_response.status.0,
            commits_behind: api_response.status.1,
        };

        Ok(serde_json::to_string_pretty(&response).unwrap())
    }
}

// Custom HTTP runner implementation with permissive security for development
#[cfg(feature = "http")]
impl TaskServer {
    /// Run HTTP server with custom security configuration
    pub async fn run_http_custom(self, addr: &str) -> Result<(), Box<dyn std::error::Error>> {
        use turbomcp_transport::streamable_http::{StreamableHttpConfigBuilder};
        use std::time::Duration;

        // Create permissive HTTP config for development
        let config = StreamableHttpConfigBuilder::new()
            .with_bind_address(addr)
            .allow_any_origin(true) // Allow any origin in development mode
            .allow_localhost(true)
            .with_rate_limit(1_000_000, Duration::from_secs(60)) // Very high limit for development
            .build();

        // Run the HTTP server with custom config (v2.3 API uses method on server)
        self.run_http_with_config(addr, config).await?;
        Ok(())
    }
}
