/**
 * useBookmarks Composable - 向下兼容层
 *
 * 此文件为向下兼容而保留，内部实际调用 useResources composable
 * 新代码应该使用 @/composables/useResources 替代
 *
 * @deprecated 请使用 useResourceActions, useTagStats, useDrawers 替代
 */
import {
  useResourceActions,
  useTagStats,
  useDrawers
} from '@/composables/useResources'
import type { Bookmark, CreateBookmarkRequest, UpdateBookmarkRequest } from '@/types'

/**
 * 书签操作逻辑
 * @deprecated 请使用 useResourceActions 替代
 */
export function useBookmarkActions() {
  const resourceActions = useResourceActions()

  // 将 Resource 相关方法映射为 Bookmark 方法
  return {
    navigateToTag: resourceActions.navigateToTag,
    toggleFavorite: resourceActions.toggleFavorite,
    editBookmark: resourceActions.editResource,
    deleteBookmark: resourceActions.deleteResource,
    handleAddBookmark: resourceActions.handleAddResource,
    isModalOpen: resourceActions.isModalOpen,
    editingBookmark: resourceActions.editingResource,
    isSubmitting: resourceActions.isSubmitting,
    handleCloseModal: resourceActions.handleCloseModal,
    handleSubmitBookmark: resourceActions.handleSubmitResource
  }
}

// 直接导出 useTagStats，因为接口相同
export { useTagStats }

// 直接导出 useDrawers，因为接口相同
export { useDrawers }
