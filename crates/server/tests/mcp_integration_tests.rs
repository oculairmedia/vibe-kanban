//! Comprehensive integration tests for MCP task and system servers
//!
//! Tests cover:
//! - Tool invocation for all 35+ MCP tools
//! - Response format validation
//! - Error handling (404, 500, validation errors)
//! - Parameter validation
//! - Complete workflows (task lifecycle, follow-ups, rebase)
//! - Performance (bulk operations, streaming, concurrency)

use std::path::{Path, PathBuf};
use std::sync::Arc;
use tempfile::TempDir;
use uuid::Uuid;

mod common;
mod task_server_tests;
mod system_server_tests;
mod workflow_tests;
mod performance_tests;
mod error_handling_tests;

/// Test fixture that provides a complete testing environment
/// with temporary database, git repos, and running services
pub struct McpTestFixture {
    pub temp_dir: TempDir,
    pub db_path: PathBuf,
    pub repo_path: PathBuf,
    pub base_url: String,
    pub project_id: Option<Uuid>,
    pub task_id: Option<Uuid>,
}

impl McpTestFixture {
    /// Create a new test fixture with initialized database and git repository
    pub async fn new() -> anyhow::Result<Self> {
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test.db");
        let repo_path = temp_dir.path().join("test_repo");

        // Initialize test database
        let db_url = format!("sqlite://{}", db_path.display());
        let pool = sqlx::SqlitePool::connect(&db_url).await?;

        // Run migrations
        sqlx::migrate!("../db/migrations").run(&pool).await?;

        // Initialize git repository
        common::init_test_repo(&repo_path)?;

        Ok(Self {
            temp_dir,
            db_path,
            repo_path,
            base_url: "http://localhost:0".to_string(), // Will be set by actual server
            project_id: None,
            task_id: None,
        })
    }

    /// Create a test project in the database
    pub async fn create_test_project(&mut self, name: &str) -> anyhow::Result<Uuid> {
        // This would use the actual API or database directly
        // For now, return a placeholder
        let project_id = Uuid::new_v4();
        self.project_id = Some(project_id);
        Ok(project_id)
    }

    /// Create a test task in the database
    pub async fn create_test_task(
        &mut self,
        project_id: Uuid,
        title: &str,
    ) -> anyhow::Result<Uuid> {
        let task_id = Uuid::new_v4();
        self.task_id = Some(task_id);
        Ok(task_id)
    }

    /// Clean up resources
    pub async fn cleanup(self) -> anyhow::Result<()> {
        // TempDir will auto-cleanup on drop
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fixture_setup() {
        let fixture = McpTestFixture::new()
            .await
            .expect("Failed to create test fixture");

        assert!(fixture.db_path.exists());
        assert!(fixture.repo_path.exists());
        assert!(fixture.repo_path.join(".git").exists());
    }
}
