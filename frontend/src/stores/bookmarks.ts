import { apiService } from '@/services/api'
import type {
  Bookmark,
  CreateBookmarkRequest,
  SearchQuery,
  UpdateBookmarkRequest
} from '@/types'
import { defineStore } from 'pinia'
import { ref } from 'vue'

export const useBookmarksStore = defineStore('bookmarks', () => {
  const bookmarks = ref<Bookmark[]>([])
  const currentBookmark = ref<Bookmark | null>(null)
  const isLoading = ref(false)
  const isLoadingMore = ref(false)
  const error = ref<string | null>(null)
  const hasMore = ref(true)
  const currentPage = ref(0)
  const pageSize = ref(20)

  const fetchBookmarks = async (params?: SearchQuery, reset = true): Promise<void> => {
    if (reset) {
      isLoading.value = true
      bookmarks.value = []
      currentPage.value = 0
      hasMore.value = true
    } else {
      isLoadingMore.value = true
    }
    
    error.value = null
    
    try {
      const requestParams = {
        ...params,
        limit: pageSize.value,
        offset: reset ? 0 : bookmarks.value.length
      }
      
      const response = await apiService.getBookmarks(requestParams)
      
      // API返回格式: {data: Array, success: true} 或 {data: {items: [...], pagination: {...}}, success: true}
      let items: any = []
      if (Array.isArray(response.data)) {
        items = response.data
      } else if (response.data?.items) {
        items = response.data.items
      }
      
      if (reset) {
        bookmarks.value = items
      } else {
        bookmarks.value.push(...items)
      }
      
      // 如果返回的数据少于请求的页面大小，说明没有更多数据了
      hasMore.value = items.length === pageSize.value
      
      if (!reset) {
        currentPage.value++
      }
    } catch (err: any) {
      error.value = err.message || 'Failed to fetch bookmarks'
      throw err
    } finally {
      isLoading.value = false
      isLoadingMore.value = false
    }
  }

  const fetchBookmark = async (id: number): Promise<void> => {
    isLoading.value = true
    error.value = null
    
    try {
      const bookmark = await apiService.getBookmark(id)
      currentBookmark.value = bookmark.data
    } catch (err: any) {
      error.value = err.message || 'Failed to fetch bookmark'
      throw err
    } finally {
      isLoading.value = false
    }
  }

  const createBookmark = async (data: CreateBookmarkRequest): Promise<Bookmark> => {
    isLoading.value = true
    error.value = null
    
    try {
      const response = await apiService.createBookmark(data)
      const newBookmark: Bookmark = response.data
      bookmarks.value.unshift(newBookmark)
      return newBookmark
    } catch (err: any) {
      error.value = err.message || 'Failed to create bookmark'
      throw err
    } finally {
      isLoading.value = false
    }
  }

  const updateBookmark = async (id: number, data: UpdateBookmarkRequest): Promise<Bookmark> => {
    isLoading.value = true
    error.value = null
    
    try {
      const response = await apiService.updateBookmark(id, data)
      const updatedBookmark: Bookmark = response.data
      const index = bookmarks.value.findIndex(b => b.id === id)
      if (index !== -1) {
        bookmarks.value[index] = updatedBookmark
      }
      if (currentBookmark.value?.id === id) {
        currentBookmark.value = updatedBookmark
      }
      return updatedBookmark
    } catch (err: any) {
      error.value = err.message || 'Failed to update bookmark'
      throw err
    } finally {
      isLoading.value = false
    }
  }

  const deleteBookmark = async (id: number): Promise<void> => {
    isLoading.value = true
    error.value = null
    
    try {
      await apiService.deleteBookmark(id)
      bookmarks.value = bookmarks.value.filter(b => b.id !== id)
      if (currentBookmark.value?.id === id) {
        currentBookmark.value = null
      }
    } catch (err: any) {
      error.value = err.message || 'Failed to delete bookmark'
      throw err
    } finally {
      isLoading.value = false
    }
  }

  const searchBookmarks = async (params: SearchQuery, reset = true): Promise<void> => {
    if (reset) {
      isLoading.value = true
      bookmarks.value = []
      currentPage.value = 0
      hasMore.value = true
    } else {
      isLoadingMore.value = true
    }
    
    error.value = null
    
    try {
      const requestParams = {
        ...params,
        limit: pageSize.value,
        offset: reset ? 0 : bookmarks.value.length
      }
      
      const response = await apiService.search(requestParams)
      
      // API返回格式: {data: {items: [...], pagination: {...}}, success: true}
      const items = response.data?.items || []
      
      if (reset) {
        bookmarks.value = items
      } else {
        bookmarks.value.push(...items)
      }
      
      // 如果返回的数据少于请求的页面大小，说明没有更多数据了
      hasMore.value = items.length === pageSize.value
      
      if (!reset) {
        currentPage.value++
      }
    } catch (err: any) {
      error.value = err.message || 'Failed to search bookmarks'
      throw err
    } finally {
      isLoading.value = false
      isLoadingMore.value = false
    }
  }

  const clearError = (): void => {
    error.value = null
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