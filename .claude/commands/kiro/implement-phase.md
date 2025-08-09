---
description: Implement all tasks in a specific phase systematically
allowed-tools: Bash, Read, Write, Edit, MultiEdit, Grep, TodoWrite, Task
---

# Implement Phase

Systematically implement all tasks in a specific phase for feature:
**$ARGUMENTS**

## Phase Implementation Strategy

### Available Phases

Parse tasks.md to identify phases:

- Phase 1: Database Setup and Data Models
- Phase 2: Backend Data Models and Types
- Phase 3: Backend Services Implementation
- Phase 4: WebSocket Implementation
- Phase 5: API Endpoints
- Phase 6: Frontend Components
- Phase 7: Frontend Services and State
- Phase 8: Integration and Testing
- Phase 9: Final Integration

### Current Status

- Tasks file: @.kiro/specs/$ARGUMENTS/tasks.md
- Spec metadata: @.kiro/specs/$ARGUMENTS/spec.json
- Phase progress: Identify completed vs pending tasks per phase

## Implementation Approach

### 1. Phase Selection

Either:

- Auto-detect next incomplete phase
- Use phase number if provided in arguments
- Show phase menu for user selection

### 2. Dependency Check

Before starting phase:

- Verify prerequisite phases are complete
- Check for blocking dependencies
- Ensure environment is ready (database, services, etc.)

### 3. Phase Preparation

Set up for phase implementation:

- List all tasks in the phase
- Create necessary directories
- Set up test infrastructure
- Load relevant steering documents

### 4. Sequential Task Execution

For each task in phase:

1. **Read task requirements**

   - Parse task description
   - Identify files to create/modify
   - Note requirements mapping

2. **Implement with TDD**

   - Write tests first
   - Implement solution
   - Verify tests pass
   - Refactor if needed

3. **Validate implementation**

   - Run all tests
   - Check type safety
   - Verify requirements met

4. **Mark complete**
   - Update task checkbox
   - Track in TodoWrite
   - Log any issues

### 5. Phase Completion

After all tasks in phase:

- Run integration tests for phase
- Verify phase objectives met
- Update spec.json with phase status
- Generate phase completion report

## Parallel Execution Strategy

### Identify Parallelizable Tasks

Some phases allow parallel work:

- Frontend and backend development (after Phase 2)
- Multiple component development
- Independent service implementations

### Task Dependencies

Track dependencies within phase:

- Tasks that must be sequential
- Tasks that can run parallel
- Shared resources or files

## Progress Tracking

### Real-time Updates

During phase implementation:

- Update TodoWrite with current task
- Show progress bar: [████████░░] 80%
- Estimate time remaining
- Log any blockers

### Phase Metrics

Track for each phase:

- Start time
- Tasks completed
- Time per task
- Issues encountered
- Test coverage

## Error Handling

### Task Failure

If a task fails:

- Log the error
- Attempt automatic fix if possible
- Prompt for user intervention
- Option to skip and continue

### Rollback Strategy

If phase cannot complete:

- Document incomplete state
- Preserve completed work
- Create recovery plan
- Update tasks with issues

## Completion Report

### Phase Summary

Generate report including:

```
Phase X: [Name] - COMPLETE
========================
✅ Tasks Completed: X/Y
⏱️ Time Taken: X hours
📊 Test Coverage: X%
🐛 Issues Resolved: X

Key Achievements:
- [Achievement 1]
- [Achievement 2]

Next Phase: [Name]
Ready to proceed: [Yes/No]
```

### Recommendations

Based on phase completion:

- Suggest next phase
- Recommend testing focus areas
- Highlight any technical debt
- Propose optimizations

## Instructions

1. **Identify target phase** from arguments or auto-detect
2. **Verify dependencies** and prerequisites
3. **Set up phase environment** with necessary files
4. **Execute tasks sequentially** following TDD
5. **Track progress** in real-time with TodoWrite
6. **Generate completion report** when phase is done
7. **Recommend next steps** based on overall progress

This enables efficient phase-by-phase implementation while maintaining quality
and tracking.
