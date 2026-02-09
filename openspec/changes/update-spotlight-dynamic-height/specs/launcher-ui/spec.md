## ADDED Requirements

### Requirement: Spotlight 风格动态窗口高度

Launcher 窗口 SHALL 在初始状态下仅展示搜索输入栏，窗口高度 MUST 等于搜索栏高度（input_height + 上下内边距）。当搜索产生内容（结果、加载态、空状态、错误态）时，窗口 SHALL 动态扩展高度以适配内容。窗口最大高度 MUST 不超过 `window_height` 设置值。当搜索词被清空时，窗口 SHALL 收缩回搜索栏高度。

#### Scenario: 初始启动仅展示搜索栏
- **WHEN** Launcher 窗口通过全局快捷键唤起
- **THEN** 窗口仅展示搜索输入栏，高度约为 input_height + padding
- **AND** 不展示任何结果面板、提示文字或快捷键说明

#### Scenario: 输入搜索词后窗口展开
- **WHEN** 用户在搜索栏中输入关键字
- **AND** 搜索返回结果（或进入加载/空结果/错误状态）
- **THEN** 窗口高度动态扩展以展示结果面板
- **AND** 窗口高度不超过 `window_height` 设置的最大值

#### Scenario: 清空搜索词后窗口收缩
- **WHEN** 用户清空搜索输入（按 Escape 或手动删除）
- **THEN** 窗口高度收缩回搜索栏高度
- **AND** 结果面板被隐藏

#### Scenario: 结果数量决定窗口高度
- **WHEN** 搜索返回 N 条结果
- **THEN** 窗口高度 = 搜索栏高度 + min(N × item_height + padding, max_content_height) + 底部栏高度
- **AND** 超出最大高度时结果区域可滚动

### Requirement: 设置面板使用固定高度

当用户打开设置面板时，窗口 SHALL 切换为 `window_height` 定义的固定高度。当用户关闭设置面板时，窗口 SHALL 恢复为搜索栏高度。

#### Scenario: 打开设置面板
- **WHEN** 用户通过快捷键或操作打开设置面板
- **THEN** 窗口高度切换为 `window_height` 设置值
- **AND** 设置面板占据完整窗口空间

#### Scenario: 关闭设置面板
- **WHEN** 用户关闭设置面板
- **THEN** 窗口高度恢复为搜索栏高度
- **AND** 搜索框获得焦点
