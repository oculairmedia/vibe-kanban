# Reviewer Feedback - Attempt 3

## Previous Build Results (Attempt 2)
**Status**: FAILED with 5 compilation errors
**Worktree**: `/var/tmp/vibe-kanban/worktrees/98ec-phase-2-enhanced`

## Critical Issue
The `upload_task_image` tool uses `reqwest::multipart` features that aren't enabled in Cargo.toml.

## Solution

**In `crates/server/Cargo.toml`**, update the reqwest dependency to include the `multipart` feature:

```toml
# Current (WRONG):
reqwest = { version = "0.12", features = ["json"] }

# Should be (CORRECT):
reqwest = { version = "0.12", features = ["json", "multipart"] }
```

## Additional Cleanup

Remove the unused import at line 4 of `task_server.rs`:

```rust
// Remove this:
use models::{
    image::Image,  // <-- Not used
    // ... keep other imports
};
```

## Error Context
Without the `multipart` feature:
- Line 897: `reqwest::multipart::Form::new()` - module not found
- Line 912: `.multipart(form)` - method not found on RequestBuilder

## Verification
After fixing, run:
```bash
cargo build --release --bin mcp_task_server
```

All 6 tools are implemented including `upload_task_image` - just need to enable the feature!
