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
  // Grays (warm neutral palette - "Gentle Slate")
  gray: {
    50: '#f8f7f6',   // Warm off-white
    100: '#f0efed',  // Soft cream
    200: '#e3e1df',  // Warmer mid-tone
    300: '#d1cfcc',  // Gentle stone
    400: '#a8a49f',  // Muted mid-gray
    500: '#7a766f',  // Balanced gray
    600: '#5c5852',  // Warm dark gray
    700: '#454139',  // Deep charcoal
    800: '#2d2a26',  // Warm dark
    900: '#3d3a36',  // Warm dark text
    950: '#1c1a17',  // Deep warm black
  },

  // Primary accent (sage green - calming)
  coral: {
    50: '#f4f7f5',   // Lightest sage
    100: '#e5ebe8',  // Very light sage
    200: '#c8d7cf',  // Light sage
    300: '#a5c0b0',  // Soft sage
    400: '#81a594',  // Medium sage
    500: '#6b9080',  // Main accent - muted sage green
    600: '#557a68',  // Darker sage for hover
    700: '#436354',  // Deep sage
    800: '#354f43',  // Very deep sage
    900: '#2a3f35',  // Darkest sage
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
    // Backgrounds - warm cream tones for reduced eye strain
    bg: {
      primary: 'rgba(248, 247, 246, 0.96)',  // Warm off-white
      secondary: primitiveColors.gray[100],
      tertiary: primitiveColors.gray[50],
      elevated: '#fdfcfb',  // Subtle cream
      overlay: 'rgba(45, 42, 38, 0.4)',  // Warm overlay
    },

    // Text - softer contrast for comfortable extended reading
    text: {
      primary: '#3d3a36',                    // Warm dark gray (not near-black)
      secondary: '#6b6660',                  // Soft secondary
      tertiary: '#8a857e',                   // Gentle tertiary
      disabled: primitiveColors.gray[400],
      inverse: '#fdfcfb',
    },

    // Borders - subtle warm tones
    border: {
      default: primitiveColors.gray[200],
      subtle: primitiveColors.gray[100],
      strong: primitiveColors.gray[300],
      focus: primitiveColors.coral[500],  // Sage green focus
    },

    // Interactive states - gentler feedback
    interactive: {
      hover: 'rgba(107, 144, 128, 0.08)',   // Sage-tinted hover
      active: 'rgba(107, 144, 128, 0.15)',  // Sage-tinted active
      selected: primitiveColors.coral[500],
      selectedText: '#ffffff',
    },

    highlight: {
      bg: 'rgba(107, 144, 128, 0.08)',      // Sage-tinted highlight
      border: 'rgba(107, 144, 128, 0.15)',
    },

    // Semantic - muted tones
    accent: primitiveColors.coral[500],
    success: primitiveColors.green[600],
    warning: primitiveColors.amber[500],
    error: '#b85450',  // Muted red for errors
    info: primitiveColors.blue[500],
  },

  dark: {
    // Backgrounds - warm charcoal for reduced eye strain in dark mode
    bg: {
      primary: 'rgba(32, 30, 28, 0.96)',     // Warm charcoal
      secondary: primitiveColors.gray[800],
      tertiary: '#252320',                    // Slightly lighter warm dark
      elevated: '#363330',                    // Elevated warm surface
      overlay: 'rgba(0, 0, 0, 0.6)',
    },

    // Text - warm off-whites for comfortable reading
    text: {
      primary: '#e8e6e3',                     // Warm off-white
      secondary: '#a8a49f',                   // Muted warm secondary
      tertiary: '#8a857e',                    // Consistent tertiary
      disabled: primitiveColors.gray[600],
      inverse: primitiveColors.gray[900],
    },

    // Borders - subtle warm tones
    border: {
      default: '#454139',                     // Warm border
      subtle: '#363330',                      // Subtle warm
      strong: '#5c5852',                      // Stronger warm
      focus: primitiveColors.coral[400],      // Lighter sage for dark mode
    },

    // Interactive states - sage-tinted feedback
    interactive: {
      hover: 'rgba(129, 165, 148, 0.12)',    // Sage-tinted hover
      active: 'rgba(129, 165, 148, 0.18)',   // Sage-tinted active
      selected: primitiveColors.coral[500],
      selectedText: '#ffffff',
    },

    highlight: {
      bg: 'rgba(129, 165, 148, 0.10)',       // Sage-tinted highlight
      border: 'rgba(129, 165, 148, 0.18)',
    },

    // Semantic - muted tones for dark mode
    accent: primitiveColors.coral[400],       // Lighter sage in dark mode
    success: primitiveColors.green[500],
    warning: primitiveColors.amber[400],
    error: '#c97370',                         // Softer red for dark mode
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
 * Typography Presets
 * Only searchInput is actively used via tailwind.config.js
 */
export const typography = {
  searchInput: {
    fontSize: fontSize.xl,
    fontWeight: fontWeight.normal,
    lineHeight: lineHeight.normal,
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
  selected: '0 2px 12px rgba(107, 144, 128, 0.2)',
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
// EXPORT ALL TOKENS
// ============================================================================

export const tokens = {
  colors: {
    primitive: primitiveColors,
    semantic: semanticColors,
  },
  spacing,
  typography,
  fontFamily,
  fontSize,
  fontWeight,
  lineHeight,
  duration,
  easing,
  shadow,
  borderRadius,
  zIndex,
} as const;

export default tokens;
