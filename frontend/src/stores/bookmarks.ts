import { defineStore } from 'pinia'
import { ref } from 'vue'
import { apiService } from '@/services/api'
import type { 
  Bookmark, 
  CreateBookmarkRequest, 
  UpdateBookmarkRequest, 
  SearchQuery,
  PaginatedResponse 
} from '@/types'

export const useBookmarksStore = defineStore('bookmarks', () => {
  const bookmarks = ref<Bookmark[]>([])
  const currentBookmark = ref<Bookmark | null>(null)
  const isLoading = ref(false)
  const error = ref<string | null>(null)
  const pagination = ref({
    total: 0,
    limit: 20,
    offset: 0,
    has_more: false
  })

  const fetchBookmarks = async (params?: SearchQuery): Promise<void> => {
    isLoading.value = true
    error.value = null
    
    try {
      const response: PaginatedResponse<Bookmark> = await apiService.getBookmarks(params)
      bookmarks.value = response.data
      pagination.value = {
        total: response.total,
        limit: response.limit,
        offset: response.offset,
        has_more: response.has_more
      }
    } catch (err: any) {
      error.value = err.message || 'Failed to fetch bookmarks'
      throw err
    } finally {
      isLoading.value = false
    }
  }

  const fetchBookmark = async (id: number): Promise<void> => {
    isLoading.value = true
    error.value = null
    
    try {
      const bookmark = await apiService.getBookmark(id)
      currentBookmark.value = bookmark
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
      const newBookmark = await apiService.createBookmark(data)
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
      const updatedBookmark = await apiService.updateBookmark(id, data)
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

  const searchBookmarks = async (params: SearchQuery): Promise<void> => {
    isLoading.value = true
    error.value = null
    
    try {
      const response: PaginatedResponse<Bookmark> = await apiService.search(params)
      bookmarks.value = response.data
      pagination.value = {
        total: response.total,
        limit: response.limit,
        offset: response.offset,
        has_more: response.has_more
      }
    } catch (err: any) {
      error.value = err.message || 'Failed to search bookmarks'
      throw err
    } finally {
      isLoading.value = false
    }
  }

  const clearError = (): void => {
    error.value = null
  }

  return {
    bookmarks,
    currentBookmark,
    isLoading,
    error,
    pagination,
    fetchBookmarks,
    fetchBookmark,
    createBookmark,
    updateBookmark,
    deleteBookmark,
    searchBookmarks,
    clearError
  }
})