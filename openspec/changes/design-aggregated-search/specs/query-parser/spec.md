# Capability: Query Parser

## Status

Proposed

## Summary

A flexible parser to interpret user input string into structured components like scope, filters, and search terms.

## Motivation

Current search treats input as a raw string (except for plugin keyword detection). To allow precise control (e.g., searching only PDFs, or only within GitHub), we need a structured query syntax that users can learn.

## Requirements

### ADDED Filter Syntax

The system MUST parse key-value filters from the query string.

#### Scenario: Type Filter

Given the input "learning rust type:pdf"
Then the parser should extract "learning rust" as terms
And key "type" with value "pdf" as a filter

#### Scenario: Scope Filter

Given the input "gh: user/repo"
Then the parser should identify "gh" as the scope provider ID
And "user/repo" as the query for that provider
And bypass all other providers

### ADDED Quoted Terms

The system MUST respect quoted strings as single terms.

#### Scenario: Exact Phrase

Given the input `"hello world" python`
Then the parser should treat "hello world" as a single exact term
And "python" as a separate term
