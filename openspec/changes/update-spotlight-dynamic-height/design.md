## Context

当前 Launcher 应用使用 Tauri 2.x 框架，窗口以固定尺寸（750×480）创建。前端 Vue 通过 `appWindow.setSize(LogicalSize)` 控制窗口物理尺寸。搜索结果展示在 `SearchCombobox.vue` 中，使用 HeadlessUI 的 Combobox 组件。

需要改为 Spotlight 风格：初始只有搜索框，结果出现时窗口动态扩展。

## Goals / Non-Goals

**Goals:**
- 初始窗口仅展示搜索栏（高度 ≈ input_height + 边距 ≈ 72px）
- 搜索有结果时，窗口平滑扩展到适配内容的高度（上限为 max_window_height）
- 搜索结果清空或搜索词清空时，窗口收缩回搜索栏高度
- 加载中/空状态/错误状态也应展开窗口（但高度适配内容而非固定）
- 窗口居中位置在高度变化时保持合理（保持顶部位置不变）

**Non-Goals:**
- 不涉及窗口宽度的动态变化（宽度保持用户配置的固定值）
- 不涉及动画/弹跳效果（仅做尺寸变更，避免性能问题）
- 不修改设置面板的窗口行为（设置面板仍使用固定高度）

## Decisions

### 1. 高度计算策略：前端计算 + setSize 调用

**决策**: 在前端（Vue 层）根据搜索状态计算目标窗口高度，通过 `appWindow.setSize()` 调用 Tauri API 动态调整。

**原因**:
- 前端最了解当前 UI 状态（结果数量、是否加载中等）
- Tauri 的 `setSize` API 已被当前代码使用，改动最小
- 避免 Rust 后端参与 UI 布局逻辑，保持关注点分离

**替代方案**:
- Rust 端通过 window event 监听前端消息再调整大小 → 多一层 IPC，增加延迟
- CSS 自动高度 + 窗口 `resizable` → Tauri 透明窗口下 CSS 自动高度不可靠

### 2. 高度计算公式

```
搜索栏高度 = input_height + padding (12px)
单条结果高度 = item_height
底部栏高度 = 32px
结果面板内边距 = 12px

窗口高度 = 搜索栏高度 + (有内容时: min(结果数 × 单条结果高度, max_content_height) + 底部栏高度 + 结果面板内边距)
```

### 3. window_height 设置项改为 max_window_height 语义

**决策**: 保留 `window_height` 字段，但其语义变为"窗口展开时的最大高度"。默认值保持 480px。

**原因**: 避免数据迁移，向后兼容已有的用户配置。

## Risks / Trade-offs

- **窗口闪烁风险** → 使用 `requestAnimationFrame` 确保 DOM 更新后再调用 `setSize`，减少闪烁
- **窗口位置跳动** → 保持窗口顶部位置不变（Tauri 默认行为即从顶部扩展/收缩，如需可手动设置 position）
- **设置面板切换** → 进入设置面板时强制使用完整高度，退出时恢复动态计算

## Open Questions

- 无（设计已明确）
