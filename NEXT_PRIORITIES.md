# Vibe Kanban - Next Priorities Analysis

## Current Status
- ✅ **Phase 1**: TurboMCP Framework - COMPLETE
- ✅ **Phase 2**: All management tools (Project, Task, System, Execution Process) - COMPLETE
- 📊 **17 MCP tools** currently deployed and working

## Remaining Issues by Priority

### 🔴 URGENT Priority (1 issue)

#### VIBEK-24: get_attempt_artifacts 
- **Priority**: Urgent
- **Component**: MCP Server
- **Status**: Backlog
- **Description**: Access work products from completed attempts
- **Why Critical**: Essential for reviewing what was produced by coding agents
- **Recommendation**: ⭐ **HIGHEST PRIORITY** - Should be next

### 🟠 HIGH Priority (4 issues)

#### VIBEK-30: push_attempt_branch
- Push attempt branch to remote repository
- Important for CI/CD workflows and collaboration

#### VIBEK-31: create_github_pr  
- Create GitHub pull requests for attempts
- Critical for code review workflow integration

#### VIBEK-26: rebase_task_attempt
- Rebase attempt branch on latest target
- Essential for keeping work up-to-date

#### VIBEK-20: get_executor_profile
- Get detailed executor profile configuration
- Important for understanding agent capabilities

### 🟡 MEDIUM Priority (10 issues)

#### Execution Process Tools
- VIBEK-42: stream_process_logs (WebSocket real-time streaming)
- VIBEK-36: replace_execution_process (restart processes)

#### Git/Branch Management
- VIBEK-32: change_target_branch (change merge target)
- VIBEK-34: get_branch_status (ahead/behind counts)
- VIBEK-29: compare_commit_to_head (diff analysis)
- VIBEK-28: get_commit_info (commit details)
- VIBEK-27: abort_conflicts (conflict resolution)

#### Configuration
- VIBEK-17: update_executor_profile
- VIBEK-19: create_executor_profile

#### Phase 3 Tracking Issues
- VIBEK-43: start_dev_server (✅ DONE)
- VIBEK-41: get_process_normalized_logs (✅ DONE)

### 🟢 LOW Priority (6 issues)

#### Phase 3 Epics
- VIBEK-50: Phase 3: Approval Workflow (4 tools)
- VIBEK-49: Phase 3: Authentication (3 tools)  
- VIBEK-48: Phase 3: Tags & Labels (5 tools)

#### Other Tools
- VIBEK-18: delete_executor_profile
- VIBEK-33: delete_attempt_file
- VIBEK-35: open_attempt_in_editor

### ⚪ No Priority (1 issue)

- VIBEK-14: Expand MCP Server with Comprehensive Tool Coverage (Epic/Tracking)

## Recommended Next Steps

### Option 1: Focus on Critical Workflow Tools (Recommended)
Complete the most critical workflow integration tools:

1. **VIBEK-24** (Urgent): get_attempt_artifacts
2. **VIBEK-31** (High): create_github_pr  
3. **VIBEK-30** (High): push_attempt_branch
4. **VIBEK-26** (High): rebase_task_attempt

**Why**: These enable complete code review and collaboration workflows. They're the missing pieces for production usage.

**Estimated Effort**: 2-3 implementation sessions (similar to Phase 2)

### Option 2: Git/Branch Management Suite
Focus on git-related tools for better version control:

1. **VIBEK-26** (High): rebase_task_attempt
2. **VIBEK-34** (Medium): get_branch_status
3. **VIBEK-29** (Medium): compare_commit_to_head
4. **VIBEK-28** (Medium): get_commit_info
5. **VIBEK-27** (Medium): abort_conflicts
6. **VIBEK-32** (Medium): change_target_branch

**Why**: Provides comprehensive git workflow management.

**Estimated Effort**: 3-4 implementation sessions

### Option 3: Phase 3 Features (Authentication & Approvals)
Implement the Phase 3 epics for advanced workflows:

1. **VIBEK-49**: Authentication tools (3 tools)
2. **VIBEK-50**: Approval workflow tools (4 tools)
3. **VIBEK-48**: Tags & labels (5 tools)

**Why**: Enables enterprise features and human-in-the-loop workflows.

**Estimated Effort**: 5-6 implementation sessions

### Option 4: Real-time & Advanced Features
Add WebSocket streaming and advanced capabilities:

1. **VIBEK-42** (Medium): stream_process_logs (WebSocket)
2. **VIBEK-36** (Medium): replace_execution_process
3. **VIBEK-20** (High): get_executor_profile

**Why**: Enhances real-time monitoring and control.

**Estimated Effort**: 2-3 implementation sessions

## My Recommendation: Option 1

**Implement the Critical Workflow Tools** (VIBEK-24, 31, 30, 26)

### Rationale:
1. **get_attempt_artifacts** is marked URGENT and is critical for reviewing work
2. **GitHub PR creation** completes the integration with standard development workflows
3. **Branch pushing** enables CI/CD pipelines
4. **Rebasing** keeps work current and mergeable

These 4 tools would make Vibe Kanban production-ready for team collaboration and code review workflows.

### Implementation Strategy:
Similar to Phase 2, these could be implemented in parallel:
- Each tool is independent
- All follow established patterns
- 2-3 require new API endpoints
- All need MCP tool wrappers

**Estimated Timeline**: Can be completed in one focused development session with parallel execution.

## Current Tool Inventory

### ✅ Implemented (17 tools)
**Basic CRUD:**
- create_task, list_tasks, get_task, update_task, delete_task
- list_projects

**Task Attempts:**  
- start_task_attempt, list_task_attempts, get_task_attempt
- create_followup_attempt, merge_task_attempt

**Execution Processes:**
- list_execution_processes, get_execution_process, stop_execution_process
- get_process_raw_logs, get_process_normalized_logs
- start_dev_server

### ⏳ Remaining (22 tools)
- 4 Urgent/High priority workflow tools
- 10 Medium priority git/config tools
- 8 Low priority Phase 3 tools

## Success Metrics for Next Phase

### If implementing Critical Workflow Tools:
- ✅ Can retrieve artifacts from completed attempts
- ✅ Can create GitHub PRs from MCP
- ✅ Can push branches to remote
- ✅ Can rebase attempts on latest changes
- ✅ Complete code review workflow from MCP

### Total Tool Count Target:
- Current: 17 tools
- After Critical Workflow: 21 tools
- After Git Management: 27 tools  
- After Phase 3: 39 tools
- Ultimate Goal: 35-40 comprehensive tools

---

**Recommendation**: Start with **VIBEK-24 (get_attempt_artifacts)** as it's marked URGENT and is likely blocking other workflows.
