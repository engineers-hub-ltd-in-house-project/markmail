---
description: Mark a task as complete and update tracking
allowed-tools: Bash, Read, Write, Edit, MultiEdit, TodoWrite
---

# Complete Task

Mark task as complete for feature: **$ARGUMENTS**

## Task Completion Process

### Current Status

- Tasks file: @.kiro/specs/$ARGUMENTS/tasks.md
- Spec metadata: @.kiro/specs/$ARGUMENTS/spec.json
- Todo tracking: Current TodoWrite status

## Completion Workflow

### 1. Verify Task Completion

Before marking complete, verify:

- [ ] All tests pass for the task
- [ ] Code follows project conventions
- [ ] Requirements are satisfied
- [ ] No remaining TODOs in code
- [ ] Documentation is updated if needed

### 2. Update Task Status

Mark the specified task as complete:

- Change `- [ ]` to `- [x]` in tasks.md
- Add completion timestamp as comment
- Note any important implementation details

### 3. Calculate Progress

After marking complete:

- Count total tasks
- Count completed tasks
- Calculate completion percentage
- Identify next task to work on

### 4. Update Spec Metadata

Update spec.json with:

```json
{
  "phases": {
    "tasks": {
      "status": "in-progress",
      "progress": {
        "completed": [number],
        "total": [number],
        "percentage": [percentage]
      }
    }
  },
  "updated_at": "[current_timestamp]"
}
```

### 5. TodoWrite Sync

Update TodoWrite tracking:

- Mark task as completed
- Remove from active todos
- Add next task if continuing

### 6. Generate Summary

Provide summary including:

- Task completed: #[number] - [description]
- Time taken: [if tracked]
- Completion rate: [X/Y] tasks ([percentage]%)
- Next task: #[number] - [description]
- Estimated remaining effort: [hours]

## Verification Checklist

### Code Quality

- Tests written and passing
- Code reviewed for best practices
- No linting errors
- Type checking passes

### Requirements

- Mapped requirements satisfied
- Acceptance criteria met
- Edge cases handled
- Error scenarios covered

### Documentation

- Code comments added where needed
- API documentation updated
- README updated if applicable

## Next Steps

### Suggest Next Action

Based on completion status:

- If more tasks in current phase: Suggest `/kiro:implement-next`
- If phase complete: Suggest moving to next phase
- If all tasks complete: Suggest `/kiro:spec-status` for final review

### Blockers and Issues

If any blockers were encountered:

- Document the issue
- Suggest resolution approach
- Update tasks if new work identified

## Instructions

1. **Verify completion criteria** are met for the task
2. **Update tasks.md** marking task with `[x]`
3. **Calculate progress** and update spec.json
4. **Sync with TodoWrite** for tracking
5. **Generate summary** of completion status
6. **Suggest next steps** based on progress

This ensures proper tracking and maintains momentum through the implementation
phase.
