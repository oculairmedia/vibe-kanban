//! Common utilities for MCP integration tests

pub mod mcp_client;

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use services::services::git::GitService;
use sqlx::{Pool, Sqlite, SqlitePool, sqlite::SqliteConnectOptions};
use std::str::FromStr;
use tempfile::TempDir;
use uuid::Uuid;
use chrono::Utc;

// Re-export the MCP client for convenience
pub use mcp_client::{McpClient, McpClientError};

// Re-export for convenience
pub use serde_json::json;
pub use uuid::Uuid as TestUuid;

/// Test fixture that provides a complete testing environment
/// with temporary database, git repos, and test data
pub struct TestFixture {
    pub temp_dir: TempDir,
    pub db_pool: Pool<Sqlite>,
    pub repo_path: PathBuf,
    pub project_id: Option<Uuid>,
    pub task_id: Option<Uuid>,
}

impl TestFixture {
    /// Create a new test fixture with initialized database and git repository
    pub async fn new() -> anyhow::Result<Self> {
        let temp_dir = TempDir::new()?;
        let repo_path = temp_dir.path().join("test_repo");

        // Create in-memory SQLite database with shared cache for multi-connection access
        let db_url = "sqlite::memory:?cache=shared";
        let options = SqliteConnectOptions::from_str(db_url)?
            .create_if_missing(true);
        
        let db_pool = SqlitePool::connect_with(options).await?;

        // Run migrations using embedded SQL
        Self::run_migrations(&db_pool).await?;

        // Initialize git repository
        init_test_repo(&repo_path)?;

        Ok(Self {
            temp_dir,
            db_pool,
            repo_path,
            project_id: None,
            task_id: None,
        })
    }

    /// Run database migrations
    async fn run_migrations(pool: &Pool<Sqlite>) -> anyhow::Result<()> {
        // Core schema
        sqlx::query(r#"
            PRAGMA foreign_keys = ON;

            CREATE TABLE IF NOT EXISTS projects (
                id            BLOB PRIMARY KEY,
                name          TEXT NOT NULL,
                git_repo_path TEXT NOT NULL DEFAULT '',
                setup_script  TEXT DEFAULT '',
                cleanup_script TEXT DEFAULT '',
                dev_script    TEXT DEFAULT '',
                dev_script_working_dir TEXT DEFAULT '',
                default_agent_working_dir TEXT DEFAULT '',
                remote_project_id TEXT DEFAULT NULL,
                created_at    TEXT NOT NULL DEFAULT (datetime('now', 'subsec')),
                updated_at    TEXT NOT NULL DEFAULT (datetime('now', 'subsec'))
            );

            CREATE TABLE IF NOT EXISTS tasks (
                id                  BLOB PRIMARY KEY,
                project_id          BLOB NOT NULL,
                title               TEXT NOT NULL,
                description         TEXT,
                status              TEXT NOT NULL DEFAULT 'todo'
                                    CHECK (status IN ('todo','inprogress','done','cancelled','inreview')),
                parent_task_attempt BLOB DEFAULT NULL,
                shared_task_id      BLOB DEFAULT NULL,
                created_at          TEXT NOT NULL DEFAULT (datetime('now', 'subsec')),
                updated_at          TEXT NOT NULL DEFAULT (datetime('now', 'subsec')),
                FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS task_attempts (
                id                  BLOB PRIMARY KEY,
                task_id             BLOB NOT NULL,
                worktree_path       TEXT NOT NULL DEFAULT '',
                container_ref       TEXT NOT NULL DEFAULT '',
                merge_commit        TEXT,
                executor            TEXT,
                branch              TEXT DEFAULT '',
                target_branch       TEXT DEFAULT 'main',
                worktree_deleted    INTEGER DEFAULT 0,
                setup_completed_at  TEXT DEFAULT NULL,
                pr_url              TEXT DEFAULT NULL,
                pr_number           INTEGER DEFAULT NULL,
                created_at          TEXT NOT NULL DEFAULT (datetime('now', 'subsec')),
                updated_at          TEXT NOT NULL DEFAULT (datetime('now', 'subsec')),
                FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS execution_processes (
                id                BLOB PRIMARY KEY,
                task_attempt_id   BLOB NOT NULL,
                run_reason        TEXT NOT NULL DEFAULT 'codingagent'
                                  CHECK (run_reason IN ('setupscript','cleanupscript','codingagent','devscript')),
                status            TEXT NOT NULL DEFAULT 'running'
                                  CHECK (status IN ('running','completed','failed','killed')),
                exit_code         INTEGER,
                executor_type     TEXT DEFAULT NULL,
                created_at        TEXT NOT NULL DEFAULT (datetime('now', 'subsec')),
                updated_at        TEXT NOT NULL DEFAULT (datetime('now', 'subsec')),
                FOREIGN KEY (task_attempt_id) REFERENCES task_attempts(id) ON DELETE CASCADE
            );
        "#).execute(pool).await?;

        Ok(())
    }

    /// Create a test project
    pub async fn create_project(&mut self, name: &str) -> anyhow::Result<Uuid> {
        let id = Uuid::new_v4();
        let now = Utc::now().to_rfc3339();
        
        sqlx::query(
            r#"INSERT INTO projects (id, name, git_repo_path, created_at, updated_at)
               VALUES (?, ?, ?, ?, ?)"#
        )
        .bind(id.as_bytes().as_slice())
        .bind(name)
        .bind(self.repo_path.to_string_lossy().to_string())
        .bind(&now)
        .bind(&now)
        .execute(&self.db_pool)
        .await?;

        self.project_id = Some(id);
        Ok(id)
    }

    /// Create a test task
    pub async fn create_task(&mut self, project_id: Uuid, title: &str, status: &str) -> anyhow::Result<Uuid> {
        let id = Uuid::new_v4();
        let now = Utc::now().to_rfc3339();
        
        sqlx::query(
            r#"INSERT INTO tasks (id, project_id, title, status, created_at, updated_at)
               VALUES (?, ?, ?, ?, ?, ?)"#
        )
        .bind(id.as_bytes().as_slice())
        .bind(project_id.as_bytes().as_slice())
        .bind(title)
        .bind(status)
        .bind(&now)
        .bind(&now)
        .execute(&self.db_pool)
        .await?;

        self.task_id = Some(id);
        Ok(id)
    }

    /// Create a test task attempt
    pub async fn create_attempt(&self, task_id: Uuid, executor: &str) -> anyhow::Result<Uuid> {
        let id = Uuid::new_v4();
        let now = Utc::now().to_rfc3339();
        let branch = format!("task/{}", id);
        
        sqlx::query(
            r#"INSERT INTO task_attempts (id, task_id, executor, branch, target_branch, created_at, updated_at)
               VALUES (?, ?, ?, ?, 'main', ?, ?)"#
        )
        .bind(id.as_bytes().as_slice())
        .bind(task_id.as_bytes().as_slice())
        .bind(executor)
        .bind(&branch)
        .bind(&now)
        .bind(&now)
        .execute(&self.db_pool)
        .await?;

        Ok(id)
    }

    /// Get project by ID
    pub async fn get_project(&self, id: Uuid) -> anyhow::Result<Option<serde_json::Value>> {
        let row = sqlx::query(
            r#"SELECT id, name, git_repo_path, created_at, updated_at 
               FROM projects WHERE id = ?"#
        )
        .bind(id.as_bytes().as_slice())
        .fetch_optional(&self.db_pool)
        .await?;

        Ok(row.map(|r| {
            let id_bytes: Vec<u8> = sqlx::Row::get(&r, "id");
            let id = Uuid::from_slice(&id_bytes).unwrap_or_default();
            json!({
                "id": id.to_string(),
                "name": sqlx::Row::get::<String, _>(&r, "name"),
                "git_repo_path": sqlx::Row::get::<String, _>(&r, "git_repo_path"),
                "created_at": sqlx::Row::get::<String, _>(&r, "created_at"),
                "updated_at": sqlx::Row::get::<String, _>(&r, "updated_at"),
            })
        }))
    }

    /// Get task by ID
    pub async fn get_task(&self, id: Uuid) -> anyhow::Result<Option<serde_json::Value>> {
        let row = sqlx::query(
            r#"SELECT id, project_id, title, description, status, created_at, updated_at 
               FROM tasks WHERE id = ?"#
        )
        .bind(id.as_bytes().as_slice())
        .fetch_optional(&self.db_pool)
        .await?;

        Ok(row.map(|r| {
            let id_bytes: Vec<u8> = sqlx::Row::get(&r, "id");
            let project_id_bytes: Vec<u8> = sqlx::Row::get(&r, "project_id");
            json!({
                "id": Uuid::from_slice(&id_bytes).unwrap_or_default().to_string(),
                "project_id": Uuid::from_slice(&project_id_bytes).unwrap_or_default().to_string(),
                "title": sqlx::Row::get::<String, _>(&r, "title"),
                "description": sqlx::Row::get::<Option<String>, _>(&r, "description"),
                "status": sqlx::Row::get::<String, _>(&r, "status"),
                "created_at": sqlx::Row::get::<String, _>(&r, "created_at"),
                "updated_at": sqlx::Row::get::<String, _>(&r, "updated_at"),
            })
        }))
    }

    /// List all tasks for a project
    pub async fn list_tasks(&self, project_id: Uuid) -> anyhow::Result<Vec<serde_json::Value>> {
        self.list_tasks_with_search(project_id, None).await
    }

    /// List tasks for a project with optional search filter (case-insensitive title match)
    pub async fn list_tasks_with_search(&self, project_id: Uuid, search: Option<&str>) -> anyhow::Result<Vec<serde_json::Value>> {
        let rows = sqlx::query(
            r#"SELECT id, project_id, title, description, status, created_at, updated_at 
               FROM tasks WHERE project_id = ? ORDER BY created_at DESC"#
        )
        .bind(project_id.as_bytes().as_slice())
        .fetch_all(&self.db_pool)
        .await?;

        let search_lower = search.map(|s| s.to_lowercase());

        Ok(rows.into_iter().filter_map(|r| {
            let title: String = sqlx::Row::get(&r, "title");
            
            // Apply search filter if provided
            if let Some(ref query) = search_lower {
                if !title.to_lowercase().contains(query) {
                    return None;
                }
            }
            
            let id_bytes: Vec<u8> = sqlx::Row::get(&r, "id");
            let project_id_bytes: Vec<u8> = sqlx::Row::get(&r, "project_id");
            Some(json!({
                "id": Uuid::from_slice(&id_bytes).unwrap_or_default().to_string(),
                "project_id": Uuid::from_slice(&project_id_bytes).unwrap_or_default().to_string(),
                "title": title,
                "description": sqlx::Row::get::<Option<String>, _>(&r, "description"),
                "status": sqlx::Row::get::<String, _>(&r, "status"),
                "created_at": sqlx::Row::get::<String, _>(&r, "created_at"),
                "updated_at": sqlx::Row::get::<String, _>(&r, "updated_at"),
            }))
        }).collect())
    }

    /// Update task status
    pub async fn update_task_status(&self, task_id: Uuid, status: &str) -> anyhow::Result<()> {
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            r#"UPDATE tasks SET status = ?, updated_at = ? WHERE id = ?"#
        )
        .bind(status)
        .bind(&now)
        .bind(task_id.as_bytes().as_slice())
        .execute(&self.db_pool)
        .await?;
        Ok(())
    }

    /// Delete a task
    pub async fn delete_task(&self, task_id: Uuid) -> anyhow::Result<bool> {
        let result = sqlx::query(
            r#"DELETE FROM tasks WHERE id = ?"#
        )
        .bind(task_id.as_bytes().as_slice())
        .execute(&self.db_pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }

    /// Get task with optional attempts included (simulates include_attempts parameter)
    pub async fn get_task_with_attempts(&self, task_id: Uuid, include_attempts: bool) -> anyhow::Result<Option<serde_json::Value>> {
        let task = self.get_task(task_id).await?;
        
        if let Some(mut task_json) = task {
            if include_attempts {
                let attempts = self.list_attempts(task_id).await?;
                task_json["attempts"] = json!(attempts);
            }
            Ok(Some(task_json))
        } else {
            Ok(None)
        }
    }

    /// List attempts for a task
    pub async fn list_attempts(&self, task_id: Uuid) -> anyhow::Result<Vec<serde_json::Value>> {
        let rows = sqlx::query(
            r#"SELECT id, task_id, branch, target_branch, executor, container_ref, 
                      worktree_deleted, setup_completed_at, created_at, updated_at 
               FROM task_attempts WHERE task_id = ? ORDER BY created_at DESC"#
        )
        .bind(task_id.as_bytes().as_slice())
        .fetch_all(&self.db_pool)
        .await?;

        Ok(rows.into_iter().map(|r| {
            let id_bytes: Vec<u8> = sqlx::Row::get(&r, "id");
            let task_id_bytes: Vec<u8> = sqlx::Row::get(&r, "task_id");
            json!({
                "id": Uuid::from_slice(&id_bytes).unwrap_or_default().to_string(),
                "task_id": Uuid::from_slice(&task_id_bytes).unwrap_or_default().to_string(),
                "branch": sqlx::Row::get::<String, _>(&r, "branch"),
                "target_branch": sqlx::Row::get::<String, _>(&r, "target_branch"),
                "executor": sqlx::Row::get::<Option<String>, _>(&r, "executor"),
                "container_ref": sqlx::Row::get::<Option<String>, _>(&r, "container_ref"),
                "worktree_deleted": sqlx::Row::get::<bool, _>(&r, "worktree_deleted"),
                "setup_completed_at": sqlx::Row::get::<Option<String>, _>(&r, "setup_completed_at"),
                "created_at": sqlx::Row::get::<String, _>(&r, "created_at"),
                "updated_at": sqlx::Row::get::<String, _>(&r, "updated_at"),
            })
        }).collect())
    }

    /// Get attempt with optional processes included (simulates include_processes parameter)
    pub async fn get_attempt_with_processes(&self, attempt_id: Uuid, include_processes: bool) -> anyhow::Result<Option<serde_json::Value>> {
        let row = sqlx::query(
            r#"SELECT id, task_id, branch, target_branch, executor, container_ref, 
                      worktree_deleted, setup_completed_at, created_at, updated_at 
               FROM task_attempts WHERE id = ?"#
        )
        .bind(attempt_id.as_bytes().as_slice())
        .fetch_optional(&self.db_pool)
        .await?;

        if let Some(r) = row {
            let id_bytes: Vec<u8> = sqlx::Row::get(&r, "id");
            let task_id_bytes: Vec<u8> = sqlx::Row::get(&r, "task_id");
            let mut attempt_json = json!({
                "id": Uuid::from_slice(&id_bytes).unwrap_or_default().to_string(),
                "task_id": Uuid::from_slice(&task_id_bytes).unwrap_or_default().to_string(),
                "branch": sqlx::Row::get::<String, _>(&r, "branch"),
                "target_branch": sqlx::Row::get::<String, _>(&r, "target_branch"),
                "executor": sqlx::Row::get::<Option<String>, _>(&r, "executor"),
                "container_ref": sqlx::Row::get::<Option<String>, _>(&r, "container_ref"),
                "worktree_deleted": sqlx::Row::get::<bool, _>(&r, "worktree_deleted"),
                "setup_completed_at": sqlx::Row::get::<Option<String>, _>(&r, "setup_completed_at"),
                "created_at": sqlx::Row::get::<String, _>(&r, "created_at"),
                "updated_at": sqlx::Row::get::<String, _>(&r, "updated_at"),
            });
            
            if include_processes {
                let processes = self.list_processes(attempt_id).await?;
                attempt_json["processes"] = json!(processes);
            }
            
            Ok(Some(attempt_json))
        } else {
            Ok(None)
        }
    }

    /// List execution processes for an attempt
    pub async fn list_processes(&self, attempt_id: Uuid) -> anyhow::Result<Vec<serde_json::Value>> {
        let rows = sqlx::query(
            r#"SELECT id, task_attempt_id, run_reason, status, executor_type,
                      exit_code, created_at, updated_at 
               FROM execution_processes WHERE task_attempt_id = ? ORDER BY created_at DESC"#
        )
        .bind(attempt_id.as_bytes().as_slice())
        .fetch_all(&self.db_pool)
        .await?;

        Ok(rows.into_iter().map(|r| {
            let id_bytes: Vec<u8> = sqlx::Row::get(&r, "id");
            let attempt_id_bytes: Vec<u8> = sqlx::Row::get(&r, "task_attempt_id");
            json!({
                "id": Uuid::from_slice(&id_bytes).unwrap_or_default().to_string(),
                "task_attempt_id": Uuid::from_slice(&attempt_id_bytes).unwrap_or_default().to_string(),
                "run_reason": sqlx::Row::get::<String, _>(&r, "run_reason"),
                "status": sqlx::Row::get::<String, _>(&r, "status"),
                "executor_type": sqlx::Row::get::<Option<String>, _>(&r, "executor_type"),
                "exit_code": sqlx::Row::get::<Option<i32>, _>(&r, "exit_code"),
                "created_at": sqlx::Row::get::<String, _>(&r, "created_at"),
                "updated_at": sqlx::Row::get::<String, _>(&r, "updated_at"),
            })
        }).collect())
    }

    /// Create an execution process for an attempt
    pub async fn create_process(&mut self, attempt_id: Uuid, run_reason: &str, status: &str) -> anyhow::Result<Uuid> {
        let id = Uuid::new_v4();
        let now = Utc::now().to_rfc3339();
        
        sqlx::query(
            r#"INSERT INTO execution_processes (id, task_attempt_id, run_reason, status, created_at, updated_at)
               VALUES (?, ?, ?, ?, ?, ?)"#
        )
        .bind(id.as_bytes().as_slice())
        .bind(attempt_id.as_bytes().as_slice())
        .bind(run_reason)
        .bind(status)
        .bind(&now)
        .bind(&now)
        .execute(&self.db_pool)
        .await?;
        
        Ok(id)
    }
}

/// Initialize a test git repository with an initial commit
pub fn init_test_repo(repo_path: &Path) -> anyhow::Result<()> {
    // Create the directory first
    fs::create_dir_all(repo_path)?;
    
    let git = GitService::new();
    git.initialize_repo_with_main_branch(repo_path)?;
    git.configure_user(repo_path, "Test User", "test@example.com")?;
    git.checkout_branch(repo_path, "main")?;

    // Create an initial file and commit
    write_file(repo_path, "README.md", "# Test Repository\n")?;
    git.commit(repo_path, "Initial commit")?;

    Ok(())
}

/// Write a file to the repository
pub fn write_file<P: AsRef<Path>>(base: P, rel: &str, content: &str) -> anyhow::Result<()> {
    let path = base.as_ref().join(rel);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut f = fs::File::create(&path)?;
    f.write_all(content.as_bytes())?;
    Ok(())
}

/// Assert that a JSON response contains expected fields
pub fn assert_json_has_field(json: &serde_json::Value, field: &str) {
    assert!(
        json.get(field).is_some(),
        "Expected JSON to have field '{}', got: {}",
        field,
        json
    );
}

/// Assert that a JSON response matches expected structure
pub fn assert_json_structure(json: &serde_json::Value, fields: &[&str]) {
    for field in fields {
        assert_json_has_field(json, field);
    }
}

/// Parse a tool response as JSON
pub fn parse_tool_response(response: &str) -> anyhow::Result<serde_json::Value> {
    Ok(serde_json::from_str(response)?)
}

/// Create a mock HTTP client for testing
pub fn create_test_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("Failed to create test client")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fixture_creation() {
        let fixture = TestFixture::new().await.expect("Failed to create fixture");
        assert!(fixture.repo_path.exists());
        assert!(fixture.repo_path.join(".git").exists());
    }

    #[tokio::test]
    async fn test_create_project() {
        let mut fixture = TestFixture::new().await.expect("Failed to create fixture");
        let project_id = fixture.create_project("Test Project").await.expect("Failed to create project");
        
        let project = fixture.get_project(project_id).await.expect("Failed to get project");
        assert!(project.is_some());
        let project = project.unwrap();
        assert_eq!(project["name"], "Test Project");
    }

    #[tokio::test]
    async fn test_create_task() {
        let mut fixture = TestFixture::new().await.expect("Failed to create fixture");
        let project_id = fixture.create_project("Test Project").await.expect("Failed to create project");
        let task_id = fixture.create_task(project_id, "Test Task", "todo").await.expect("Failed to create task");
        
        let task = fixture.get_task(task_id).await.expect("Failed to get task");
        assert!(task.is_some());
        let task = task.unwrap();
        assert_eq!(task["title"], "Test Task");
        assert_eq!(task["status"], "todo");
    }

    #[tokio::test]
    async fn test_list_tasks() {
        let mut fixture = TestFixture::new().await.expect("Failed to create fixture");
        let project_id = fixture.create_project("Test Project").await.expect("Failed to create project");
        
        fixture.create_task(project_id, "Task 1", "todo").await.expect("Failed to create task 1");
        fixture.create_task(project_id, "Task 2", "inprogress").await.expect("Failed to create task 2");
        fixture.create_task(project_id, "Task 3", "done").await.expect("Failed to create task 3");
        
        let tasks = fixture.list_tasks(project_id).await.expect("Failed to list tasks");
        assert_eq!(tasks.len(), 3);
    }

    #[tokio::test]
    async fn test_update_task_status() {
        let mut fixture = TestFixture::new().await.expect("Failed to create fixture");
        let project_id = fixture.create_project("Test Project").await.expect("Failed to create project");
        let task_id = fixture.create_task(project_id, "Test Task", "todo").await.expect("Failed to create task");
        
        fixture.update_task_status(task_id, "inprogress").await.expect("Failed to update status");
        
        let task = fixture.get_task(task_id).await.expect("Failed to get task").unwrap();
        assert_eq!(task["status"], "inprogress");
    }

    #[tokio::test]
    async fn test_delete_task() {
        let mut fixture = TestFixture::new().await.expect("Failed to create fixture");
        let project_id = fixture.create_project("Test Project").await.expect("Failed to create project");
        let task_id = fixture.create_task(project_id, "Test Task", "todo").await.expect("Failed to create task");
        
        let deleted = fixture.delete_task(task_id).await.expect("Failed to delete task");
        assert!(deleted);
        
        let task = fixture.get_task(task_id).await.expect("Failed to get task");
        assert!(task.is_none());
    }

    #[test]
    fn test_init_repo() {
        let td = TempDir::new().unwrap();
        let repo_path = td.path().join("test_repo");

        init_test_repo(&repo_path).expect("Failed to init repo");

        assert!(repo_path.exists());
        assert!(repo_path.join(".git").exists());
        assert!(repo_path.join("README.md").exists());
    }

    #[test]
    fn test_write_file() {
        let td = TempDir::new().unwrap();
        write_file(td.path(), "test.txt", "hello").unwrap();
        let content = fs::read_to_string(td.path().join("test.txt")).unwrap();
        assert_eq!(content, "hello");
    }

    #[test]
    fn test_assert_json_has_field() {
        let json = serde_json::json!({ "foo": "bar", "baz": 123 });
        assert_json_has_field(&json, "foo");
        assert_json_has_field(&json, "baz");
    }

    #[test]
    #[should_panic(expected = "Expected JSON to have field")]
    fn test_assert_json_missing_field() {
        let json = serde_json::json!({ "foo": "bar" });
        assert_json_has_field(&json, "missing");
    }

    #[test]
    fn test_parse_tool_response() {
        let response = r#"{"task_id": "123", "status": "ok"}"#;
        let json = parse_tool_response(response).unwrap();
        assert_eq!(json["task_id"], "123");
        assert_eq!(json["status"], "ok");
    }
}
