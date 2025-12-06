import { ref, computed } from 'vue'
import { useRouter } from 'vue-router'
import { useResourcesStore } from '@/stores/resources'
import { useCollectionsStore } from '@/stores/collections'
import type { Resource, CreateResourceRequest, UpdateResourceRequest } from '@/types'

/**
 * 资源操作逻辑
 * 提供资源的创建、编辑、删除等操作
 */
export function useResourceActions() {
  const router = useRouter()
  const resourcesStore = useResourcesStore()
  const collectionsStore = useCollectionsStore()

  // 模态框状态
  const isModalOpen = ref(false)
  const editingResource = ref<Resource | undefined>()
  const isSubmitting = ref(false)

  /**
   * 导航到标签页面
   * @param tagName 标签名称
   */
  const navigateToTag = (tagName: string) => {
    // 这个函数现在在视图组件中被重新定义
    console.log('Navigate to tag:', tagName)
  }

  /**
   * 切换收藏状态
   * @param resource 资源对象
   */
  const toggleFavorite = async (resource: Resource) => {
    try {
      await resourcesStore.updateResource(resource.id, {
        is_favorite: !resource.is_favorite
      })
    } catch (error) {
      console.error('切换收藏状态失败:', error)
    }
  }

  /**
   * 编辑资源
   * @param resource 资源对象
   */
  const editResource = (resource: Resource) => {
    editingResource.value = resource
    isModalOpen.value = true
  }

  /**
   * 删除资源
   * @param id 资源ID
   */
  const deleteResource = async (id: number) => {
    if (!confirm('确定要删除这个资源吗？')) return
    try {
      await resourcesStore.deleteResource(id)
    } catch (error) {
      console.error('删除资源失败:', error)
    }
  }

  /**
   * 打开新增资源模态框
   */
  const handleAddResource = () => {
    editingResource.value = undefined
    isModalOpen.value = true
  }

  /**
   * 关闭模态框
   */
  const handleCloseModal = () => {
    isModalOpen.value = false
    editingResource.value = undefined
  }

  /**
   * 提交资源数据（创建或更新）
   * @param data 资源数据
   */
  const handleSubmitResource = async (data: CreateResourceRequest | UpdateResourceRequest) => {
    try {
      isSubmitting.value = true

      console.log('提交资源数据:', data)
      console.log('编辑模式:', !!editingResource.value)

      if (editingResource.value) {
        // 更新资源
        console.log('更新资源 ID:', editingResource.value.id)
        const result = await resourcesStore.updateResource(
          editingResource.value.id,
          data as UpdateResourceRequest
        )
        console.log('更新结果:', result)
      } else {
        // 创建资源
        console.log('创建新资源')
        const result = await resourcesStore.createResource(data as CreateResourceRequest)
        console.log('创建结果:', result)
      }

      // 先重置提交状态，再关闭模态框
      isSubmitting.value = false
      handleCloseModal()
    } catch (error) {
      console.error('保存资源失败:', error)
      console.error('错误详情:', error?.message, error?.status, error?.code)
      isSubmitting.value = false
    }
  }

  return {
    navigateToTag,
    toggleFavorite,
    editResource,
    deleteResource,
    handleAddResource,
    isModalOpen,
    editingResource,
    isSubmitting,
    handleCloseModal,
    handleSubmitResource
  }
}

/**
 * 标签统计逻辑
 * 统计资源中的标签使用情况
 */
export function useTagStats(resources: Resource[]) {
  const allTags = computed(() => {
    const tagMap = new Map<string, number>()

    if (Array.isArray(resources)) {
      resources.forEach(resource => {
        if (resource.tags) {
          resource.tags.forEach(tag => {
            tagMap.set(tag, (tagMap.get(tag) || 0) + 1)
          })
        }
      })
    }

    return Array.from(tagMap.entries())
      .map(([name, count]) => ({ name, count }))
      .sort((a, b) => b.count - a.count)
  })

  return { allTags }
}

/**
 * 抽屉状态管理
 * 管理左右侧边栏的显示状态
 */
export function useDrawers() {
  const leftDrawerOpen = ref(false)
  const rightDrawerOpen = ref(false)

  const toggleLeftDrawer = () => {
    leftDrawerOpen.value = !leftDrawerOpen.value
  }

  const toggleRightDrawer = () => {
    rightDrawerOpen.value = !rightDrawerOpen.value
  }

  return {
    leftDrawerOpen,
    rightDrawerOpen,
    toggleLeftDrawer,
    toggleRightDrawer
  }
}
