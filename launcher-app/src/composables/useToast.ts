/**
 * useToast Composable
 *
 * Provides a simple API for displaying toast notifications using vue-sonner.
 * Supports success, error, info, and warning toast types with consistent styling.
 *
 * Usage:
 * ```ts
 * const { success, error, info, warning } = useToast()
 *
 * success('Bookmark opened successfully')
 * error('Failed to open file', 'Please check the file path')
 * ```
 */

import { toast } from 'vue-sonner'

export interface ToastOptions {
  /** Main message */
  title: string
  /** Additional description (optional) */
  description?: string
  /** Duration in milliseconds (default: auto) */
  duration?: number
  /** Show action button */
  action?: {
    label: string
    onClick: () => void
  }
}

/**
 * Toast notification composable
 */
export function useToast() {
  /**
   * Show success toast (green)
   * Default duration: 3000ms
   */
  const success = (title: string, description?: string, duration?: number) => {
    toast.success(title, {
      description,
      duration: duration ?? 3000,
    })
  }

  /**
   * Show error toast (red)
   * Default duration: 5000ms (longer for errors)
   */
  const error = (title: string, description?: string, duration?: number) => {
    toast.error(title, {
      description,
      duration: duration ?? 5000,
    })
  }

  /**
   * Show info toast (blue)
   * Default duration: 3000ms
   */
  const info = (title: string, description?: string, duration?: number) => {
    toast.info(title, {
      description,
      duration: duration ?? 3000,
    })
  }

  /**
   * Show warning toast (amber)
   * Default duration: 4000ms
   */
  const warning = (title: string, description?: string, duration?: number) => {
    toast.warning(title, {
      description,
      duration: duration ?? 4000,
    })
  }

  /**
   * Show custom toast with action button
   */
  const custom = (options: ToastOptions) => {
    toast(options.title, {
      description: options.description,
      duration: options.duration ?? 3000,
      action: options.action,
    })
  }

  /**
   * Show loading toast (returns toast ID for dismissal)
   */
  const loading = (title: string, description?: string) => {
    return toast.loading(title, {
      description,
    })
  }

  /**
   * Dismiss a specific toast by ID
   */
  const dismiss = (toastId?: string | number) => {
    toast.dismiss(toastId)
  }

  /**
   * Dismiss all toasts
   */
  const dismissAll = () => {
    toast.dismiss()
  }

  /**
   * Show promise-based toast (auto-updates based on promise state)
   */
  const promise = <T,>(
    promise: Promise<T>,
    messages: {
      loading: string
      success: string | ((data: T) => string)
      error: string | ((error: Error) => string)
    }
  ) => {
    return toast.promise(promise, messages)
  }

  return {
    success,
    error,
    info,
    warning,
    custom,
    loading,
    dismiss,
    dismissAll,
    promise,
  }
}

/**
 * Convenience exports for direct usage
 */
export const showSuccess = (title: string, description?: string) => {
  toast.success(title, { description, duration: 3000 })
}

export const showError = (title: string, description?: string) => {
  toast.error(title, { description, duration: 5000 })
}

export const showInfo = (title: string, description?: string) => {
  toast.info(title, { description, duration: 3000 })
}

export const showWarning = (title: string, description?: string) => {
  toast.warning(title, { description, duration: 4000 })
}
