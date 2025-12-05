<template>
  <div class="h-screen flex flex-col bg-background">
    <!-- 顶部导航栏 -->
    <div class="border-b border-border/40 bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60 px-6 py-4">
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-4">
          <button @click="toggleLeftDrawer" class="p-2 rounded-md hover:bg-accent transition-colors" title="切换收藏夹栏">
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16" />
            </svg>
          </button>

          <div>
            <h1 class="text-xl font-bold tracking-tight">书签</h1>
            <p class="text-sm text-muted-foreground">共 {{ bookmarksStore.bookmarks?.length || 0 }} 个书签</p>
          </div>
        </div>

        <div class="flex items-center gap-2">
          <select v-model="filters.sortBy" class="px-3 py-1.5 text-sm border border-border rounded-md bg-background" @change="applyFilters">
            <option value="created_at">最新创建</option>
            <option value="updated_at">最近更新</option>
            <option value="title">标题</option>
          </select>

          <button @click="toggleRightDrawer" class="p-2 rounded-md hover:bg-accent transition-colors" title="切换标签栏">
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 7h.01M7 3h5c.512 0 1.024.195 1.414.586l7 7a2 2 0 010 2.828l-7 7a2 2 0 01-2.828 0l-7-7A1.994 1.994 0 013 12V7a4 4 0 014-4z" />
            </svg>
          </button>
        </div>
      </div>
    </div>

    <!-- 主内容区域 -->
    <div class="flex-1 flex min-h-0 relative">
      <CollectionDrawer
        :is-open="leftDrawerOpen"
        :collections="collectionsStore.collections || []"
        :selected-id="filters.collectionId"
        @select="selectCollection"
      />

      <div
        class="flex-1 overflow-y-auto transition-all duration-300"
        :class="{ 'ml-64': leftDrawerOpen, 'mr-64': rightDrawerOpen, 'ml-0': !leftDrawerOpen, 'mr-0': !rightDrawerOpen }"
      >
        <div class="p-6">
          <BookmarkGrid
            :bookmarks="bookmarksStore.bookmarks || []"
            :is-loading="bookmarksStore.isLoading"
            :is-loading-more="bookmarksStore.isLoadingMore"
            :has-more="bookmarksStore.hasMore"
            @load-more="loadMore"
            @toggle-favorite="toggleFavorite"
            @edit="editBookmark"
            @delete="deleteBookmark"
            @tag-click="navigateToTag"
            @add-bookmark="handleAddBookmark"
          />
        </div>
      </div>

      <TagDrawer :is-open="rightDrawerOpen" :tags="allTags" :selected-tag="filters.tag" @tag-click="handleTagClick" />
    </div>

    <!-- 书签编辑模态框 -->
    <BookmarkModal
      :is-open="isModalOpen"
      :bookmark="editingBookmark"
      :collections="collectionsStore.collections || []"
      :is-submitting="isSubmitting"
      @close="handleCloseModal"
      @submit="handleSubmitBookmark"
    />
  </div>
</template>

<script setup lang="ts">
import { onMounted, reactive, computed } from 'vue'
import { useBookmarksStore } from '@/stores/bookmarks'
import { useCollectionsStore } from '@/stores/collections'
import { useTagsStore } from '@/stores/tags'
import { useDrawers, useBookmarkActions, useTagStats } from '@/composables/useBookmarks'
import { BookmarkModal } from '@/components/bookmarks'
import CollectionDrawer from '@/components/bookmarks/CollectionDrawer.vue'
import TagDrawer from '@/components/bookmarks/TagDrawer.vue'
import BookmarkGrid from '@/components/bookmarks/BookmarkGrid.vue'

const bookmarksStore = useBookmarksStore()
const collectionsStore = useCollectionsStore()
const tagsStore = useTagsStore()

const { leftDrawerOpen, rightDrawerOpen, toggleLeftDrawer, toggleRightDrawer } = useDrawers()
const { 
  toggleFavorite, 
  editBookmark, 
  deleteBookmark, 
  handleAddBookmark,
  isModalOpen,
  editingBookmark,
  isSubmitting,
  handleCloseModal,
  handleSubmitBookmark
} = useBookmarkActions()

// 本地标签处理函数
const handleTagClick = (tagName: string) => {
  filters.tag = filters.tag === tagName ? '' : tagName
  applyFilters()
}

const filters = reactive({ collectionId: '', sortBy: 'created_at', tag: '' })

const allTags = computed(() => {
  const bookmarks = bookmarksStore.bookmarks || []
  const { allTags } = useTagStats(bookmarks)
  return allTags.value
})

const selectCollection = (collectionId: string | null) => {
  filters.collectionId = collectionId || ''
  applyFilters()
}

const applyFilters = async () => {
  const params: any = { sort_by: filters.sortBy }
  if (filters.collectionId) params.collection_id = parseInt(filters.collectionId)
  if (filters.tag) params.tags = filters.tag // 后端期望字符串，不是数组
  console.log('应用过滤器参数:', params)
  await bookmarksStore.fetchBookmarks(params, true)
}

const loadMore = async () => {
  const params: any = { sort_by: filters.sortBy }
  if (filters.collectionId) params.collection_id = parseInt(filters.collectionId)
  if (filters.tag) params.tags = filters.tag // 后端期望字符串，不是数组
  await bookmarksStore.fetchBookmarks(params, false)
}

onMounted(async () => {
  await Promise.all([
    collectionsStore.fetchCollections(), 
    tagsStore.fetchTags(),
    applyFilters()
  ])
})
</script>
