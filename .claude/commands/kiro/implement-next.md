---
description: Implement the next pending task from the specification
allowed-tools: Bash, Read, Write, Edit, MultiEdit, Grep, TodoWrite
---

# Implement Next Task

Start implementation of the next uncompleted task for feature: **$ARGUMENTS**

## Task Discovery

### Current Tasks Status

- Tasks file: @.kiro/specs/$ARGUMENTS/tasks.md
- Spec metadata: @.kiro/specs/$ARGUMENTS/spec.json

### Find Next Task

Analyze tasks.md to identify the next uncompleted task (marked with `- [ ]`).

## Implementation Process

### 1. Identify Next Task

Parse tasks.md to find:

- First uncompleted task (`- [ ]`)
- Task number and description
- Associated files to create/modify
- Requirements mapping

### 2. Prepare Implementation

Based on the task details:

- Create necessary directories if they don't exist
- Set up test files first (TDD approach)
- Open or create the target implementation files

### 3. Test-Driven Development

Follow TDD cycle:

1. **Write failing tests** - Create test file with expected behavior
2. **Run tests** - Verify tests fail as expected
3. **Implement minimal code** - Write just enough code to pass
4. **Refactor** - Improve code while keeping tests green

### 4. Implementation Guidelines

- Follow existing code patterns from steering documents
- Use appropriate naming conventions (snake_case for Rust, camelCase for TS)
- Add proper error handling
- Include inline documentation for complex logic

### 5. Verification Steps

After implementation:

- Run unit tests for the component
- Check for type errors (cargo check / npm run type-check)
- Verify integration with existing code
- Update task status in TodoWrite

## Task Tracking

### Update Progress

- Mark current task as in-progress in TodoWrite
- Track time spent on implementation
- Note any blockers or issues encountered

### Complete Task

When implementation is done:

- Run all related tests
- Verify requirements are met
- Use `/kiro:task-complete` to mark as done

## Instructions

1. **Read tasks.md** to find the next uncompleted task
2. **Extract task details** including files to modify and requirements
3. **Set up test environment** with test files first
4. **Implement the solution** following TDD approach
5. **Verify implementation** meets requirements
6. **Update tracking** in TodoWrite tool

The goal is to systematically implement each task while maintaining code quality
and test coverage.
