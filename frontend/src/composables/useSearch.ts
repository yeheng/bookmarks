import { ref, computed, watch, nextTick } from 'vue'
import { debounceWithCancel, SEARCH_DEBOUNCE_DELAY, isValidSearchQuery, getSearchQueryStatus } from '@/utils/debounce'
import type { SearchFilters } from '@/types'

/**
 * 搜索功能的组合式函数
 */
export function useSearch(
  onSearch: (query: string, filters: SearchFilters) => Promise<void>
) {
  // 搜索状态
  const searchQuery = ref('')
  const isSearching = ref(false)
  const hasSearched = ref(false)
  const searchTime = ref(0)
  const searchError = ref<string | null>(null)
  const isTyping = ref(false)
  const isLoadingMore = ref(false)

  // 搜索过滤器
  const filters = ref<SearchFilters>({
    collectionId: '',
    tagId: '',
    sortBy: 'relevance'
  })

  // 创建可取消的防抖搜索
  const { debounced: debouncedSearch, cancel: cancelDebounce, flush: flushSearch } = debounceWithCancel(
    async (query: string) => {
      await performSearch(query)
    },
    SEARCH_DEBOUNCE_DELAY
  )

  // 搜索查询状态
  const queryStatus = computed(() => getSearchQueryStatus(searchQuery.value))

  // 是否可以执行搜索
  const canSearch = computed(() => queryStatus.value.isValid)

  // 执行搜索的核心函数
  const performSearch = async (query?: string) => {
    const searchKeyword = query || searchQuery.value.trim()

    if (!isValidSearchQuery(searchKeyword)) {
      return
    }

    isSearching.value = true
    hasSearched.value = true
    searchError.value = null
    isTyping.value = false

    try {
      const startTime = Date.now()

      await onSearch(searchKeyword, filters.value)

      searchTime.value = ((Date.now() - startTime) / 1000).toFixed(3)
    } catch (error: any) {
      console.error('搜索失败:', error)
      searchError.value = error.message || '搜索失败，请稍后重试'
    } finally {
      isSearching.value = false
    }
  }

  // 处理输入事件
  const handleInput = (event: Event) => {
    const target = event.target as HTMLInputElement
    const value = target.value

    searchQuery.value = value
    isTyping.value = true

    // 如果查询无效，直接清除防抖
    if (!canSearch.value) {
      cancelDebounce()
      // 如果查询为空且之前有搜索结果，清除搜索
      if (!value.trim() && hasSearched.value) {
        clearSearch()
      }
      return
    }

    // 使用防抖搜索
    debouncedSearch(value)
  }

  // 手动触发搜索（点击搜索按钮或按回车）
  const triggerSearch = async () => {
    if (!canSearch.value) {
      return
    }

    // 取消防抖，立即执行搜索
    flushSearch(searchQuery.value)
  }

  // 重试搜索
  const retrySearch = () => {
    searchError.value = null
    performSearch()
  }

  // 清除搜索
  const clearSearch = () => {
    cancelDebounce()

    searchQuery.value = ''
    hasSearched.value = false
    searchError.value = null
    isTyping.value = false
    searchTime.value = 0
    isLoadingMore.value = false

    // 重置过滤器
    filters.value = {
      collectionId: '',
      tagId: '',
      sortBy: 'relevance'
    }
  }

  // 加载更多结果
  const loadMore = async () => {
    if (isLoadingMore.value || !canSearch.value) {
      return
    }

    isLoadingMore.value = true

    try {
      // 这里需要根据具体的搜索实现来处理加载更多
      // 可以通过回调函数或者额外的参数来处理
      console.log('加载更多搜索结果...')
    } catch (error) {
      console.error('加载更多结果失败:', error)
    } finally {
      isLoadingMore.value = false
    }
  }

  // 更新过滤器
  const updateFilter = (filterName: keyof SearchFilters, value: string) => {
    filters.value[filterName] = value

    // 如果已经搜索过且查询有效，立即重新搜索
    if (hasSearched.value && canSearch.value) {
      performSearch()
    }
  }

  // 应用搜索建议
  const applySuggestion = (suggestion: string) => {
    searchQuery.value = suggestion
    // 立即搜索，不需要防抖
    triggerSearch()
  }

  // 监听过滤器变化
  watch(
    () => [filters.value.collectionId, filters.value.tagId, filters.value.sortBy],
    () => {
      if (hasSearched.value && canSearch.value) {
        // 取消防抖，立即执行搜索
        cancelDebounce()
        performSearch()
      }
    },
    { deep: true }
  )

  // 组件卸载时清理
  const cleanup = () => {
    cancelDebounce()
  }

  return {
    // 状态
    searchQuery,
    isSearching,
    hasSearched,
    searchTime,
    searchError,
    isTyping,
    isLoadingMore,
    filters,
    queryStatus,
    canSearch,

    // 方法
    handleInput,
    triggerSearch,
    retrySearch,
    clearSearch,
    loadMore,
    updateFilter,
    applySuggestion,
    cleanup,

    // 内部方法（供高级使用）
    performSearch,
    cancelDebounce
  }
}