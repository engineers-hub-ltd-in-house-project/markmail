# Implementation Plan: Email Analytics Dashboard

## Phase 1: Database Setup and Data Models

- [ ] 1. Create database migrations for analytics tables

  - Create migration file `migrations/add_analytics_tables.sql` using
    `sqlx migrate add`
  - Define email_events table with id, campaign_id, subscriber_id, event_type,
    metadata, created_at columns
  - Add indexes on campaign_id, subscriber_id, and event_type for query
    performance
  - Convert email_events to TimescaleDB hypertable for time-series optimization
  - _Requirements: 2.1, 2.2, 7.1, 8.1_

- [ ] 2. Create subscriber engagement and export tables
  - Add subscriber_engagement table with engagement scores and tiers
  - Create analytics_exports table for tracking export jobs
  - Create materialized view campaign_metrics for pre-aggregated data
  - Run migration and verify table creation with `sqlx migrate run`
  - _Requirements: 4.1, 4.2, 5.1, 7.3_

## Phase 2: Backend Data Models and Types

- [ ] 3. Implement Rust data models for analytics

  - Create `backend/src/models/analytics.rs` with EmailEvent, CampaignMetrics
    structs
  - Add derive macros for Serialize, Deserialize, and sqlx::FromRow
  - Define EventType enum (sent, delivered, opened, clicked, bounced,
    unsubscribed)
  - Add validation methods for event types and metadata
  - _Requirements: 2.5, 2.6, 8.1_

- [ ] 4. Create TypeScript types for frontend
  - Create `frontend/src/lib/types/analytics.ts` with matching TypeScript
    interfaces
  - Define EmailEvent, CampaignMetrics, TimeSeriesDataPoint interfaces
  - Add EngagementDistribution type for subscriber segmentation
  - Export all types for use in components and services
  - _Requirements: 1.1, 2.1, 3.1, 4.1_

## Phase 3: Backend Services Implementation

- [ ] 5. Implement AnalyticsService core functionality

  - Create `backend/src/services/analytics_service.rs` with struct and impl
    block
  - Write unit tests for get_overview method using mock data
  - Implement get_overview to query aggregated metrics from database
  - Add Redis caching with 5-minute TTL for overview data
  - _Requirements: 1.1, 1.4, 7.1, 7.4_

- [ ] 6. Add campaign metrics calculation methods

  - Write tests for calculate_open_rate and calculate_click_rate formulas
  - Implement methods using formula: (unique events / delivered) × 100
  - Add get_campaign_metrics method to fetch detailed campaign data
  - Include bounce rate, unsubscribe rate, and delivery rate calculations
  - _Requirements: 2.5, 2.6, 2.3, 8.1_

- [ ] 7. Implement time-series data aggregation

  - Write tests for get_time_series with different date ranges
  - Implement hourly aggregation for 7-day range, daily for 30+ days
  - Add support for multiple campaign comparison
  - Cache aggregated results in Redis for performance
  - _Requirements: 3.1, 3.2, 3.4, 7.3_

- [ ] 8. Create ExportService for CSV generation
  - Create `backend/src/services/export_service.rs` with ExportService struct
  - Write tests for CSV generation with mock data
  - Implement export_to_csv using csv crate with UTF-8 BOM encoding
  - Add queue_large_export for async processing of >10k rows
  - _Requirements: 5.1, 5.2, 5.3, 5.5_

## Phase 4: WebSocket Implementation

- [ ] 9. Implement WebSocket handler for real-time updates

  - Create `backend/src/api/websocket.rs` with WebSocketHandler
  - Write integration test for WebSocket connection establishment
  - Implement handle_connection with authentication via JWT
  - Add connection pool management with Arc<RwLock<HashMap>>
  - _Requirements: 6.1, 6.2, 6.3_

- [ ] 10. Add Redis pub/sub for event distribution
  - Write tests for event publishing and subscription
  - Implement Redis channel subscription for email events
  - Add broadcast_update method to push updates to connected clients
  - Set up 10-second interval for campaign metrics updates
  - _Requirements: 6.1, 6.2, 6.4, 6.5_

## Phase 5: API Endpoints

- [ ] 11. Create analytics API routes

  - Create `backend/src/api/analytics.rs` with Router configuration
  - Write integration tests for GET /api/analytics/overview endpoint
  - Implement overview endpoint with time range parameter support
  - Add authentication middleware to all analytics routes
  - _Requirements: 1.1, 1.4, 1.5, 7.1_

- [ ] 12. Implement campaign and time-series endpoints

  - Write tests for campaign list and detail endpoints
  - Implement GET /api/analytics/campaigns with sorting and filtering
  - Add GET /api/analytics/time-series with aggregation support
  - Include proper error handling and status codes
  - _Requirements: 2.1, 2.2, 3.1, 3.5_

- [ ] 13. Add export and engagement endpoints
  - Write tests for export endpoint with small and large datasets
  - Implement POST /api/analytics/export with job queuing for large exports
  - Add GET /api/analytics/engagement for distribution data
  - Implement rate limiting for export endpoints
  - _Requirements: 4.1, 5.1, 5.2, 5.4_

## Phase 6: Frontend Components

- [ ] 14. Create dashboard layout and routing

  - Create `frontend/src/routes/analytics/+page.svelte` for main dashboard
  - Write component tests for dashboard layout rendering
  - Implement responsive grid layout with TailwindCSS
  - Add loading states and error boundaries
  - _Requirements: 1.1, 1.4, 1.6_

- [ ] 15. Build MetricsSummary component

  - Create `frontend/src/lib/components/analytics/MetricsSummary.svelte`
  - Write tests for metric card rendering with different values
  - Implement cards for total sent, open rate, click rate
  - Add tooltips explaining metric calculations
  - _Requirements: 1.1, 1.6, 2.5, 2.6_

- [ ] 16. Implement CampaignTable component

  - Create `frontend/src/lib/components/analytics/CampaignTable.svelte`
  - Write tests for table sorting and row expansion
  - Implement sortable columns with click handlers
  - Add expandable rows showing detailed metrics
  - _Requirements: 2.1, 2.2, 2.3, 2.4_

- [ ] 17. Create TimeSeriesChart component

  - Create `frontend/src/lib/components/analytics/TimeSeriesChart.svelte`
  - Write tests for chart rendering with Chart.js
  - Implement line chart with opens and clicks data
  - Add hover tooltips and legend for multiple series
  - _Requirements: 3.1, 3.2, 3.3, 3.6_

- [ ] 18. Build ExportButton and DateRangePicker
  - Create ExportButton component with download functionality
  - Write tests for date range selection logic
  - Implement DateRangePicker with preset options (7, 30, 90 days)
  - Add CSV download handling with FileSaver.js
  - _Requirements: 1.4, 5.1, 5.3, 5.4_

## Phase 7: Frontend Services and State

- [ ] 19. Create analytics service client

  - Create `frontend/src/lib/services/analyticsService.ts`
  - Write unit tests for API calls with mock fetch
  - Implement methods for overview, campaigns, time-series, export
  - Add error handling and retry logic for failed requests
  - _Requirements: 1.1, 2.1, 3.1, 5.1_

- [ ] 20. Implement WebSocket client for real-time updates

  - Create `frontend/src/lib/services/websocketService.ts`
  - Write tests for connection establishment and reconnection
  - Implement WebSocket client with automatic reconnection
  - Add event handlers for metric updates
  - _Requirements: 6.1, 6.2, 6.3, 6.4_

- [ ] 21. Create analytics store for state management
  - Create `frontend/src/lib/stores/analyticsStore.ts` with Svelte stores
  - Write tests for store updates and subscriptions
  - Implement stores for metrics, campaigns, and real-time status
  - Add methods to update store from WebSocket events
  - _Requirements: 1.1, 2.1, 6.4, 6.5_

## Phase 8: Integration and Testing

- [ ] 22. Wire frontend components with services

  - Update `frontend/src/routes/analytics/+page.svelte` to use analyticsService
  - Write integration tests for data fetching and display
  - Connect WebSocket service for real-time updates
  - Implement error handling and user feedback
  - _Requirements: 1.1, 1.4, 6.1, 7.1_

- [ ] 23. Implement end-to-end analytics flow tests

  - Create `frontend/tests/analytics.e2e.test.ts` with Playwright
  - Write E2E test for dashboard load and metric display
  - Test campaign table sorting and expansion
  - Verify export functionality with file download
  - _Requirements: 1.1, 2.1, 3.1, 5.1_

- [ ] 24. Add performance and load tests
  - Create `backend/tests/analytics_performance_test.rs`
  - Write load tests for dashboard API with 10k concurrent users
  - Test WebSocket scalability with multiple connections
  - Verify 3-second load time and 200ms API response requirements
  - _Requirements: 7.1, 7.2, 7.3, 7.4_

## Phase 9: Final Integration

- [ ] 25. Add analytics to main navigation

  - Update `frontend/src/routes/+layout.svelte` with analytics menu item
  - Write test for navigation visibility based on user permissions
  - Implement role-based access control for analytics features
  - Add analytics icon from Lucide Svelte library
  - _Requirements: 1.5, Security Matrix_

- [ ] 26. Integrate event collection with email service

  - Update `backend/src/services/email_service.rs` to emit events
  - Write integration tests for event collection flow
  - Add event publishing to Redis on email open/click
  - Verify events are stored in email_events table
  - _Requirements: 2.1, 6.1, 8.1, 8.2_

- [ ] 27. Complete system integration verification
  - Run full test suite including unit, integration, and E2E tests
  - Verify all requirements are met with working implementation
  - Test complete flow: send email → track event → view analytics → export data
  - Ensure WebSocket updates work across multiple browser sessions
  - _Requirements: All requirements verification_

## Task Summary

Total Tasks: 27 Estimated Time: ~60-80 hours Priority: High (addresses critical
business need for analytics visibility)

Each task is designed to be completed in 1-3 hours and builds upon previous
tasks. All tasks focus exclusively on code implementation and automated testing,
with no manual testing or deployment tasks included.
