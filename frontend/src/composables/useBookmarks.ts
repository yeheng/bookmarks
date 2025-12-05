import { ref, computed } from 'vue'
import { useRouter } from 'vue-router'
import { useBookmarksStore } from '@/stores/bookmarks'
import type { Bookmark } from '@/types'

/**
 * 书签操作逻辑
 */
export function useBookmarkActions() {
  const router = useRouter()
  const bookmarksStore = useBookmarksStore()

  const navigateToTag = (tagName: string) => {
    router.push({ name: 'search', query: { tags: tagName } })
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
    console.log('编辑书签:', bookmark)
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
    console.log('添加书签')
  }

  return {
    navigateToTag,
    toggleFavorite,
    editBookmark,
    deleteBookmark,
    handleAddBookmark
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
