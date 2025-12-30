# Repository Guidelines

## Project Structure & Module Organization
- `crates/`: Rust workspace crates — `server` (API + bins), `db` (SQLx models/migrations), `executors`, `services`, `utils`, `deployment`, `local-deployment`, `remote`.
- `frontend/`: React + TypeScript app (Vite, Tailwind). Source in `frontend/src`.
- `frontend/src/components/dialogs`: Dialog components for the frontend.
- `remote-frontend/`: Remote deployment frontend.
- `shared/`: Generated TypeScript types (`shared/types.ts`). Do not edit directly.
- `assets/`, `dev_assets_seed/`, `dev_assets/`: Packaged and local dev assets.
- `npx-cli/`: Files published to the npm CLI package.
- `scripts/`: Dev helpers (ports, DB preparation).
- `docs/`: Documentation files.

## Managing Shared Types Between Rust and TypeScript

ts-rs allows you to derive TypeScript types from Rust structs/enums. By annotating your Rust types with #[derive(TS)] and related macros, ts-rs will generate .ts declaration files for those types.
When making changes to the types, you can regenerate them using `pnpm run generate-types`
Do not manually edit shared/types.ts, instead edit crates/server/src/bin/generate_types.rs

## Build, Test, and Development Commands
- Install: `pnpm i`
- Run dev (frontend + backend with ports auto-assigned): `pnpm run dev`
- Backend (watch): `pnpm run backend:dev:watch`
- Frontend (dev): `pnpm run frontend:dev`
- Type checks: `pnpm run check` (frontend) and `pnpm run backend:check` (Rust cargo check)
- Rust tests: `cargo test --workspace`
- Generate TS types from Rust: `pnpm run generate-types` (or `generate-types:check` in CI)
- Prepare SQLx (offline): `pnpm run prepare-db`
- Prepare SQLx (remote package, postgres): `pnpm run remote:prepare-db`
- Local NPX build: `pnpm run build:npx` then `pnpm pack` in `npx-cli/`

## Coding Style & Naming Conventions
- Rust: `rustfmt` enforced (`rustfmt.toml`); group imports by crate; snake_case modules, PascalCase types.
- TypeScript/React: ESLint + Prettier (2 spaces, single quotes, 80 cols). PascalCase components, camelCase vars/functions, kebab-case file names where practical.
- Keep functions small, add `Debug`/`Serialize`/`Deserialize` where useful.

## Testing Guidelines
- Rust: prefer unit tests alongside code (`#[cfg(test)]`), run `cargo test --workspace`. Add tests for new logic and edge cases.
- Frontend: ensure `pnpm run check` and `pnpm run lint` pass. If adding runtime logic, include lightweight tests (e.g., Vitest) in the same directory.

## Security & Config Tips
- Use `.env` for local overrides; never commit secrets. Key envs: `FRONTEND_PORT`, `BACKEND_PORT`, `HOST` 
- Dev ports and assets are managed by `scripts/setup-dev-environment.js`.

## MCP Server Development & Testing

### Building MCP Servers (Dev Mode)
For faster iteration when testing MCP changes, build in dev mode instead of release:
```bash
cd /opt/stacks/vibe-kanban/.git/beads-worktrees/main  # or main repo dir
cargo build --package server --bin mcp_task_server --bin mcp_system_server
```

### Running MCP Servers Locally (for testing)
Stop Docker containers first, then run from host:
```bash
# Stop Docker MCP containers
docker stop vibe-mcp-task vibe-mcp-system

# Start task server (port 9717)
TRANSPORT=http RUST_LOG=info BASE_URL=http://localhost:3105 MCP_PORT=9717 \
  nohup ./target/debug/mcp_task_server > /tmp/mcp_task.log 2>&1 &

# Start system server (port 9718)  
TRANSPORT=http RUST_LOG=info BASE_URL=http://localhost:3105 MCP_PORT=9718 \
  nohup ./target/debug/mcp_system_server > /tmp/mcp_system.log 2>&1 &
```

### Testing MCP Endpoints
```bash
# List tools
curl -s http://localhost:9717/mcp -X POST -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' | jq '.result.tools | length'

# Check logs
tail -f /tmp/mcp_task.log
```

### Key Environment Variables
- `TRANSPORT=http` - Use HTTP transport (required for testing via curl)
- `BASE_URL=http://localhost:3105` - Backend API URL
- `MCP_PORT=9717` - Port for the MCP server
- `RUST_LOG=info` - Log level

---

## MCP Docker Container Builds (CRITICAL - READ BEFORE MODIFYING)

### Overview
The MCP servers (`mcp_task_server` and `mcp_system_server`) are containerized via:
- `Dockerfile.mcp-task`
- `Dockerfile.mcp-system`
- `.github/workflows/mcp-build.yml`

### Why Debian Instead of Alpine (DO NOT CHANGE)

**The Dockerfiles MUST use Debian-based images (`rustlang/rust:nightly-bookworm`), NOT Alpine.**

Alpine uses musl libc which has a fundamental incompatibility with `libsqlite3-sys` + `bindgen`:
- `bindgen` requires `libclang` for generating Rust bindings
- `libclang` requires dynamic library loading (`dlopen`)
- musl does NOT support dynamic loading of libclang
- This causes build failures with: `"Dynamic loading not supported"`

Multiple attempted workarounds that FAILED on Alpine/musl:
1. `LIBSQLITE3_SYS_USE_PKG_CONFIG=1` - Still triggers bindgen
2. `LIBSQLITE3_SYS_USE_BUNDLED_BINDINGS=1` - Not a valid env var
3. `libsqlite3-sys` with `bundled` feature - sqlx's `pkg-config` feature still enables bindgen
4. Removing `sqlite-preupdate-hook` feature - Helped but didn't fully resolve
5. `SQLX_OFFLINE=true` - Doesn't prevent libsqlite3-sys from needing bindgen

**The only working solution is using glibc-based Debian images.**

### Why Nightly Rust (DO NOT CHANGE)

The Dockerfiles use `rustlang/rust:nightly-bookworm` because:
- `turbomcp` crate requires Rust edition 2024
- Edition 2024 requires Rust 1.85+ (currently only available in nightly)
- Standard `rust:1.87-slim-bookworm` will fail with edition errors

### Required Files in Docker Build Context

The following MUST be copied to the build context:
- `assets/` folder - Required by `RustEmbed` derive macro at compile time
- `crates/` folder - All workspace crates
- `Cargo.toml` and `Cargo.lock` - Workspace configuration

### Dockerfile Requirements Checklist

When modifying MCP Dockerfiles, ensure:
- [ ] Base image is `rustlang/rust:nightly-bookworm` (NOT Alpine, NOT stable Rust)
- [ ] Build dependencies include: `pkg-config`, `libssl-dev`, `libsqlite3-dev`, `clang`, `llvm`
- [ ] `COPY assets/ ./assets/` is present before `cargo build`
- [ ] User creation uses Debian syntax: `groupadd`/`useradd` (NOT `addgroup`/`adduser`)
- [ ] Runtime image is `debian:bookworm-slim` (NOT Alpine)
- [ ] Runtime has `ca-certificates` and `libsqlite3-0` installed

### Build Triggers

The `mcp-build.yml` workflow triggers on:
- Push to `main` branch
- Push to `sync-upstream` branch
- Manual `workflow_dispatch`

### Troubleshooting Build Failures

| Error | Cause | Solution |
|-------|-------|----------|
| `Dynamic loading not supported` | Alpine/musl + libclang | Use Debian image |
| `edition 2024 is not stable` | Rust version too old | Use nightly Rust |
| `addgroup: not found` | Alpine syntax on Debian | Use `groupadd`/`useradd` |
| `assets/sounds not found` | Missing COPY in Dockerfile | Add `COPY assets/ ./assets/` |
| `libsqlite3-sys` bindgen errors | pkg-config triggering bindgen | Use Debian (glibc supports dlopen) |

### Files to Restore After Rebase

If these files are lost during a rebase from upstream, restore them:
1. `.github/workflows/mcp-build.yml` - Container build workflow
2. `Dockerfile.mcp-task` - Task server container
3. `Dockerfile.mcp-system` - System server container
4. `crates/server/src/bin/mcp_system_server.rs` - System server binary
5. `crates/server/src/mcp/system_server.rs` - System server module
6. `crates/server/src/mcp/mod.rs` - Must export `system_server` module

### Testing Container Builds Locally

Use `act` to test GitHub Actions locally before pushing:
```bash
# Install act
curl -s https://raw.githubusercontent.com/nektos/act/master/install.sh | sudo bash

# Test MCP build workflow
cd /opt/stacks/vibe-kanban
act -W .github/workflows/mcp-build.yml -j build-mcp-task --dryrun
```

### MCP Tool Status

Working tools:
- `list_projects`, `get_project`, `list_tasks`, `get_task`
- `list_task_attempts`, `get_task_attempt`
- `get_config`, `list_executor_profiles` (after endpoint URL fixes)
- `create_task`, `update_task`, `delete_task`
- And other CRUD operations

Disabled tools (commented out - no backend endpoints exist):
- `get_project_branches` - Would need `/api/repos/{repo_id}/branches`
- `search_project_files` - Route exists but returns 404
- `get_attempt_artifacts` - No endpoint
- `list_execution_processes` - No list-by-attempt endpoint
- `get_branch_status` - No endpoint
- `get_attempt_commits` - No endpoint

These are commented out in `crates/server/src/mcp/task_server.rs` to avoid wasting tokens describing non-functional tools to AI agents.
