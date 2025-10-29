use axum::{
    extract::{Query, State},
    response::Json as ResponseJson,
    Extension,
};
use db::models::{
    execution_process::ExecutionProcess,
    execution_process_logs::ExecutionProcessLogs,
    task_attempt::TaskAttempt,
};
use deployment::Deployment;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utils::response::ApiResponse;

use crate::{error::ApiError, routes::task_attempts::util::ensure_worktree_path, DeploymentImpl};

/// Type of artifact
#[derive(Debug, Clone, Serialize, Deserialize, TS, schemars::JsonSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[ts(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ArtifactType {
    GitDiff,
    GitCommit,
    ExecutionLog,
}

/// Individual artifact from an attempt
#[derive(Debug, Clone, Serialize, Deserialize, TS, schemars::JsonSchema)]
pub struct Artifact {
    /// Type of artifact
    pub artifact_type: ArtifactType,
    /// Execution process ID this artifact came from
    pub process_id: String,
    /// Content of the artifact (diff text, commit message, log lines, etc.)
    pub content: Option<String>,
    /// Size in bytes
    pub size_bytes: usize,
    /// Git commit SHA (for commit artifacts)
    pub commit_sha: Option<String>,
    /// Git commit subject/message (for commit artifacts)
    pub commit_subject: Option<String>,
    /// Before commit SHA (for diff artifacts)
    pub before_commit: Option<String>,
    /// After commit SHA (for diff artifacts)
    pub after_commit: Option<String>,
}

/// Query parameters for filtering artifacts
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ArtifactFilters {
    /// Filter by artifact type
    pub artifact_type: Option<ArtifactType>,
    /// Maximum number of artifacts to return
    pub limit: Option<usize>,
    /// Offset for pagination
    pub offset: Option<usize>,
}

/// Response containing attempt artifacts
#[derive(Debug, Serialize, Deserialize, TS, schemars::JsonSchema)]
pub struct AttemptArtifactsResponse {
    pub attempt_id: String,
    pub artifacts: Vec<Artifact>,
    pub total_count: usize,
}

/// Get all artifacts for a task attempt
pub async fn get_attempt_artifacts(
    Extension(task_attempt): Extension<TaskAttempt>,
    Query(filters): Query<ArtifactFilters>,
    State(deployment): State<DeploymentImpl>,
) -> Result<ResponseJson<ApiResponse<AttemptArtifactsResponse>>, ApiError> {
    let pool = &deployment.db().pool;

    // Fetch all execution processes for this attempt (excluding dropped ones)
    let execution_processes =
        ExecutionProcess::find_by_task_attempt_id(pool, task_attempt.id, false).await?;

    let mut artifacts = Vec::new();

    // Try to get worktree path for git operations
    let worktree_path = match ensure_worktree_path(&deployment, &task_attempt).await {
        Ok(path) => Some(path),
        Err(_) => None,
    };

    for process in &execution_processes {
        // Skip if filtering by type and this doesn't match
        let should_skip_commits = matches!(&filters.artifact_type, Some(ArtifactType::GitDiff) | Some(ArtifactType::ExecutionLog));
        let should_skip_diffs = matches!(&filters.artifact_type, Some(ArtifactType::GitCommit) | Some(ArtifactType::ExecutionLog));
        let should_skip_logs = matches!(&filters.artifact_type, Some(ArtifactType::GitDiff) | Some(ArtifactType::GitCommit));

        // Collect git commits
        if !should_skip_commits {
            if let Some(commit_sha) = &process.after_head_commit {
                let commit_subject = if let Some(ref wt_path) = worktree_path {
                    deployment
                        .git()
                        .get_commit_subject(std::path::Path::new(wt_path), commit_sha)
                        .ok()
                } else {
                    None
                };

                let subject_str = commit_subject.clone().unwrap_or_else(|| commit_sha[..7].to_string());

                artifacts.push(Artifact {
                    artifact_type: ArtifactType::GitCommit,
                    process_id: process.id.to_string(),
                    content: commit_subject,
                    size_bytes: subject_str.len(),
                    commit_sha: Some(commit_sha.clone()),
                    commit_subject: Some(subject_str),
                    before_commit: None,
                    after_commit: None,
                });
            }
        }

        // Collect git diffs
        if !should_skip_diffs {
            if let (Some(before), Some(after)) =
                (&process.before_head_commit, &process.after_head_commit)
            {
                // Get diff content if worktree is available
                let diff_content = if let Some(ref wt_path) = worktree_path {
                    deployment
                        .git()
                        .get_diff_between_commits(std::path::Path::new(wt_path), before, after)
                        .ok()
                } else {
                    None
                };

                let size = diff_content.as_ref().map(|c| c.len()).unwrap_or(0);

                artifacts.push(Artifact {
                    artifact_type: ArtifactType::GitDiff,
                    process_id: process.id.to_string(),
                    content: diff_content,
                    size_bytes: size,
                    commit_sha: None,
                    commit_subject: None,
                    before_commit: Some(before.clone()),
                    after_commit: Some(after.clone()),
                });
            }
        }

        // Collect execution logs
        if !should_skip_logs {
            if let Some(logs) = ExecutionProcessLogs::find_by_execution_id(pool, process.id).await?
            {
                artifacts.push(Artifact {
                    artifact_type: ArtifactType::ExecutionLog,
                    process_id: process.id.to_string(),
                    content: Some(logs.logs.clone()),
                    size_bytes: logs.byte_size as usize,
                    commit_sha: None,
                    commit_subject: None,
                    before_commit: None,
                    after_commit: None,
                });
            }
        }
    }

    // Apply pagination
    let total_count = artifacts.len();
    let offset = filters.offset.unwrap_or(0);
    let limit = filters.limit.unwrap_or(usize::MAX);

    let paginated_artifacts: Vec<Artifact> = artifacts
        .into_iter()
        .skip(offset)
        .take(limit)
        .collect();

    let response = AttemptArtifactsResponse {
        attempt_id: task_attempt.id.to_string(),
        artifacts: paginated_artifacts,
        total_count,
    };

    Ok(ResponseJson(ApiResponse::success(response)))
}
