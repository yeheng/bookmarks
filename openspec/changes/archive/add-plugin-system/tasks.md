## 1. Foundation — Plugin Manifest & Models
- [x] 1.1 定义 Rust 数据模型: `PluginManifest`, `PluginCommand`, `PluginPreference` structs（含 serde 反序列化）
- [x] 1.2 实现 `plugin.toml` 解析器（使用 `toml` crate），含校验逻辑（必填字段、api_version 兼容性）
- [x] 1.3 定义 TypeScript 类型: `Plugin`, `PluginCommand`, `PluginResultItem`, `PluginAction`
- [x] 1.4 编写 manifest 解析单元测试（有效/无效/缺失字段/版本不兼容场景）

**验证**: `cargo test manifest` 全部通过

## 2. Foundation — Plugin Registry (Rust)
- [x] 2.1 新增 SQLite 表: `plugins` + `plugin_preferences`（在 `db/mod.rs` 中增加 migration）
- [x] 2.2 实现 `PluginRegistry` struct: `discover()`, `register()`, `unregister()`, `enable()`, `disable()`, `list()`, `get()`
- [x] 2.3 实现启动时插件目录扫描 + 注册表同步逻辑
- [x] 2.4 实现 keyword 注册表（HashMap 内存缓存）+ 唯一性检查
- [x] 2.5 实现 preference 存储: `get_preferences()`, `set_preference()`, 密码加密
- [x] 2.6 编写 registry 集成测试（安装/卸载/启用/禁用/keyword 冲突）

**验证**: `cargo test registry` 全部通过

**依赖**: 1.1, 1.2 完成后可开始

## 3. Core — Plugin Runtime Executor (Rust)
- [x] 3.1 实现 `RuntimeResolver`: 根据 manifest `runtime` 字段确定执行器路径
- [x] 3.2 实现运行时可用性检测（`which node`, `which python3`）
- [x] 3.3 实现 `PluginExecutor`: 子进程 spawn + stdin JSON 写入 + stdout JSON 读取
- [x] 3.4 实现超时保护（tokio timeout + 进程 kill）
- [x] 3.5 实现 stderr 日志捕获 + 写入文件
- [x] 3.6 实现环境变量注入（`LAUNCHER_PLUGIN_DIR`, `LAUNCHER_DATA_DIR`, preferences）
- [x] 3.7 实现结果缓存层（基于 query + plugin_id + TTL）
- [x] 3.8 编写 executor 测试（成功执行、超时、崩溃、无效 JSON、runtime 缺失）

**验证**: `cargo test executor` 全部通过；使用测试脚本验证端到端执行

**依赖**: 1.x 完成后可开始；与 2.x 可并行

## 4. Integration — Tauri Commands (Rust)
- [x] 4.1 新增 `commands/plugins.rs`: `list_plugins`, `install_plugin`, `uninstall_plugin`, `enable_plugin`, `disable_plugin`
- [x] 4.2 新增 `commands/plugins.rs`: `execute_plugin_command`, `get_plugin_preferences`, `set_plugin_preference`
- [x] 4.3 新增 `commands/plugins.rs`: `get_plugin_log`
- [x] 4.4 在 `lib.rs` 中注册所有新 commands
- [x] 4.5 将 `PluginRegistry` + `PluginExecutor` 加入 `AppState`
- [x] 4.6 在应用启动流程中初始化 plugin 目录 + 执行 registry discover

**验证**: `cargo build` 成功；手动调用 invoke 验证

**依赖**: 2.x, 3.x 完成后开始

## 5. Frontend — Keyword Detection & Plugin Result Rendering
- [x] 5.1 扩展 `SearchResult` 类型，增加 `type: 'plugin'` 及 `PluginResultItem` 字段
- [x] 5.2 实现 keyword 检测逻辑: 在 `SearchCombobox.vue` 中识别 `keyword:query` 模式
- [x] 5.3 调用 `execute_plugin_command` Tauri command 并处理返回结果
- [x] 5.4 扩展 `SearchResultItem.vue` 支持渲染 plugin 类型结果（icon 解析、badge 显示）
- [x] 5.5 实现 plugin 结果 action 执行逻辑（open-url, copy, open-file, paste）
- [x] 5.6 实现 loading 状态和 error 状态 UI

**验证**: 手动测试——安装示例插件 → 输入 keyword → 看到结果 → 执行 action

**依赖**: 4.x 完成后开始

## 6. Frontend — Plugin Management UI
- [x] 6.1 在 `SettingsPanel.vue` 新增 "Plugins" 标签页
- [x] 6.2 实现插件列表展示（名称、描述、版本、状态、图标）
- [x] 6.3 实现启用/禁用 toggle
- [x] 6.4 实现卸载按钮（含确认对话框）
- [x] 6.5 实现插件配置表单自动生成（基于 manifest preferences 声明）
- [x] 6.6 实现"查看日志"入口

**验证**: 手动测试——设置面板中管理插件、配置 preferences

**依赖**: 4.x 完成后开始；与 5.x 可并行

## 7. Example Plugin & Validation
- [x] 7.1 编写示例插件 `hello-world`（Node.js）: 返回固定结果列表
- [x] 7.2 编写示例插件 `system-info`（Python）: 返回系统信息
- [x] 7.3 编写示例插件 `quick-calc`（Bash）: 简单计算器
- [x] 7.4 端到端验证: 安装 → keyword 搜索 → 查看结果 → 执行 action → 管理设置

**验证**: 三个示例插件全部可安装、运行、管理

**依赖**: 5.x, 6.x 完成后开始

## 8. Testing & Documentation
- [x] 8.1 编写 Rust 集成测试覆盖核心流程（manifest → registry → execute → result）
- [x] 8.2 编写前端组件测试（keyword 检测、result 渲染）
- [x] 8.3 编写插件开发者指南（plugin.toml 格式、JSON 协议、示例）
- [x] 8.4 确保 `cargo test` 和 `npm run build` 通过

**验证**: CI 构建通过；测试覆盖率 ≥ 70% 核心逻辑

**依赖**: 7.x 完成后开始

---

### 并行化建议

```
Phase A (Foundation):  1.x ─────────────────────►
Phase B (Registry):           2.x ────────────►    (依赖 1.x)
Phase C (Runtime):            3.x ────────────►    (依赖 1.x, 并行 2.x)
Phase D (Commands):                  4.x ──────►   (依赖 2.x + 3.x)
Phase E (Frontend):                        5.x ──► (依赖 4.x)
Phase F (Mgmt UI):                         6.x ──► (依赖 4.x, 并行 5.x)
Phase G (Examples):                             7.x (依赖 5.x + 6.x)
Phase H (Testing):                               8.x
```
