<template>
  <div class="container mx-auto px-4 py-8 h-screen flex flex-col">
    <!-- Page header with filters -->
    <div class="mb-6 flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4 flex-shrink-0">
      <div>
        <h1 class="text-2xl font-bold tracking-tight">书签</h1>
        <p class="text-muted-foreground">
          共 {{ bookmarksStore.bookmarks?.length || 0 }} 个书签
        </p>
      </div>
      
      <!-- Filters -->
      <div class="flex items-center gap-2">
        <select 
          v-model="filters.collectionId" 
          class="px-3 py-1.5 text-sm border border-border rounded-md bg-background"
          @change="applyFilters"
        >
          <option value="">所有收藏夹</option>
          <option 
            v-for="collection in collectionsStore.collections" 
            :key="collection.id" 
            :value="collection.id"
          >
            {{ collection.name }}
          </option>
        </select>
        
        <select 
          v-model="filters.sortBy" 
          class="px-3 py-1.5 text-sm border border-border rounded-md bg-background"
          @change="applyFilters"
        >
          <option value="created_at">最新创建</option>
          <option value="updated_at">最近更新</option>
          <option value="title">标题</option>
          <option value="visit_count">访问次数</option>
        </select>
      </div>
    </div>

    <!-- Infinite scroll container -->
    <div class="flex-1 min-h-0">
      <InfiniteScroll
        :items="bookmarksStore.bookmarks"
        :is-loading="bookmarksStore.isLoading"
        :is-loading-more="bookmarksStore.isLoadingMore"
        :has-more="bookmarksStore.hasMore"
        @load-more="loadMore"
      >
        <template #default="{ items, isLoading, isLoadingMore }">
          <!-- Bookmarks grid -->
          <div v-if="!isLoading && items.length > 0" class="grid gap-4">
            <div
              v-for="bookmark in items"
              :key="bookmark.id"
              class="group bg-card border border-border/50 rounded-lg p-4 hover:shadow-sm transition-all duration-200"
            >
              <div class="flex items-start justify-between gap-4">
                <!-- Bookmark info -->
                <div class="flex-1 min-w-0">
                  <div class="flex items-center gap-2 mb-1">
                    <h3 class="font-medium truncate hover:text-primary cursor-pointer" @click="openBookmark(bookmark.url)">
                      {{ bookmark.title }}
                    </h3>
                    <!-- Status indicators -->
                    <div class="flex items-center gap-1">
                      <span v-if="bookmark.is_favorite" class="text-yellow-500" title="收藏">⭐</span>
                      <span v-if="bookmark.is_read" class="text-green-500" title="已读">✓</span>
                      <span v-if="bookmark.is_archived" class="text-gray-500" title="归档">📁</span>
                    </div>
                  </div>
                  
                  <p class="text-sm text-muted-foreground mb-2 truncate">{{ bookmark.url }}</p>
                  
                  <p v-if="bookmark.description" class="text-sm text-muted-foreground mb-2 line-clamp-2">
                    {{ bookmark.description }}
                  </p>
                  
                  <!-- Tags -->
                  <div v-if="bookmark.tags && bookmark.tags.length > 0" class="flex flex-wrap gap-1 mb-2">
                    <span
                      v-for="tag in bookmark.tags"
                      :key="tag"
                      class="inline-flex items-center px-2 py-0.5 rounded-full text-xs bg-blue-100 text-blue-800"
                    >
                      {{ tag }}
                    </span>
                  </div>
                  
                  <!-- Meta info -->
                  <div class="flex items-center gap-4 text-xs text-muted-foreground">
                    <span>{{ formatDate(bookmark.created_at) }}</span>
                    <span v-if="bookmark.visit_count > 0">{{ bookmark.visit_count }} 次访问</span>
                    <span v-if="bookmark.collection_name">{{ bookmark.collection_name }}</span>
                  </div>
                </div>
                
                <!-- Actions -->
                <div class="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                  <button
                    @click="toggleFavorite(bookmark)"
                    class="p-1.5 rounded hover:bg-accent transition-colors"
                    :title="bookmark.is_favorite ? '取消收藏' : '添加收藏'"
                  >
                    <span :class="bookmark.is_favorite ? 'text-yellow-500' : 'text-muted-foreground'">
                      {{ bookmark.is_favorite ? '⭐' : '☆' }}
                    </span>
                  </button>
                  <button
                    @click="editBookmark(bookmark)"
                    class="p-1.5 rounded hover:bg-accent transition-colors text-muted-foreground"
                    title="编辑"
                  >
                    ✏️
                  </button>
                  <button
                    @click="deleteBookmark(bookmark.id)"
                    class="p-1.5 rounded hover:bg-accent transition-colors text-red-500"
                    title="删除"
                  >
                    🗑️
                  </button>
                </div>
              </div>
            </div>
          </div>

          <!-- Loading state -->
          <div v-else-if="isLoading" class="flex justify-center py-12">
            <div class="text-center">
              <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary mx-auto mb-4"></div>
              <p class="text-muted-foreground">加载书签中...</p>
            </div>
          </div>

          <!-- Empty state -->
          <div v-else class="flex justify-center py-12">
            <EmptyState
              title="暂无书签"
              description="使用顶部导航栏的 + 按钮添加第一个书签"
              action-text="添加书签"
              icon-type="bookmark"
              @action="handleAddBookmark"
            />
          </div>
        </template>
      </InfiniteScroll>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, reactive } from 'vue'
import { useBookmarksStore } from '@/stores/bookmarks'
import { useCollectionsStore } from '@/stores/collections'
import { EmptyState } from '@/components/ui/empty-state'
import { InfiniteScroll } from '@/components/ui/infinite-scroll'
import type { Bookmark, UpdateBookmarkRequest } from '@/types'

const bookmarksStore = useBookmarksStore()
const collectionsStore = useCollectionsStore()

// 筛选条件
const filters = reactive({
  collectionId: '',
  sortBy: 'created_at'
})

// 打开书签
const openBookmark = (url: string) => {
  window.open(url, '_blank')
}

// 格式化日期
const formatDate = (timestamp: number) => {
  const date = new Date(timestamp * 1000)
  const now = new Date()
  const diff = now.getTime() - date.getTime()
  const days = Math.floor(diff / (1000 * 60 * 60 * 24))
  
  if (days === 0) return '今天'
  if (days === 1) return '昨天'
  if (days < 7) return `${days}天前`
  if (days < 30) return `${Math.floor(days / 7)}周前`
  return date.toLocaleDateString()
}

// 应用筛选
const applyFilters = async () => {
  const params: any = {
    sort_by: filters.sortBy
  }
  
  if (filters.collectionId) {
    params.collection_id = parseInt(filters.collectionId)
  }
  
  await bookmarksStore.fetchBookmarks(params, true)
}

// 加载更多
const loadMore = async () => {
  const params: any = {
    sort_by: filters.sortBy
  }
  
  if (filters.collectionId) {
    params.collection_id = parseInt(filters.collectionId)
  }
  
  await bookmarksStore.fetchBookmarks(params, false)
}

// 切换收藏状态
const toggleFavorite = async (bookmark: Bookmark) => {
  try {
    await bookmarksStore.updateBookmark(bookmark.id, {
      is_favorite: !bookmark.is_favorite
    })
  } catch (error) {
    console.error('切换收藏状态失败:', error)
  }
}

// 编辑书签
const editBookmark = (bookmark: Bookmark) => {
  // TODO: 实现编辑书签功能
  console.log('编辑书签:', bookmark)
}

// 删除书签
const deleteBookmark = async (id: number) => {
  if (!confirm('确定要删除这个书签吗？')) return
  
  try {
    await bookmarksStore.deleteBookmark(id)
  } catch (error) {
    console.error('删除书签失败:', error)
  }
}

// 添加书签
const handleAddBookmark = () => {
  // TODO: 触发全局添加书签功能
  console.log('添加书签')
}

// 初始化
onMounted(async () => {
  await Promise.all([
    collectionsStore.fetchCollections(),
    applyFilters()
  ])
})
</script>