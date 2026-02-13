# Design Extensible Aggregated Search

## Summary

Design a robust, extensible search aggregation system that allows dynamic registration of search providers (plugins, remote sources), supports advanced query syntax (filters, scopes), and implements context-aware result ranking.

## Motivation

The current search implementation provides a solid foundation but lacks flexibility for power users and developers. Extending search requires modifying core code or adhering to rigid plugin structures. A truly extensible system should allow:
1.  **Dynamic Providers**: Plugins should be able to register new search sources at runtime without app restarts or core code changes.
2.  **Advanced Queries**: Users need precise control (e.g., `type:pdf`, `site:github.com`) beyond simple keyword matching.
3.  **Context Ranking**: Results should be ranked based on current context (e.g., active app, time of day) not just static scores.

## Goals

1.  **Dynamic Registry**: Create a `SearchRegistry` that allows adding/removing providers on the fly.
2.  **Query Language**: Define a syntax for filters, scopes, and commands within the search bar.
3.  **Pluggable Ranking**: Allow different ranking strategies (e.g., frecency, relevance, AI-based) to be swapped or combined.
4.   **Result Actions**: Standardize how results expose actions (open, copy, run script) across all providers.

## Non-Goals

-   Implementing a full SQL engine.
-   Replacing Tantivy (we build *on top* of it).
