# Capability: Ranking Strategy

## Status

Existing - No Changes

## Summary

Keep the existing `Score*0.7 + Frecency*0.3` ranking formula. No separate RankingEngine abstraction needed.

## Motivation

The current approach is simple, fast, and effective. Adding a separate `RankingEngine` trait with pluggable strategies is over-abstraction for a single weighted formula.

## Requirements

### EXISTING Score + Frecency

The system MUST continue using the proven weighted ranking.

```rust
const SCORE_WEIGHT: f64 = 0.7;
const FRECENCY_WEIGHT: f64 = 0.3;

global_score = norm_score * SCORE_WEIGHT + norm_frecency * FRECENCY_WEIGHT;
```

### EXISTING Per-Provider Normalization

The system MUST normalize scores per-provider using min-max before combining.

## Rejected Enhancements

| Idea | Why Rejected |
|------|--------------|
| ContextAware Ranking | Requires tracking active app/window, OS permissions, unclear benefit |
| AI Re-ranking | Adds 50-200ms latency, conflicts with fast search goal |
| Pluggable Strategies | Over-abstraction for single formula |
| Boosting Map | Can be added later if needed, not v1 requirement |
