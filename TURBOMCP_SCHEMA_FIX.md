# TurboMCP Schema Generation Fix

## Problem
Vibe Kanban's MCP tools had **empty `inputSchema`** - all tools showed:
```json
{
  "type": "object",
  "properties": {},
  "required": [],
  "additionalProperties": false
}
```

This prevented LLMs from knowing what parameters to pass to the tools.

## Root Cause
The `schemars` feature was not enabled as a **crate feature** in `crates/server/Cargo.toml`. 

While the TurboMCP dependency had `features = ["http", "schemars"]`, this only enables the feature for the dependency itself. TurboMCP's schema generation macros check if the **consuming crate** has the `schemars` feature enabled.

## Solution
Added the `schemars` feature to the server crate's feature configuration:

**File: `crates/server/Cargo.toml`**

### Before
```toml
[features]
default = ["http"]
http = ["turbomcp/http", "turbomcp-transport/http"]
```

### After
```toml
[features]
default = ["http", "schemars"]
http = ["turbomcp/http", "turbomcp-transport/http"]
schemars = ["turbomcp/schemars"]
```

## Verification

### Task Server (11 tools)
```bash
curl -s -X POST http://192.168.50.90:9717/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc": "2.0", "method": "tools/list", "id": 1}' \
  | jq '.result.tools[0].inputSchema'
```

**Result:**
```json
{
  "type": "object",
  "properties": {
    "title": {
      "description": "The title of the task",
      "type": "string"
    },
    "description": {
      "description": "Optional description of the task",
      "type": ["string", "null"]
    },
    "project_id": {
      "description": "The ID of the project to create the task in. This is required!",
      "type": "string",
      "format": "uuid"
    }
  },
  "required": ["project_id", "title"]
}
```

### System Server (9 tools)
```bash
curl -s -X POST http://192.168.50.90:9718/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc": "2.0", "method": "tools/list", "id": 1}' \
  | jq '.result.tools[] | select(.name == "update_config") | .inputSchema'
```

**Result:**
```json
{
  "type": "object",
  "properties": {
    "editor": {
      "description": "Preferred editor",
      "type": ["string", "null"]
    },
    "executor_profile": {
      "description": "Default executor profile",
      "type": ["string", "null"]
    },
    "git_branch_prefix": {
      "description": "Git branch prefix for task branches",
      "type": ["string", "null"]
    },
    "telemetry_enabled": {
      "description": "Enable telemetry",
      "type": ["boolean", "null"]
    },
    "analytics_enabled": {
      "description": "Enable analytics",
      "type": ["boolean", "null"]
    }
  }
}
```

## Key Learnings

1. **Dependency features != Crate features**: Enabling a feature in a dependency's feature list doesn't automatically enable it as a feature of your crate.

2. **TurboMCP macro behavior**: The `#[tool]` and `#[turbomcp::server]` macros check for the `schemars` feature in the **consuming crate**, not just in the TurboMCP dependency.

3. **Letta MCP worked because**: The Letta MCP server had this configuration from the start:
   ```toml
   [features]
   default = ["http", "schemars"]
   schemars = ["turbomcp/schemars"]
   ```

4. **Request struct pattern required**: TurboMCP only generates schemas from request structs with `#[derive(schemars::JsonSchema)]`, not from individual function parameters.

## Related Documentation
- Letta MCP: `/opt/stacks/letta-MCP-server/SCHEMA_FIX_DOCUMENTATION.md`
- TurboMCP fork: `https://github.com/oculairmedia/turbomcp.git` branch `feature/flatten-structs`

## Build Commands
```bash
cd /opt/stacks/vibe-kanban

# Rebuild both servers
cargo build --bin mcp_task_server
cargo build --bin mcp_system_server

# Start with HTTP transport
TRANSPORT=http MCP_PORT=9717 ./target/debug/mcp_task_server &
TRANSPORT=http MCP_PORT=9718 ./target/debug/mcp_system_server &
```

## Status
✅ **RESOLVED** - All 20 MCP tools (11 task + 9 system) now have fully populated schemas with descriptions, types, and required fields.
