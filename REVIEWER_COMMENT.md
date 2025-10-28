# Reviewer Feedback - Phase 2 Branches

## vk/6d69-phase-2-system-c Review

**Status**: FAILED with 19 compilation errors
**Worktree**: `/var/tmp/vibe-kanban/worktrees/6d69-phase-2-system-c`

### Critical Issue
All 19 errors are the same type mismatch pattern throughout `system_server.rs`:

```rust
// WRONG - Type mismatch: Some() expects &str but gets String
return Self::err("Failed to list git repositories", Some(e.to_string()));
//                                                   ^^^^ expected `&str`, found `String`
```

### Solution (Choose One)

#### Option 1: Quick Fix (Recommended)
Add `&` before each `e.to_string()` in all error handlers:

```rust
return Self::err("Failed to list git repositories", Some(&e.to_string()));
//                                                         ^ add this
```

**Affected lines** (all in `crates/server/src/mcp/system_server.rs`):
287, 312, 335, 356, 374, 399, 411, 438, 450, 460, 472, 490, 511, and 6 more

#### Option 2: Better Long-term Fix
Change the `err_str` function signature to accept `String`:

```rust
fn err_str(msg: &str, details: Option<String>) -> McpError {
    let mut error_msg = msg.to_string();
    if let Some(d) = details {
        error_msg.push_str(&format!(": {}", d));
    }
    McpError::internal_error(error_msg)
}
```

Then all existing `Some(e.to_string())` calls will work without modification.

### Verification
```bash
cargo build --release --bin mcp_system_server
```

All 8 tools are implemented - just need to fix these type errors!

---

## vk/98ec-phase-2-enhanced Review

**Status**: FAILED with 5 compilation errors
**Worktree**: `/var/tmp/vibe-kanban/worktrees/98ec-phase-2-enhanced`

### Critical Issue
The `upload_task_image` tool uses `reqwest::multipart` features that aren't enabled in Cargo.toml.

### Solution

**In `crates/server/Cargo.toml`**, update the reqwest dependency to include the `multipart` feature:

```toml
# Current (WRONG):
reqwest = { version = "0.12", features = ["json"] }

# Should be (CORRECT):
reqwest = { version = "0.12", features = ["json", "multipart"] }
```

### Additional Cleanup

Remove the unused import at line 4 of `task_server.rs`:

```rust
// Remove this:
use models::{
    image::Image,  // <-- Not used
    // ... keep other imports
};
```

### Error Context
Without the `multipart` feature:
- Line 897: `reqwest::multipart::Form::new()` - module not found
- Line 912: `.multipart(form)` - method not found on RequestBuilder

### Verification
```bash
cargo build --release --bin mcp_task_server
```

All 6 tools are implemented including `upload_task_image` - just need to enable the feature!

---

## vk/b609-phase-2-enhanced Review

**Status**: FAILED with 9 compilation errors
**Worktree**: `/var/tmp/vibe-kanban/worktrees/b609-phase-2-enhanced`

### Critical Issue
The `ProjectSummary` struct is missing the `Deserialize` derive, but it's used in structs that require deserialization.

### Solution

Find the `ProjectSummary` struct definition (likely in one of these locations):
- `shared/src/types.rs`
- `services/src/models.rs`
- Or wherever domain models are defined

```rust
// Current (WRONG):
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct ProjectSummary {
    pub id: Uuid,
    pub name: String,
}

// Should be (CORRECT):
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ProjectSummary {
    pub id: Uuid,
    pub name: String,
}
```

### Why This Is Needed

The `TaskWithAttempt` struct (task_server.rs:290) uses `ProjectSummary` and has `Deserialize` derive:

```rust
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct TaskWithAttempt {
    pub project: ProjectSummary,  // ProjectSummary MUST have Deserialize!
    // ... other fields
}
```

### Verification
```bash
cargo build --release --bin mcp_task_server
```

All 8 tools are implemented - just need to add the missing derive!
