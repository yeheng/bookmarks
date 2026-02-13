## Context

Launcher 当前是一个硬编码功能的桌面搜索工具（书签 + 文件）。为了扩展为平台级产品，需要引入插件系统。

经过对 Raycast（Node.js + React + JSON-RPC）和 Alfred（Script Filter + JSON stdout）的深度调研，结合本项目的技术栈（Tauri 2.x + Rust + Vue.js）和**最小化产品原则**，选择了 **Script-based + JSON 宿主渲染**方案。

### 技术约束
- Tauri 2.x 架构（Rust backend + Web frontend）
- 现有搜索响应 < 100ms 硬性要求
- 空闲内存 < 100MB
- 跨平台（macOS/Windows/Linux）
- 隐私优先，无遥测

## Goals / Non-Goals

### Goals
- 用户可以安装/卸载第三方插件扩展 Launcher 功能
- 插件通过 keyword 触发，返回结构化搜索结果
- 插件崩溃不影响宿主应用稳定性
- 插件开发者可以用任何语言编写插件（JS/Python/Shell/Go）
- 保持核心搜索 < 100ms（插件结果可异步加载）

### Non-Goals
- ❌ 不做沙盒化强制隔离（复杂度过高，MVP 不需要）
- ❌ 不做自定义 UI 渲染（插件不能定义自己的 Vue 组件）
- ❌ 不做插件间依赖（插件独立运行）
- ❌ 不做插件商店/市场（MVP 阶段通过本地安装）
- ❌ 不做 AI Tools 集成（Phase 2）

## Decisions

### Decision 1: Script-based Runtime（而非嵌入式 JS 引擎）

**选择**: 子进程执行脚本 + JSON stdout 协议

**替代方案**:
| 方案 | 优点 | 缺点 | 复杂度 |
|------|------|------|--------|
| **Script-based (选择)** | 语言无关、简单可靠、Alfred 验证 | 启动开销、无内置沙盒 | 低 |
| Deno Core 嵌入 | 内置沙盒、V8 性能 | 仅限 JS/TS、Rust 集成复杂 | 高 |
| Node.js 子进程 | npm 生态、Raycast 验证 | 依赖 Node 安装、体积大 | 中 |
| WASM | 天然沙盒、高性能 | 生态不成熟、开发体验差 | 高 |

**理由**: Script-based 是最小化方案。Alfred 用此模型支撑了庞大的 workflow 生态。对 MVP 来说，简单 > 完美。

### Decision 2: JSON 宿主渲染（而非插件自定义 UI）

**选择**: 插件返回结构化 JSON，宿主用已有 Vue 组件渲染

**协议设计**（借鉴 Alfred Script Filter 格式）:
```json
{
  "items": [
    {
      "uid": "unique-id",
      "title": "Result Title",
      "subtitle": "Description",
      "arg": "action-argument",
      "icon": { "path": "icon.png" },
      "actions": [
        { "type": "open-url", "url": "https://..." },
        { "type": "copy", "text": "..." },
        { "type": "run-command", "command": "detail", "arg": "..." }
      ]
    }
  ],
  "cache": { "ttl_seconds": 300 }
}
```

**理由**:
- 与现有 `SearchResult` 类型高度对齐
- UI 一致性由宿主保证
- 插件开发零前端知识要求
- 安全——插件无法注入任意 HTML/JS

### Decision 3: TOML Manifest（而非 JSON/YAML）

**选择**: `plugin.toml` 作为插件清单格式

**理由**:
- Rust 生态标准（`Cargo.toml`）
- 对开发者可读性优于 JSON
- `toml` crate 成熟稳定
- 比 YAML 更明确（无隐式类型转换）

### Decision 4: Keyword 触发（而非全局混合搜索）

**选择**: 插件通过 keyword 前缀触发（如 `gh:search-term`）

**替代方案**: 所有插件结果混入全局搜索

**理由**:
- 用户意图明确，避免搜索结果噪音
- 核心搜索性能不受插件影响（< 100ms 保证）
- 实现简单——前端只需检测 keyword 前缀
- 后续可增加"全局搜索混合"作为可选功能

### Decision 5: 进程隔离 + 超时保护

**策略**:
- 每个插件命令在独立子进程中运行
- 默认超时 10 秒（可配置）
- 内存不做硬限制（依赖 OS 进程管理）
- 崩溃/超时自动终止，宿主显示错误提示
- stderr 输出写入插件日志文件

## Architecture

```
┌──────────────────────────────────────────┐
│              Launcher (Tauri)             │
│                                          │
│  ┌─────────┐  ┌─────────────────────┐    │
│  │ Vue.js  │  │    Rust Backend     │    │
│  │ Frontend│  │                     │    │
│  │         │←─┤  ┌───────────────┐  │    │
│  │ Search  │  │  │ Plugin        │  │    │
│  │ Combobox│  │  │ Registry      │  │    │
│  │  +      │  │  │ (discover,    │  │    │
│  │ Plugin  │  │  │  load, CRUD)  │  │    │
│  │ Results │  │  └───────┬───────┘  │    │
│  │         │  │          │          │    │
│  └─────────┘  │  ┌───────▼───────┐  │    │
│               │  │ Plugin        │  │    │
│               │  │ Runtime       │  │    │
│               │  │ (executor)    │  │    │
│               │  └───────┬───────┘  │    │
│               │          │          │    │
│               └──────────┼──────────┘    │
└──────────────────────────┼───────────────┘
                           │ spawn subprocess
              ┌────────────┼────────────┐
              │            │            │
         ┌────▼────┐ ┌────▼────┐ ┌────▼────┐
         │ Plugin  │ │ Plugin  │ │ Plugin  │
         │   A     │ │   B     │ │   C     │
         │ (node)  │ │(python) │ │ (bash)  │
         └─────────┘ └─────────┘ └─────────┘
              │            │            │
              └──── JSON stdout ────────┘
```

### 数据流

```
1. 用户输入 "gh:react"
2. Frontend 检测到 keyword "gh" → 匹配插件 "github-search"
3. Frontend invoke → Rust "execute_plugin_command"
4. Rust 查找 registry → 找到插件 manifest
5. Rust spawn 子进程: `node /plugins/github-search/dist/index.js`
6. 子进程 stdin 接收: {"command": "search", "query": "react", "preferences": {...}}
7. 子进程 stdout 返回: {"items": [...]}
8. Rust 解析 JSON → 返回给 Frontend
9. Frontend 用 SearchResultItem 渲染插件结果
```

## Risks / Trade-offs

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| 子进程启动延迟 | 首次搜索慢 200-500ms | 显示 loading 状态；后续考虑进程池 |
| 恶意插件执行任意代码 | 安全风险 | MVP: 信任模型（用户自行安装）；Phase 2: 权限系统 |
| 插件质量参差不齐 | 用户体验差 | 提供开发指南 + 模板 + 校验工具 |
| JSON 协议不够灵活 | 无法支持复杂 UI | 当前 List/Detail 够用；后续按需扩展 |
| 跨平台脚本兼容 | Windows 上 bash 不可用 | 推荐 Node.js/Python；记录平台支持情况 |

## Open Questions

1. 插件存储目录是放在应用数据目录（`~/.launcher/plugins/`）还是用户自选目录？
   - 建议: 默认应用数据目录，但支持 `plugin_dirs` 配置
2. 是否需要在 MVP 阶段支持插件配置 UI（preference 表单自动生成）？
   - 建议: MVP 支持，复杂度不高，但大幅提升体验
3. 插件日志如何暴露给用户调试？
   - 建议: stderr → 日志文件，前端提供"查看日志"入口
