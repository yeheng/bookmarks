import { tokens } from './src/design-system/tokens'

/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{vue,js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      // Color tokens - Semantic colors for light and dark themes
      colors: {
        // Primitive colors
        gray: tokens.colors.primitive.gray,
        coral: tokens.colors.primitive.coral,
        green: tokens.colors.primitive.green,
        amber: tokens.colors.primitive.amber,
        blue: tokens.colors.primitive.blue,

        // Semantic colors - Light theme
        'bg-primary': tokens.colors.semantic.light.bg.primary,
        'bg-secondary': tokens.colors.semantic.light.bg.secondary,
        'bg-elevated': tokens.colors.semantic.light.bg.elevated,
        'bg-overlay': tokens.colors.semantic.light.bg.overlay,

        'text-primary': tokens.colors.semantic.light.text.primary,
        'text-secondary': tokens.colors.semantic.light.text.secondary,
        'text-tertiary': tokens.colors.semantic.light.text.tertiary,
        'text-disabled': tokens.colors.semantic.light.text.disabled,

        'border-default': tokens.colors.semantic.light.border.default,
        'border-subtle': tokens.colors.semantic.light.border.subtle,
        'border-focus': tokens.colors.semantic.light.border.focus,

        'accent': tokens.colors.semantic.light.accent,
        'success': tokens.colors.semantic.light.success,
        'warning': tokens.colors.semantic.light.warning,
        'error': tokens.colors.semantic.light.error,
        'info': tokens.colors.semantic.light.info,

         'hover-bg': tokens.colors.semantic.light.interactive.hover,
         'active-bg': tokens.colors.semantic.light.interactive.active,
         'selected': tokens.colors.semantic.light.interactive.selected,
         'selected-text': tokens.colors.semantic.light.interactive.selectedText,

         // Ghost highlight
         'highlight-bg': tokens.colors.semantic.light.highlight.bg,
         'highlight-border': tokens.colors.semantic.light.highlight.border,

         // Legacy compatibility (maps to new tokens)
         dark: {
           bg: tokens.colors.semantic.dark.bg.primary,
           'input-bg': tokens.colors.semantic.dark.bg.secondary,
           'text-primary': tokens.colors.semantic.dark.text.primary,
           'text-secondary': tokens.colors.semantic.dark.text.secondary,
           accent: tokens.colors.semantic.dark.accent,
           border: tokens.colors.semantic.dark.border.default,
           'highlight-bg': tokens.colors.semantic.dark.highlight.bg,
           'highlight-border': tokens.colors.semantic.dark.highlight.border,
         },
         light: {
           bg: tokens.colors.semantic.light.bg.primary,
           'input-bg': tokens.colors.semantic.light.bg.secondary,
           'text-primary': tokens.colors.semantic.light.text.primary,
           'text-secondary': tokens.colors.semantic.light.text.secondary,
           accent: tokens.colors.semantic.light.accent,
           border: tokens.colors.semantic.light.border.default,
           'highlight-bg': tokens.colors.semantic.light.highlight.bg,
           'highlight-border': tokens.colors.semantic.light.highlight.border,
         }
      },

      // Typography tokens
      fontFamily: {
        sans: tokens.fontFamily.sans.split(', '),
        mono: tokens.fontFamily.mono.split(', '),
      },
      fontSize: {
        xs: tokens.fontSize.xs,
        sm: tokens.fontSize.sm,
        base: tokens.fontSize.base,
        lg: tokens.fontSize.lg,
        xl: tokens.fontSize.xl,
        '2xl': tokens.fontSize['2xl'],
        '3xl': tokens.fontSize['3xl'],
        '4xl': tokens.fontSize['4xl'],
        // Component-specific (legacy)
        'search-input': tokens.typography.searchInput.fontSize,
        'result-title': tokens.fontSize.base,
        'result-subtitle': tokens.fontSize.sm,
      },
      fontWeight: {
        normal: tokens.fontWeight.normal,
        medium: tokens.fontWeight.medium,
        semibold: tokens.fontWeight.semibold,
        bold: tokens.fontWeight.bold,
      },
      lineHeight: {
        none: tokens.lineHeight.none,
        tight: tokens.lineHeight.tight,
        snug: tokens.lineHeight.snug,
        normal: tokens.lineHeight.normal,
        relaxed: tokens.lineHeight.relaxed,
        loose: tokens.lineHeight.loose,
      },

      // Spacing tokens (8px grid system)
      spacing: {
        px: tokens.spacing.px,
        0: tokens.spacing[0],
        0.5: tokens.spacing[0.5],
        1: tokens.spacing[1],
        1.5: tokens.spacing[1.5],
        2: tokens.spacing[2],
        2.5: tokens.spacing[2.5],
        3: tokens.spacing[3],
        3.5: tokens.spacing[3.5],
        4: tokens.spacing[4],
        5: tokens.spacing[5],
        6: tokens.spacing[6],
        8: tokens.spacing[8],
        10: tokens.spacing[10],
        12: tokens.spacing[12],
        16: tokens.spacing[16],
        20: tokens.spacing[20],
        24: tokens.spacing[24],
        32: tokens.spacing[32],
        // Legacy compatibility
        'window': tokens.spacing[4],
      },

      // Border radius tokens
      borderRadius: {
        none: tokens.borderRadius.none,
        sm: tokens.borderRadius.sm,
        DEFAULT: tokens.borderRadius.default,
        md: tokens.borderRadius.md,
        lg: tokens.borderRadius.lg,
        xl: tokens.borderRadius.xl,
        '2xl': tokens.borderRadius['2xl'],
        full: tokens.borderRadius.full,
        // Legacy compatibility
        'input': tokens.borderRadius.lg,
        'result': tokens.borderRadius.md,
      },

      // Shadow tokens
      boxShadow: {
        sm: tokens.shadow.sm,
        DEFAULT: tokens.shadow.default,
        md: tokens.shadow.md,
        lg: tokens.shadow.lg,
        xl: tokens.shadow.xl,
        hover: tokens.shadow.hover,
        selected: tokens.shadow.selected,
        elevated: tokens.shadow.elevated,
        none: tokens.shadow.none,
      },

      // Animation tokens
      transitionDuration: {
        instant: tokens.duration.instant,
        fast: tokens.duration.fast,
        normal: tokens.duration.normal,
        slow: tokens.duration.slow,
        slower: tokens.duration.slower,
        slowest: tokens.duration.slowest,
        // Legacy compatibility
        'window': tokens.duration.normal,
        'highlight': tokens.duration.fast,
      },
      transitionTimingFunction: {
        linear: tokens.easing.linear,
        in: tokens.easing.easeIn,
        out: tokens.easing.easeOut,
        'in-out': tokens.easing.easeInOut,
        bounce: tokens.easing.bounce,
        smooth: tokens.easing.smooth,
      },

      // Z-index tokens
      zIndex: {
        base: tokens.zIndex.base,
        dropdown: tokens.zIndex.dropdown,
        sticky: tokens.zIndex.sticky,
        fixed: tokens.zIndex.fixed,
        overlay: tokens.zIndex.overlay,
        modal: tokens.zIndex.modal,
        popover: tokens.zIndex.popover,
        toast: tokens.zIndex.toast,
        tooltip: tokens.zIndex.tooltip,
      },
    },
  },
  plugins: [],
}
