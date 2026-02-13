# Capability: Search Registry

## Status

Proposed

## Summary

A central registry for managing the lifecycle of search providers, enabling dynamic registration and discovery at runtime.

## Motivation

Currently, search providers are hardcoded or limited to specific types (bookmarks, files, plugins). To support a truly extensible ecosystem, we need a unified registry where any component (core or plugin) can register a capability to provide search results.

## Requirements

### ADDED Dynamic Registration

The system MUST allow registering new search providers at runtime.

#### Scenario: Plugin Registration

Given a running application
When a plugin "GitHub Search" connects and registers with ID "github"
Then the registry should include "github" in the list of active providers
And subsequent search queries should be routed to this provider

#### Scenario: Duplicate ID Prevention

Given a provider registered with ID "files"
When another provider attempts to register with ID "files"
Then the registration should fail with an "ID already in use" error

### ADDED Provider Lifecycle

The system MUST handle provider unregistration and cleanup.

#### Scenario: Plugin Disconnect

Given a registered plugin provider "github"
When the plugin process terminates or disconnects
Then the provider should be automatically unregistered
And active queries to this provider should be cancelled gracefully
