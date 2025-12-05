import { ref, computed } from 'vue'
import { useRouter } from 'vue-router'
import { useBookmarksStore } from '@/stores/bookmarks'
import { useCollectionsStore } from '@/stores/collections'
import type { Bookmark, CreateBookmarkRequest, UpdateBookmarkRequest } from '@/types'

/**
 * 书签操作逻辑
 */
export function useBookmarkActions() {
  const router = useRouter()
  const bookmarksStore = useBookmarksStore()
  const collectionsStore = useCollectionsStore()

  // 模态框状态
  const isModalOpen = ref(false)
  const editingBookmark = ref<Bookmark | undefined>()
  const isSubmitting = ref(false)

  const navigateToTag = (tagName: string) => {
    // 这个函数现在在 BookmarksView 中被重新定义
    console.log('Navigate to tag:', tagName)
  }

  const toggleFavorite = async (bookmark: Bookmark) => {
    try {
      await bookmarksStore.updateBookmark(bookmark.id, {
        is_favorite: !bookmark.is_favorite
      })
    } catch (error) {
      console.error('切换收藏状态失败:', error)
    }
  }

  const editBookmark = (bookmark: Bookmark) => {
    editingBookmark.value = bookmark
    isModalOpen.value = true
  }

  const deleteBookmark = async (id: number) => {
    if (!confirm('确定要删除这个书签吗？')) return
    try {
      await bookmarksStore.deleteBookmark(id)
    } catch (error) {
      console.error('删除书签失败:', error)
    }
  }

  const handleAddBookmark = () => {
    editingBookmark.value = undefined
    isModalOpen.value = true
  }

  const handleCloseModal = () => {
    isModalOpen.value = false
    editingBookmark.value = undefined
  }

  const handleSubmitBookmark = async (data: CreateBookmarkRequest | UpdateBookmarkRequest) => {
    try {
      isSubmitting.value = true
      
      console.log('提交书签数据:', data)
      console.log('编辑模式:', !!editingBookmark.value)
      
      if (editingBookmark.value) {
        // 更新书签
        console.log('更新书签 ID:', editingBookmark.value.id)
        const result = await bookmarksStore.updateBookmark(editingBookmark.value.id, data as UpdateBookmarkRequest)
        console.log('更新结果:', result)
      } else {
        // 创建书签
        console.log('创建新书签')
        const result = await bookmarksStore.createBookmark(data as CreateBookmarkRequest)
        console.log('创建结果:', result)
      }
      
      // 先重置提交状态，再关闭模态框
      isSubmitting.value = false
      handleCloseModal()
    } catch (error) {
      console.error('保存书签失败:', error)
      console.error('错误详情:', error?.message, error?.status, error?.code)
      isSubmitting.value = false
    }
  }

  return {
    navigateToTag,
    toggleFavorite,
    editBookmark,
    deleteBookmark,
    handleAddBookmark,
    isModalOpen,
    editingBookmark,
    isSubmitting,
    handleCloseModal,
    handleSubmitBookmark
  }
}

/**
 * 标签统计逻辑
 */
export function useTagStats(bookmarks: Bookmark[]) {
  const allTags = computed(() => {
    const tagMap = new Map<string, number>()

    if (Array.isArray(bookmarks)) {
      bookmarks.forEach(bookmark => {
        if (bookmark.tags) {
          bookmark.tags.forEach(tag => {
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
