# Viable Work Analysis - Vibe Kanban MCP Tools

**Analysis Date:** 2025-10-29  
**Backend Version:** NPM Package v0.0.111 (port 3105)  
**Purpose:** Identify which Huly issues can be implemented without custom backend deployment

---

## Summary

**Total Issues:** 39  
**Completed:** 17 ✅  
**In Backlog:** 18  
**Blocked (Awaiting NPM Release):** 4 🔄

---

## ✅ IMMEDIATELY VIABLE - Can Be Implemented Now

These issues have all required API endpoints available in the current NPM package:

### High Priority

#### VIBEK-34: Implement get_branch_status tool
- **Endpoint:** `GET /api/task-attempts/{id}/branch-status` ✅ Available
- **Complexity:** Low - straightforward API call
- **Value:** Medium - useful for monitoring git status
- **Recommendation:** ⭐ **Quick win, implement next**

#### VIBEK-20: Implement get_executor_profile tool
- **Endpoint:** Requires new endpoint - NOT AVAILABLE ❌
- **Status:** BLOCKED - needs custom API endpoint

### Medium Priority

#### VIBEK-32: Implement change_target_branch tool
- **Endpoint:** `POST /api/task-attempts/{id}/change-target-branch` ✅ Available
- **Complexity:** Low - simple API call
- **Value:** Medium - allows flexible branch targeting
- **Recommendation:** ⭐ **Good candidate**

#### VIBEK-28: Implement get_commit_info tool
- **Endpoint:** `GET /api/task-attempts/{id}/commit-info?sha={sha}` ✅ Available
- **Complexity:** Low - query parameter API call
- **Value:** Medium - provides commit metadata
- **Recommendation:** ⭐ **Good candidate**

#### VIBEK-29: Implement compare_commit_to_head tool
- **Endpoint:** `GET /api/task-attempts/{id}/commit-compare?sha={sha}` ✅ Available
- **Complexity:** Low - query parameter API call
- **Value:** Medium - helps understand divergence
- **Recommendation:** ⭐ **Good candidate**

#### VIBEK-27: Implement abort_conflicts tool
- **Endpoint:** `POST /api/task-attempts/{id}/conflicts/abort` ✅ Available
- **Complexity:** Low - simple POST
- **Value:** High - critical for conflict resolution workflows
- **Recommendation:** ⭐⭐ **High value, implement soon**

#### VIBEK-36: Implement replace_execution_process tool
- **Endpoint:** `POST /api/task-attempts/{id}/replace-process` ✅ Available
- **Complexity:** Medium - requires process ID and prompt
- **Value:** High - enables retry/redo workflows
- **Recommendation:** ⭐⭐ **High value, good next step**

#### VIBEK-42: Implement stream_process_logs tool
- **Endpoint:** `GET /api/execution-processes/{id}/raw-logs/ws` (WebSocket) ✅ Available
- **Endpoint:** `GET /api/execution-processes/{id}/normalized-logs/ws` (WebSocket) ✅ Available
- **Complexity:** High - requires WebSocket handling in MCP
- **Value:** High - real-time monitoring
- **Recommendation:** ⚠️ **High complexity, defer until WebSocket support confirmed**

### Low Priority

#### VIBEK-35: Implement open_attempt_in_editor tool
- **Endpoint:** `POST /api/task-attempts/{id}/open-editor` ✅ Available
- **Complexity:** Low - simple POST
- **Value:** Low - convenience feature
- **Recommendation:** Implement if time permits

#### VIBEK-33: Implement delete_attempt_file tool
- **Endpoint:** `POST /api/task-attempts/{id}/delete-file?file_path={path}` ✅ Available
- **Complexity:** Low - query parameter POST
- **Value:** Low - niche use case
- **Recommendation:** Implement if time permits

---

## 🔄 BLOCKED - Awaiting NPM Package Release

These are **fully implemented** in code but disabled until next NPM release:

### VIBEK-24: get_attempt_artifacts
- **Status:** Code complete (commit `246958b`), temporarily disabled
- **Reason:** `/api/task-attempts/{id}/artifacts` not in NPM package
- **Action:** Comment added explaining status

### VIBEK-26: rebase_task_attempt
- **Status:** Code complete (commit `10b13fe`), temporarily disabled
- **Reason:** `/api/task-attempts/{id}/rebase` not in NPM package
- **Action:** Comment added explaining status

### VIBEK-30: push_attempt_branch
- **Status:** Code complete (commit `4958cac`), temporarily disabled
- **Reason:** `/api/task-attempts/{id}/push` not in NPM package
- **Action:** Comment added explaining status

### VIBEK-31: create_github_pr
- **Status:** Code complete (commit `fd98a7d`), temporarily disabled
- **Reason:** `/api/task-attempts/{id}/pr` not in NPM package
- **Action:** Comment added explaining status

---

## ❌ NOT VIABLE - Require Custom Backend Endpoints

These issues need new API endpoints that don't exist yet:

### VIBEK-17: Implement update_executor_profile tool
- **Required Endpoint:** `PUT /api/executor-profiles/{id}`
- **Status:** NO SUCH ENDPOINT - would need custom backend

### VIBEK-18: Implement delete_executor_profile tool
- **Required Endpoint:** `DELETE /api/executor-profiles/{id}`
- **Status:** NO SUCH ENDPOINT - would need custom backend

### VIBEK-19: Implement create_executor_profile tool
- **Required Endpoint:** `POST /api/executor-profiles`
- **Status:** NO SUCH ENDPOINT - would need custom backend

### VIBEK-20: Implement get_executor_profile tool
- **Required Endpoint:** `GET /api/executor-profiles/{id}`
- **Status:** NO SUCH ENDPOINT - would need custom backend

### Phase 3 Epics (Low Priority)

#### VIBEK-48: Phase 3: Tags & Labels (5 tools)
- **Endpoints:** Tags API exists (`/api/tags`) ✅
- **Complexity:** Medium - 5 tools to implement
- **Value:** Medium - organizational features
- **Recommendation:** ⭐ **Viable but lower priority**

#### VIBEK-49: Phase 3: Authentication (3 tools)
- **Endpoints:** Auth API exists (`/api/auth/github/*`) ✅
- **Complexity:** Medium - 3 tools to implement
- **Value:** High - enables GitHub workflows
- **Recommendation:** ⭐⭐ **Good candidate after core tools**

#### VIBEK-50: Phase 3: Approval Workflow (4 tools)
- **Endpoints:** Approvals API exists (`/api/approvals/*`) ✅
- **Complexity:** Medium - 4 tools to implement
- **Value:** High - human-in-the-loop workflows
- **Recommendation:** ⭐⭐ **Good candidate after core tools**

---

## 📊 Recommended Implementation Order

Based on **value, complexity, and endpoint availability**:

### Batch 1: High-Value Quick Wins (Implement First)
1. **VIBEK-27** - abort_conflicts (High value, low complexity) ⭐⭐
2. **VIBEK-36** - replace_execution_process (High value, medium complexity) ⭐⭐
3. **VIBEK-34** - get_branch_status (Medium value, low complexity) ⭐

### Batch 2: Git Operations Suite
4. **VIBEK-28** - get_commit_info (Complements existing tools) ⭐
5. **VIBEK-29** - compare_commit_to_head (Completes git suite) ⭐
6. **VIBEK-32** - change_target_branch (Flexible workflows) ⭐

### Batch 3: Convenience Features
7. **VIBEK-35** - open_attempt_in_editor (Low complexity)
8. **VIBEK-33** - delete_attempt_file (Low complexity)

### Batch 4: Phase 3 Epics (If Time Permits)
9. **VIBEK-49** - Phase 3: Authentication (3 tools)
10. **VIBEK-50** - Phase 3: Approval Workflow (4 tools)
11. **VIBEK-48** - Phase 3: Tags & Labels (5 tools)

### Future: WebSocket & Complex Features
12. **VIBEK-42** - stream_process_logs (High complexity - WebSocket)

---

## 🎯 Next Steps Recommendation

**For immediate progress:**

1. **Start with VIBEK-27** (abort_conflicts)
   - Quick implementation
   - Unblocks conflict resolution workflows
   - High user value

2. **Follow with VIBEK-36** (replace_execution_process)
   - Enables retry/redo patterns
   - Critical for iterative development
   - Complements existing process tools

3. **Complete git suite with VIBEK-34, 28, 29, 32**
   - Provides comprehensive git operations
   - Logical grouping for testing
   - Natural workflow progression

4. **Consider Phase 3 epics** once core tools are stable
   - Auth, Approvals, Tags all have available endpoints
   - Would significantly expand MCP capabilities
   - Good opportunity for broader testing

---

## ⚠️ Important Notes

### Endpoint Verification Strategy
Before implementing any tool:
1. Test endpoint availability: `curl -s http://localhost:3105/api/[endpoint]`
2. Check response structure matches expectations
3. Verify all required query parameters work
4. Document any discrepancies

### Risk Assessment

**Low Risk:**
- Tools using existing GET endpoints (28, 29, 34)
- Simple POST operations (27, 35)

**Medium Risk:**
- Tools with complex payloads (36)
- Multi-step operations

**High Risk:**
- WebSocket tools (42) - May require MCP protocol updates
- Phase 3 epics - Broader surface area for bugs

### Testing Approach

For each tool implementation:
1. Unit test the MCP tool handler
2. Integration test against live backend
3. Manual verification with realistic use case
4. Document in `MCP_TOOLS_COMPLETE.md`

---

## 📈 Progress Tracking

**Current Status:**
- ✅ 17 tools complete and working
- 🔄 4 tools complete but awaiting deployment
- ⭐ 8-10 tools immediately implementable
- ❌ 4 tools blocked on new backend endpoints
- 📦 3 phase epics (12 tools) viable for future work

**Estimated Effort:**
- Batch 1 (3 tools): 2-3 hours
- Batch 2 (4 tools): 3-4 hours
- Batch 3 (2 tools): 1-2 hours
- Phase 3 epics (12 tools): 8-12 hours

**Total Viable Work Available:** ~20 tools ready to implement with current backend!
