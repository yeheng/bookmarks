# Change: 实现 Spotlight 风格的动态高度搜索界面

## Why

当前 Launcher 窗口在启动时即以固定高度（480px）展示搜索框 + 结果区域，即使没有任何搜索结果也会占用大量屏幕空间。这与 macOS Spotlight 的交互范式不一致——Spotlight 启动时仅展示一个紧凑的搜索框，只有在产生搜索结果后才向下展开结果面板。当前的固定高度设计让界面看上去空洞且不够精致。

## What Changes

- **窗口初始高度**：启动时窗口仅显示搜索输入栏（约 60px 高度），而非固定的完整高度
- **动态高度调整**：当搜索产生结果时，窗口高度动态扩展以适配结果列表；当结果被清空时，窗口收缩回仅搜索栏状态
- **结果面板条件渲染**：结果区域（包括空状态、加载态、初始提示）仅在用户输入搜索关键字后才展示
- **前端 → Tauri 联动**：前端通过 `appWindow.setSize()` 动态调整窗口物理尺寸
- **移除 window_height 静态设置**：窗口高度不再由用户在设置中固定配置，改为自动计算

## Impact

- Affected specs: `launcher-ui`（新建）
- Affected code:
  - `launcher-app/src/App.vue` — 移除固定窗口高度的 watch 逻辑，新增动态高度计算
  - `launcher-app/src/components/SearchCombobox.vue` — 条件渲染结果面板，移除无搜索词时的初始提示
  - `launcher-app/src-tauri/tauri.conf.json` — 初始窗口高度调整为搜索栏高度
  - `launcher-app/src-tauri/src/models/settings.rs` — window_height 默认值调整或语义变更
  - `launcher-app/src/types/settings.ts` — 相应类型调整
