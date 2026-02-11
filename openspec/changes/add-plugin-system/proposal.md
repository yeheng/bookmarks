# Change: Add Plugin System

## Why

当前 Launcher 的功能（书签搜索、文件搜索）是硬编码的。用户无法扩展新的搜索源、自定义动作或集成第三方服务。对标 Raycast 和 Alfred，插件系统是从"工具"升级为"平台"的关键一步。

遵循**最小化产品原则**，本提案设计一个 Script-based 的插件架构——插件通过子进程执行脚本并返回 JSON，宿主端用已有 Vue 组件渲染结果。这是最简单且已被 Alfred 验证过的模型。

## What Changes

### Phase 1 (MVP)
- **Plugin Manifest**: 定义 `plugin.toml` 清单格式（元数据、命令声明、配置项声明）
- **Plugin Registry**: Rust 侧插件注册表（发现、加载、启用/禁用、卸载）
- **Plugin Runtime**: Script-based 执行引擎（子进程 + JSON stdout 协议）
- **Plugin UI Rendering**: 结构化 JSON 结果集成到现有搜索 UI

### Phase 2 (后续)
- AI Tools 声明与调度
- 插件商店 / GitHub 分发
- 权限系统增强
- 热重载开发工具

## Impact
- Affected specs: (新建) `plugin-manifest`, `plugin-runtime`, `plugin-registry`, `plugin-ui-rendering`
- Affected code:
  - `src-tauri/src/` — 新增 `plugins/` 模块（manifest 解析、registry、runtime executor）
  - `src-tauri/src/commands/` — 新增 plugin 相关 Tauri commands
  - `src-tauri/src/models/` — 新增 Plugin, PluginCommand, PluginResult 模型
  - `src/` — 扩展搜索结果类型支持 plugin 结果、新增插件管理 UI
  - `src/types/` — 新增 plugin 相关 TypeScript 类型
