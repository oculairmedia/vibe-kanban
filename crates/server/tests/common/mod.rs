//! Common utilities for MCP integration tests

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use services::services::git::GitService;

/// Initialize a test git repository with an initial commit
pub fn init_test_repo(repo_path: &Path) -> anyhow::Result<()> {
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
    use tempfile::TempDir;

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
