# Plugin Search Integration

Plugins can integrate with the launcher's unified search system to provide custom search results alongside bookmarks and files.

## How It Works

When a user types a query that starts with a plugin's **keyword**, the search system routes the query to that plugin automatically. For example, if a plugin registers the keyword `gh`, typing `gh search rust` will:

1. Detect the `gh` keyword
2. Execute the plugin's search command with `search rust` as the query
3. Display plugin results alongside other search results

## Plugin Manifest (`plugin.toml`)

To enable search integration, your plugin must declare a command with `mode = "search"`:

```toml
[plugin]
id = "my-search-plugin"
name = "My Search Plugin"
version = "1.0.0"
api_version = "0.1"

[[commands]]
name = "search"
title = "My Search"
description = "Search my data source"
keyword = "my"
mode = "search"
script = "search.js"
runtime = "node"
timeout = 5
```

### Key Fields

| Field | Required | Description |
|-------|----------|-------------|
| `keyword` | Yes | The trigger word that activates this plugin's search |
| `mode` | Yes | Must be `"search"` for unified search integration |
| `script` | Yes | Path to the script that handles search queries |
| `runtime` | Yes | Runtime to execute the script (`node`, `python`, `bash`, `binary`) |
| `timeout` | No | Max execution time in seconds (default: 10) |

## Search Request/Response

### Request (via stdin)

Your script receives a JSON object on stdin:

```json
{
  "command": "search",
  "query": "the user's query after the keyword",
  "preferences": {
    "api_key": "user-configured-value"
  }
}
```

### Response (via stdout)

Return a JSON object with search results:

```json
{
  "items": [
    {
      "uid": "unique-id-1",
      "title": "Result Title",
      "subtitle": "Additional info",
      "arg": "https://example.com",
      "icon": {
        "emoji": "🔍"
      },
      "actions": [
        {
          "type": "open-url",
          "url": "https://example.com",
          "title": "Open in browser"
        }
      ],
      "badge": "Source"
    }
  ],
  "cache": {
    "ttl_seconds": 300
  }
}
```

### Result Item Fields

| Field | Required | Description |
|-------|----------|-------------|
| `uid` | Yes | Unique identifier for the result |
| `title` | Yes | Primary display text |
| `subtitle` | No | Secondary display text |
| `arg` | No | Primary argument (e.g., URL to open) |
| `icon` | No | Icon with `emoji`, `url`, or `path` |
| `actions` | No | Available actions (open-url, copy, paste, open-file, run-command) |
| `badge` | No | Badge text shown on the result |

### Caching

Include a `cache` object in your response to enable result caching:

```json
{
  "cache": {
    "ttl_seconds": 300
  }
}
```

## Architecture

```
User types: "gh search rust"
     │
     ▼
unified_search command
     │
     ├─▶ BookmarkSearchProvider (skipped — keyword detected)
     ├─▶ FileSearchProvider (skipped — keyword detected)  
     └─▶ PluginSearchProvider
              │
              ├── detect_keyword("gh") → match!
              ├── resolve_keyword → plugin_id, command
              └── execute(plugin_dir, command, "search rust")
                       │
                       └── subprocess: node search.js
                                stdin: {"command":"search","query":"search rust"}
                                stdout: {"items":[...]}
```
