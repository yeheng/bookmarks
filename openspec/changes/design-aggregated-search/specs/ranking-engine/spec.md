# Capability: Ranking Engine

## Status

Proposed

## Summary

A pluggable ranking system that orders search results based on flexible strategies, including context awareness (app usage, time) and machine learning relevance.

## Motivation

Current ranking uses a static formula (Frecency + BM25). While effective, it cannot adapt to different user needs (e.g., prioritizing coding documents while VS Code is open). Pluggable ranking allows smarter, context-aware results.

## Requirements

### ADDED Context Awareness

The system MUST allow boosting results based on the active application or environment context.

#### Scenario: Code Context

Given the user is currently focused on "VS Code"
When searching for "config"
Then file results with extension ".json" or ".toml" should receive a score boost

### ADDED Pluggable Strategies

The system MUST allow swapping the ranking algorithm at runtime.

#### Scenario: AI Reranking

Given a plugin that provides an AI ranking model
When the user enables "AI Ranking" in settings
Then the system should forward top results to the AI model for re-scoring before display

### MODIFIED Normalize Scores

The system MUST normalize scores across disparate providers fairly.

#### Scenario: Variable Ranges

Given provider A returns scores [0-100] and provider B [0-1]
Then the ranking engine should normalize both to a [0-1] scale before merging
To prevent one provider from dominating results
