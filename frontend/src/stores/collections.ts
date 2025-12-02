import { defineStore } from 'pinia'
import { ref } from 'vue'
import { apiService } from '@/services/api'
import type { 
  Collection, 
  CreateCollectionRequest, 
  UpdateCollectionRequest 
} from '@/types'

export const useCollectionsStore = defineStore('collections', () => {
  const collections = ref<Collection[]>([])
  const currentCollection = ref<Collection | null>(null)
  const isLoading = ref(false)
  const error = ref<string | null>(null)

  const fetchCollections = async (): Promise<void> => {
    isLoading.value = true
    error.value = null
    
    try {
      const fetchedCollections = await apiService.getCollections()
      collections.value = fetchedCollections
    } catch (err: any) {
      error.value = err.message || 'Failed to fetch collections'
      throw err
    } finally {
      isLoading.value = false
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
    error,
    fetchCollections,
    fetchCollection,
    createCollection,
    updateCollection,
    deleteCollection,
    clearError
  }
})