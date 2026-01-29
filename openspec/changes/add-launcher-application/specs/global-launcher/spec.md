## ADDED Requirements

### Requirement: Global Hotkey Registration

The application SHALL register a configurable global hotkey that summons the launcher window from any application.

#### Scenario: Default hotkey activation
- **WHEN** user presses Cmd+Space (macOS) or Ctrl+Space (Windows/Linux)
- **THEN** the launcher window appears centered on the screen
- **AND** the search input receives focus

#### Scenario: Custom hotkey configuration
- **WHEN** user configures a custom hotkey in settings
- **THEN** the application registers the new hotkey
- **AND** unregisters the previous hotkey
- **AND** the new hotkey summons the launcher

#### Scenario: Hotkey conflict detection
- **WHEN** the configured hotkey is already registered by another application
- **THEN** the application SHALL display an error notification
- **AND** SHALL prompt the user to choose a different hotkey

### Requirement: Window Visibility Management

The application SHALL manage launcher window visibility with smooth transitions and auto-hide behavior.

#### Scenario: Show window on hotkey
- **WHEN** global hotkey is triggered and window is hidden
- **THEN** the window SHALL fade in (150ms transition)
- **AND** position itself at screen center
- **AND** set itself as always-on-top
- **AND** focus the search input

#### Scenario: Hide window on Escape key
- **WHEN** user presses Escape key while launcher is visible
- **THEN** the window SHALL fade out (150ms transition)
- **AND** clear the search input
- **AND** reset search results

#### Scenario: Hide window on focus loss
- **WHEN** user clicks outside the launcher window
- **THEN** the window SHALL fade out (150ms transition)
- **AND** clear the search input
- **AND** reset search results

#### Scenario: Hide window on result selection
- **WHEN** user selects a search result (Enter key or click)
- **THEN** the window SHALL hide immediately
- **AND** clear the search input
- **AND** open the selected resource

### Requirement: Frameless Window Design

The application SHALL display a frameless, transparent window with modern visual styling.

#### Scenario: Window appearance
- **WHEN** launcher window is visible
- **THEN** window SHALL be frameless (no title bar)
- **AND** SHALL have transparent background with blur effect (macOS/Windows)
- **AND** SHALL have rounded corners (12px radius)
- **AND** SHALL display a shadow for visual depth
- **AND** SHALL be 680px wide and up to 480px tall

#### Scenario: Window positioning
- **WHEN** launcher window appears
- **THEN** it SHALL be centered horizontally on the active screen
- **AND** positioned in the upper third vertically (similar to Spotlight)
- **AND** SHALL remain always-on-top while visible

### Requirement: Cross-Platform Support

The application SHALL run on macOS, Windows, and Linux with consistent functionality.

#### Scenario: macOS compatibility
- **WHEN** application runs on macOS 10.15+
- **THEN** all features SHALL work including global hotkey, window management, and blur effects

#### Scenario: Windows compatibility
- **WHEN** application runs on Windows 10/11
- **THEN** all features SHALL work including global hotkey, window management, and blur effects

#### Scenario: Linux compatibility
- **WHEN** application runs on Linux with X11 or Wayland
- **THEN** all features SHALL work including global hotkey and window management
- **AND** blur effects MAY be degraded based on compositor support
