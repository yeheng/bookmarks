/**
 * Bookmarks Store - 向下兼容层
 *
 * 此文件为向下兼容而保留，内部实际调用 resources store
 * 新代码应该使用 @/stores/resources 替代
 *
 * @deprecated 请使用 useResourcesStore 替代
 */
import { useResourcesStore } from '@/stores/resources'
import type {
  Bookmark,
  CreateBookmarkRequest,
  SearchQuery,
  UpdateBookmarkRequest
} from '@/types'
import { defineStore } from 'pinia'
import { computed } from 'vue'

export const useBookmarksStore = defineStore('bookmarks', () => {
  // 获取 resources store 实例
  const resourcesStore = useResourcesStore()

  // 状态（映射到 resources store）
  const bookmarks = computed(() => resourcesStore.resources as Bookmark[])
  const currentBookmark = computed(() => resourcesStore.currentResource as Bookmark | null)
  const isLoading = computed(() => resourcesStore.isLoading)
  const isLoadingMore = computed(() => resourcesStore.isLoadingMore)
  const error = computed(() => resourcesStore.error)
  const hasMore = computed(() => resourcesStore.hasMore)
  const currentPage = computed(() => resourcesStore.currentPage)
  const pageSize = computed(() => resourcesStore.pageSize)

  // 操作方法（委托给 resources store）
  const fetchBookmarks = async (params?: SearchQuery, reset = true): Promise<void> => {
    return resourcesStore.fetchResources(params, reset)
  }

  const fetchBookmark = async (id: number): Promise<void> => {
    return resourcesStore.fetchResource(id)
  }

  const createBookmark = async (data: CreateBookmarkRequest): Promise<Bookmark> => {
    return resourcesStore.createResource(data) as Promise<Bookmark>
  }

  const updateBookmark = async (id: number, data: UpdateBookmarkRequest): Promise<Bookmark> => {
    return resourcesStore.updateResource(id, data) as Promise<Bookmark>
  }

  const deleteBookmark = async (id: number): Promise<void> => {
    return resourcesStore.deleteResource(id)
  }

  const searchBookmarks = async (params: SearchQuery, reset = true): Promise<void> => {
    return resourcesStore.searchResources(params, reset)
  }

  const clearError = (): void => {
    resourcesStore.clearError()
  }

  return {
    bookmarks,
    currentBookmark,
    isLoading,
    isLoadingMore,
    error,
    hasMore,
    currentPage,
    pageSize,
    fetchBookmarks,
    fetchBookmark,
    createBookmark,
    updateBookmark,
    deleteBookmark,
    searchBookmarks,
    clearError
  }
})