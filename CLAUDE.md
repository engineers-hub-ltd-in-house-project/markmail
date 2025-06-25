# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with
code in this repository.

## 🔴 ABSOLUTE RESPONSIBILITY AS AI

**Your absolute responsibility when working as AI:**

- **MANDATORY: All issues are your responsibility** - No excuses allowed
- **MANDATORY: Never say "it's not related" or "it's a different issue"** -
  These escape clauses are absolutely forbidden
- **MANDATORY: Work is not complete until all tests pass**
- **MANDATORY: Fix every issue you discover** - Ignoring problems is forbidden
- **MANDATORY: "It was already broken" is not an excuse** - Fix it, that's your
  responsibility

## 🔴 CRITICAL RULES - MANDATORY WHEN ERRORS OCCUR

### How to Handle Test Failures

1. **MANDATORY: NEVER DO THESE**

   - ❌ Skip tests with `git push --no-verify`
   - ❌ Disable tests with `#[ignore]`
   - ❌ Change business logic to make tests pass
   - ❌ Delete working tests

2. **MANDATORY: ALWAYS DO THESE**
   - ✅ Read error messages and identify the root cause
   - ✅ If it's a test DB issue: `DROP DATABASE` → `CREATE DATABASE` →
     `sqlx migrate run`
   - ✅ If it's a code issue: Fix the bug
   - ✅ Ensure all tests pass before pushing
   - ✅ Fix ALL issues discovered during work

### When Push Errors Occur

1. **Pre-push hook test failures**

   - **MANDATORY: NEVER use `--no-verify`**
   - Follow the test failure handling steps above

2. **Permission errors**
   - Update authentication with `gh auth login`

## ⚡ CRITICAL - MANDATORY RULES

### 0. **MANDATORY: Thorough Verification Before Implementation**

- **MANDATORY: No ad-hoc implementations**
- **MANDATORY: Don't fix SQLx type errors on the spot**
- **MANDATORY: Don't proceed without running tests**
- **MANDATORY: Before implementation, always:**
  1. Research existing code patterns
  2. Identify impact scope
  3. Run tests to understand current state
  4. Verify with small changes
  5. Immediately revert if issues arise

### 0.1. **MANDATORY: SQLx Work Rules**

- **MANDATORY: Don't carelessly change type casts**
- **MANDATORY: Stop when you see `Option<Option<T>>` errors**
- **MANDATORY: Mass changes to .sqlx files are danger signs**
- **MANDATORY: When SQLx type errors occur:**
  1. First understand the cause
  2. Check existing similar patterns
  3. Respond with minimal changes
  4. Always run tests after `cargo sqlx prepare`

### 0.2. **MANDATORY: Caution When Modifying Shared Modules**

- **MANDATORY: Don't carelessly modify shared modules when implementing new
  features**
- **MANDATORY: Be extra careful with core modules like subscriptions.rs,
  users.rs**
- **MANDATORY: When modifying shared modules:**
  1. Identify all tests affected by changes
  2. Confirm tests pass before changes
  3. Keep changes minimal
  4. Run all tests after changes

### 0.3. **MANDATORY: Handling Test Failures**

- **MANDATORY: Don't ignore test failures and proceed**
- **MANDATORY: Don't think "it's not my fault"**
- **MANDATORY: Don't downplay regressions**
- **MANDATORY: Don't make excuses like "not related to current context"**
- **MANDATORY: Don't dismiss as "a different issue"**
- **MANDATORY: When tests fail:**
  1. Stop work immediately
  2. Identify the cause of failure
  3. **Fix it regardless of the reason - no excuses**
  4. If fix is difficult, revert changes
  5. **NEVER push until all tests pass**

### 1. **MANDATORY: Never Delete or Modify Existing Migration Files**

- **MANDATORY: NEVER delete or modify database migration files
  (`backend/migrations/*.sql`)**
- Create new migration files with new timestamps when changes are needed
- Applied migrations are immutable

### 2. **MANDATORY: Never Disable Tests**

- **MANDATORY: When tests fail, fix the code, not the test**
- Using `#[ignore]` or `skip` is forbidden
- **MANDATORY: NEVER change business logic to make tests pass**
- Tests verify existing logic; don't change logic to fit tests
- **MANDATORY: NEVER delete working tests!**

### 3. **MANDATORY: Never Manipulate Database Directly**

- **MANDATORY: NEVER execute destructive operations like `DROP TABLE`,
  `DROP DATABASE` (except test DB)**
- Always use migration files for schema changes
- **MANDATORY: Always maintain consistency between migration files and
  database**
  - ❌ NEVER: Delete migration files and leave database state unchanged
  - ❌ NEVER: Create tables directly in database without migration files
  - ✅ ALWAYS: When deleting migration files, revert corresponding database
    changes
  - ✅ ALWAYS: Keep `_sqlx_migrations` table and migrations directory in sync

### 3.1. **MANDATORY: Migration File is the ONLY Source of Truth**

- **MANDATORY: NEVER manually fix SQLx compile errors by modifying database
  schema directly**
- **MANDATORY: DDL defined in migration files is ABSOLUTE - NEVER bypass it**
- **MANDATORY: When SQLx compile errors occur due to schema mismatch:**
  1. Check the migration file for the intended schema
  2. Run `sqlx migrate run` to apply migrations
  3. Run `cargo sqlx prepare` to update offline cache
  4. NEVER manually alter tables to "fix" compile errors
- **MANDATORY: Design database schemas carefully BEFORE creating migration
  files**
- **MANDATORY: Review migration SQL thoroughly - it cannot be changed after
  creation**
- **MANDATORY: Small, incremental migrations are better than large changes**

### 4. **MANDATORY: Never Expose Secrets**

- Never commit or display `.env` file contents
- Never hardcode API keys or passwords

### 5. **MANDATORY: Never Use Overconfident Language**

- **MANDATORY: NEVER use the word "perfect"** - Issues always arise after
  implementation
- Avoid definitive expressions like "no problem" or "it's fine"
- Always use cautious expressions like "it seems" or "it should"
- Always be aware of potential issues even after implementation

### 6. **MANDATORY: Never Evade Responsibility**

- **MANDATORY: NEVER say "it's not related because I didn't touch it"**
- **MANDATORY: Don't blame pre-existing issues**
- **MANDATORY: Don't dismiss as "unrelated to current changes"**
- **MANDATORY: Take responsibility for ALL issues**
- **MANDATORY: Fix ALL issues discovered during work**
- **MANDATORY: Show through actions, not excuses**

### 7. **MANDATORY: Careful and Deliberate Development**

- **MANDATORY: Design types, interfaces, and structs carefully - NO sloppy
  implementations**
- **MANDATORY: Small, incremental changes are better than large, risky changes**
- **MANDATORY: Always consider the impact on existing code before making
  changes**
- **MANDATORY: Review your changes thoroughly before committing**
- **MANDATORY: When in doubt, research existing patterns in the codebase**
- **MANDATORY: Never rush implementation - correctness over speed**

### 8. **MANDATORY: Best Practice Development Process**

When implementing new features, follow this proven approach:

1. **MANDATORY: Create clear task list with TodoWrite tool**

   - Break down the work into specific, manageable tasks
   - Track progress systematically
   - Mark tasks as completed immediately when done

2. **MANDATORY: Study existing patterns BEFORE implementation**

   - Use Task tool to research similar code patterns
   - Understand how existing services are structured
   - Follow established conventions religiously

3. **MANDATORY: Start with data models**

   - Design your types and structs carefully
   - Consider all fields and their relationships
   - Use appropriate derives (Debug, Clone, Serialize, Deserialize)

4. **MANDATORY: Design database schema with extreme care**

   - Create migration files with descriptive names
   - Include appropriate constraints and indexes
   - NEVER modify migration files after creation
   - Run migrations and `cargo sqlx prepare` immediately

5. **MANDATORY: Implement services following provider pattern**

   - Create trait definitions for abstraction
   - Implement concrete providers
   - Use proper error handling with custom error types
   - Add #[allow(dead_code)] for WIP code to avoid warnings

6. **MANDATORY: Update module declarations**

   - Add new modules to mod.rs files
   - Maintain alphabetical order
   - Check compilation frequently with `cargo check`

7. **MANDATORY: Test continuously**

   - Run tests after each significant change
   - All tests must pass before committing
   - Use `cargo test -- --test-threads=1` to avoid flaky tests

8. **MANDATORY: Commit incrementally**
   - Make small, focused commits
   - Write clear commit messages
   - Never commit broken code

## 🛠️ Common Development Commands

### ⚠️ IMPORTANT: Development Server Control

**MANDATORY Rules:**

- ❌ **MANDATORY: Never start/stop dev servers automatically** - Server control
  is managed by developers
- ❌ Don't auto-execute server startup commands like `cargo run`, `npm run dev`
- ✅ Only present the commands when server startup is needed
- ✅ Guide with "Please start the server with the following command"

### Backend (Rust)

```bash
cd backend

# Development
cargo run                          # Start dev server (port 3000)
cargo watch -c -w src -w .env -x run  # Auto-reload dev server ⭐ RECOMMENDED
./watch.sh                         # Same as above (script version)

# Testing (IMPORTANT: Run in single thread to avoid failures)
cargo test -- --test-threads=1     # Run all tests in single thread ⭐ MANDATORY
cargo test test_name -- --test-threads=1  # Run specific test in single thread

cargo clippy -- -D warnings        # Run linter
cargo fmt                          # Format code

# Install cargo-watch (first time only)
cargo install cargo-watch

# Database
export DATABASE_URL="postgresql://postgres:postgres@localhost:5432/markmail"
sqlx migrate run                   # Run migrations
sqlx migrate add migration_name    # Create new migration
cargo sqlx prepare                 # Update sqlx-data.json for offline compilation ⭐ MANDATORY after schema changes

# SQLx Offline Cache Update (IMPORTANT)
# MANDATORY: Run after database schema changes or adding new queries
cargo sqlx prepare                 # Update .sqlx directory
git add backend/.sqlx              # MANDATORY: Include changes in commit
```

### Frontend (SvelteKit)

```bash
cd frontend

# Development
npm run dev                        # Start dev server (port 5173)
npm run build                      # Production build
npm test                          # Run all tests
npm test -- --run                  # Run tests once
npm run check                      # Type checking
npm run lint                       # ESLint
npm run format                     # Format code
```

### Infrastructure (AWS CDK)

```bash
cd infrastructure
npm test                           # Run infrastructure tests
npm run build                      # Compile TypeScript
npm run deploy                     # Deploy to AWS
cdk synth                         # Generate CloudFormation template
```

### Project-wide Commands

```bash
# From project root
docker-compose up -d               # Start all services (PostgreSQL, Redis, MailHog)
npm run format                     # Format entire codebase
npm run lint                      # Lint entire codebase
./scripts/setup-lefthook.sh       # Setup Git hooks
```

### AI Feature Configuration

```bash
# Add to .env file
AI_PROVIDER=openai                 # or 'anthropic'
OPENAI_API_KEY=sk-xxxx            # OpenAI API key
ANTHROPIC_API_KEY=sk-ant-xxxx     # Anthropic API key

# Access AI features:
# 1. Click "AI Features" in navigation menu
# 2. Three features available:
#    - Marketing scenario generation
#    - Content generation
#    - Subject line optimization
```

## 🏗️ High-Level Architecture

### System Overview

Application designed with clear separation of concerns:

- **Frontend**: SvelteKit SPA with client-side routing (SSR disabled)
- **Backend**: Rust/Axum REST API with JWT authentication
- **Database**: PostgreSQL with SQLx for compile-time query verification
- **Infrastructure**: AWS CDK for Infrastructure as Code
- **Background Processing**: Tokio-based async workers

### Backend Architecture (Rust)

```
backend/src/
├── api/           # HTTP endpoint handlers (route definitions)
├── database/      # Database query functions (repository layer)
├── models/        # Domain models and request/response types
├── services/      # Business logic layer
├── workers/       # Background workers
├── middleware/    # Auth, CORS, logging middleware
├── utils/         # Shared utilities (JWT, password hashing, validation)
└── ai/            # AI features module ⭐ NEW
    ├── providers/ # Provider implementations (OpenAI, Anthropic)
    ├── services/  # AI services (scenario builder, content generation)
    └── models/    # AI-related data models and prompts
```

**Key Patterns**:

- All API routes use Axum's `from_fn` middleware for authentication
- Database queries verified at compile time with SQLx
- Service layer handles business logic, handlers stay thin
- Models define both database entities and API contracts
- Error handling with custom error types for proper HTTP status codes
- Background workers run as independent Tokio tasks

### Frontend Architecture (SvelteKit)

```
frontend/src/
├── routes/        # SvelteKit pages and API routes
├── lib/
│   ├── services/  # API client services
│   ├── stores/    # Svelte stores (auth, global state)
│   └── types/     # TypeScript type definitions
└── tests/         # Test files mirroring src structure
```

**Key Patterns**:

- SPA mode via `ssr = false` and `prerender = false` in `+layout.js`
- All API calls go through service layer with proper error handling
- Auth state persisted in `authStore` with localStorage
- Form components share logic between create and edit modes
- TypeScript types match backend API contracts

### Database Schema

Key tables and relationships:

- `users` → `templates`, `campaigns`, `subscribers`, `forms`, `sequences`
- `campaigns` → `templates` (many-to-one)
- `forms` → `form_fields` (one-to-many)
- `sequences` → `sequence_steps` (one-to-many)
- `sequence_steps` → `templates` (many-to-one)
- `form_submissions` → `forms` (many-to-one)
- `sequence_enrollments` → `sequences`, `subscribers` (many-to-one)
- `sequence_step_logs` → `sequence_enrollments`, `sequence_steps` (many-to-one)

## 📋 Important Development Considerations

### Database Migrations

- **MANDATORY: Never modify existing migration files** - Once applied, they're
  immutable
- Always create new migrations with timestamps: `sqlx migrate add description`
- After migration, run `cargo sqlx prepare` to update offline compilation data

#### 🚨 Migration Consistency Checklist

**MANDATORY: Follow these steps when making database changes:**

1. **Pre-implementation Check**

   ```bash
   # Verify migration file count matches DB records
   ls -1 migrations/*.sql | wc -l
   docker exec markmail-postgres-1 psql -U markmail -d markmail_dev -c "SELECT COUNT(*) FROM _sqlx_migrations;"
   ```

2. **Creating New Migration**

   ```bash
   # Correct procedure
   sqlx migrate add your_feature_description
   # Write SQL
   sqlx migrate run
   cargo sqlx prepare
   ```

3. **Feature Removal/Rollback**

   ```bash
   # ❌ MANDATORY: NEVER DO THIS
   rm migrations/20250621_your_feature.sql  # Deleting file only, leaving DB unchanged

   # ✅ MANDATORY: Correct procedure
   # 1. First revert DB state
   docker exec markmail-postgres-1 psql -U markmail -d markmail_dev -c "DROP TABLE your_table CASCADE;"
   # 2. Delete migration record
   docker exec markmail-postgres-1 psql -U markmail -d markmail_dev -c "DELETE FROM _sqlx_migrations WHERE version = 'your_version';"
   # 3. Delete file
   rm migrations/20250621_your_feature.sql
   # 4. Regenerate SQLx metadata
   cargo sqlx prepare
   ```

4. **Consistency Repair**
   ```bash
   # Check current state
   diff <(ls -1 migrations/*.sql | sed 's/.*\///' | sed 's/_.*$//' | sort) \
        <(docker exec markmail-postgres-1 psql -U markmail -d markmail_dev -t -c "SELECT version FROM _sqlx_migrations ORDER BY version;" | grep -v '^$' | tr -d ' ')
   ```

### Testing Philosophy

- **MANDATORY: Never disable failing tests** - Fix the root cause
- Test naming: `test_feature_scenario` (e.g.,
  `test_create_campaign_with_invalid_template`)
- Backend tests use isolated test database with automatic cleanup
- Frontend tests use mock services to avoid API dependencies

### Authentication Flow

1. Login returns JWT (24h) + refresh token (30d)
2. Frontend stores tokens in localStorage via authStore
3. API requests include `Authorization: Bearer <token>` header
4. 401 response triggers automatic logout
5. Protected routes check auth state before rendering

### Form Builder System

Forms have complex field structure:

- Backend uses `form_fields` (snake_case)
- Frontend components use `form.form_fields` (NOT `form.fields`)
- Field types: text, email, textarea, select, radio, checkbox, etc.
- Public forms accessible without auth at `/forms/[id]/public`

### Email Service Architecture

- Provider abstraction trait switches between MailHog (dev) and AWS SES (prod)
- Environment variable `EMAIL_PROVIDER` controls provider
- Rate limiting for batch sending in production
- Template variables use `{{variable_name}}` syntax

### Sequence Automation System

- Background worker runs every 60 seconds to process pending steps
- Trigger-based auto-enrollment (form submission, subscriber creation, etc.)
- Step types: email (send email), wait (delay), condition (branching), tag
  (tagging)
- Full automation from form submission to subscriber creation and sequence
  enrollment

### Common Pitfalls

1. **SvelteKit Dynamic Routes**: Can't be prerendered, use SPA mode
2. **CORS in Development**: Backend allows localhost:5173, production uses same
   domain
3. **SQLx Offline Mode**: Run `cargo sqlx prepare` after schema changes
4. **Lefthook Formatting**: Runs on commit, don't bypass with `--no-verify`

## 🚀 AWS Deployment Notes

### Build Configuration

- Frontend uses static adapter with `fallback: "index.html"` for SPA
- Dockerfile copies from `/app/build`, not `.svelte-kit/output`
- VITE_API_URL environment variable set at build time for API endpoint

### Infrastructure Stacks

- ECS Fargate for containerized services
- RDS Aurora PostgreSQL Serverless v2
- Application Load Balancer with path-based routing
- CloudWatch for logging and monitoring
- CodePipeline for CI/CD from GitHub

### Environment Variables

Required production variables:

- `DATABASE_URL`: PostgreSQL connection string
- `JWT_SECRET`: Secret for JWT signing
- `VITE_API_URL`: Frontend API endpoint (build time)
- `EMAIL_PROVIDER`: mailhog or aws_ses
- `AWS_REGION`, `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`: For SES

## 🚫 Common Mistakes and Prevention

### 0. **Breaking Existing Features During New Feature Implementation**

- ❌ **WORST**: Modifying subscriptions.rs for AI usage tracking and breaking
  tests
- ❌ **Bad**: Fixing SQLx type errors ad-hoc causing more issues
- ❌ **Bad**: Dismissing test failures as "not caused by my changes"
- ❌ **WORST EXCUSE**: "It's a different issue unrelated to current fix"
- ❌ **PEAK IRRESPONSIBILITY**: "It's a parallel execution issue, unrelated to
  AI destruction"
- ✅ **Good**:
  1. Implement new features as independent modules
  2. Keep changes to existing modules minimal
  3. Confirm all tests pass before changes
  4. Confirm all tests pass after changes
  5. Immediately revert if issues arise
  6. **NEVER give up until all tests pass**

### 1. **Changing Logic to Pass Tests**

- ❌ Bad: Change business logic to match failing tests
- ✅ Good: Fix tests if logic is correct, fix logic if it has bugs

### 2. **Skipping Tests to Push**

- ❌ **WORST**: Skip tests with `git push --no-verify`
- ❌ Bad: Add `#[ignore]` when tests fail
- ✅ Good: Investigate failure cause and fix before pushing

### 3. **Deleting or Modifying Migration Files**

- ❌ Bad: Delete existing migration files when errors occur
- ❌ Bad: Edit existing migration files directly
- ✅ Good: Add new migration with new timestamp for fixes

### 4. **Suppressing Errors**

- ❌ Bad: Change `unwrap()` to `.unwrap_or_default()` when it fails
- ✅ Good: Investigate error cause and implement proper handling

### 5. **Working Directory Confusion**

- ❌ Bad: Execute commands without checking current directory
- ✅ Good: Always check with `pwd` and navigate to correct directory

## 🚨 External Library/SDK Integration Requirements

### 1. **MANDATORY: Prioritize Official Documentation**

**MANDATORY: Before introducing new libraries or SDKs, always follow these
steps:**

1. **Check Official Site**

   ```bash
   # Example: For Stripe
   # 1. Check https://docs.stripe.com/sdks for official SDKs
   # 2. Check community SDK section
   # 3. Use recommended libraries
   ```

2. **Get Latest Information**

   - Search for current year (2025) information
   - Check library's latest version
   - Check latest releases on GitHub

3. **Pre-implementation Verification**

   - Check official implementation examples
   - Verify dependency compatibility
   - Check consistency with existing code

4. **Record Decision Process**
   - Clearly record why you chose that library
   - Other options considered and rejection reasons
   - Save official documentation URLs

### 2. **MANDATORY: No Rollback Work**

**MANDATORY: Never do these:**

- ❌ Repeatedly change libraries once decided
- ❌ "Try and switch if it doesn't work" approach
- ❌ Proceed based on guesses without official info

**Correct Approach:**

- ✅ Conduct thorough research first
- ✅ Adopt officially recommended methods
- ✅ Clarify approach before implementation

### 3. **Example: Correct Stripe SDK Integration**

```bash
# 1. Check official documentation
WebFetch: https://docs.stripe.com/sdks
# → Check community SDK section

# 2. Identify recommended library
WebFetch: https://docs.stripe.com/sdks/community
# → For Rust: async-stripe by Alex Lyon

# 3. Check latest version and usage
WebSearch: "async-stripe arlyon GitHub latest version 2025"
WebFetch: https://github.com/arlyon/async-stripe
# → Version 0.31, check features

# 4. Check implementation examples before starting
WebFetch: https://github.com/arlyon/async-stripe/blob/master/examples/
```

## 🚨 New Service Implementation Checklist

**MANDATORY: When creating new service files, always refer to existing service
file patterns.**

### 1. **Unify API URL Construction Patterns**

```typescript
// ❌ Bad: Creating your own pattern
const API_BASE = import.meta.env.VITE_API_URL || "http://localhost:3000";
const response = await fetch(`${API_BASE}/api${path}`, ...);

// ✅ Good: Follow existing patterns
const API_BASE_URL = "/api";
const response = await fetch(`${API_BASE_URL}${path}`, ...);
```

### 2. **Pre-creation Verification Steps**

1. **Search for existing similar files**

   ```bash
   # Example: Before creating new service
   find . -name "*Service.ts" -o -name "*service.ts"
   ```

2. **Check existing patterns**

   ```bash
   # Example: Check API call patterns
   grep -r "fetch.*api" --include="*.ts"
   ```

3. **Base on most similar file**
   - Reference basic service files like `api.ts`
   - Don't invent your own patterns

### 3. **Avoid Environment Variables**

- Use relative paths (`/api`) in frontend
- Keep environment-dependent settings minimal
- Don't use env vars if existing services don't

### 4. **Code Review Checklist**

- [ ] Using same patterns as existing service files
- [ ] Unified API URL construction
- [ ] Consistent error handling
- [ ] Unified auth token handling
- [ ] Proper TypeScript type definitions

## 📝 Commit Message Convention

```
<type>: <subject>

<body>
```

Types:

- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation only changes
- `style`: Changes that don't affect code meaning
- `refactor`: Refactoring
- `test`: Adding/modifying tests
- `chore`: Build process or tool changes

## 🔧 Recommended New Feature Implementation Process

### 1. **Pre-verify Database Constraints**

Before implementing new features, always check database constraints:

```bash
# Check table structure and constraints
docker exec markmail-postgres-1 psql -U markmail -d markmail_dev -c "\d table_name"

# Especially check:
# - CHECK constraints (allowed values)
# - UNIQUE constraints (columns that don't allow duplicates)
# - Foreign key constraints
# - Data types (especially UUID vs INTEGER)
```

### 2. **Backend and Frontend Type Consistency**

Before implementation, verify:

1. **Backend Model Definitions** (`backend/src/models/`)

   - Field names (snake_case)
   - Data types (UUID, String, i32, etc.)
   - Required/optional fields

2. **Database Schema** (`backend/migrations/`)

   - Column names and types
   - Constraints (CHECK, UNIQUE, etc.)
   - Default values

3. **Frontend Type Definitions** (`frontend/src/lib/types/`)
   - Type definitions matching backend
   - IDs are usually `string` (UUID)
   - Enum values for status/type match

### 3. **API Implementation Checklist**

1. **Verify Endpoints**

   ```bash
   # Check routing in backend/src/api/mod.rs
   grep -n "route.*api" backend/src/api/mod.rs
   ```

2. **Understand Special Endpoints**

   - Detail fetch: `/api/resources/:id` vs `/api/resources/:id/full`
   - Nested resources: `/api/resources/:id/sub-resources`

3. **Verify Response Format**
   - Single object vs wrapper object
   - Pagination format

### 4. **Common Implementation Mistakes and Solutions**

#### ❌ Type Mismatch

```typescript
// Bad
type Status = 'active' | 'inactive'; // DB includes 'draft' too

// Good - Check DB constraints first
type Status = 'draft' | 'active' | 'inactive';
```

#### ❌ Field Name Mismatch

```typescript
// Bad
trigger_conditions?: Record<string, any>;  // DB uses trigger_config

// Good - Match backend model
trigger_config?: Record<string, any>;
```

#### ❌ Not Considering Duplicates

```typescript
// Bad
step_order: steps.length + 1; // Can duplicate after deletion

// Good
step_order: Math.max(...steps.map(s => s.step_order)) + 1;
```

#### ❌ Enum Type String Comparison

```rust
// Bad
if sequence.trigger_type == TriggerType::FormSubmission {
    // Type error: String != TriggerType
}

// Good - Use as_str() method
if sequence.trigger_type == TriggerType::FormSubmission.as_str() {
    // Works correctly
}
```

#### ❌ Async Task Error Handling

```rust
// Bad
tokio::spawn(async move {
    process_sequences().await; // Errors are swallowed
});

// Good - Log errors
tokio::spawn(async move {
    if let Err(e) = process_sequences().await {
        error!("Sequence processing error: {}", e);
    }
});
```

### 5. **Debugging Steps**

1. **First check logs when errors occur**

   - Browser console
   - Backend terminal output

2. **Verify actual database data**

   ```bash
   docker exec markmail-postgres-1 psql -U markmail -d markmail_dev -c "SELECT * FROM table_name;"
   ```

3. **Check API communication**
   - Browser DevTools > Network tab
   - Verify request/response payloads

## ⚠️ AWS CDK Deployment Critical Requirements

### **MANDATORY: Domain Environment Variables Required**

**Problem**: Without domain environment variables, these severe issues occur:

1. ALBStack won't create HTTPS listener
2. ECSServiceStack tries to reference HTTPS listener and fails
3. Both stacks get stuck in UPDATE_ROLLBACK_COMPLETE state
4. Dependent stacks like MonitoringStack can't deploy

**Solution**: Set environment variables before CDK deployment

```bash
# For development
export DEV_DOMAIN=dev.markmail.engineers-hub.ltd

# For staging
export STAGING_DOMAIN=staging.markmail.engineers-hub.ltd

# For production
export PROD_DOMAIN=markmail.engineers-hub.ltd

# Run deployment
npm run cdk -- deploy StackName --profile your-profile
```

**MANDATORY: Never do these:**

- ❌ Run CDK deploy without environment variables
- ❌ Manually create/modify resources with AWS CLI
- ❌ Ignore inter-stack dependencies

## 🔧 AWS RDS Operations

### Connecting to RDS

AWS RDS cannot be directly connected due to security requirements. Use these
methods:

#### 1. Connection via Bastion Host

```bash
# Create bastion host
cd infrastructure
CREATE_BASTION=true npm run cdk -- deploy MarkMail-dev-BastionStack --profile your-profile

# Get bastion host instance ID
aws ec2 describe-instances \
  --filters "Name=tag:Name,Values=markmail-dev-bastion" \
  --query 'Reservations[*].Instances[*].[InstanceId]' \
  --output text \
  --profile your-profile

# Connect via SSM Session Manager
aws ssm start-session \
  --target i-xxxxxxxxxxxxx \
  --profile your-profile

# Connect to RDS from bastion host
PGPASSWORD=your-password psql \
  -h your-rds-endpoint.rds.amazonaws.com \
  -U markmail \
  -d markmail
```

#### 2. Remote Execution via SSM Send Command

```bash
# Execute command
aws ssm send-command \
  --instance-ids i-xxxxxxxxxxxxx \
  --document-name "AWS-RunShellScript" \
  --parameters 'commands=["your-command-here"]' \
  --profile your-profile \
  --query 'Command.CommandId' \
  --output text

# Check execution result
aws ssm get-command-invocation \
  --command-id command-id-here \
  --instance-id i-xxxxxxxxxxxxx \
  --profile your-profile
```

### Database Migration

#### Automatic Migration via ECS

Migrations run automatically on application startup:

```bash
# Force redeploy ECS service
aws ecs update-service \
  --cluster markmail-dev \
  --service markmail-dev-backend \
  --force-new-deployment \
  --profile your-profile
```

#### Manual Migration

Via bastion host:

```bash
# Run migration on bastion host
aws ssm send-command \
  --instance-ids i-xxxxxxxxxxxxx \
  --document-name "AWS-RunShellScript" \
  --parameters 'commands=[
    "git clone https://github.com/your-repo/markmail.git",
    "cd markmail/backend",
    "export DATABASE_URL=\"postgresql://user:pass@endpoint:5432/dbname\"",
    "sqlx migrate run"
  ]' \
  --profile your-profile
```

### Database Reset

⚠️ **WARNING**: NEVER execute in production

```bash
# Terminate connections and recreate database
aws ssm send-command \
  --instance-ids i-xxxxxxxxxxxxx \
  --document-name "AWS-RunShellScript" \
  --parameters 'commands=[
    "export PGPASSWORD=\"your-password\"",
    "psql -h endpoint -U markmail -d postgres -c \"SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = '"'"'markmail'"'"' AND pid <> pg_backend_pid();\"",
    "psql -h endpoint -U markmail -d postgres -c \"DROP DATABASE IF EXISTS markmail;\"",
    "psql -h endpoint -U markmail -d postgres -c \"CREATE DATABASE markmail;\""
  ]' \
  --profile your-profile
```

### Resolve Migration Version Mismatch

When migration checksums differ between local and AWS:

1. **Check migration history**

   ```sql
   SELECT version, checksum FROM _sqlx_migrations ORDER BY version;
   ```

2. **Update checksum**

   ```sql
   UPDATE _sqlx_migrations
   SET checksum = 'new-checksum-here'
   WHERE version = 'version-number';
   ```

3. **Delete and rerun specific migration**
   ```sql
   DELETE FROM _sqlx_migrations WHERE version = 'version-number';
   ```

### Troubleshooting

#### Bastion host not found

```bash
# Check instance status
aws ec2 describe-instances \
  --filters "Name=tag:Name,Values=markmail-dev-bastion" \
  "Name=instance-state-name,Values=running,stopped" \
  --query 'Reservations[*].Instances[*].[InstanceId,State.Name]' \
  --output table \
  --profile your-profile
```

#### Check RDS endpoint

```bash
aws rds describe-db-instances \
  --query 'DBInstances[*].[DBInstanceIdentifier,Endpoint.Address]' \
  --output table \
  --profile your-profile
```

#### Get database password

```bash
aws secretsmanager get-secret-value \
  --secret-id markmail-dev-db-secret \
  --query 'SecretString' \
  --output text \
  --profile your-profile | jq -r '.password'
```

### Important Notes

- **Bastion host is temporary resource** - Consider deletion after use
- **Be extra careful in production**
- **Verify database backups exist**

## 🔐 Managing AI API Keys with AWS Secrets Manager

### Initial Setup for AI Secrets

In AWS environments, AI API keys (OPENAI_API_KEY, ANTHROPIC_API_KEY, etc.) are
managed via Secrets Manager.

#### 1. Update Secrets

```bash
# Update secret values
aws secretsmanager update-secret \
  --secret-id markmail-dev-ai-secret \
  --secret-string '{
    "OPENAI_API_KEY": "sk-xxxxxxxxxxxxxxxxxxxxxxxx",
    "ANTHROPIC_API_KEY": "sk-ant-xxxxxxxxxxxxxxxx",
    "AI_PROVIDER": "openai",
    "OPENAI_MODEL": "gpt-4",
    "ANTHROPIC_MODEL": "claude-3-opus-20240229"
  }' \
  --profile your-profile
```

#### 2. Verify Secrets

```bash
# Verify current values (WARNING: displays actual API keys)
aws secretsmanager get-secret-value \
  --secret-id markmail-dev-ai-secret \
  --query 'SecretString' \
  --output text \
  --profile your-profile | jq '.'
```

#### 3. Redeploy ECS Service

After updating secrets, redeploy ECS service to reflect new values:

```bash
# Force redeploy backend service
aws ecs update-service \
  --cluster markmail-dev \
  --service markmail-dev-backend \
  --force-new-deployment \
  --profile your-profile
```

#### 4. Secret Rotation

Regularly rotate API keys:

```bash
# Update secret with new API key
aws secretsmanager update-secret \
  --secret-id markmail-dev-ai-secret \
  --secret-string '{
    "OPENAI_API_KEY": "sk-new-key-xxxxxxxxxxxxxxxx",
    "ANTHROPIC_API_KEY": "sk-ant-new-key-xxxxxxxxx",
    "AI_PROVIDER": "openai",
    "OPENAI_MODEL": "gpt-4",
    "ANTHROPIC_MODEL": "claude-3-opus-20240229"
  }' \
  --profile your-profile
```

### Important Notes

- **Secrets are separated by environment** (dev, staging, prod)
- **Secret naming convention**: `markmail-{environment}-ai-secret`
- **ECS tasks automatically fetch latest secrets**
- **Use `.env` file for local development**

## 🔧 Troubleshooting

### Backend Test Failures

**IMPORTANT: Prevent test failures from parallel execution**

```bash
# ❌ Bad: Tests may fail due to parallel execution
cargo test

# ✅ Good: Run in single thread for reliability
cargo test -- --test-threads=1

# Run specific test in single thread
cargo test test_create_campaign -- --test-threads=1

# For more detailed logs
RUST_LOG=debug cargo test -- --test-threads=1 --nocapture
```

**Handling test failures**:

1. For database connection issues
   - Check PostgreSQL status with `docker-compose ps`
   - Drop and recreate test DB if needed
2. For parallel execution issues
   - Always use `--test-threads=1`
3. For migration issues
   - Run `sqlx migrate run`

### Database Connection Errors

```bash
# Check if PostgreSQL is running
docker-compose ps

# If not running
docker-compose up -d postgres

# Run migrations
cd backend
sqlx migrate run
```

### Build Errors

```bash
# Rust
cargo clean
cargo build

# Frontend
cd frontend
rm -rf node_modules .svelte-kit
npm install
```

### Cleaning Untracked Files

```bash
# Check untracked files
git clean -n

# Delete untracked files and directories
git clean -fd
```

Follow these guidelines for safe and high-quality code development.
