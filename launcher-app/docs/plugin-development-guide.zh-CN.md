# 插件开发指南

> 使用任何脚本语言构建 Launcher 插件 — Node.js、Python、Bash 或编译型二进制文件。

---

## 架构概览

```
┌─────────────────────────────────────────────┐
│  Launcher 主机                              │
│                                             │
│   搜索输入 ──► 关键词检测                   │
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
│                    渲染结果                  │
└─────────────────────────────────────────────┘
```

**核心契约**：插件是一个包含 `plugin.toml` 清单文件和一个或多个脚本的目录。主机将脚本作为子进程启动，向 **stdin** 写入 JSON 请求，并从 **stdout** 读取 JSON 响应。

---

## 快速开始

### 1. 创建插件目录

```
my-plugin/
├── plugin.toml       # 清单文件（必需）
├── index.js          # 脚本（在清单中引用）
└── assets/           # 可选图标
    └── icon.png
```

### 2. 编写清单文件

```toml
[plugin]
name = "my-plugin"
title = "我的插件"
description = "做一些有用的事情。"
version = "1.0.0"
author = "你的名字"
api_version = "0.1"
icon = "🔧"

[[commands]]
name = "search"
title = "搜索"
description = "搜索内容"
keyword = "mp"
mode = "search"
script = "index.js"
runtime = "node"
timeout = 10
```

### 3. 编写脚本

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
        title: `结果：${query}`,
        subtitle: '一个有用的描述',
        icon: { emoji: '🔧' },
        actions: [
          { type: 'copy', text: query, title: '复制查询' },
        ],
      },
    ],
  };

  process.stdout.write(JSON.stringify(response));
});
```

### 4. 安装

将插件目录复制到：
```
~/Library/Application Support/com.bookmarks.launcher/plugins/my-plugin/
```

重启 Launcher 或使用插件管理界面刷新。

### 5. 使用

在搜索栏中输入 `mp your query`（其中 `mp` 是你的关键词）。

---

## 清单文件参考 (`plugin.toml`)

### `[plugin]` — 元数据（必需）

| 字段         | 类型   | 必需   | 描述                                                         |
|---------------|--------|--------|-------------------------------------------------------------|
| `name`        | string | ✅     | 唯一标识符，仅限 kebab-case（`a-z`、`0-9`、`-`）           |
| `title`       | string | ✅     | 在 UI 中显示的人类可读名称                                   |
| `description` | string | ✅     | 此插件的作用                                                 |
| `version`     | string | ✅     | 语义化版本号（例如 `"1.0.0"`）                              |
| `api_version` | string | ✅     | 所需的最低主机 API 版本（当前为 `"0.1"`）                    |
| `author`      | string | –      | 作者名称或昵称                                               |
| `icon`        | string | –      | Emoji 或相对于 `assets/` 目录的文件名                        |

### `[[commands]]` — 命令（至少需要一项）

| 字段         | 类型   | 必需   | 默认值     | 描述                                                 |
|---------------|--------|--------|-----------|-----------------------------------------------------|
| `name`        | string | ✅     | –         | 命令 ID（kebab-case）                              |
| `title`       | string | ✅     | –         | 显示标题                                             |
| `description` | string | ✅     | –         | 此命令的作用                                         |
| `keyword`     | string | ✅     | –         | 触发关键词（在所有插件中必须唯一）                   |
| `script`      | string | ✅     | –         | 相对于插件目录的脚本路径                             |
| `mode`        | string | –      | `"search"` | 可选：`search`、`action`、`detail`                 |
| `runtime`     | string | –      | `"node"`   | 可选：`node`、`python`、`bash`、`binary`           |
| `timeout`     | int    | –      | `10`      | 最长执行时间（秒）                                   |

### `[[preferences]]` — 用户偏好（可选）

| 字段         | 类型   | 必需   | 描述                                                         |
|---------------|--------|--------|-------------------------------------------------------------|
| `name`        | string | ✅     | 偏好设置键名                                                 |
| `type`        | string | ✅     | 可选：`text`、`password`、`number`、`boolean`、`select`    |
| `title`       | string | ✅     | 显示标签                                                     |
| `required`    | bool   | –      | 插件工作前是否必须设置（默认：`false`）                      |
| `description` | string | –      | 帮助文本                                                     |
| `default`     | string | –      | 默认值                                                       |
| `options`     | array  | `select` 类型必需 | 选项列表：`[{ label, value }]`                        |

**带偏好设置的示例：**

```toml
[[preferences]]
name = "api_key"
type = "password"
title = "API 密钥"
description = "服务的 API 密钥"
required = true

[[preferences]]
name = "region"
type = "select"
title = "区域"
default = "us"
options = [
    { label = "美国", value = "us" },
    { label = "欧洲", value = "eu" },
    { label = "亚太", value = "ap" },
]

[[preferences]]
name = "max_results"
type = "number"
title = "最大结果数"
default = "10"
```

---

## JSON 协议

### 请求（stdin）

主机会向你的脚本的 stdin 写入一个 JSON 对象，然后关闭管道：

```json
{
  "command": "search",
  "query": "用户的搜索文本",
  "preferences": {
    "api_key": "用户配置的值",
    "region": "us"
  }
}
```

| 字段         | 类型              | 描述                                  |
|---------------|-------------------|--------------------------------------|
| `command`     | string            | 清单中被调用的命令 `name`             |
| `query`       | string            | 关键词后的用户输入（可能为空）        |
| `preferences` | `{string: string}`| 用户配置的偏好设置，键值对形式        |

### 响应（stdout）

你的脚本向 stdout 写入一个 JSON 对象：

```json
{
  "items": [
    {
      "uid": "unique-result-id",
      "title": "结果标题",
      "subtitle": "描述文本",
      "arg": "可选参数",
      "badge": "分类",
      "icon": {
        "emoji": "🔍"
      },
      "actions": [
        { "type": "copy", "text": "要复制的文本", "title": "复制" },
        { "type": "open-url", "url": "https://example.com", "title": "打开" }
      ]
    }
  ],
  "cache": {
    "ttl_seconds": 60
  }
}
```

### 结果项字段

| 字段      | 类型     | 必需   | 描述                                  |
|------------|----------|--------|--------------------------------------|
| `uid`      | string   | ✅     | 此结果的唯一标识符                    |
| `title`    | string   | ✅     | 主要显示文本                          |
| `subtitle` | string   | –      | 次要文本                              |
| `arg`      | string   | –      | 传递给操作的参数                      |
| `badge`    | string   | –      | 显示在结果旁边的标签                  |
| `icon`     | object   | –      | 图标（见下方）                        |
| `actions`  | array    | ✅     | 至少一个操作（第一个 = 回车时的默认） |

### 图标对象

指定以下之一：

| 字段   | 描述                  | 示例                     |
|---------|-----------------------|--------------------------|
| `emoji` | Emoji 字符            | `{ "emoji": "🔍" }`      |
| `url`   | 远程图片 URL          | `{ "url": "https://..." }` |
| `path`  | 本地文件路径（绝对）  | `{ "path": "/path/to/icon.png" }` |

### 操作类型

| 类型          | 字段                          | 描述                       |
|---------------|-------------------------------|---------------------------|
| `copy`        | `text`、`title?`              | 复制文本到剪贴板           |
| `open-url`    | `url`、`title?`               | 在默认浏览器中打开 URL     |
| `open-file`   | `path`、`title?`              | 在系统中打开文件/目录      |
| `paste`       | `text`、`title?`              | 将文本粘贴到最前面的应用   |
| `run-command` | `command`、`arg?`、`title?`   | 运行系统命令               |

**数组中的第一个操作** 是用户按回车时的默认操作。

### 缓存指令（可选）

在响应中包含 `cache` 字段以告诉主机缓存结果：

```json
{
  "items": [...],
  "cache": { "ttl_seconds": 300 }
}
```

主机会以 `(plugin_id, command, query)` 为键缓存响应。使用相同输入的后续调用将返回缓存结果，而无需启动子进程。

---

## 环境变量

你的脚本会收到以下环境变量：

| 变量名                  | 描述                                   |
|-------------------------|---------------------------------------|
| `LAUNCHER_PLUGIN_DIR`   | 此插件目录的绝对路径                  |
| `LAUNCHER_DATA_DIR`     | 插件数据目录的路径（可写）            |
| `LAUNCHER_API_VERSION`  | 主机 API 版本（例如 `"0.1"`）         |
| `LAUNCHER_PREF_{NAME}`  | 每个偏好设置都以 `LAUNCHER_PREF_` + 大写名称形式 |

例如，名为 `api_key` 的偏好设置会变成 `LAUNCHER_PREF_API_KEY`。

---

## 支持的运行时

| 运行时  | 解析为            | 要求                          |
|----------|-------------------|-------------------------------|
| `node`   | `node`            | 已安装 Node.js 并在 `$PATH` 中 |
| `python` | `python3`/`python` | 已安装 Python 3 并在 `$PATH` 中 |
| `bash`   | `bash`            | Bash 可用（macOS/Linux 内置） |
| `binary` | 直接执行          | 独立可执行文件                |

### 运行时解析

1. 主机使用 `which` 检查运行时是否可用
2. 对于 Python，先尝试 `python3`，然后回退到 `python`
3. 对于 `binary`，直接执行脚本路径本身

---

## 错误处理

### stderr → 日志

写入 **stderr** 的任何内容都会被捕获并保存到 `<plugin_dir>/logs/latest.log`。使用 stderr 进行调试输出：

```python
import sys
print("调试：正在处理查询...", file=sys.stderr)
```

从 Launcher 设置 → 插件 → "查看日志" 按钮查看日志。

### 优雅失败

如果脚本遇到错误，返回空的项目数组：

```json
{ "items": [] }
```

或返回用户可见的错误结果：

```json
{
  "items": [
    {
      "uid": "error",
      "title": "出现错误",
      "subtitle": "请在插件设置中检查你的 API 密钥",
      "icon": { "emoji": "❌" },
      "actions": []
    }
  ]
}
```

### 超时

如果脚本超过配置的 `timeout`（默认 10 秒），主机会终止进程。对于长时间运行的操作，考虑：
- 在清单中增加 `timeout`
- 使用缓存指令避免重复执行
- 将工作分解为更小的块

---

## 示例

### Node.js — Hello World

```toml
# plugin.toml
[plugin]
name = "hello-world"
title = "Hello World"
description = "一个最小示例插件。"
version = "1.0.0"
api_version = "0.1"
icon = "👋"

[[commands]]
name = "greet"
title = "Say Hello"
description = "返回友好的问候"
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
          title: `你好，${name}！`,
          subtitle: '一个友好的问候',
          icon: { emoji: '👋' },
          actions: [
            { type: 'copy', text: `你好，${name}！`, title: '复制' },
          ],
        },
      ],
    }));
  } catch (err) {
    process.stderr.write(`错误：${err.message}\n`);
    process.stdout.write('{"items":[]}');
  }
});
```

### Python — 系统信息

```toml
# plugin.toml
[plugin]
name = "system-info"
title = "系统信息"
description = "显示系统信息。"
version = "1.0.0"
api_version = "0.1"
icon = "💻"

[[commands]]
name = "info"
title = "系统信息"
description = "显示系统详情"
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
            "title": f"操作系统：{platform.system()} {platform.release()}",
            "subtitle": platform.platform(),
            "icon": {"emoji": "🖥️"},
            "actions": [{"type": "copy", "text": platform.platform(), "title": "复制"}],
        },
    ]

    # 如果提供了查询，则进行过滤
    if query.strip():
        q = query.lower()
        items = [i for i in items if q in i["title"].lower()]

    print(json.dumps({"items": items}))

if __name__ == "__main__":
    main()
```

### Bash — 快速计算器

```toml
# plugin.toml
[plugin]
name = "quick-calc"
title = "快速计算器"
description = "计算数学表达式。"
version = "1.0.0"
api_version = "0.1"
icon = "🧮"

[[commands]]
name = "calc"
title = "计算"
description = "计算数学表达式"
keyword = "calc"
script = "calc.sh"
runtime = "bash"
timeout = 5
```

```bash
#!/bin/bash
read -r input
query=$(echo "$input" | sed 's/.*\"query\":\"\\([^\"]*\\)\".*/\\1/')

if [ -z "$query" ]; then
  echo '{"items":[{"uid":"help","title":"输入一个数学表达式","subtitle":"示例：2+2","icon":{"emoji":"🧮"},"actions":[]}]}'
  exit 0
fi

result=$(echo "scale=6; $query" 2>/dev/null | bc 2>/dev/null)

if [ -n "$result" ]; then
  clean=$(echo "$result" | sed 's/\.0*$//;s/\(\.[0-9]*[1-9]\)0*$/\1/')
  echo "{\"items\":[{\"uid\":\"result\",\"title\":\"= ${clean}\",\"subtitle\":\"${query}\",\"icon\":{\"emoji\":\"🧮\"},\"actions\":[{\"type\":\"copy\",\"text\":\"${clean}\",\"title\":\"复制\"}]}]}"
else
  echo '{"items":[{"uid":"error","title":"无效的表达式","icon":{"emoji":"❌"},"actions":[]}]}'
fi
```

---

## 最佳实践

1. **保持快速** — 插件同步运行。目标响应时间 < 1 秒。
2. **使用缓存** — 对于不经常变化的结果，返回 `cache.ttl_seconds`。
3. **处理空查询** — 当 `query` 为空时显示有用的建议。
4. **使用 stderr 调试** — 永远不要将调试输出写入 stdout；这会破坏 JSON 响应。
5. **验证输入** — 不要信任查询字符串；在 shell 命令或 URL 中使用前要进行清理。
6. **提供有意义的操作** — 数组中的第一个操作是默认操作（按回车触发）。
7. **使用偏好设置存储密钥** — 为 API 密钥声明 `type = "password"` 偏好设置。
8. **优雅失败** — 即使出错也要返回有效的 JSON。使用 `{"items":[]}` 作为回退。
9. **独立测试** — 你可以从终端测试插件：
   ```bash
   echo '{"command":"search","query":"test","preferences":{}}' | node index.js
   ```

---

## 插件目录结构

安装后，插件位于应用数据目录中：

```
~/Library/Application Support/com.bookmarks.launcher/
└── plugins/
    └── my-plugin/
        ├── plugin.toml
        ├── index.js
        ├── assets/
        │   └── icon.png
        ├── data/          # 可写数据目录（LAUNCHER_DATA_DIR）
        └── logs/
            └── latest.log # stderr 捕获
```

---

## API 版本兼容性

当前主机 API 版本是 **`0.1`**。

- 插件声明所需的最低 `api_version`
- 主机检查：插件的主版本必须与主机的主版本匹配，且插件的次版本必须 ≤ 主机的次版本
- 示例：`api_version = "0.1"` 的插件可用于主机 `0.1` 和 `0.2`，但不能用于 `1.0`

随着 API 的发展，版本号将递增以表示新功能或破坏性更改。
