# Design Document: UI/UX Redesign - Raycast Style

## Context

The Bookmarks Launcher application needs a comprehensive UI/UX redesign to improve visual refinement, interaction clarity, and overall user experience. The current design, while functional, has identified pain points in window behavior, visual details, interaction feedback, and layout hierarchy.

**Target Users:** Personal daily users + open-source project showcase

**Design Direction:** macOS native style with Raycast-inspired modern refinements

## Goals / Non-Goals

**Goals:**
- Create a refined, polished visual system with glass-morphism effects
- Implement smooth, natural window animations and transitions
- Establish clear interaction feedback and visual hierarchy
- Improve accessibility and keyboard navigation experience
- Maintain high performance (<100ms search response)

**Non-Goals:**
- Complete architectural rewrite of the application
- Adding new features beyond visual/interaction improvements
- Changing the core search functionality

## Design System

### Visual System - "Refined Glass" Design Language

#### Color Palette

**Light Mode:**
```
Background:      #FFFFFF with blur: 40px, opacity: 0.92
Secondary BG:    #F5F5F7 (Apple Gray)
Primary Text:    #1D1D1F (Deep Ink Gray)
Secondary Text:  #86868B
Accent Color:    #007AFF (System Blue) or user-customizable
```

**Dark Mode:**
```
Background:      rgba(28,28,30,0.85) with blur: 50px
Secondary BG:    #2C2C2E
Primary Text:    #F5F5F7
Secondary Text:  #98989D
Accent Color:    #0A84FF (Dark Mode System Blue)
```

#### Border Radius

```
Window:          14px (larger, softer)
Result Items:    8px
Icon Containers: 10px
Small Elements:  6px
```

#### Shadow System

```
Window Shadow:   0 25px 50px -12px rgba(0,0,0,0.25)
Hover State:     0 4px 12px rgba(0,0,0,0.08)
Selected State:  0 2px 8px rgba(0,122,255,0.15)
```

#### Typography

```
Search Input:    SF Pro / system-ui, 22px, weight 400
Title:           14px, weight 500
Subtitle:        12px, weight 400
Auxiliary:       11px, weight 400, mono (file size, time, etc.)
```

### Layout System

#### Window Dimensions

```
Width:           680px (Raycast standard)
Max Height:      520px (dynamic adjustment)
Min Height:      64px (search bar only)

Position:
- Horizontal:    Screen center
- Vertical:      38% of screen height (slightly above center)
- Multi-monitor: Follow current focus monitor
```

#### Internal Layout Structure

```
+-------------------------------------+
|  [Search Icon] Search input (56px)  |  <- Search bar area
+-------------------------------------+
|  > Bookmarks (3)              [v]  |  <- Group header (collapsible)
|  +---------------------------------+|
|  | Icon  Result Item 1            ||
|  |       subtitle . metadata      ||  <- Result item (48px each)
|  +---------------------------------+|
|  | Icon  Result Item 2            ||
|  |       subtitle . metadata      ||
|  +---------------------------------+|
+-------------------------------------+
|  5 results                    [settings] |  <- Bottom bar (32px)
+-------------------------------------+
```

#### Spacing System

```
Padding:
- Search bar:        16px horizontal
- Results list:      12px horizontal, 8px vertical
- Result items:      12px horizontal, 10px vertical

Gaps:
- Search bar to results: 1px divider
- Between result items:   2px
- Group header to items:  8px
```

#### Information Density

```
Item height:     48px (more spacious than current 40px)
Max items shown: 8 (more comfortable than current 10)
Icon size:       24px (clearer than current 18px)
```

### Interaction Design

#### Keyboard Navigation

```
Core Shortcuts:
- Up/Down or Ctrl+P/N: Navigate between results
- Enter:               Open selected item
- Tab:                 Expand group / switch focus
- Shift+Tab:           Collapse group
- Esc:                 Close window (first press clears search, second closes)
- Cmd+,:               Open settings

Advanced Navigation:
- Cmd+1~9:             Direct select Nth result
- Cmd+Enter:           Open in background (bookmarks)
- Opt+Enter:           Copy link/path
```

#### Selection State Feedback

```
Visual Feedback:
- Background:   Accent color gradient fill
- Text:         Primary text turns white, secondary semi-transparent white
- Icon:         Slight scale up (1.05)
- Border radius: Maintained at 8px
- Transition:   150ms ease-out

Selection Animation:
- Enter:        Scale pulse (1.0 -> 1.01 -> 1.0)
- Leave:        Fade (opacity 1 -> 0.8)
- Duration:     100ms
```

#### Hover State

```
Unselected Item Hover:
- Background:   rgba(0,0,0,0.04) light / rgba(255,255,255,0.06) dark
- Transition:   80ms ease-out
- No shadow (keep minimal)

Cursor Changes:
- Default:      text (in input)
- Result item:  pointer
- Collapsible:  pointer
```

#### Input Feedback

```
Search Icon Animation:
- Typing:       Slight pulse (opacity fluctuation)
- Loading:      Rotation animation (360deg loop)
- Idle:         Static

Clear Button:
- Show X button when content exists (right side)
- Hide when empty
- After click:  Clear input + focus input
```

### Animation System

#### Window Entrance Animation

```
Duration: 150ms
Properties:
- Opacity:      0 -> 1
- Scale:        0.96 -> 1.0
- Y-translate:  -12px -> 0
- Easing:       cubic-bezier(0.16, 1, 0.3, 1) (spring feel)
- Background blur: Sync fade-in

Flow: Hotkey -> Position window -> Play animation -> Focus input
```

#### Window Exit Animation

```
Duration: 120ms
Properties:
- Opacity:      1 -> 0
- Scale:        1.0 -> 0.98
- Y-translate:  0 -> -8px
- Easing:       cubic-bezier(0.4, 0, 1, 1) (quick settle)
- Trigger:      After selection or Esc key
```

#### Height Dynamic Adjustment

```
Duration: 180ms
Easing:   cubic-bezier(0.25, 0.1, 0.25, 1)
Triggers:
- Results appear after input
- Result count changes
- Group collapse/expand toggle
Debounce: 100ms (avoid frequent adjustments)
```

#### Result List Animation

```
Item Entrance (staggered):
- Per-item delay: 30ms (offset entrance)
- Single item (100ms):
  - Opacity:     0 -> 1
  - X-translate: -8px -> 0
  - Easing:      ease-out
- Max total:     300ms (8 items x 30ms + 100ms)

Scroll Behavior:
- Scrollbar:    4px width, semi-transparent
- Hover:        Darken
- Smooth scroll: Enabled
```

#### Group Collapse Animation

```
Duration: 200ms
Properties:
- Height:       0 <-> auto
- Opacity:      0 <-> 1
- Easing:       ease-in-out
- Arrow rotate: 0deg <-> 90deg
```

#### Micro-interactions

```
Click Feedback:
- Result select:    Scale 1.0 -> 0.98 -> 1.0 (80ms)
- Button click:     Scale 1.0 -> 0.95 (60ms)
- Settings hover:   Rotate 0deg -> 30deg

Focus Indication:
- Input focus:      No extra border (minimal)
- Keyboard focus:   Selection state visual feedback
```

#### Reduced Motion Mode

```
prefers-reduced-motion: reduce
- All animation durations -> 0ms
- Transitions -> Instant switch
- Keep only necessary state changes
```

## Component Architecture

### File Structure

```
src/components/
+-- SearchCombobox.vue (refactor)
|   +-- Search input area
|   +-- Results list container
|   +-- Bottom status bar
|
+-- SearchResultItem.vue (refactor)
|   +-- Icon container
|   +-- Content area (title + subtitle)
|   +-- Action hints
|
+-- ResultGroupHeader.vue (optimize)
|   +-- Group title + collapse button
|
+-- SkeletonLoader.vue (optimize)
|   +-- Loading skeleton
|
+-- new/
    +-- SearchIcon.vue (new)
    |   +-- Animated search icon
    +-- ClearButton.vue (new)
        +-- Clear input button
```

### Design Tokens Update

```
src/design-system/tokens.ts
+-- Update primitiveColors (new palette)
+-- Update semanticColors (new semantic colors)
+-- Update borderRadius (larger radius)
+-- Add shadowTokens (shadow system)
+-- Add animationTokens (animation parameters)
```

### CSS Variables Mapping

```css
:root {
  /* Window */
  --window-bg: ...;
  --window-blur: 50px;
  --window-radius: 14px;

  /* Animation */
  --ease-spring: cubic-bezier(0.16, 1, 0.3, 1);
  --ease-out: cubic-bezier(0, 0, 0.2, 1);
  --duration-fast: 80ms;
  --duration-normal: 150ms;
  --duration-slow: 200ms;

  /* Shadow */
  --shadow-window: ...;
  --shadow-elevated: ...;
}
```

## Implementation Phases

### Phase 1: Visual Foundation (High Priority)
- Update design tokens
- Refactor color system
- Update CSS variables

### Phase 2: Layout Optimization (High Priority)
- Adjust window dimensions and spacing
- Optimize result item height
- Update group styles

### Phase 3: Animation System (Medium Priority)
- Implement window entrance/exit animations
- Implement result item staggered entrance
- Implement selection state animations

### Phase 4: Interaction Polish (Medium Priority)
- Optimize keyboard navigation feedback
- Add clear button
- Perfect status indicators

## Testing Strategy

### Visual Testing
- Light/dark mode switching
- Layout with different result counts
- Animation behavior under reduce-motion

### Interaction Testing
- Keyboard navigation completeness
- Animation performance (60fps)
- Window positioning accuracy

## Risks / Trade-offs

### Risk: Animation performance on older hardware
**Mitigation:**
- Use CSS transforms and opacity (GPU accelerated)
- Implement reduce-motion support
- Profile on lower-end devices

### Risk: Cross-platform consistency
**Mitigation:**
- Test on macOS, Windows, Linux
- Use platform-agnostic CSS where possible
- Document platform-specific behaviors

### Risk: Breaking existing functionality
**Mitigation:**
- Incremental rollout by phase
- Maintain backward compatibility for settings
- Comprehensive testing before each phase merge

## Success Criteria

1. **Visual Refinement**: Design feels polished and professional
2. **Interaction Clarity**: Users understand navigation and feedback intuitively
3. **Performance**: Animations run at 60fps, search remains <100ms
4. **Accessibility**: Keyboard navigation works flawlessly
5. **Consistency**: Light/dark modes both feel native and cohesive
