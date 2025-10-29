# Letta Agent Integration for Vibe Kanban

## Vision

Create a self-managing Vibe Kanban system where Letta agents autonomously:
- Monitor project progress and task completion
- Assign tasks to appropriate coding agents based on expertise
- Coordinate parallel work to avoid conflicts
- Review code changes and provide feedback
- Escalate blockers to human developers
- Learn from past successes/failures to improve task routing

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                    Letta Agent Layer                        │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐    │
│  │   Project    │  │     Task     │  │    Code      │    │
│  │   Manager    │  │  Coordinator │  │   Reviewer   │    │
│  │    Agent     │  │    Agent     │  │    Agent     │    │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘    │
│         │                 │                  │             │
│         └─────────────────┴──────────────────┘             │
│                           │                                │
├───────────────────────────┼────────────────────────────────┤
│                    Vibe Kanban MCP Tools                   │
├───────────────────────────┼────────────────────────────────┤
│                           │                                │
│  ┌────────────────────────▼─────────────────────────┐     │
│  │         Vibe Kanban Task Server                  │     │
│  │  • Tasks   • Attempts   • Processes             │     │
│  │  • Projects • Executors  • Git Operations       │     │
│  └──────────────────────────────────────────────────┘     │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## Agent Roles

### 1. Project Manager Agent
**Purpose**: High-level project oversight and planning

**Archival Memory**:
- Project goals and success criteria
- Team member expertise profiles
- Historical task completion times
- Common blocker patterns
- Project dependencies and relationships

**Core Memory**:
```json
{
  "persona": "Senior engineering manager with expertise in task prioritization",
  "current_projects": ["project-uuid-1", "project-uuid-2"],
  "active_focus": "Ensuring MCP tool expansion stays on track",
  "known_blockers": []
}
```

**Tools**:
- `list_projects`
- `list_tasks` (with status filtering)
- `get_task`
- `create_task`
- `update_task`
- `list_task_attempts`

**Responsibilities**:
- Monitor project health (velocity, blocker count)
- Create new tasks from high-level requirements
- Prioritize task backlog
- Identify dependency chains
- Report project status to humans

**Example Behavior**:
```
Every 30 minutes:
1. Check for blocked tasks (attempts with no progress)
2. Review completed tasks and close them
3. Identify next high-priority tasks
4. Create new tasks if backlog is low
5. Update project documentation
```

### 2. Task Coordinator Agent
**Purpose**: Intelligent task assignment and coordination

**Archival Memory**:
- Coding agent capability profiles
  - CLAUDE_CODE: General purpose, good at Rust/TypeScript
  - GEMINI: Fast iterations, good at testing
  - CODEX: API integrations, documentation
- Historical success rates per agent per task type
- Parallel work conflict patterns
- Optimal task batch sizes

**Core Memory**:
```json
{
  "persona": "Technical lead coordinating multiple developers",
  "active_attempts": {
    "8cf78a17-2d6c-44ef-ab2f-e4977176865c": {
      "task": "get_execution_process",
      "agent": "CLAUDE_CODE",
      "started": "2025-10-29T05:41:19Z",
      "status": "running",
      "last_check": "2025-10-29T05:45:00Z"
    }
  },
  "parallel_work_map": {
    "task_server.rs": ["8cf78a17-...", "a376f56e-..."]
  },
  "next_batch_ready": ["task-id-1", "task-id-2"]
}
```

**Tools**:
- All task/attempt tools
- `list_execution_processes`
- `get_execution_process`
- `stop_execution_process`
- `start_task_attempt`
- `create_followup_attempt`
- `list_executor_profiles`

**Decision Algorithm**:
```python
def assign_task(task):
    # 1. Analyze task type
    task_type = classify_task(task.description)
    
    # 2. Check for conflicts with active work
    conflicts = find_file_conflicts(task, active_attempts)
    if conflicts:
        return "WAIT"
    
    # 3. Select best agent based on:
    #    - Historical success rate for this task type
    #    - Current workload
    #    - Availability
    agent = select_optimal_agent(task_type, historical_data)
    
    # 4. Start attempt
    create_task_attempt(task.id, agent, base_branch="main")
    
    # 5. Add to active monitoring
    register_active_attempt(attempt_id, task, agent)
```

**Responsibilities**:
- Monitor task queue for new TODO tasks
- Select appropriate coding agent for each task
- Start task attempts with correct executor profiles
- Detect and prevent parallel work conflicts
- Restart failed attempts with different agents
- Coordinate batch processing of similar tasks

**Example Behavior**:
```
Every 5 minutes:
1. Check for TODO tasks without attempts
2. Identify safe-to-parallelize tasks
3. Check agent availability/workload
4. Start attempts for 3-5 tasks in parallel
5. Monitor active attempts for progress
6. Detect stalled attempts (no commits in 15min)
7. Stop/restart problematic attempts
```

### 3. Code Reviewer Agent
**Purpose**: Automated code review and quality assurance

**Archival Memory**:
- Code style guidelines (Rust, TypeScript)
- Common anti-patterns to catch
- Test coverage requirements
- Performance considerations
- Security best practices

**Core Memory**:
```json
{
  "persona": "Senior code reviewer focused on quality and best practices",
  "review_queue": ["attempt-id-1", "attempt-id-2"],
  "recent_reviews": [
    {
      "attempt_id": "8cf78a17-...",
      "verdict": "approved_with_suggestions",
      "key_points": ["Good error handling", "Missing edge case test"]
    }
  ],
  "patterns_to_watch": [
    "Ensure schemars descriptions on all fields",
    "Verify error handling follows err_str pattern"
  ]
}
```

**Tools**:
- `get_task_attempt`
- `list_execution_processes`
- `get_process_raw_logs` (to review output)
- `get_commit_info`
- `compare_commit_to_head`
- `create_followup_attempt` (for fixes)
- `merge_task_attempt` (when approved)

**Review Criteria**:
```yaml
automated_checks:
  - build_success: true
  - tests_pass: true
  - no_warnings: preferred
  - follows_patterns: true

code_quality:
  - error_handling: "Uses McpError consistently"
  - documentation: "All public functions documented"
  - schema_quality: "All fields have descriptions"
  - type_safety: "No unwrap() in production code"
  - testing: "Happy path and error cases covered"

review_process:
  1. Check build/test results from execution logs
  2. Review git diff for code changes
  3. Verify follows established patterns
  4. Check for security issues (API exposure, auth)
  5. Provide feedback or approve
```

**Responsibilities**:
- Monitor completed task attempts
- Review code changes automatically
- Provide detailed feedback comments
- Approve and merge when quality bar is met
- Request followup attempts for fixes
- Learn patterns from merged code

**Example Behavior**:
```
Every 10 minutes:
1. List task attempts with completed processes
2. For each completed attempt:
   a. Get execution logs
   b. Check for build/test success
   c. Get commit diff
   d. Run automated quality checks
   e. If passed: merge attempt
   f. If issues: create followup with feedback
3. Update review history in memory
```

## Letta Agent Service Design

### Service: `letta_agent_coordinator.py`

**Location**: `/opt/stacks/vibe-kanban/services/letta_agent_coordinator.py`

**Purpose**: Bridge between Vibe Kanban and Letta agents

**Architecture**:
```python
class LettaAgentCoordinator:
    def __init__(self):
        self.letta_client = LettaClient(base_url=LETTA_API_URL)
        self.vibe_kanban_url = BACKEND_URL
        
        # Initialize agents
        self.project_manager = self.get_or_create_agent(
            name="vk_project_manager",
            tools=["list_projects", "list_tasks", "create_task", ...]
        )
        
        self.task_coordinator = self.get_or_create_agent(
            name="vk_task_coordinator", 
            tools=["start_task_attempt", "list_execution_processes", ...]
        )
        
        self.code_reviewer = self.get_or_create_agent(
            name="vk_code_reviewer",
            tools=["get_task_attempt", "merge_task_attempt", ...]
        )
    
    async def run_coordination_loop(self):
        """Main coordination loop"""
        while True:
            # Project Manager: Every 30 minutes
            if should_run_project_manager():
                await self.run_project_manager_cycle()
            
            # Task Coordinator: Every 5 minutes
            if should_run_task_coordinator():
                await self.run_task_coordinator_cycle()
            
            # Code Reviewer: Every 10 minutes
            if should_run_code_reviewer():
                await self.run_code_reviewer_cycle()
            
            await asyncio.sleep(60)  # Check every minute
    
    async def run_task_coordinator_cycle(self):
        """Task assignment and monitoring"""
        prompt = f"""
        You are coordinating task execution for Vibe Kanban.
        
        Current time: {datetime.now()}
        
        Please:
        1. Check for TODO tasks without attempts
        2. Analyze which tasks can be worked in parallel
        3. Start task attempts for 3-5 safe tasks
        4. Check on active attempts for progress
        5. Restart any stalled attempts
        
        Use the Vibe Kanban MCP tools to accomplish this.
        Provide a summary of actions taken.
        """
        
        response = await self.letta_client.send_message(
            agent_id=self.task_coordinator.id,
            messages=[{"role": "user", "content": prompt}]
        )
        
        log_coordination_action("task_coordinator", response)
```

### Agent Provisioning Script

**Location**: `/opt/stacks/vibe-kanban/scripts/provision_letta_agents.py`

```python
#!/usr/bin/env python3
"""
Provision Letta agents for Vibe Kanban coordination
"""

import os
from letta import LettaClient

LETTA_API_URL = os.getenv("LETTA_API_URL", "https://letta.oculair.ca")
LETTA_PASSWORD = os.getenv("LETTA_PASSWORD")

# Vibe Kanban MCP server configs
VIBE_KANBAN_MCP_SERVERS = {
    "vibe-kanban-tasks": "http://192.168.50.90:9717/mcp",
    "vibe-kanban-system": "http://192.168.50.90:9718/mcp"
}

def provision_project_manager():
    """Create project manager agent"""
    
    agent_config = {
        "name": "vk_project_manager",
        "persona": """You are a senior engineering manager overseeing multiple projects.
        
Your responsibilities:
- Monitor project health and velocity
- Prioritize task backlog
- Create new tasks from requirements
- Report project status
- Identify blockers and dependencies

Your approach:
- Data-driven decisions
- Clear communication
- Proactive problem identification
- Team empowerment
""",
        "human": """You work with a team of AI coding agents (CLAUDE_CODE, GEMINI, CODEX, etc).
        
Your reports are for the human engineering lead who checks in periodically.
Keep them informed of progress, blockers, and key decisions.
""",
        "tools": [
            "list_projects",
            "list_tasks", 
            "get_task",
            "create_task",
            "update_task",
            "list_task_attempts"
        ],
        "mcp_servers": ["vibe-kanban-tasks"]
    }
    
    # Archival memory setup
    archival_memory = {
        "project_guidelines": """
        Vibe Kanban Project Guidelines:
        - MCP tools follow TurboMCP patterns
        - All fields must have schemars descriptions
        - Error handling uses McpError types
        - Backend integration via REST API
        - Independent tools can be parallelized
        """,
        
        "task_classification": """
        Task Types:
        - mcp_tool: Implementing new MCP tools (1-2 hours)
        - backend_api: New REST endpoints (2-4 hours)
        - ui_feature: Frontend components (3-6 hours)
        - bug_fix: Fixing issues (1-3 hours)
        - documentation: Updating docs (30min-1 hour)
        """,
        
        "agent_expertise": """
        Coding Agent Capabilities:
        - CLAUDE_CODE: Best all-rounder, Rust/TypeScript/React
        - GEMINI: Fast iterations, good for quick fixes
        - CODEX: API integrations, clear documentation
        - AMP: Complex refactoring, performance optimization
        - CURSOR: UI/UX focused, React expertise
        """
    }
    
    return create_agent(agent_config, archival_memory)

def provision_task_coordinator():
    """Create task coordinator agent"""
    
    agent_config = {
        "name": "vk_task_coordinator",
        "persona": """You are a technical lead coordinating multiple developers.

Your responsibilities:
- Assign tasks to appropriate coding agents
- Start task attempts with correct executor profiles
- Monitor active attempts for progress
- Detect and prevent parallel work conflicts
- Restart failed attempts
- Coordinate batch processing

Your approach:
- Intelligent task routing based on agent strengths
- Proactive conflict detection
- Continuous monitoring
- Quick recovery from failures
""",
        "human": """You work with AI coding agents and coordinate their work.
        
Report significant decisions, conflicts resolved, and any issues requiring human judgment.
""",
        "tools": [
            "list_tasks",
            "get_task", 
            "start_task_attempt",
            "list_task_attempts",
            "get_task_attempt",
            "create_followup_attempt",
            "list_execution_processes",
            "get_execution_process",
            "stop_execution_process",
            "list_executor_profiles"
        ],
        "mcp_servers": ["vibe-kanban-tasks", "vibe-kanban-system"]
    }
    
    archival_memory = {
        "conflict_detection": """
        Conflict Detection Rules:
        - Same file modifications: Check git diffs
        - task_server.rs: Max 2 parallel (top/bottom of file)
        - system_server.rs: Max 2 parallel
        - New files: No conflicts
        - Different routes: Safe to parallelize
        """,
        
        "task_routing": """
        Task Routing Algorithm:
        1. Classify task type (mcp_tool, backend_api, etc)
        2. Check agent historical success rates
        3. Verify no file conflicts with active work
        4. Select agent with best match
        5. Start attempt with appropriate executor
        
        Fallback strategy:
        - Primary fails -> Try secondary agent
        - Multiple failures -> Escalate to human
        """,
        
        "monitoring_patterns": """
        Progress Indicators:
        - Git commits within 15 minutes
        - Build/test execution
        - Process status changes
        - Log output patterns
        
        Stall Detection:
        - No commits in 15 minutes after start
        - Process stuck in running >30 minutes
        - Repeated compilation errors
        - No log output in 10 minutes
        """
    }
    
    return create_agent(agent_config, archival_memory)

def provision_code_reviewer():
    """Create code reviewer agent"""
    
    agent_config = {
        "name": "vk_code_reviewer",
        "persona": """You are a senior code reviewer focused on quality and best practices.

Your responsibilities:
- Review completed task attempts
- Verify build and test success
- Check code quality and patterns
- Provide constructive feedback
- Approve and merge quality code
- Request fixes when needed

Your approach:
- Thorough but pragmatic
- Educational feedback
- Consistent standards
- Recognize good work
""",
        "human": """You ensure code quality before merging.
        
Report approval decisions, quality trends, and patterns needing human review.
""",
        "tools": [
            "get_task",
            "get_task_attempt",
            "list_task_attempts",
            "list_execution_processes",
            "get_process_raw_logs",
            "get_commit_info",
            "compare_commit_to_head",
            "create_followup_attempt",
            "merge_task_attempt"
        ],
        "mcp_servers": ["vibe-kanban-tasks"]
    }
    
    archival_memory = {
        "review_checklist": """
        Code Review Checklist:
        
        Build & Tests:
        ✓ Cargo build succeeds
        ✓ All tests pass
        ✓ No new warnings (or justified)
        
        Code Quality:
        ✓ Follows established patterns
        ✓ Error handling with McpError
        ✓ All fields have schemars descriptions
        ✓ No unwrap() in production paths
        ✓ Descriptive variable names
        
        Documentation:
        ✓ Tool description is clear
        ✓ Function comments added
        ✓ Complex logic explained
        
        Testing:
        ✓ Happy path covered
        ✓ Error cases tested
        ✓ Edge cases considered
        """,
        
        "common_issues": """
        Common Issues to Catch:
        
        1. Missing schema descriptions
           - All #[schemars(description = "...")] present
        
        2. Inconsistent error handling
           - Use McpError::internal or invalid_request
           - Provide context with err_str helper
        
        3. Type mismatches
           - UUIDs as strings in responses
           - Option types handled correctly
        
        4. Pattern violations
           - Request/Response struct naming
           - Tool placement in impl block
           - Consistent formatting
        """,
        
        "approval_criteria": """
        Auto-Approve When:
        - Build succeeds with no warnings
        - All tests pass
        - Follows established patterns exactly
        - Simple, focused change
        - Similar to merged examples
        
        Request Fixes When:
        - Build failures or test failures
        - Missing schema descriptions
        - Inconsistent patterns
        - Security concerns
        - Poor error handling
        
        Escalate to Human When:
        - Major architectural changes
        - New dependencies added
        - Performance implications
        - Security-sensitive code
        - Unclear requirements
        """
    }
    
    return create_agent(agent_config, archival_memory)

def main():
    """Provision all Letta agents"""
    client = LettaClient(base_url=LETTA_API_URL, token=LETTA_PASSWORD)
    
    print("Provisioning Letta Agents for Vibe Kanban...")
    
    # 1. Project Manager
    print("\n1. Creating Project Manager Agent...")
    pm = provision_project_manager()
    print(f"   ✓ Created: {pm.id}")
    
    # 2. Task Coordinator
    print("\n2. Creating Task Coordinator Agent...")
    tc = provision_task_coordinator()
    print(f"   ✓ Created: {tc.id}")
    
    # 3. Code Reviewer
    print("\n3. Creating Code Reviewer Agent...")
    cr = provision_code_reviewer()
    print(f"   ✓ Created: {cr.id}")
    
    print("\n✅ All agents provisioned successfully!")
    print("\nNext steps:")
    print("1. Start coordinator service: python services/letta_agent_coordinator.py")
    print("2. Monitor logs: tail -f logs/letta_coordinator.log")
    print("3. Check agent status via Letta UI")

if __name__ == "__main__":
    main()
```

## Integration Points

### 1. MCP Tool Requirements

The Letta agents need these Vibe Kanban MCP tools to be available:

**Essential (Already Implemented)**:
- ✅ `list_projects`
- ✅ `list_tasks`
- ✅ `get_task`
- ✅ `create_task`
- ✅ `update_task`
- ✅ `start_task_attempt`
- ✅ `list_task_attempts`
- ✅ `get_task_attempt`
- ✅ `create_followup_attempt`
- ✅ `merge_task_attempt`
- ✅ `list_execution_processes`

**In Progress** (5 parallel attempts active):
- ⏳ `get_execution_process`
- ⏳ `stop_execution_process`
- ⏳ `get_process_raw_logs`
- ⏳ `get_process_normalized_logs`
- ⏳ `start_dev_server`

**Still Needed**:
- ❌ `get_commit_info`
- ❌ `compare_commit_to_head`

### 2. Letta Server Configuration

The Letta agents need access to Vibe Kanban MCP servers:

```json
{
  "mcp_servers": {
    "vibe-kanban-tasks": {
      "transport": "http",
      "url": "http://192.168.50.90:9717/mcp"
    },
    "vibe-kanban-system": {
      "transport": "http", 
      "url": "http://192.168.50.90:9718/mcp"
    }
  }
}
```

### 3. Coordinator Service Structure

```
/opt/stacks/vibe-kanban/
├── services/
│   ├── letta_agent_coordinator.py     # Main service
│   ├── agent_definitions/
│   │   ├── project_manager.py
│   │   ├── task_coordinator.py
│   │   └── code_reviewer.py
│   └── utils/
│       ├── letta_client.py
│       ├── vibe_kanban_client.py
│       └── logging_config.py
├── scripts/
│   └── provision_letta_agents.py      # One-time setup
└── config/
    └── agent_config.yaml              # Agent configurations
```

## Workflow Example

### Scenario: User Creates High-Level Task

```
1. Human creates task via UI:
   "Implement complete Git operations suite"

2. Project Manager Agent (30min cycle):
   - Sees new high-level task
   - Breaks down into subtasks:
     * get_commit_info
     * compare_commit_to_head  
     * get_branch_status
     * change_target_branch
   - Creates 4 TODO tasks with proper descriptions
   - Updates archival memory with decomposition pattern

3. Task Coordinator Agent (5min cycle):
   - Detects 4 new TODO tasks
   - Analyzes for conflicts:
     * All are in task_server.rs (limit 2 parallel)
   - Starts 2 task attempts:
     * get_commit_info -> CLAUDE_CODE
     * get_branch_status -> GEMINI
   - Queues remaining 2 for next batch
   - Monitors both attempts every 5 minutes

4. After 15 minutes:
   - get_commit_info: 3 commits, tests passing
   - get_branch_status: 2 commits, build successful
   
5. Task Coordinator (next cycle):
   - Both attempts progressing well
   - Starts next 2 attempts:
     * compare_commit_to_head -> CODEX
     * change_target_branch -> CLAUDE_CODE

6. Code Reviewer Agent (10min cycle):
   - Checks get_commit_info attempt
   - Reviews execution logs: Build ✓, Tests ✓
   - Gets commit diff
   - Verifies: Schema descriptions ✓, Error handling ✓
   - Auto-approves and merges
   - Creates comment: "Great work! Schema is complete."

7. After all 4 complete:
   - Project Manager updates task status
   - Reports to human: "Git operations suite complete"
   - Suggests next priority: "Process management tools"
```

## Monitoring & Observability

### Agent Dashboards

Create monitoring UI showing:
- Agent status (active/idle/error)
- Current tasks being coordinated
- Approval/merge statistics
- Agent decision logs
- Conflict detections prevented
- Historical performance metrics

### Metrics to Track

```python
metrics = {
    "task_throughput": {
        "tasks_assigned_per_hour": 12,
        "tasks_completed_per_hour": 10,
        "average_task_duration_minutes": 45
    },
    
    "agent_performance": {
        "CLAUDE_CODE": {
            "success_rate": 0.95,
            "average_duration": 40,
            "task_types": ["mcp_tool", "backend_api"]
        }
    },
    
    "review_stats": {
        "auto_approved": 45,
        "requested_fixes": 5,
        "escalated_to_human": 2
    },
    
    "coordination": {
        "conflicts_prevented": 8,
        "parallel_batches": 15,
        "stalled_attempts_restarted": 3
    }
}
```

## Next Steps

### Phase 1: Foundation (Week 1)
1. ✅ Complete core MCP tools (in progress)
2. Create Letta agent provisioning script
3. Set up coordinator service infrastructure
4. Deploy project manager agent

### Phase 2: Automation (Week 2)
5. Deploy task coordinator agent
6. Test parallel task assignment
7. Implement conflict detection
8. Add monitoring dashboards

### Phase 3: Quality (Week 3)
9. Deploy code reviewer agent
10. Tune auto-approval criteria
11. Add learning from feedback
12. Performance optimization

### Phase 4: Production (Week 4)
13. Load testing with 20+ parallel tasks
14. Failure recovery testing
15. Human escalation workflows
16. Documentation and training

## Benefits

### For Developers
- **Reduced cognitive load**: Agents handle task routing
- **Faster feedback**: Auto-review in minutes vs hours
- **Better parallelization**: No manual coordination needed
- **Learning system**: Agents improve over time

### For Projects
- **Higher throughput**: 3-5x more tasks completed
- **Consistent quality**: Automated review standards
- **Faster iteration**: Quick feedback loops
- **Better visibility**: Real-time project dashboards

### For Organization
- **Scalability**: Handle 100+ tasks across projects
- **Knowledge capture**: Agent memory preserves patterns
- **Resource optimization**: Right agent for each task
- **24/7 operation**: Continuous progress

## Risk Mitigation

### Agent Failures
- **Problem**: Agent makes wrong decisions
- **Mitigation**: 
  - Human review of high-risk actions
  - Rollback capabilities
  - Decision logging for audit
  - Gradual autonomy increase

### Coordination Conflicts
- **Problem**: Multiple agents interfere
- **Mitigation**:
  - Clear role boundaries
  - Shared state via Letta memory
  - Lock mechanisms for critical operations
  - Conflict resolution protocols

### Quality Issues
- **Problem**: Auto-approved bad code
- **Mitigation**:
  - Conservative auto-approve criteria
  - Sample human review (10% random)
  - Revert capabilities
  - Continuous criteria tuning

## Conclusion

This Letta agent integration will transform Vibe Kanban from a task management tool into an **autonomous development orchestration platform**. The agents will handle the tedious coordination work, allowing humans to focus on high-level design and complex problem-solving.

The key is starting simple (project manager only), proving value, then gradually expanding autonomy as confidence grows.

---

**Status**: Design Complete  
**Ready for**: Implementation Phase 1  
**Dependencies**: Core MCP tools (95% complete)  
**Timeline**: 4 weeks to production  
