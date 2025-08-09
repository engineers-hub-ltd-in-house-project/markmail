# Product Overview

## What is MarkMail?

MarkMail is an AI-driven marketing automation tool designed specifically for
engineers. It automates marketing funnel creation with a single prompt, creates
AI-understandable emails in Markdown format, and provides unlimited
extensibility through APIs with an engineer-first design philosophy.

## Core Features

- **AI Marketing Scenario Generation**: Automatically build sequences,
  templates, and forms by specifying industry and purpose
- **AI Content Assistant**: Auto-generate and optimize email content, A/B test
  subject line suggestions
- **Intelligent Segmentation**: Analyze subscriber behavior with machine
  learning for optimal delivery timing
- **Smart Automation**: AI continuously suggests sequence optimizations
- **Multi-language Support** (In Development): AI-generated content in Japanese
  and English
- **Template Management**: Markdown-based email templates with variable
  substitution
- **Campaign Management**: Create, schedule, and send email campaigns
- **Subscriber Management**: Import, tag, and manage subscriber lists with
  custom fields
- **Form Builder**: Dynamic forms with automatic subscriber creation
- **Sequence Automation**: Multi-step email sequences with triggers and
  conditional logic

## Target Use Cases

### Primary Use Cases

- **Email Marketing Automation**: Automated drip campaigns and nurture sequences
- **Lead Generation**: Form-based lead capture with Salesforce integration
- **Customer Onboarding**: Welcome sequences for new user activation
- **Product Updates**: Regular newsletter and announcement campaigns
- **Event Marketing**: Registration forms and follow-up sequences

### Target Audience

- **Engineers and Technical Teams**: Who prefer code-based configuration and
  API-first approach
- **Startups and Small Businesses**: Looking for affordable marketing automation
- **Marketing Teams**: Working in technical environments or with developer
  resources
- **SaaS Companies**: Needing programmatic email capabilities

## Key Value Propositions

### Unique Benefits

- **Engineer-First Design**: Markdown format, API-driven, version-controllable
  configurations
- **AI-Powered Efficiency**: Complete marketing funnel generation from a single
  prompt
- **Full Transparency**: Open-source codebase with self-hosting option
- **Cost-Effective**: Competitive pricing with generous free tier
- **Developer Experience**: Clean APIs, comprehensive documentation, webhook
  support
- **Extensibility**: Custom integrations via REST API and webhooks

### Differentiators

- **Markdown Native**: All content in developer-friendly Markdown format
- **AI Integration**: Built-in AI capabilities for content generation and
  optimization
- **Infrastructure as Code**: AWS CDK for complete infrastructure automation
- **Multi-Provider Support**: Switch between email providers (MailHog, AWS SES)
- **Type-Safe**: Full TypeScript support in frontend, Rust type safety in
  backend

## Pricing Tiers

### Free Plan (¥0/month)

- 100 contacts, 1,000 emails/month
- 3 campaigns, 5 templates, 3 forms, 1 sequence
- AI features: 10 uses/month (limited)
- No API access, A/B testing, or custom domain

### Pro Plan (¥4,980/month)

- 10,000 contacts, 100,000 emails/month
- 50 campaigns, 100 templates, 50 forms, 20 sequences
- AI features: 500 uses/month
- API access (rate-limited), A/B testing, advanced analytics

### Business Plan (¥19,800/month)

- 100,000 contacts, 1,000,000 emails/month
- Unlimited campaigns, templates, forms, sequences
- Unlimited AI features
- Full API access, custom domain, white-label, priority support

## Product Roadmap

### Completed Features

- Authentication system (JWT + refresh tokens)
- Template management with Markdown support
- Campaign creation and sending
- Subscriber management with CSV import
- Form builder with public submission
- Sequence automation engine
- AWS SES email integration
- AI content generation (OpenAI/Anthropic)

### In Development

- Subscription billing (Stripe integration)
- AI usage tracking and limits
- E2E testing with Playwright
- Multi-language AI content generation

### Future Plans

- Intelligent segmentation with ML
- Salesforce/HubSpot integrations
- Advanced analytics and reporting
- Webhook system for external integrations
