/**
 * 搜索功能单元测试
 */

import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest'
import { nextTick } from 'vue'
import { useSearch } from '@/composables/useSearch'
import { debounceWithCancel, isValidSearchQuery, getSearchQueryStatus } from '@/utils/debounce'

// Mock 搜索函数
const mockSearchFunction = vi.fn()

describe('useSearch', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    mockSearchFunction.mockResolvedValue(undefined)
  })

  afterEach(() => {
    vi.restoreAllMocks()
  })

  it('should initialize with correct default values', () => {
    const {
      searchQuery,
      isSearching,
      hasSearched,
      searchError,
      isTyping,
      isLoadingMore,
      filters,
      queryStatus,
      canSearch
    } = useSearch(mockSearchFunction)

    expect(searchQuery.value).toBe('')
    expect(isSearching.value).toBe(false)
    expect(hasSearched.value).toBe(false)
    expect(searchError.value).toBe(null)
    expect(isTyping.value).toBe(false)
    expect(isLoadingMore.value).toBe(false)
    expect(filters.value).toEqual({
      collectionId: '',
      tagId: '',
      sortBy: 'relevance'
    })
    expect(queryStatus.value.message).toBe('请输入搜索关键词')
    expect(canSearch.value).toBe(false)
  })

  it('should update query status correctly', async () => {
    const { searchQuery, queryStatus, canSearch, handleInput } = useSearch(mockSearchFunction)

    // 测试空输入
    expect(queryStatus.value.isValid).toBe(false)
    expect(queryStatus.value.message).toBe('请输入搜索关键词')
    expect(canSearch.value).toBe(false)

    // 测试1个字符
    searchQuery.value = 'a'
    await nextTick()
    expect(queryStatus.value.isValid).toBe(false)
    expect(queryStatus.value.message).toContain('请输入至少3个字符')
    expect(canSearch.value).toBe(false)

    // 测试2个字符
    searchQuery.value = 'ab'
    await nextTick()
    expect(queryStatus.value.isValid).toBe(false)
    expect(canSearch.value).toBe(false)

    // 测试3个字符（边界情况）
    searchQuery.value = 'abc'
    await nextTick()
    expect(queryStatus.value.isValid).toBe(true)
    expect(queryStatus.value.message).toBe('可以开始搜索')
    expect(canSearch.value).toBe(true)
  })

  it('should handle input changes with debouncing', async () => {
    const { handleInput, isTyping } = useSearch(mockSearchFunction)
    vi.useFakeTimers()

    const mockEvent = {
      target: { value: 'test query' }
    } as unknown as Event

    // 开始输入
    handleInput(mockEvent)
    expect(isTyping.value).toBe(true)

    // 快速连续输入
    handleInput(mockEvent)
    handleInput(mockEvent)

    // 在防抖时间内，不应该触发搜索
    expect(mockSearchFunction).not.toHaveBeenCalled()

    // 等待防抖时间
    vi.advanceTimersByTime(500)
    await nextTick()

    // 防抖时间后，应该触发搜索
    expect(mockSearchFunction).toHaveBeenCalledTimes(1)
    expect(mockSearchFunction).toHaveBeenCalledWith('test query', {
      collectionId: '',
      tagId: '',
      sortBy: 'relevance'
    })

    vi.useRealTimers()
  })

  it('should trigger immediate search when requested', async () => {
    const { searchQuery, triggerSearch } = useSearch(mockSearchFunction)

    searchQuery.value = 'test'
    await nextTick()

    await triggerSearch()

    expect(mockSearchFunction).toHaveBeenCalledWith('test', {
      collectionId: '',
      tagId: '',
      sortBy: 'relevance'
    })
  })

  it('should not search with invalid query', async () => {
    const { searchQuery, triggerSearch } = useSearch(mockSearchFunction)

    searchQuery.value = 'ab' // 少于3个字符
    await nextTick()

    await triggerSearch()

    expect(mockSearchFunction).not.toHaveBeenCalled()
  })

  it('should update filters correctly', async () => {
    const { updateFilter, filters } = useSearch(mockSearchFunction)

    updateFilter('collectionId', '123')
    expect(filters.value.collectionId).toBe('123')

    updateFilter('tagId', '456')
    expect(filters.value.tagId).toBe('456')

    updateFilter('sortBy', 'created_at')
    expect(filters.value.sortBy).toBe('created_at')
  })

  it('should clear search correctly', () => {
    const { clearSearch, searchQuery, hasSearched, searchError, filters } = useSearch(mockSearchFunction)

    // 设置一些状态
    searchQuery.value = 'test'
    hasSearched.value = true
    searchError.value = 'Some error'
    filters.value.collectionId = '123'

    clearSearch()

    expect(searchQuery.value).toBe('')
    expect(hasSearched.value).toBe(false)
    expect(searchError.value).toBe(null)
    expect(filters.value).toEqual({
      collectionId: '',
      tagId: '',
      sortBy: 'relevance'
    })
  })
})

describe('debounce utilities', () => {
  beforeEach(() => {
    vi.useFakeTimers()
  })

  afterEach(() => {
    vi.useRealTimers()
  })

  it('should debounce function calls', () => {
    const mockFn = vi.fn()
    const { debounced } = debounceWithCancel(mockFn, 100)

    // 快速连续调用
    debounced('call1')
    debounced('call2')
    debounced('call3')

    // 在防抖时间内不应该执行
    expect(mockFn).not.toHaveBeenCalled()

    // 等待防抖时间
    vi.advanceTimersByTime(100)

    // 应该只执行最后一次调用
    expect(mockFn).toHaveBeenCalledTimes(1)
    expect(mockFn).toHaveBeenCalledWith('call3')
  })

  it('should cancel pending calls', () => {
    const mockFn = vi.fn()
    const { debounced, cancel } = debounceWithCancel(mockFn, 100)

    debounced('test')
    expect(mockFn).not.toHaveBeenCalled()

    cancel()

    // 即使等待防抖时间，也不应该执行
    vi.advanceTimersByTime(100)
    expect(mockFn).not.toHaveBeenCalled()
  })

  it('should flush immediate execution', () => {
    const mockFn = vi.fn()
    const { debounced, flush } = debounceWithCancel(mockFn, 100)

    debounced('test')
    expect(mockFn).not.toHaveBeenCalled()

    flush('test')

    // 立即执行，不需要等待
    expect(mockFn).toHaveBeenCalledTimes(1)
    expect(mockFn).toHaveBeenCalledWith('test')
  })

  it('should report pending status correctly', () => {
    const mockFn = vi.fn()
    const { debounced, pending } = debounceWithCancel(mockFn, 100)

    expect(pending()).toBe(false)

    debounced('test')
    expect(pending()).toBe(true)

    vi.advanceTimersByTime(100)
    expect(pending()).toBe(false)
  })
})

describe('search validation utilities', () => {
  it('should validate search queries correctly', () => {
    expect(isValidSearchQuery('')).toBe(false)
    expect(isValidSearchQuery('a')).toBe(false)
    expect(isValidSearchQuery('ab')).toBe(false)
    expect(isValidSearchQuery('abc')).toBe(true)
    expect(isValidSearchQuery('abcd')).toBe(true)
    expect(isValidSearchQuery('  abc  ')).toBe(true)
    expect(isValidSearchQuery('   ')).toBe(false)
  })

  it('should provide correct query status', () => {
    // 空输入
    let status = getSearchQueryStatus('')
    expect(status.isValid).toBe(false)
    expect(status.message).toBe('请输入搜索关键词')
    expect(status.type).toBe('info')

    // 少于3个字符
    status = getSearchQueryStatus('ab')
    expect(status.isValid).toBe(false)
    expect(status.message).toContain('请输入至少3个字符')
    expect(status.type).toBe('warning')

    // 有效输入
    status = getSearchQueryStatus('test')
    expect(status.isValid).toBe(true)
    expect(status.message).toBe('可以开始搜索')
    expect(status.type).toBe('success')
  })
})