## ADDED Requirements

### Requirement: Script-Based Plugin Execution
Launcher MUST 通过子进程执行插件脚本，并通过 stdin/stdout JSON 协议通信。

**执行流程**:
1. Rust 根据 manifest 的 `runtime` 字段确定执行器（node/python/bash/binary）
2. spawn 子进程，工作目录设为插件根目录
3. 通过 stdin 发送 JSON 请求
4. 读取 stdout JSON 响应
5. stderr 输出写入插件日志文件

**请求协议** (stdin → plugin):
```json
{
  "command": "search",
  "query": "search term",
  "preferences": {
    "api_key": "user-configured-value"
  }
}
```

**响应协议** (plugin → stdout):
```json
{
  "items": [
    {
      "uid": "unique-id",
      "title": "Result Title",
      "subtitle": "Description text",
      "arg": "action-argument",
      "icon": { "path": "assets/icon.png" },
      "actions": [
        { "type": "open-url", "url": "https://..." },
        { "type": "copy", "text": "copy content" }
      ]
    }
  ],
  "cache": { "ttl_seconds": 300 }
}
```

#### Scenario: Plugin command executed successfully
- **WHEN** a plugin command is triggered with a query
- **THEN** the system SHALL spawn a subprocess with the correct runtime
- **THEN** the system SHALL send the request JSON via stdin
- **THEN** the system SHALL read and parse the response JSON from stdout
- **THEN** the system SHALL return parsed items to the frontend

#### Scenario: Plugin execution timeout
- **WHEN** a plugin subprocess does not respond within the configured timeout (default 10s)
- **THEN** the system SHALL kill the subprocess
- **THEN** the system SHALL return a timeout error to the frontend
- **THEN** the system SHALL log the timeout event

#### Scenario: Plugin crash handling
- **WHEN** a plugin subprocess exits with non-zero code or produces invalid JSON
- **THEN** the system SHALL capture stderr output into the plugin log file
- **THEN** the system SHALL return an error to the frontend with a user-friendly message
- **THEN** the host application SHALL remain stable

#### Scenario: Plugin stderr logged
- **WHEN** a plugin subprocess writes to stderr during execution
- **THEN** the system SHALL append the output to `{plugin_dir}/logs/latest.log`

### Requirement: Runtime Resolver
系统 MUST 根据 manifest 声明的 `runtime` 字段确定脚本执行器。

| runtime 值 | 执行器 | 说明 |
|-----------|--------|------|
| `node` | `node {script}` | 需要系统安装 Node.js |
| `python` | `python3 {script}` | 需要系统安装 Python 3 |
| `bash` | `bash {script}` | macOS/Linux 原生；Windows 需 WSL/Git Bash |
| `binary` | `./{script}` | 预编译二进制，直接执行 |

#### Scenario: Node.js script executed
- **WHEN** a plugin declares `runtime = "node"`
- **THEN** the system SHALL execute via `node {script_path}`
- **THEN** the system SHALL pass the plugin directory as working directory

#### Scenario: Runtime not available
- **WHEN** a plugin requires a runtime that is not installed on the system
- **THEN** the system SHALL return a clear error indicating which runtime is missing
- **THEN** the plugin SHALL be marked as "unavailable" in the registry

### Requirement: Execution Environment
插件子进程 MUST 在受控的环境变量上下文中运行。

环境变量注入:
- `LAUNCHER_PLUGIN_DIR` — 插件根目录绝对路径
- `LAUNCHER_DATA_DIR` — 插件数据目录绝对路径
- `LAUNCHER_API_VERSION` — 宿主 API 版本
- 用户配置的 preferences 以 `LAUNCHER_PREF_{NAME}` 格式注入

#### Scenario: Environment variables available to plugin
- **WHEN** a plugin subprocess is spawned
- **THEN** the system SHALL set `LAUNCHER_PLUGIN_DIR`, `LAUNCHER_DATA_DIR`, `LAUNCHER_API_VERSION`
- **THEN** the system SHALL set all user preferences as `LAUNCHER_PREF_{UPPER_CASE_NAME}`
