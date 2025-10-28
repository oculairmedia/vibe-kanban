# Reviewer Feedback - Attempt 3

## Previous Build Results (Attempt 2)
**Status**: FAILED with 19 compilation errors
**Worktree**: `/var/tmp/vibe-kanban/worktrees/6d69-phase-2-system-c`

## Critical Issue
All 19 errors are the same type mismatch pattern throughout `system_server.rs`:

```rust
// WRONG - Type mismatch: Some() expects &str but gets String
return Self::err("Failed to list git repositories", Some(e.to_string()));
//                                                   ^^^^ expected `&str`, found `String`
```

## Solution (Choose One)

### Option 1: Quick Fix (Recommended)
Add `&` before each `e.to_string()` in all error handlers:

```rust
return Self::err("Failed to list git repositories", Some(&e.to_string()));
//                                                         ^ add this
```

**Affected lines** (all in `crates/server/src/mcp/system_server.rs`):
287, 312, 335, 356, 374, 399, 411, 438, 450, 460, 472, 490, 511, and 6 more

### Option 2: Better Long-term Fix
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

## Verification
After fixing, run:
```bash
cargo build --release --bin mcp_system_server
```

All 8 tools are implemented - just need to fix these type errors!
