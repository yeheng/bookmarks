## ADDED Requirements

### Requirement: Plugin Result Rendering
Launcher 前端 MUST 将插件返回的 JSON 结果渲染为与原生搜索结果一致的 UI。

插件结果项映射到现有 `SearchResult` 类型:
```typescript
interface PluginResultItem {
  uid: string;                    // 唯一标识
  title: string;                  // 主标题
  subtitle?: string;              // 副标题
  arg?: string;                   // 动作参数
  icon?: {
    path?: string;                // 相对于插件 assets 目录
    url?: string;                 // 远程图标 URL
    emoji?: string;               // Emoji 图标
  };
  actions?: PluginAction[];       // 可执行动作列表
  badge?: string;                 // 右侧徽标文本
}

type PluginAction =
  | { type: "open-url"; url: string; title?: string }
  | { type: "copy"; text: string; title?: string }
  | { type: "paste"; text: string; title?: string }
  | { type: "open-file"; path: string; title?: string }
  | { type: "run-command"; command: string; arg?: string; title?: string };
```

#### Scenario: Plugin results displayed in search list
- **WHEN** a plugin command returns items
- **THEN** the frontend SHALL render each item using the existing `SearchResultItem` component
- **THEN** the item SHALL display title, subtitle, and icon
- **THEN** the source plugin name SHALL be shown as a group header or badge

#### Scenario: Plugin result action executed
- **WHEN** user selects a plugin result and presses Enter
- **THEN** the system SHALL execute the first action (or default action) from the item's actions list
- **THEN** for `open-url`, the system SHALL open the URL in default browser
- **THEN** for `copy`, the system SHALL copy text to clipboard
- **THEN** for `open-file`, the system SHALL open the file with system default

#### Scenario: Plugin result with modifier key actions
- **WHEN** user holds a modifier key (Cmd/Ctrl) and selects a plugin result
- **THEN** the system SHALL execute the corresponding secondary action if defined

### Requirement: Plugin Loading State
前端 MUST 在插件执行期间显示加载状态。

#### Scenario: Loading indicator shown during execution
- **WHEN** a plugin command is triggered
- **THEN** the frontend SHALL display a loading spinner or skeleton UI
- **THEN** the loading state SHALL persist until results are received or timeout occurs

#### Scenario: Error state displayed on failure
- **WHEN** a plugin command fails (timeout, crash, invalid output)
- **THEN** the frontend SHALL display an error message in the result area
- **THEN** the error message SHALL include the plugin name and a "View Logs" link

### Requirement: Plugin Result Caching
系统 MUST 支持插件声明的结果缓存策略。

#### Scenario: Cached results returned
- **WHEN** a plugin command is triggered with the same query within cache TTL
- **THEN** the system SHALL return cached results immediately
- **THEN** the system SHALL NOT spawn a new subprocess

#### Scenario: Cache expired
- **WHEN** cached results exceed the declared TTL
- **THEN** the system SHALL spawn a new subprocess for fresh results
- **THEN** the system SHALL replace the cached results

### Requirement: Plugin Management UI
前端 MUST 提供插件管理界面，集成到现有设置面板中。

#### Scenario: View installed plugins
- **WHEN** user navigates to Settings → Plugins
- **THEN** the system SHALL display a list of installed plugins with name, description, version, status
- **THEN** each plugin SHALL have enable/disable toggle and uninstall button

#### Scenario: Configure plugin preferences
- **WHEN** user clicks "Configure" on a plugin
- **THEN** the system SHALL auto-generate a form based on the plugin's `[[preferences]]` declarations
- **THEN** password fields SHALL be masked
- **THEN** changes SHALL be saved immediately
