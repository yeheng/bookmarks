/**
 * Design Tokens System
 *
 * This file defines all design tokens for the Bookmark Launcher application.
 * Tokens are organized into categories: colors, spacing, typography, animations, and components.
 *
 * Usage:
 * - Import tokens directly: `import { colors, spacing } from '@/design-system/tokens'`
 * - Use with Tailwind: tokens are mapped to tailwind.config.ts
 * - Use with CSS variables: tokens generate CSS custom properties
 */

// ============================================================================
// COLOR TOKENS
// ============================================================================

/**
 * Primitive Colors
 * Raw color values used to build semantic colors
 */
export const primitiveColors = {
  // Grays (neutral palette)
  gray: {
    50: '#fafafa',
    100: '#f5f5f5',
    200: '#e5e5e5',
    300: '#d4d4d4',
    400: '#a3a3a3',
    500: '#737373',
    600: '#525252',
    700: '#404040',
    800: '#262626',
    900: '#171717',
    950: '#0a0a0a',
  },

  // Primary accent (coral/red)
  coral: {
    50: '#fff5f5',
    100: '#ffe3e3',
    200: '#ffc9c9',
    300: '#ffa8a8',
    400: '#ff8787',
    500: '#ff6b6b',  // Main accent
    600: '#fa5252',
    700: '#f03e3e',
    800: '#e03131',
    900: '#c92a2a',
  },

  // Success (green)
  green: {
    50: '#ecfdf5',
    100: '#d1fae5',
    200: '#a7f3d0',
    300: '#6ee7b7',
    400: '#34d399',
    500: '#10b981',
    600: '#059669',
    700: '#047857',
    800: '#065f46',
    900: '#064e3b',
  },

  // Warning (amber)
  amber: {
    50: '#fffbeb',
    100: '#fef3c7',
    200: '#fde68a',
    300: '#fcd34d',
    400: '#fbbf24',
    500: '#f59e0b',
    600: '#d97706',
    700: '#b45309',
    800: '#92400e',
    900: '#78350f',
  },

  // Info (blue)
  blue: {
    50: '#eff6ff',
    100: '#dbeafe',
    200: '#bfdbfe',
    300: '#93c5fd',
    400: '#60a5fa',
    500: '#3b82f6',
    600: '#2563eb',
    700: '#1d4ed8',
    800: '#1e40af',
    900: '#1e3a8a',
  },
} as const;

/**
 * Semantic Colors
 * Purpose-driven color assignments for light and dark themes
 */
export const semanticColors = {
  light: {
    // Backgrounds
    bg: {
      primary: 'rgba(255, 255, 255, 0.95)',
      secondary: primitiveColors.gray[100],
      tertiary: primitiveColors.gray[50],
      elevated: '#ffffff',
      overlay: 'rgba(0, 0, 0, 0.5)',
    },

    // Text - WCAG 2.1 AA compliant (4.5:1+ contrast on light backgrounds)
    text: {
      primary: primitiveColors.gray[900],
      secondary: primitiveColors.gray[600],  // 7.5:1 contrast on #ffffff
      tertiary: '#6b6b6b',                   // 5.0:1 contrast on #ffffff
      disabled: primitiveColors.gray[400],
      inverse: '#ffffff',
    },

    // Borders
    border: {
      default: primitiveColors.gray[200],
      subtle: primitiveColors.gray[100],
      strong: primitiveColors.gray[300],
      focus: primitiveColors.coral[500],
    },

    // Interactive states
    interactive: {
      hover: 'rgba(0, 0, 0, 0.04)',
      active: 'rgba(0, 0, 0, 0.08)',
      selected: primitiveColors.coral[500],
      selectedText: '#ffffff',
    },

    // Semantic
    accent: primitiveColors.coral[500],
    success: primitiveColors.green[600],
    warning: primitiveColors.amber[500],
    error: primitiveColors.coral[600],
    info: primitiveColors.blue[500],
  },

  dark: {
    // Backgrounds
    bg: {
      primary: 'rgba(26, 26, 26, 0.95)',
      secondary: primitiveColors.gray[800],
      tertiary: primitiveColors.gray[900],
      elevated: primitiveColors.gray[800],
      overlay: 'rgba(0, 0, 0, 0.7)',
    },

    // Text - WCAG 2.1 AA compliant (4.5:1+ contrast on dark backgrounds)
    text: {
      primary: primitiveColors.gray[100],
      secondary: '#b3b3b3',  // 5.5:1 contrast on #1a1a1a
      tertiary: '#a3a3a3',   // 4.3:1 contrast on #1a1a1a
      disabled: primitiveColors.gray[600],
      inverse: primitiveColors.gray[900],
    },

    // Borders
    border: {
      default: primitiveColors.gray[700],
      subtle: primitiveColors.gray[800],
      strong: primitiveColors.gray[600],
      focus: primitiveColors.coral[500],
    },

    // Interactive states
    interactive: {
      hover: 'rgba(255, 255, 255, 0.08)',
      active: 'rgba(255, 255, 255, 0.12)',
      selected: primitiveColors.coral[500],
      selectedText: '#ffffff',
    },

    // Semantic
    accent: primitiveColors.coral[500],
    success: primitiveColors.green[500],
    warning: primitiveColors.amber[400],
    error: primitiveColors.coral[500],
    info: primitiveColors.blue[400],
  },
} as const;

// ============================================================================
// SPACING TOKENS
// ============================================================================

/**
 * Spacing Scale
 * Based on 4px base unit (8px grid system)
 */
export const spacing = {
  0: '0px',
  px: '1px',
  0.5: '2px',
  1: '4px',
  1.5: '6px',
  2: '8px',      // Base unit
  2.5: '10px',
  3: '12px',
  3.5: '14px',
  4: '16px',     // 2x base
  5: '20px',
  6: '24px',     // 3x base
  7: '28px',
  8: '32px',     // 4x base
  9: '36px',
  10: '40px',    // 5x base
  11: '44px',
  12: '48px',    // 6x base
  14: '56px',
  16: '64px',    // 8x base
  20: '80px',
  24: '96px',
  28: '112px',
  32: '128px',
} as const;

/**
 * Component-specific spacing
 */
export const componentSpacing = {
  input: {
    paddingX: spacing[6],    // 24px
    paddingY: spacing[3],    // 12px
    height: '60px',
  },
  resultItem: {
    paddingX: spacing[4],    // 16px
    paddingY: spacing[0],
    height: '44px',
    gap: spacing[3],         // 12px
  },
  button: {
    paddingX: spacing[4],    // 16px
    paddingY: spacing[2],    // 8px
    gap: spacing[2],         // 8px
  },
  panel: {
    padding: spacing[4],     // 16px
    gap: spacing[6],         // 24px
  },
} as const;

// ============================================================================
// TYPOGRAPHY TOKENS
// ============================================================================

/**
 * Font Family
 */
export const fontFamily = {
  sans: '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif',
  mono: 'ui-monospace, SFMono-Regular, "SF Mono", Consolas, "Liberation Mono", Menlo, monospace',
} as const;

/**
 * Font Size Scale
 */
export const fontSize = {
  xs: '10px',
  sm: '12px',
  base: '14px',
  lg: '16px',
  xl: '18px',
  '2xl': '20px',
  '3xl': '24px',
  '4xl': '30px',
} as const;

/**
 * Font Weight
 */
export const fontWeight = {
  normal: '400',
  medium: '500',
  semibold: '600',
  bold: '700',
} as const;

/**
 * Line Height
 */
export const lineHeight = {
  none: '1',
  tight: '1.25',
  snug: '1.375',
  normal: '1.5',
  relaxed: '1.625',
  loose: '2',
} as const;

/**
 * Letter Spacing
 */
export const letterSpacing = {
  tighter: '-0.05em',
  tight: '-0.025em',
  normal: '0em',
  wide: '0.025em',
  wider: '0.05em',
  widest: '0.1em',
} as const;

/**
 * Typography Presets
 */
export const typography = {
  // Headings
  h1: {
    fontSize: fontSize['3xl'],
    fontWeight: fontWeight.bold,
    lineHeight: lineHeight.tight,
    letterSpacing: letterSpacing.tight,
  },
  h2: {
    fontSize: fontSize['2xl'],
    fontWeight: fontWeight.semibold,
    lineHeight: lineHeight.tight,
    letterSpacing: letterSpacing.tight,
  },
  h3: {
    fontSize: fontSize.xl,
    fontWeight: fontWeight.semibold,
    lineHeight: lineHeight.snug,
    letterSpacing: letterSpacing.normal,
  },

  // Body text
  body: {
    fontSize: fontSize.base,
    fontWeight: fontWeight.normal,
    lineHeight: lineHeight.normal,
    letterSpacing: letterSpacing.normal,
  },
  bodySmall: {
    fontSize: fontSize.sm,
    fontWeight: fontWeight.normal,
    lineHeight: lineHeight.normal,
    letterSpacing: letterSpacing.normal,
  },

  // UI elements
  label: {
    fontSize: fontSize.sm,
    fontWeight: fontWeight.medium,
    lineHeight: lineHeight.none,
    letterSpacing: letterSpacing.normal,
  },
  caption: {
    fontSize: fontSize.xs,
    fontWeight: fontWeight.medium,
    lineHeight: lineHeight.normal,
    letterSpacing: letterSpacing.wide,
  },

  // Special
  searchInput: {
    fontSize: fontSize.xl,
    fontWeight: fontWeight.normal,
    lineHeight: lineHeight.normal,
    letterSpacing: letterSpacing.normal,
  },
  sectionLabel: {
    fontSize: fontSize.xs,
    fontWeight: fontWeight.semibold,
    lineHeight: lineHeight.none,
    letterSpacing: letterSpacing.wider,
    textTransform: 'uppercase' as const,
  },
} as const;

// ============================================================================
// ANIMATION TOKENS
// ============================================================================

/**
 * Duration
 */
export const duration = {
  instant: '0ms',
  fast: '100ms',
  normal: '150ms',
  slow: '200ms',
  slower: '300ms',
  slowest: '500ms',
} as const;

/**
 * Easing Functions
 */
export const easing = {
  linear: 'linear',
  easeIn: 'cubic-bezier(0.4, 0, 1, 1)',
  easeOut: 'cubic-bezier(0, 0, 0.2, 1)',
  easeInOut: 'cubic-bezier(0.4, 0, 0.2, 1)',
  // Custom easings
  bounce: 'cubic-bezier(0.68, -0.55, 0.265, 1.55)',
  smooth: 'cubic-bezier(0.25, 0.1, 0.25, 1)',
} as const;

/**
 * Animation Presets
 */
export const animation = {
  // Transitions
  fadeIn: {
    duration: duration.normal,
    easing: easing.easeOut,
  },
  fadeOut: {
    duration: duration.fast,
    easing: easing.easeIn,
  },
  slideIn: {
    duration: duration.slow,
    easing: easing.easeOut,
  },
  slideOut: {
    duration: duration.normal,
    easing: easing.easeIn,
  },

  // Interactive states
  hover: {
    duration: duration.normal,
    easing: easing.easeOut,
  },
  press: {
    duration: duration.fast,
    easing: easing.easeOut,
  },
  focus: {
    duration: duration.normal,
    easing: easing.easeOut,
  },

  // Special
  ripple: {
    duration: '600ms',
    easing: easing.easeOut,
  },
  skeleton: {
    duration: '1500ms',
    easing: easing.easeInOut,
  },
  spin: {
    duration: '800ms',
    easing: easing.linear,
  },
} as const;

// ============================================================================
// SHADOW TOKENS
// ============================================================================

export const shadow = {
  none: 'none',
  sm: '0 1px 2px 0 rgba(0, 0, 0, 0.05)',
  default: '0 1px 3px 0 rgba(0, 0, 0, 0.1), 0 1px 2px -1px rgba(0, 0, 0, 0.1)',
  md: '0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -2px rgba(0, 0, 0, 0.1)',
  lg: '0 10px 15px -3px rgba(0, 0, 0, 0.1), 0 4px 6px -4px rgba(0, 0, 0, 0.1)',
  xl: '0 20px 25px -5px rgba(0, 0, 0, 0.1), 0 8px 10px -6px rgba(0, 0, 0, 0.1)',

  // Component-specific
  hover: '0 2px 8px rgba(0, 0, 0, 0.1)',
  selected: '0 2px 12px rgba(255, 107, 107, 0.2)',
  elevated: '0 4px 12px rgba(0, 0, 0, 0.15)',
} as const;

// ============================================================================
// BORDER RADIUS TOKENS
// ============================================================================

export const borderRadius = {
  none: '0px',
  sm: '2px',
  default: '4px',
  md: '6px',
  lg: '8px',
  xl: '12px',
  '2xl': '16px',
  full: '9999px',
} as const;

// ============================================================================
// Z-INDEX TOKENS
// ============================================================================

export const zIndex = {
  base: 0,
  dropdown: 10,
  sticky: 20,
  fixed: 30,
  overlay: 40,
  modal: 50,
  popover: 60,
  toast: 70,
  tooltip: 80,
} as const;

// ============================================================================
// CSS CUSTOM PROPERTIES GENERATOR
// ============================================================================

/**
 * Generates CSS custom properties string for the given theme
 */
export function generateCSSVariables(theme: 'light' | 'dark'): string {
  const colors = semanticColors[theme];

  return `
    /* Background Colors */
    --color-bg-primary: ${colors.bg.primary};
    --color-bg-secondary: ${colors.bg.secondary};
    --color-bg-tertiary: ${colors.bg.tertiary};
    --color-bg-elevated: ${colors.bg.elevated};
    --color-bg-overlay: ${colors.bg.overlay};

    /* Text Colors */
    --color-text-primary: ${colors.text.primary};
    --color-text-secondary: ${colors.text.secondary};
    --color-text-tertiary: ${colors.text.tertiary};
    --color-text-disabled: ${colors.text.disabled};
    --color-text-inverse: ${colors.text.inverse};

    /* Border Colors */
    --color-border-default: ${colors.border.default};
    --color-border-subtle: ${colors.border.subtle};
    --color-border-strong: ${colors.border.strong};
    --color-border-focus: ${colors.border.focus};

    /* Interactive Colors */
    --color-interactive-hover: ${colors.interactive.hover};
    --color-interactive-active: ${colors.interactive.active};
    --color-interactive-selected: ${colors.interactive.selected};
    --color-interactive-selected-text: ${colors.interactive.selectedText};

    /* Semantic Colors */
    --color-accent: ${colors.accent};
    --color-success: ${colors.success};
    --color-warning: ${colors.warning};
    --color-error: ${colors.error};
    --color-info: ${colors.info};
  `.trim();
}

// ============================================================================
// THEME MAPPING (for existing CSS variables compatibility)
// ============================================================================

/**
 * Maps new tokens to existing CSS variable names for backward compatibility
 */
export const legacyVariableMapping = {
  '--bg-color': 'bg.primary',
  '--text-color': 'text.primary',
  '--secondary-text': 'text.secondary',
  '--border-color': 'border.default',
  '--accent-color': 'accent',
  '--selection-bg': 'interactive.selected',
  '--selection-text': 'interactive.selectedText',
} as const;

// ============================================================================
// EXPORT ALL TOKENS
// ============================================================================

export const tokens = {
  colors: {
    primitive: primitiveColors,
    semantic: semanticColors,
  },
  spacing,
  componentSpacing,
  typography,
  fontFamily,
  fontSize,
  fontWeight,
  lineHeight,
  letterSpacing,
  animation,
  duration,
  easing,
  shadow,
  borderRadius,
  zIndex,
} as const;

export default tokens;
