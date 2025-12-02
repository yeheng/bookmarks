# Technical Decisions Log

_Auto-updated during discovery and planning sessions - you can also add information here yourself_

## Purpose

This document captures technical decisions, preferences, and constraints discovered during project discussions. It serves as input for solution-architecture.md and solution design documents.

## Confirmed Decisions

<!-- Technical choices explicitly confirmed by the team/user -->

### Frontend Technology Stack (Updated: 2025-11-30)
- **Framework**: SvelteKit 2.0+ (changed from React + Selvlet UI)
- **UI Library**: shadcn/ui (based on Radix UI) (changed from Selvlet UI)
- **Language**: TypeScript 5.0+
- **Styling**: Tailwind CSS 3.3+
- **State Management**: Svelte Stores (changed from Zustand)
- **Form Handling**: SvelteKit Forms + Zod (changed from React Hook Form)
- **Data Fetching**: SvelteKit Load Functions (changed from React Query)
- **Testing**: Vitest + Svelte Testing Library + Playwright
- **Build Tool**: Vite (SvelteKit built-in)

### Backend Technology Stack (Confirmed)
- **Framework**: Axum 0.7+
- **Language**: Rust 1.75+
- **Database**: PostgreSQL 15+
- **ORM**: SQLx 0.7+
- **Authentication**: JWT + bcrypt

## Preferences

<!-- Non-binding preferences mentioned during discussions -->

### Frontend Preferences
- Prefer server-side rendering for better SEO and performance
- Prefer progressive enhancement for forms
- Prefer type-safe form validation
- Prefer component-based architecture
- Prefer responsive design with mobile-first approach

### Development Preferences
- Prefer TypeScript for type safety
- Prefer automated testing
- Prefer code formatting with Prettier
- Prefer linting with ESLint

## Constraints

<!-- Hard requirements from infrastructure, compliance, or integration needs -->

### Technical Constraints
- Must maintain compatibility with existing Rust backend API
- Must support modern browsers (Chrome, Firefox, Safari, Edge)
- Must be responsive for mobile and desktop
- Must support both light and dark themes
- Must be accessible (WCAG 2.1 AA)

### Performance Constraints
- First Contentful Paint < 1.5s
- Largest Contentful Paint < 2.5s
- Cumulative Layout Shift < 0.1
- First Input Delay < 100ms

## To Investigate

<!-- Technical questions that need research or architect input -->

### SvelteKit Migration Considerations
- [ ] Investigate optimal SvelteKit adapter for target deployment platform
- [ ] Research SvelteKit authentication patterns with JWT
- [ ] Evaluate SvelteKit form actions vs. traditional API calls
- [ ] Investigate shadcn/ui customization and theming options
- [ ] Research SvelteKit SEO best practices

### Performance Optimization
- [ ] Investigate image optimization strategies
- [ ] Research caching strategies for SvelteKit applications
- [ ] Evaluate bundle size optimization techniques
- [ ] Investigate service worker implementation

### Integration Considerations
- [ ] Research optimal API client patterns for SvelteKit
- [ ] Investigate error handling strategies
- [ ] Research testing strategies for SvelteKit applications
- [ ] Evaluate deployment options and CI/CD pipelines

## Notes

- This file is automatically updated when technical information is mentioned
- Decisions here are inputs, not final architecture
- Final technical decisions belong in solution-architecture.md
- Implementation details belong in solutions/\*.md and story context or dev notes
- The migration from React + Selvlet UI to SvelteKit + shadcn/ui was decided on 2025-11-30
- SvelteKit provides built-in SSR, routing, and form handling capabilities
- shadcn/ui offers better accessibility and customization compared to Selvlet UI
- Svelte stores provide more efficient state management than Zustand for this use case
