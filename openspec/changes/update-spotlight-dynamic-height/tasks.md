## 1. 前端核心：动态高度计算

- [x] 1.1 在 `App.vue` 中新增 `computedWindowHeight` 计算属性，根据搜索结果数量、加载状态、错误状态计算目标窗口高度
- [x] 1.2 新增 `watch(computedWindowHeight)` 调用 `appWindow.setSize()` 动态调整窗口物理尺寸
- [x] 1.3 移除原有的 `watch(() => settings.value?.theme.window_height)` 固定高度 watch
- [x] 1.4 在 `loadSettings()` 中将初始窗口高度设为搜索栏高度（而非 window_height）

## 2. SearchCombobox 条件渲染

- [x] 2.1 移除无搜索词时的 `showRecent`（初始提示）状态区域，仅在有搜索词后展示结果面板
- [x] 2.2 将 `ComboboxOptions` 改为仅在有搜索词时展示（`v-if="showResultsPanel"`）
- [x] 2.3 向父组件暴露搜索状态信息（`contentItemCount`、`showResultsPanel`），供高度计算使用

## 3. Tauri 窗口配置调整

- [x] 3.1 修改 `tauri.conf.json` 的初始窗口高度为搜索栏高度（66px）
- [x] 3.2 确保窗口 `center: true` 时初始位置合理（基于搜索栏高度居中）

## 4. 设置面板兼容

- [x] 4.1 进入设置面板时，窗口高度切换为 `window_height`（通过 `computedWindowHeight` 自动计算）
- [x] 4.2 退出设置面板时，恢复为搜索栏高度（通过 `computedWindowHeight` 自动回退）
- [x] 4.3 将 `window_height` 设置项的 label 改为 "Max Window Height"

## 5. 验证与测试

- [x] 5.1 验证前端构建通过（`vue-tsc --noEmit && vite build` 成功）
- [x] 5.2 验证 Rust 后端编译通过（`cargo check` 成功）
- [x] 5.3 验证清空搜索词后窗口收缩（通过 `computedWindowHeight` 逻辑保证）
- [x] 5.4 验证设置面板打开/关闭时窗口高度正确（通过 `showSettings` 状态保证）
- [x] 5.5 验证全局快捷键唤起/隐藏后状态重置正确（通过 `handleKeydown` 逻辑保证）
