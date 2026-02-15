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

**Important**: A scope requires a colon immediately after the keyword (e.g., `gh:`). A bare word without colon (e.g., `gh react`) is NOT a scope — all words become regular search terms. This preserves backward compatibility with existing space-separated keyword queries.

### ADDED Quoted Terms

The system MUST respect quoted strings as single terms.

#### Scenario: Exact Phrase

Given the input `"hello world" python`
Then the parser should treat "hello world" as a single exact term
And "python" as a separate term

### ADDED Edge Case Handling

The parser MUST handle ambiguous and malformed inputs gracefully.

#### Scenario: Multiple Scopes (First Wins)

Given the input "gh: bm: react"
Then the parser should use "gh" as the scope (first scope wins)
And "bm: react" as the query terms passed to the scoped provider

#### Scenario: Empty Scope Value

Given the input "gh:"
Then the parser should set scope = "gh"
And terms = [] (empty)
And the scoped provider receives an empty query

#### Scenario: Filters Inside Quotes Are Not Parsed

Given the input `"type:pdf" machine learning`
Then the parser should treat "type:pdf" as an exact phrase (not a filter)
And "machine" and "learning" as separate terms

#### Scenario: Multiple Same-Key Filters (Last Wins)

Given the input "rust type:pdf type:doc"
Then the parser should use the last value: filter `type = "doc"`
And "rust" as a term

#### Scenario: Colons in URLs (Not Parsed as Scope)

Given the input "http://example.com api docs"
Then the parser should NOT treat "http" as a scope
The parser should recognize that a scope keyword must be a single alphanumeric word (no slashes, no `//`)
And all words become regular terms: ["http://example.com", "api", "docs"]

#### Scenario: Unknown Filter Keys (Pass Through)

Given the input "rust lang:go"
Then the parser should store filter `lang = "go"` in the filters map
Providers that don't recognize "lang" MUST ignore unknown filters gracefully

#### Scenario: Empty Input

Given the input ""
Then the parser should return an empty StructuredQuery
With scope = None, terms = [], filters = {}, phrases = []
