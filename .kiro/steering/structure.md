# Project Structure

## Root Directory Organization

```
markmail/
├── backend/                 # Rust backend API server
├── frontend/                # SvelteKit web application
├── infrastructure/          # AWS CDK infrastructure code
├── scripts/                 # Utility and setup scripts
├── docs/                    # Project documentation
├── .claude/                 # Claude Code configuration
├── .kiro/                   # Kiro spec-driven development
├── .github/                 # GitHub Actions workflows
├── .vscode/                 # VS Code configuration
├── docker-compose.yml       # Local development environment
├── lefthook.yml            # Git hooks configuration
├── package.json            # Root package scripts
├── CLAUDE.md               # AI developer guidelines
└── README.md               # Project documentation
```

## Subdirectory Structures

### Backend (`backend/`)

```
backend/
├── src/
│   ├── main.rs             # Application entry point
│   ├── api/                # API route handlers
│   │   ├── auth.rs         # Authentication endpoints
│   │   ├── campaigns.rs    # Campaign management
│   │   ├── forms.rs        # Form handling
│   │   ├── sequences.rs    # Email sequences
│   │   ├── subscribers.rs  # Subscriber management
│   │   ├── templates.rs    # Template CRUD
│   │   └── ai.rs           # AI content generation
│   ├── services/           # Business logic layer
│   │   ├── email_service.rs
│   │   ├── ai_service.rs
│   │   └── auth_service.rs
│   ├── models/             # Data models and DTOs
│   │   ├── user.rs
│   │   ├── campaign.rs
│   │   ├── template.rs
│   │   └── subscriber.rs
│   ├── database/           # Database utilities
│   │   ├── connection.rs
│   │   └── migrations.rs
│   ├── middleware/         # HTTP middleware
│   │   ├── auth.rs         # JWT validation
│   │   ├── cors.rs         # CORS handling
│   │   └── logging.rs      # Request logging
│   └── utils/              # Utility functions
│       ├── markdown.rs     # Markdown processing
│       ├── jwt.rs          # JWT utilities
│       └── validation.rs   # Input validation
├── migrations/             # SQLx database migrations
├── tests/                  # Integration tests
├── Cargo.toml             # Rust dependencies
├── Dockerfile.dev         # Development container
├── rustfmt.toml           # Rust formatting config
└── .sqlx/                 # SQLx query cache
```

### Frontend (`frontend/`)

```
frontend/
├── src/
│   ├── routes/            # SvelteKit pages/routes
│   │   ├── +layout.svelte # Root layout
│   │   ├── +page.svelte   # Home page
│   │   ├── auth/          # Authentication pages
│   │   │   ├── login/
│   │   │   ├── register/
│   │   │   └── reset-password/
│   │   ├── campaigns/     # Campaign management
│   │   ├── subscribers/   # Subscriber management
│   │   ├── templates/     # Template editor
│   │   ├── forms/         # Form builder
│   │   ├── sequences/     # Sequence automation
│   │   └── ai/            # AI features
│   ├── lib/               # Shared code
│   │   ├── components/    # Reusable components
│   │   ├── services/      # API client services
│   │   │   ├── api.ts
│   │   │   ├── campaignService.ts
│   │   │   └── subscriberService.ts
│   │   ├── stores/        # Svelte stores
│   │   │   └── authStore.ts
│   │   ├── types/         # TypeScript types
│   │   │   ├── campaign.ts
│   │   │   ├── template.ts
│   │   │   └── subscriber.ts
│   │   └── utils/         # Utility functions
│   ├── app.d.ts           # Global type definitions
│   └── app.html           # HTML template
├── static/                # Static assets
├── tests/                 # Test files
├── package.json          # Node dependencies
├── svelte.config.js      # SvelteKit configuration
├── tailwind.config.js    # TailwindCSS configuration
├── tsconfig.json         # TypeScript configuration
├── vite.config.js        # Vite bundler configuration
└── Dockerfile.dev        # Development container
```

### Infrastructure (`infrastructure/`)

```
infrastructure/
├── lib/
│   ├── stacks/           # CDK stack definitions
│   │   ├── network-stack.ts      # VPC, subnets
│   │   ├── database-stack.ts     # RDS setup
│   │   ├── container-stack.ts    # ECS/Fargate
│   │   ├── ses-stack.ts          # Email service
│   │   └── pipeline-stack.ts     # CI/CD
│   └── constructs/       # Reusable constructs
├── bin/
│   └── infrastructure.ts # CDK app entry point
├── test/                 # Infrastructure tests
├── cdk.json             # CDK configuration
├── package.json         # Node dependencies
└── tsconfig.json        # TypeScript configuration
```

### Scripts (`scripts/`)

```
scripts/
├── salesforce-integration/  # Salesforce tools
│   ├── testing/            # Test scripts
│   ├── utilities/          # Utility scripts
│   └── form-management/    # Form utilities
├── setup-lefthook.sh       # Git hooks setup
└── deploy.sh               # Deployment script
```

## Code Organization Patterns

### Backend Patterns

- **Layered Architecture**: Controllers → Services → Models → Database
- **Trait-based Abstraction**: Email providers, AI providers use traits
- **Error Handling**: Custom error types with thiserror
- **Async/Await**: All I/O operations are async with Tokio
- **Dependency Injection**: Services passed through Axum state

### Frontend Patterns

- **Component-based**: Reusable Svelte components
- **Service Layer**: API calls abstracted in service modules
- **Store Pattern**: Reactive state management with Svelte stores
- **Type Safety**: Full TypeScript coverage with strict mode
- **Route-based Code Splitting**: Automatic with SvelteKit

## File Naming Conventions

### General Rules

- **Directories**: lowercase with hyphens (e.g., `form-management`)
- **Source Files**:
  - Rust: snake_case (e.g., `email_service.rs`)
  - TypeScript/JavaScript: camelCase (e.g., `campaignService.ts`)
  - Svelte Components: PascalCase (e.g., `TemplateEditor.svelte`)
- **Configuration**: lowercase with extensions (e.g., `tsconfig.json`)
- **Documentation**: UPPERCASE or Title Case (e.g., `README.md`)

### Special Files

- **Route Files**: `+page.svelte`, `+layout.svelte` (SvelteKit convention)
- **Test Files**: `*.test.ts`, `*.spec.ts` for frontend, `#[test]` in Rust files
- **Type Definitions**: `*.d.ts` for TypeScript declarations
- **Environment**: `.env` for local, `.env.example` for templates

## Import Organization

### Rust Imports

```rust
// Standard library
use std::collections::HashMap;

// External crates
use axum::{Router, Json};
use serde::{Deserialize, Serialize};

// Internal modules
use crate::models::User;
use crate::services::EmailService;
```

### TypeScript Imports

```typescript
// External packages
import { writable } from 'svelte/store';
import type { RequestHandler } from '@sveltejs/kit';

// Internal absolute imports
import { api } from '$lib/services/api';
import type { Campaign } from '$lib/types/campaign';

// Relative imports
import TemplateEditor from './TemplateEditor.svelte';
```

## Key Architectural Principles

### Separation of Concerns

- **API Routes**: Handle HTTP requests and responses only
- **Services**: Contain business logic and orchestration
- **Models**: Define data structures and validation
- **Database**: Handle persistence and queries

### Security First

- All endpoints require authentication by default
- Public endpoints explicitly marked
- Input validation at API boundary
- SQL injection prevention via parameterized queries

### Type Safety

- Compile-time query verification with SQLx
- Full TypeScript coverage in frontend
- Shared type definitions between frontend and API docs

### Testing Strategy

- Unit tests for business logic
- Integration tests for API endpoints
- E2E tests for critical user journeys
- Infrastructure tests for CDK stacks

### Code Reusability

- Shared components library in frontend
- Trait-based implementations in backend
- CDK constructs for infrastructure patterns
- Utility functions in dedicated modules

### Performance Optimization

- Database connection pooling
- Redis caching for frequent queries
- Lazy loading of routes in frontend
- Batch processing for bulk operations

### Developer Experience

- Hot reload in development
- Automatic code formatting on commit
- Type checking in CI/CD pipeline
- Comprehensive error messages

### Documentation Standards

- README in each major directory
- JSDoc/RustDoc for public APIs
- Inline comments for complex logic
- Architecture decision records in docs/
