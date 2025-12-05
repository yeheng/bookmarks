/**
 * 防抖动工具函数
 * 用于延迟执行函数，直到用户停止操作一段时间
 */

/**
 * 防抖动函数
 * @param func 需要防抖的函数
 * @param delay 延迟时间（毫秒）
 * @param immediate 是否立即执行（第一次调用时）
 * @returns 防抖后的函数
 */
export function debounce<T extends (...args: any[]) => any>(
  func: T,
  delay: number,
  immediate: boolean = false
): (...args: Parameters<T>) => void {
  let timeoutId: NodeJS.Timeout | null = null
  let lastCallTime = 0

  return function debounced(...args: Parameters<T>) {
    const now = Date.now()
    const timeSinceLastCall = now - lastCallTime

    // 清除之前的定时器
    if (timeoutId) {
      clearTimeout(timeoutId)
    }

    // 立即执行条件：第一次调用且 immediate 为 true
    if (immediate && !lastCallTime) {
      lastCallTime = now
      func.apply(this, args)
      return
    }

    // 设置新的定时器
    timeoutId = setTimeout(() => {
      lastCallTime = Date.now()
      timeoutId = null
      func.apply(this, args)
    }, delay)
  }
}

/**
 * 可取消的防抖动函数
 * 返回包含 cancel 方法的防抖函数
 */
export function debounceWithCancel<T extends (...args: any[]) => any>(
  func: T,
  delay: number
) {
  let timeoutId: NodeJS.Timeout | null = null
  let isPending = false

  const debounced = function debounced(...args: Parameters<T>) {
    // 清除之前的定时器
    if (timeoutId) {
      clearTimeout(timeoutId)
    }

    isPending = true

    // 设置新的定时器
    timeoutId = setTimeout(() => {
      timeoutId = null
      isPending = false
      func.apply(this, args)
    }, delay)
  }

  // 取消方法
  const cancel = () => {
    if (timeoutId) {
      clearTimeout(timeoutId)
      timeoutId = null
      isPending = false
    }
  }

  // 立即执行方法
  const flush = (...args: Parameters<T>) => {
    cancel()
    func.apply(this, args)
  }

  // 检查是否有待处理的调用
  const pending = () => isPending

  return {
    debounced,
    cancel,
    flush,
    pending
  }
}

/**
 * 节流函数
 * 限制函数的执行频率
 */
export function throttle<T extends (...args: any[]) => any>(
  func: T,
  delay: number
): (...args: Parameters<T>) => void {
  let timeoutId: NodeJS.Timeout | null = null
  let lastExecTime = 0

  return function throttled(...args: Parameters<T>) {
    const now = Date.now()

    // 如果距离上次执行时间超过延迟，立即执行
    if (now - lastExecTime > delay) {
      lastExecTime = now
      func.apply(this, args)
    } else {
      // 否则设置定时器在延迟后执行
      if (timeoutId) {
        clearTimeout(timeoutId)
      }
      timeoutId = setTimeout(() => {
        lastExecTime = Date.now()
        timeoutId = null
        func.apply(this, args)
      }, delay - (now - lastExecTime))
    }
  }
}

/**
 * 搜索专用的防抖动Hook配置
 */
export const SEARCH_DEBOUNCE_DELAY = 500 // 搜索防抖延迟（毫秒）
export const SEARCH_MIN_CHARS = 3 // 搜索最少字符数

/**
 * 验证搜索查询是否有效
 */
export function isValidSearchQuery(query: string): boolean {
  return query.trim().length >= SEARCH_MIN_CHARS
}

/**
 * 获取搜索查询的状态描述
 */
export function getSearchQueryStatus(query: string): {
  isValid: boolean
  message: string
  type: 'info' | 'warning' | 'success'
} {
  const trimmedQuery = query.trim()

  if (!trimmedQuery) {
    return {
      isValid: false,
      message: '请输入搜索关键词',
      type: 'info'
    }
  }

  if (trimmedQuery.length < SEARCH_MIN_CHARS) {
    return {
      isValid: false,
      message: `请输入至少${SEARCH_MIN_CHARS}个字符开始搜索`,
      type: 'warning'
    }
  }

  return {
    isValid: true,
    message: '可以开始搜索',
    type: 'success'
  }
}