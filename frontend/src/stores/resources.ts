import { commandApiService } from '@/services/command-api'
import type {
  CreateResourceRequest,
  Resource,
  SearchQuery,
  UpdateResourceRequest
} from '@/types'
import { defineStore } from 'pinia'
import { ref } from 'vue'

/**
 * 资源管理 Store
 * 管理所有类型的资源（链接、笔记、代码片段、文件等）
 */
export const useResourcesStore = defineStore('resources', () => {
  // 状态
  const resources = ref<Resource[]>([])
  const currentResource = ref<Resource | null>(null)
  const isLoading = ref(false)
  const isLoadingMore = ref(false)
  const error = ref<string | null>(null)
  const hasMore = ref(true)
  const currentPage = ref(0)
  const pageSize = ref(20)

  /**
   * 获取资源列表
   * @param params 查询参数
   * @param reset 是否重置列表
   */
  const fetchResources = async (params?: SearchQuery, reset = true): Promise<void> => {
    if (reset) {
      isLoading.value = true
      resources.value = []
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
        offset: reset ? 0 : resources.value.length
      }

      const response = await commandApiService.getResources(requestParams)

      // API返回格式: {items: [...], pagination: {...}} 或直接是数组
      let items: any = []
      if (Array.isArray(response)) {
        items = response
      } else if (response.items) {
        items = response.items
      }

      if (reset) {
        resources.value = items
      } else {
        resources.value.push(...items)
      }

      // 如果返回的数据少于请求的页面大小，说明没有更多数据了
      hasMore.value = items.length === pageSize.value

      if (!reset) {
        currentPage.value++
      }
    } catch (err: any) {
      error.value = err.message || 'Failed to fetch resources'
      throw err
    } finally {
      isLoading.value = false
      isLoadingMore.value = false
    }
  }

  /**
   * 获取单个资源详情
   * @param id 资源ID
   */
  const fetchResource = async (id: number): Promise<void> => {
    isLoading.value = true
    error.value = null

    try {
      const resource = await commandApiService.getResource(id)
      currentResource.value = resource
    } catch (err: any) {
      error.value = err.message || 'Failed to fetch resource'
      throw err
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 创建新资源
   * @param data 资源数据
   * @returns 创建的资源
   */
  const createResource = async (data: CreateResourceRequest): Promise<Resource> => {
    isLoading.value = true
    error.value = null

    try {
      const newResource: Resource = await commandApiService.createResource(data)
      resources.value.unshift(newResource)
      return newResource
    } catch (err: any) {
      error.value = err.message || 'Failed to create resource'
      throw err
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 更新资源
   * @param id 资源ID
   * @param data 更新数据
   * @returns 更新后的资源
   */
  const updateResource = async (id: number, data: UpdateResourceRequest): Promise<Resource> => {
    isLoading.value = true
    error.value = null

    try {
      const updatedResource: Resource = await commandApiService.updateResource(id, data)
      const index = resources.value.findIndex(r => r.id === id)
      if (index !== -1) {
        resources.value[index] = updatedResource
      }
      if (currentResource.value?.id === id) {
        currentResource.value = updatedResource
      }
      return updatedResource
    } catch (err: any) {
      error.value = err.message || 'Failed to update resource'
      throw err
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 删除资源
   * @param id 资源ID
   */
  const deleteResource = async (id: number): Promise<void> => {
    isLoading.value = true
    error.value = null

    try {
      await commandApiService.deleteResource(id)
      resources.value = resources.value.filter(r => r.id !== id)
      if (currentResource.value?.id === id) {
        currentResource.value = null
      }
    } catch (err: any) {
      error.value = err.message || 'Failed to delete resource'
      throw err
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 搜索资源
   * @param params 搜索参数
   * @param reset 是否重置列表
   */
  const searchResources = async (params: SearchQuery, reset = true): Promise<void> => {
    if (reset) {
      isLoading.value = true
      resources.value = []
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
        offset: reset ? 0 : resources.value.length
      }

      const response = await commandApiService.searchResources(requestParams)

      // API返回格式: {items: [...], pagination: {...}}
      const items = response.items || []

      if (reset) {
        resources.value = items
      } else {
        resources.value.push(...items)
      }

      // 如果返回的数据少于请求的页面大小，说明没有更多数据了
      hasMore.value = items.length === pageSize.value

      if (!reset) {
        currentPage.value++
      }
    } catch (err: any) {
      error.value = err.message || 'Failed to search resources'
      throw err
    } finally {
      isLoading.value = false
      isLoadingMore.value = false
    }
  }

  /**
   * 创建资源引用关系
   * @param resourceId 源资源ID
   * @param targetId 目标资源ID
   * @param type 引用类型（可选）
   */
  const createResourceReference = async (
    resourceId: number,
    targetId: number,
    type?: string
  ): Promise<void> => {
    error.value = null

    try {
      await commandApiService.createResourceReference(resourceId, targetId, type)
    } catch (err: any) {
      error.value = err.message || 'Failed to create resource reference'
      throw err
    }
  }

  /**
   * 删除资源引用关系
   * @param resourceId 源资源ID
   * @param targetId 目标资源ID
   */
  const deleteResourceReference = async (resourceId: number, targetId: number): Promise<void> => {
    error.value = null

    try {
      await apiService.deleteResourceReference(resourceId, targetId)
    } catch (err: any) {
      error.value = err.message || 'Failed to delete resource reference'
      throw err
    }
  }

  /**
   * 获取资源的引用列表
   * @param resourceId 资源ID
   * @param params 查询参数
   * @returns 引用的资源列表
   */
  const getResourceReferences = async (
    resourceId: number,
    params?: { type?: string; limit?: number; offset?: number }
  ): Promise<Resource[]> => {
    error.value = null

    try {
      const response = await apiService.getResourceReferences(resourceId, params)
      const items = response.data?.items || []
      return items
    } catch (err: any) {
      error.value = err.message || 'Failed to get resource references'
      throw err
    }
  }

  /**
   * 清除错误信息
   */
  const clearError = (): void => {
    error.value = null
  }

  return {
    // 状态
    resources,
    currentResource,
    isLoading,
    isLoadingMore,
    error,
    hasMore,
    currentPage,
    pageSize,
    // 操作方法
    fetchResources,
    fetchResource,
    createResource,
    updateResource,
    deleteResource,
    searchResources,
    createResourceReference,
    deleteResourceReference,
    getResourceReferences,
    clearError
  }
})
