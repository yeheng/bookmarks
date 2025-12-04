import { defineStore } from 'pinia'
import { ref } from 'vue'
import { apiService } from '@/services/api'
import type { 
  Tag, 
  CreateTagRequest, 
  UpdateTagRequest 
} from '@/types'

export const useTagsStore = defineStore('tags', () => {
  const tags = ref<Tag[]>([])
  const currentTag = ref<Tag | null>(null)
  const isLoading = ref(false)
  const isLoadingMore = ref(false)
  const error = ref<string | null>(null)
  const hasMore = ref(true)
  const currentPage = ref(0)
  const pageSize = ref(20)

  const fetchTags = async (params?: { limit?: number; offset?: number; search?: string }, reset = true): Promise<void> => {
    if (reset) {
      isLoading.value = true
      tags.value = []
      currentPage.value = 0
      hasMore.value = true
    } else {
      isLoadingMore.value = true
    }
    
    error.value = null
    
    try {
      const requestParams = {
        limit: pageSize.value,
        offset: reset ? 0 : tags.value.length,
        ...params
      }
      
      const response = await apiService.getTags(requestParams)
      
      if (reset) {
        tags.value = response
      } else {
        tags.value.push(...response)
      }
      
      // 如果返回的数据少于请求的页面大小，说明没有更多数据了
      hasMore.value = response.length === pageSize.value
      
      if (!reset) {
        currentPage.value++
      }
    } catch (err: any) {
      error.value = err.message || 'Failed to fetch tags'
      throw err
    } finally {
      isLoading.value = false
      isLoadingMore.value = false
    }
  }

  const fetchTag = async (id: number): Promise<void> => {
    isLoading.value = true
    error.value = null
    
    try {
      const tag = await apiService.getTag(id)
      currentTag.value = tag
    } catch (err: any) {
      error.value = err.message || 'Failed to fetch tag'
      throw err
    } finally {
      isLoading.value = false
    }
  }

  const createTag = async (data: CreateTagRequest): Promise<Tag> => {
    isLoading.value = true
    error.value = null
    
    try {
      const newTag = await apiService.createTag(data)
      tags.value.push(newTag)
      return newTag
    } catch (err: any) {
      error.value = err.message || 'Failed to create tag'
      throw err
    } finally {
      isLoading.value = false
    }
  }

  const updateTag = async (id: number, data: UpdateTagRequest): Promise<Tag> => {
    isLoading.value = true
    error.value = null
    
    try {
      const updatedTag = await apiService.updateTag(id, data)
      const index = tags.value.findIndex(t => t.id === id)
      if (index !== -1) {
        tags.value[index] = updatedTag
      }
      if (currentTag.value?.id === id) {
        currentTag.value = updatedTag
      }
      return updatedTag
    } catch (err: any) {
      error.value = err.message || 'Failed to update tag'
      throw err
    } finally {
      isLoading.value = false
    }
  }

  const deleteTag = async (id: number): Promise<void> => {
    isLoading.value = true
    error.value = null
    
    try {
      await apiService.deleteTag(id)
      tags.value = tags.value.filter(t => t.id !== id)
      if (currentTag.value?.id === id) {
        currentTag.value = null
      }
    } catch (err: any) {
      error.value = err.message || 'Failed to delete tag'
      throw err
    } finally {
      isLoading.value = false
    }
  }

  const clearError = (): void => {
    error.value = null
  }

  return {
    tags,
    currentTag,
    isLoading,
    isLoadingMore,
    error,
    hasMore,
    currentPage,
    pageSize,
    fetchTags,
    fetchTag,
    createTag,
    updateTag,
    deleteTag,
    clearError
  }
})