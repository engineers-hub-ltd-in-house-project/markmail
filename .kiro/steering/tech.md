# Technology Stack

## Architecture

### High-Level System Design

```
Frontend (SvelteKit) → Backend (Rust/Axum) → Database (PostgreSQL)
                                           → Cache (Redis)
                                           → Email (AWS SES/MailHog)
```

### Deployment Architecture

- **Container Platform**: Docker, ECS Fargate on AWS
- **Load Balancing**: Application Load Balancer (ALB)
- **Database**: RDS Aurora PostgreSQL Serverless v2
- **CI/CD**: GitHub Actions + AWS CodePipeline
- **Infrastructure as Code**: AWS CDK v2 (TypeScript)

## Frontend

### Framework and Libraries

- **Framework**: SvelteKit 2.x
- **Language**: TypeScript 5.x
- **UI Library**: TailwindCSS 3.x
- **Icons**: Lucide Svelte
- **Markdown**: marked, DOMPurify
- **HTTP Client**: Native fetch with custom API wrapper
- **State Management**: Svelte stores

### Build Tools

- **Bundler**: Vite 6.x
- **Package Manager**: npm
- **Linting**: ESLint with Svelte plugin
- **Formatting**: Prettier with Svelte plugin
- **Type Checking**: svelte-check

## Backend

### Core Technologies

- **Language**: Rust (latest stable)
- **Web Framework**: Axum 0.7
- **Async Runtime**: Tokio
- **Database ORM**: SQLx with compile-time checked queries
- **Authentication**: JWT (jsonwebtoken crate) + bcrypt
- **Serialization**: Serde + serde_json

### Additional Backend Libraries

- **Email**: lettre (SMTP), AWS SDK for SES
- **Markdown Processing**: pulldown-cmark
- **Environment**: dotenvy
- **Logging**: tracing + tracing-subscriber
- **Testing**: Built-in Rust test framework
- **Hot Reload**: cargo-watch (development)

### AI Integration

- **Providers**: OpenAI (GPT-4), Anthropic (Claude)
- **Libraries**: reqwest for API calls
- **Token Counting**: tiktoken-rs

## Database

### Primary Database

- **System**: PostgreSQL 16.x
- **Migration Tool**: SQLx migrate
- **Connection Pooling**: SQLx built-in pool

### Cache Layer

- **System**: Redis 7.x
- **Use Cases**: Session storage, rate limiting, temporary data

## Development Environment

### Required Tools

- **Rust**: 1.75+ (rustup recommended)
- **Node.js**: 20.x LTS
- **Docker**: 24.x with Docker Compose v2
- **AWS CLI**: v2 (for deployment)
- **Git**: 2.x with lefthook for hooks

### Optional Tools

- **cargo-watch**: For backend auto-reload
- **VS Code**: With recommended extensions
- **PostgreSQL client**: psql or GUI tool

## Common Commands

### Development Server

```bash
# Start all services
docker-compose up -d

# Backend development (with auto-reload)
cd backend && cargo watch -c -w src -w .env -x run
# OR
cd backend && ./watch.sh

# Frontend development
cd frontend && npm run dev
```

### Database Operations

```bash
# Run migrations
cd backend
export DATABASE_URL="postgresql://postgres:postgres@localhost:5432/markmail"
sqlx migrate run

# Create new migration
sqlx migrate add <migration_name>
```

### Testing

```bash
# Backend tests
cd backend && cargo test

# Frontend tests
cd frontend && npm test

# Infrastructure tests
cd infrastructure && npm test
```

### Formatting and Linting

```bash
# Format all code
npm run format

# Lint all code
npm run lint

# Format backend only
npm run format:backend

# Format frontend only
npm run format:frontend
```

### Building

```bash
# Build backend
cd backend && cargo build --release

# Build frontend
cd frontend && npm run build

# Build Docker images
docker-compose build
```

## Environment Variables

### Essential Configuration

```env
# Database
DATABASE_URL=postgresql://postgres:postgres@localhost:5432/markmail
REDIS_URL=redis://localhost:6379

# JWT
JWT_SECRET=your-secret-key-here
JWT_EXPIRATION=86400  # 24 hours in seconds

# Server
SERVER_PORT=3000
FRONTEND_URL=http://localhost:5173

# Email Provider
EMAIL_PROVIDER=mailhog  # mailhog | aws_ses
SMTP_HOST=localhost
SMTP_PORT=1025
SMTP_FROM=noreply@markmail.dev

# AI Provider (optional)
AI_PROVIDER=openai  # openai | anthropic
OPENAI_API_KEY=sk-xxx
ANTHROPIC_API_KEY=sk-ant-xxx
```

### AWS Configuration (Production)

```env
AWS_REGION=ap-northeast-1
AWS_ACCESS_KEY_ID=xxx
AWS_SECRET_ACCESS_KEY=xxx
AWS_SES_FROM_EMAIL=noreply@example.com
AWS_SES_CONFIGURATION_SET=markmail-configuration-set
```

### Rate Limiting

```env
EMAIL_RATE_LIMIT=14  # emails per second
EMAIL_BATCH_SIZE=50  # batch size for bulk sending
```

## Port Configuration

### Development Ports

- **3000**: Backend API server (Rust/Axum)
- **5173**: Frontend dev server (SvelteKit)
- **5432**: PostgreSQL database
- **6379**: Redis cache
- **1025**: MailHog SMTP server
- **8025**: MailHog Web UI

### Production Ports

- **80/443**: ALB (HTTPS redirect)
- **3000**: Backend container port (internal)
- **5432**: RDS PostgreSQL (VPC internal)

## Security Considerations

### Authentication

- JWT tokens with 24-hour expiration
- Refresh tokens with 30-day expiration
- bcrypt password hashing with cost factor 12
- Secure cookie storage for tokens

### API Security

- CORS configuration for frontend origin
- Rate limiting on sensitive endpoints
- Input validation and sanitization
- SQL injection prevention via parameterized queries

### Infrastructure Security

- VPC isolation for database and backend
- Security groups with minimal access
- Secrets Manager for sensitive configuration
- HTTPS only in production with SSL/TLS

## Monitoring and Logging

### Application Monitoring

- CloudWatch Logs for application logs
- Container Insights for ECS metrics
- Custom CloudWatch metrics for business KPIs

### Error Tracking

- Structured logging with tracing
- Error context preservation
- Request ID tracking across services

### Performance Monitoring

- Database query performance via SQLx logging
- API response time tracking
- Email delivery success rates
