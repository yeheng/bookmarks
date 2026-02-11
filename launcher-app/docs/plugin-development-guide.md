# Plugin Development Guide

> Build plugins for Launcher using any scripting language — Node.js, Python, Bash, or compiled binaries.

---

## Architecture Overview

```
┌─────────────────────────────────────────────┐
│  Launcher Host                              │
│                                             │
│   Search Input ──► Keyword Detection        │
│                          │                  │
│                    PluginRegistry            │
│                    (keyword → plugin)        │
│                          │                  │
│                    PluginExecutor            │
│                    ┌─────┴──────┐           │
│                    │ subprocess │           │
│               stdin│  (plugin)  │stdout     │
│              JSON ─┤            ├─ JSON     │
│                    │            │           │
│                    └────────────┘           │
│                          │                  │
│                    Render Results            │
└─────────────────────────────────────────────┘
```

**Core contract**: A plugin is a directory containing a `plugin.toml` manifest and one or more scripts. The host spawns the script as a subprocess, writes a JSON request to **stdin**, and reads a JSON response from **stdout**.

---

## Quick Start

### 1. Create the plugin directory

```
my-plugin/
├── plugin.toml       # Manifest (required)
├── index.js          # Script (referenced in manifest)
└── assets/           # Optional icons
    └── icon.png
```

### 2. Write the manifest

```toml
[plugin]
name = "my-plugin"
title = "My Plugin"
description = "Does something useful."
version = "1.0.0"
author = "Your Name"
api_version = "0.1"
icon = "🔧"

[[commands]]
name = "search"
title = "Search"
description = "Search for things"
keyword = "mp"
mode = "search"
script = "index.js"
runtime = "node"
timeout = 10
```

### 3. Write the script

```javascript
#!/usr/bin/env node

let input = '';
process.stdin.setEncoding('utf8');
process.stdin.on('data', (chunk) => { input += chunk; });
process.stdin.on('end', () => {
  const { command, query, preferences } = JSON.parse(input);

  const response = {
    items: [
      {
        uid: 'result-1',
        title: `Result for: ${query}`,
        subtitle: 'A helpful description',
        icon: { emoji: '🔧' },
        actions: [
          { type: 'copy', text: query, title: 'Copy query' },
        ],
      },
    ],
  };

  process.stdout.write(JSON.stringify(response));
});
```

### 4. Install

Copy the plugin directory into:
```
~/Library/Application Support/com.bookmarks.launcher/plugins/my-plugin/
```

Restart Launcher or use the plugin management UI to refresh.

### 5. Use

Type `mp your query` in the search bar (where `mp` is your keyword).

---

## Manifest Reference (`plugin.toml`)

### `[plugin]` — Metadata (required)

| Field         | Type   | Required | Description                                                  |
|---------------|--------|----------|--------------------------------------------------------------|
| `name`        | string | ✅       | Unique ID, kebab-case only (`a-z`, `0-9`, `-`)              |
| `title`       | string | ✅       | Human-readable name shown in UI                              |
| `description` | string | ✅       | What this plugin does                                        |
| `version`     | string | ✅       | Semantic version (e.g. `"1.0.0"`)                            |
| `api_version` | string | ✅       | Minimum host API version required (currently `"0.1"`)        |
| `author`      | string | –        | Author name or handle                                        |
| `icon`        | string | –        | Emoji or filename relative to `assets/` directory            |

### `[[commands]]` — Commands (at least one required)

| Field         | Type   | Required | Default    | Description                                           |
|---------------|--------|----------|------------|-------------------------------------------------------|
| `name`        | string | ✅       | –          | Command ID (kebab-case)                               |
| `title`       | string | ✅       | –          | Display title                                         |
| `description` | string | ✅       | –          | What this command does                                |
| `keyword`     | string | ✅       | –          | Trigger keyword (must be unique across all plugins)   |
| `script`      | string | ✅       | –          | Script path relative to plugin directory              |
| `mode`        | string | –        | `"search"` | One of: `search`, `action`, `detail`                  |
| `runtime`     | string | –        | `"node"`   | One of: `node`, `python`, `bash`, `binary`            |
| `timeout`     | int    | –        | `10`       | Max execution time in seconds                         |

### `[[preferences]]` — User Preferences (optional)

| Field         | Type   | Required | Description                                         |
|---------------|--------|----------|-----------------------------------------------------|
| `name`        | string | ✅       | Preference key                                      |
| `type`        | string | ✅       | One of: `text`, `password`, `number`, `boolean`, `select` |
| `title`       | string | ✅       | Display label                                       |
| `required`    | bool   | –        | Must be set before plugin works (default: `false`)  |
| `description` | string | –        | Help text                                           |
| `default`     | string | –        | Default value                                       |
| `options`     | array  | for `select` | Selection options: `[{ label, value }]`          |

**Example with preferences:**

```toml
[[preferences]]
name = "api_key"
type = "password"
title = "API Key"
description = "Your API key for the service"
required = true

[[preferences]]
name = "region"
type = "select"
title = "Region"
default = "us"
options = [
    { label = "US", value = "us" },
    { label = "EU", value = "eu" },
    { label = "Asia", value = "ap" },
]

[[preferences]]
name = "max_results"
type = "number"
title = "Max Results"
default = "10"
```

---

## JSON Protocol

### Request (stdin)

The host writes a single JSON object to your script's stdin, then closes the pipe:

```json
{
  "command": "search",
  "query": "user's search text",
  "preferences": {
    "api_key": "user-configured-value",
    "region": "us"
  }
}
```

| Field         | Type              | Description                                       |
|---------------|-------------------|---------------------------------------------------|
| `command`     | string            | The command `name` from manifest that was invoked  |
| `query`       | string            | User input after the keyword (may be empty)        |
| `preferences` | `{string: string}`| User-configured preferences as key-value pairs     |

### Response (stdout)

Your script writes a single JSON object to stdout:

```json
{
  "items": [
    {
      "uid": "unique-result-id",
      "title": "Result Title",
      "subtitle": "Description text",
      "arg": "optional-argument",
      "badge": "Category",
      "icon": {
        "emoji": "🔍"
      },
      "actions": [
        { "type": "copy", "text": "text to copy", "title": "Copy" },
        { "type": "open-url", "url": "https://example.com", "title": "Open" }
      ]
    }
  ],
  "cache": {
    "ttl_seconds": 60
  }
}
```

### Result Item Fields

| Field      | Type     | Required | Description                                    |
|------------|----------|----------|------------------------------------------------|
| `uid`      | string   | ✅       | Unique ID for this result                      |
| `title`    | string   | ✅       | Primary display text                           |
| `subtitle` | string   | –        | Secondary text                                 |
| `arg`      | string   | –        | Argument passed to actions                     |
| `badge`    | string   | –        | Badge label shown alongside the result         |
| `icon`     | object   | –        | Icon (see below)                               |
| `actions`  | array    | ✅       | At least one action (first = default on Enter) |

### Icon Object

Specify one of:

| Field   | Description                    | Example                |
|---------|--------------------------------|------------------------|
| `emoji` | Emoji character                | `{ "emoji": "🔍" }`   |
| `url`   | Remote image URL               | `{ "url": "https://..." }` |
| `path`  | Local file path (absolute)     | `{ "path": "/path/to/icon.png" }` |

### Action Types

| Type          | Fields                         | Description                         |
|---------------|--------------------------------|-------------------------------------|
| `copy`        | `text`, `title?`               | Copy text to clipboard              |
| `open-url`    | `url`, `title?`                | Open URL in default browser         |
| `open-file`   | `path`, `title?`               | Open file/directory in system       |
| `paste`       | `text`, `title?`               | Paste text into frontmost app       |
| `run-command` | `command`, `arg?`, `title?`    | Run a system command                |

**First action** in the array is the default action executed when the user presses Enter.

### Cache Directive (optional)

Include a `cache` field in the response to tell the host to cache results:

```json
{
  "items": [...],
  "cache": { "ttl_seconds": 300 }
}
```

The host caches the response keyed by `(plugin_id, command, query)`. Subsequent calls with the same inputs will return the cached result without spawning a subprocess.

---

## Environment Variables

Your script receives these environment variables:

| Variable                | Description                                 |
|-------------------------|---------------------------------------------|
| `LAUNCHER_PLUGIN_DIR`   | Absolute path to this plugin's directory    |
| `LAUNCHER_DATA_DIR`     | Path to plugin's data directory (writable)  |
| `LAUNCHER_API_VERSION`  | Host API version (e.g. `"0.1"`)             |
| `LAUNCHER_PREF_{NAME}`  | Each preference as `LAUNCHER_PREF_` + uppercase name |

For example, a preference named `api_key` becomes `LAUNCHER_PREF_API_KEY`.

---

## Supported Runtimes

| Runtime  | Resolved To      | Requirements                          |
|----------|------------------|---------------------------------------|
| `node`   | `node`           | Node.js installed and on `$PATH`      |
| `python` | `python3`/`python` | Python 3 installed and on `$PATH`   |
| `bash`   | `bash`           | Bash available (macOS/Linux built-in) |
| `binary` | Direct execution | Self-contained executable             |

### Runtime Resolution

1. The host checks if the runtime is available using `which`
2. For Python, it tries `python3` first, then falls back to `python`
3. For `binary`, the script path itself is executed directly

---

## Error Handling

### stderr → Logs

Anything written to **stderr** is captured and saved to `<plugin_dir>/logs/latest.log`. Use stderr for debug output:

```python
import sys
print("Debug: processing query...", file=sys.stderr)
```

View logs from the Launcher settings → Plugins → "View Logs" button.

### Graceful Failures

If your script encounters an error, return an empty items array:

```json
{ "items": [] }
```

Or return an error result that the user can see:

```json
{
  "items": [
    {
      "uid": "error",
      "title": "Something went wrong",
      "subtitle": "Check your API key in plugin settings",
      "icon": { "emoji": "❌" },
      "actions": []
    }
  ]
}
```

### Timeout

If your script exceeds the configured `timeout` (default 10s), the host will kill the process. For long-running operations, consider:
- Increasing `timeout` in the manifest
- Using the cache directive to avoid re-execution
- Breaking work into smaller chunks

---

## Examples

### Node.js — Hello World

```toml
# plugin.toml
[plugin]
name = "hello-world"
title = "Hello World"
description = "A minimal example plugin."
version = "1.0.0"
api_version = "0.1"
icon = "👋"

[[commands]]
name = "greet"
title = "Say Hello"
description = "Returns a friendly greeting"
keyword = "hello"
script = "index.js"
runtime = "node"
```

```javascript
// index.js
#!/usr/bin/env node
let input = '';
process.stdin.setEncoding('utf8');
process.stdin.on('data', (chunk) => { input += chunk; });
process.stdin.on('end', () => {
  try {
    const { query } = JSON.parse(input);
    const name = query.trim() || 'World';
    process.stdout.write(JSON.stringify({
      items: [
        {
          uid: 'greeting',
          title: `Hello, ${name}!`,
          subtitle: 'A friendly greeting',
          icon: { emoji: '👋' },
          actions: [
            { type: 'copy', text: `Hello, ${name}!`, title: 'Copy' },
          ],
        },
      ],
    }));
  } catch (err) {
    process.stderr.write(`Error: ${err.message}\n`);
    process.stdout.write('{"items":[]}');
  }
});
```

### Python — System Info

```toml
# plugin.toml
[plugin]
name = "system-info"
title = "System Info"
description = "Displays system information."
version = "1.0.0"
api_version = "0.1"
icon = "💻"

[[commands]]
name = "info"
title = "System Information"
description = "Show system details"
keyword = "sys"
script = "main.py"
runtime = "python"
```

```python
#!/usr/bin/env python3
import json, sys, platform

def main():
    request = json.loads(sys.stdin.read())
    query = request.get("query", "")

    items = [
        {
            "uid": "os",
            "title": f"OS: {platform.system()} {platform.release()}",
            "subtitle": platform.platform(),
            "icon": {"emoji": "🖥️"},
            "actions": [{"type": "copy", "text": platform.platform(), "title": "Copy"}],
        },
    ]

    # Filter by query if provided
    if query.strip():
        q = query.lower()
        items = [i for i in items if q in i["title"].lower()]

    print(json.dumps({"items": items}))

if __name__ == "__main__":
    main()
```

### Bash — Quick Calculator

```toml
# plugin.toml
[plugin]
name = "quick-calc"
title = "Quick Calculator"
description = "Evaluate math expressions."
version = "1.0.0"
api_version = "0.1"
icon = "🧮"

[[commands]]
name = "calc"
title = "Calculate"
description = "Evaluate a math expression"
keyword = "calc"
script = "calc.sh"
runtime = "bash"
timeout = 5
```

```bash
#!/bin/bash
read -r input
query=$(echo "$input" | sed 's/.*"query":"\([^"]*\)".*/\1/')

if [ -z "$query" ]; then
  echo '{"items":[{"uid":"help","title":"Type a math expression","subtitle":"Example: 2+2","icon":{"emoji":"🧮"},"actions":[]}]}'
  exit 0
fi

result=$(echo "scale=6; $query" 2>/dev/null | bc 2>/dev/null)

if [ -n "$result" ]; then
  clean=$(echo "$result" | sed 's/\.0*$//;s/\(\.[0-9]*[1-9]\)0*$/\1/')
  echo "{\"items\":[{\"uid\":\"result\",\"title\":\"= ${clean}\",\"subtitle\":\"${query}\",\"icon\":{\"emoji\":\"🧮\"},\"actions\":[{\"type\":\"copy\",\"text\":\"${clean}\",\"title\":\"Copy\"}]}]}"
else
  echo '{"items":[{"uid":"error","title":"Invalid expression","icon":{"emoji":"❌"},"actions":[]}]}'
fi
```

---

## Best Practices

1. **Keep it fast** — Plugins run synchronously. Aim for < 1 second response time.
2. **Use caching** — Return `cache.ttl_seconds` for results that don't change frequently.
3. **Handle empty queries** — Show helpful suggestions when `query` is empty.
4. **Use stderr for debug** — Never write debug output to stdout; it corrupts the JSON response.
5. **Validate inputs** — Don't trust the query string; sanitize before using in shell commands or URLs.
6. **Provide meaningful actions** — The first action in the array is the default (triggered by Enter).
7. **Use preferences for secrets** — Declare `type = "password"` preferences for API keys.
8. **Fail gracefully** — Always return valid JSON, even on error. Use `{"items":[]}` as fallback.
9. **Test standalone** — You can test your plugin from the terminal:
   ```bash
   echo '{"command":"search","query":"test","preferences":{}}' | node index.js
   ```

---

## Plugin Directory Structure

When installed, plugins live in the application data directory:

```
~/Library/Application Support/com.bookmarks.launcher/
└── plugins/
    └── my-plugin/
        ├── plugin.toml
        ├── index.js
        ├── assets/
        │   └── icon.png
        ├── data/          # Writable data directory (LAUNCHER_DATA_DIR)
        └── logs/
            └── latest.log # stderr capture
```

---

## API Version Compatibility

The current host API version is **`0.1`**.

- Plugins declare the minimum `api_version` they require
- The host checks: plugin's major version must match host's major version, and plugin's minor version must be ≤ host's minor version
- Example: a plugin with `api_version = "0.1"` works with host `0.1` and `0.2`, but not `1.0`

As the API evolves, the version will increment to signal new capabilities or breaking changes.
