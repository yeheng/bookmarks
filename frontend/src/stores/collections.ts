import { apiService } from '@/services/api'
import type {
  Collection,
  CollectionsQuery,
  CreateCollectionRequest,
  UpdateCollectionRequest
} from '@/types'
import { defineStore } from 'pinia'
import { ref } from 'vue'

export const useCollectionsStore = defineStore('collections', () => {
  const collections = ref<Collection[]>([])
  const currentCollection = ref<Collection | null>(null)
  const isLoading = ref(false)
  const isLoadingMore = ref(false)
  const error = ref<string | null>(null)
  const hasMore = ref(true)
  const currentPage = ref(0)
  const pageSize = ref(20)

  const fetchCollections = async (params?: CollectionsQuery, reset = true): Promise<void> => {
    if (reset) {
      isLoading.value = true
      collections.value = []
      currentPage.value = 0
      hasMore.value = true
    } else {
      isLoadingMore.value = true
    }
    
    error.value = null
    
    try {
      const requestParams = {
        limit: pageSize.value,
        offset: reset ? 0 : collections.value.length,
        ...params
      }
      
      const response = await apiService.getCollections(requestParams)
      
      // API返回格式: Array 或 {items: [...], pagination: {...}}
      let items: any = []
      if (Array.isArray(response)) {
        items = response
      } else if (response.items) {
        items = response.items
      }
      
      if (reset) {
        collections.value = items
      } else {
        collections.value.push(...items)
      }
      
      // 如果返回的数据少于请求的页面大小，说明没有更多数据了
      hasMore.value = items.length === pageSize.value
      
      if (!reset) {
        currentPage.value++
      }
    } catch (err: any) {
      error.value = err.message || 'Failed to fetch collections'
      throw err
    } finally {
      isLoading.value = false
      isLoadingMore.value = false
    }
  }

  const fetchCollection = async (id: number): Promise<void> => {
    isLoading.value = true
    error.value = null
    
    try {
      const collection = await apiService.getCollection(id)
      currentCollection.value = collection
    } catch (err: any) {
      error.value = err.message || 'Failed to fetch collection'
      throw err
    } finally {
      isLoading.value = false
    }
  }

  const createCollection = async (data: CreateCollectionRequest): Promise<Collection> => {
    isLoading.value = true
    error.value = null
    
    try {
      const newCollection = await apiService.createCollection(data)
      collections.value.push(newCollection)
      return newCollection
    } catch (err: any) {
      error.value = err.message || 'Failed to create collection'
      throw err
    } finally {
      isLoading.value = false
    }
  }

  const updateCollection = async (id: number, data: UpdateCollectionRequest): Promise<Collection> => {
    isLoading.value = true
    error.value = null
    
    try {
      const updatedCollection = await apiService.updateCollection(id, data)
      const index = collections.value.findIndex(c => c.id === id)
      if (index !== -1) {
        collections.value[index] = updatedCollection
      }
      if (currentCollection.value?.id === id) {
        currentCollection.value = updatedCollection
      }
      return updatedCollection
    } catch (err: any) {
      error.value = err.message || 'Failed to update collection'
      throw err
    } finally {
      isLoading.value = false
    }
  }

  const deleteCollection = async (id: number): Promise<void> => {
    isLoading.value = true
    error.value = null
    
    try {
      await apiService.deleteCollection(id)
      collections.value = collections.value.filter(c => c.id !== id)
      if (currentCollection.value?.id === id) {
        currentCollection.value = null
      }
    } catch (err: any) {
      error.value = err.message || 'Failed to delete collection'
      throw err
    } finally {
      isLoading.value = false
    }
  }

  const clearError = (): void => {
    error.value = null
  }

  return {
    collections,
    currentCollection,
    isLoading,
    isLoadingMore,
    error,
    hasMore,
    currentPage,
    pageSize,
    fetchCollections,
    fetchCollection,
    createCollection,
    updateCollection,
    deleteCollection,
    clearError
  }
})