## ADDED Requirements

### Requirement: Minimalist Search Interface

The application SHALL provide a minimalist search interface using HeadlessUI/Vue components.

#### Scenario: Search input design
- **WHEN** launcher window is visible
- **THEN** it SHALL display a prominent search input field
- **AND** input SHALL have placeholder text "Search bookmarks and files..."
- **AND** input SHALL auto-focus on window show
- **AND** input SHALL clear on Escape key
- **AND** input SHALL accept text immediately without lag

#### Scenario: Real-time search feedback
- **WHEN** user types in search input
- **THEN** search results SHALL update within 100ms
- **AND** display loading indicator if search takes > 50ms
- **AND** debounce input (50ms) to avoid excessive searches

#### Scenario: Keyboard navigation
- **WHEN** user presses arrow keys (Up/Down)
- **THEN** the application SHALL highlight next/previous result
- **AND** scroll result into view if needed
- **AND** wrap around at list boundaries

#### Scenario: Result selection
- **WHEN** user presses Enter or clicks a result
- **THEN** the application SHALL open the selected resource
- **AND** hide the launcher window
- **AND** record the access in usage history

### Requirement: Results List Display

The application SHALL display search results in a visually clear, scannable list.

#### Scenario: Result item layout
- **WHEN** search results are displayed
- **THEN** each result SHALL show:
  - Icon (favicon for bookmarks, file type icon for files)
  - Title (bookmark title or filename)
  - Subtitle (URL for bookmarks, file path for files)
  - Type badge (optional, e.g., "Bookmark", "PDF", "Folder")
- **AND** use consistent spacing and alignment
- **AND** highlight matched text in title and subtitle

#### Scenario: Empty state
- **WHEN** search query returns no results
- **THEN** the application SHALL display "No results found"
- **AND** suggest checking spelling or trying different keywords

#### Scenario: Recent items (empty query)
- **WHEN** search input is empty
- **THEN** the application SHALL display recent items (last 10 accessed)
- **AND** label the section as "Recent"

#### Scenario: Result limit display
- **WHEN** more than 10 results match query
- **THEN** the application SHALL show first 10 results
- **AND** display "Press ↓ to see more" at bottom
- **AND** load additional results on scroll

### Requirement: Visual Theme and Styling

The application SHALL implement a minimalist visual theme inspired by Raycast/Alfred/Spotlight.

#### Scenario: Dark theme (default)
- **WHEN** application uses dark theme
- **THEN** it SHALL use color palette:
  - Background: `#1a1a1a`
  - Input background: `#2a2a2a`
  - Text primary: `#e0e0e0`
  - Text secondary: `#9a9a9a`
  - Accent (selection): `#ff6b6b`
  - Border: `#3a3a3a`

#### Scenario: Light theme
- **WHEN** user switches to light theme in settings
- **THEN** it SHALL use color palette:
  - Background: `#ffffff`
  - Input background: `#f5f5f5`
  - Text primary: `#1a1a1a`
  - Text secondary: `#6a6a6a`
  - Accent (selection): `#ff6b6b`
  - Border: `#e0e0e0`

#### Scenario: Typography
- **WHEN** rendering text in UI
- **THEN** it SHALL use:
  - Font family: System default (`-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif`)
  - Input font size: `18px`
  - Result title font size: `14px`
  - Result subtitle font size: `12px`

#### Scenario: Spacing and layout
- **WHEN** rendering UI elements
- **THEN** it SHALL use:
  - Window padding: `16px`
  - Input padding: `12px 16px`
  - Result item padding: `12px 16px`
  - Result item spacing: `4px`
  - Border radius: `8px` for input, `6px` for results

### Requirement: Smooth Animations and Transitions

The application SHALL use subtle animations for polished user experience.

#### Scenario: Window fade transition
- **WHEN** launcher window shows or hides
- **THEN** it SHALL fade in/out with 150ms duration
- **AND** use ease-out timing function

#### Scenario: Result highlight transition
- **WHEN** user navigates between results with keyboard
- **THEN** highlight SHALL move smoothly with 100ms duration
- **AND** use ease-in-out timing function

#### Scenario: Loading state animation
- **WHEN** search is in progress (> 50ms)
- **THEN** display subtle spinner or loading indicator
- **AND** fade in after 50ms delay to avoid flicker

### Requirement: Accessibility Support

The application SHALL be accessible via keyboard and follow WAI-ARIA guidelines.

#### Scenario: Keyboard-only navigation
- **WHEN** user interacts with launcher
- **THEN** all functionality SHALL be accessible via keyboard
- **AND** Tab/Shift+Tab SHALL navigate between input and settings
- **AND** Arrow keys SHALL navigate results
- **AND** Enter SHALL select highlighted result
- **AND** Escape SHALL close launcher

#### Scenario: Screen reader support
- **WHEN** screen reader is active
- **THEN** the application SHALL announce:
  - Number of search results found
  - Currently selected result
  - Result type (bookmark or file)
- **AND** use proper ARIA labels and roles

#### Scenario: Focus management
- **WHEN** launcher window appears
- **THEN** focus SHALL move to search input
- **AND** focus SHALL remain within launcher (focus trap)
- **AND** focus SHALL restore to previous application on hide

### Requirement: Responsive Layout

The application SHALL adapt layout to different screen sizes and resolutions.

#### Scenario: Standard display
- **WHEN** launcher displays on standard resolution (1920x1080)
- **THEN** window SHALL be 680px wide × up to 480px tall
- **AND** show up to 8 results without scrolling

#### Scenario: High DPI display
- **WHEN** launcher displays on high DPI screen (Retina, 4K)
- **THEN** all graphics and text SHALL render sharply
- **AND** use appropriate scaling factors

#### Scenario: Small screen
- **WHEN** launcher displays on small screen (< 1366px wide)
- **THEN** window width SHALL scale to 80% of screen width
- **AND** maintain minimum width of 500px
