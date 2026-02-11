#!/usr/bin/env python3
"""
System Info Plugin — displays OS, hostname, memory, and CPU details.

Protocol:
  - Reads JSON from stdin: { command, query, preferences }
  - Writes JSON to stdout: { items: [...] }
"""

import json
import platform
import os
import sys


def get_system_items(query: str) -> list:
    """Gather system information and return as plugin result items."""
    items = []

    # OS info
    os_info = f"{platform.system()} {platform.release()}"
    items.append({
        "uid": "os",
        "title": f"OS: {os_info}",
        "subtitle": platform.platform(),
        "icon": {"emoji": "🖥️"},
        "actions": [{"type": "copy", "text": os_info, "title": "Copy OS info"}],
    })

    # Hostname
    hostname = platform.node()
    items.append({
        "uid": "hostname",
        "title": f"Hostname: {hostname}",
        "subtitle": "Machine network name",
        "icon": {"emoji": "🏠"},
        "actions": [{"type": "copy", "text": hostname, "title": "Copy hostname"}],
    })

    # Python version
    py_ver = platform.python_version()
    items.append({
        "uid": "python",
        "title": f"Python: {py_ver}",
        "subtitle": sys.executable,
        "icon": {"emoji": "🐍"},
        "actions": [{"type": "copy", "text": py_ver, "title": "Copy Python version"}],
    })

    # Architecture
    arch = platform.machine()
    items.append({
        "uid": "arch",
        "title": f"Architecture: {arch}",
        "subtitle": f"{platform.processor() or 'Unknown processor'}",
        "icon": {"emoji": "⚙️"},
        "actions": [{"type": "copy", "text": arch, "title": "Copy architecture"}],
    })

    # User
    user = os.environ.get("USER", os.environ.get("USERNAME", "unknown"))
    items.append({
        "uid": "user",
        "title": f"User: {user}",
        "subtitle": os.path.expanduser("~"),
        "icon": {"emoji": "👤"},
        "actions": [{"type": "copy", "text": user, "title": "Copy username"}],
    })

    # Filter by query if provided
    if query.strip():
        q = query.lower()
        items = [item for item in items if q in item["title"].lower() or q in item.get("subtitle", "").lower()]

    return items


def main():
    input_data = sys.stdin.read()
    try:
        request = json.loads(input_data)
        query = request.get("query", "")
        items = get_system_items(query)
        response = {"items": items}
        print(json.dumps(response))
    except Exception as e:
        print(json.dumps({"items": []}), file=sys.stdout)
        print(f"Error: {e}", file=sys.stderr)


if __name__ == "__main__":
    main()
