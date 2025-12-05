<template>
  <div class="h-screen flex flex-col bg-background">
    <!-- Simplified header -->
    <div class="border-b border-border/40 bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60 px-6 py-4">
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-4">
          <!-- Toggle buttons for drawers -->
          <button
            @click="toggleLeftDrawer"
            class="p-2 rounded-md hover:bg-accent transition-colors"
            title="切换收藏夹栏"
          >
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16" />
            </svg>
          </button>
          
          <div>
            <h1 class="text-xl font-bold tracking-tight">书签</h1>
            <p class="text-sm text-muted-foreground">
              共 {{ bookmarksStore.bookmarks?.length || 0 }} 个书签
            </p>
          </div>
        </div>
        
        <!-- Sort controls -->
        <div class="flex items-center gap-2">
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
          
          <button
            @click="toggleRightDrawer"
            class="p-2 rounded-md hover:bg-accent transition-colors"
            title="切换标签栏"
          >
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 7h.01M7 3h5c.512 0 1.024.195 1.414.586l7 7a2 2 0 010 2.828l-7 7a2 2 0 01-2.828 0l-7-7A1.994 1.994 0 013 12V7a4 4 0 014-4z" />
            </svg>
          </button>
        </div>
      </div>
    </div>

    <!-- Main content area with drawers -->
    <div class="flex-1 flex min-h-0 relative">
      <!-- Left drawer - Collections -->
      <div
        :class="[
          'absolute left-0 top-0 h-full bg-background border-r border-border/40 transition-all duration-300 z-10',
          leftDrawerOpen ? 'w-64' : 'w-0 overflow-hidden'
        ]"
      >
        <div class="w-64 h-full flex flex-col">
          <div class="p-4 border-b border-border/40">
            <h3 class="font-semibold text-sm text-muted-foreground uppercase tracking-wider">收藏夹</h3>
          </div>
          <div class="flex-1 overflow-y-auto p-2">
            <div class="space-y-1">
              <button
                @click="selectCollection(null)"
                :class="[
                  'w-full text-left px-3 py-2 rounded-md text-sm transition-colors',
                  !filters.collectionId ? 'bg-accent text-accent-foreground' : 'hover:bg-accent/50'
                ]"
              >
                <div class="flex items-center gap-2">
                  <span class="w-2 h-2 rounded-full bg-primary"></span>
                  <span>全部书签</span>
                </div>
              </button>
              
              <button
                v-for="collection in collectionsStore.collections"
                :key="collection.id"
                @click="selectCollection(collection.id)"
                :class="[
                  'w-full text-left px-3 py-2 rounded-md text-sm transition-colors',
                  filters.collectionId === collection.id ? 'bg-accent text-accent-foreground' : 'hover:bg-accent/50'
                ]"
              >
                <div class="flex items-center gap-2">
                  <span 
                    class="w-2 h-2 rounded-full" 
                    :style="{ backgroundColor: collection.color }"
                  ></span>
                  <span class="truncate">{{ collection.name }}</span>
                  <span class="text-xs text-muted-foreground ml-auto">{{ collection.bookmark_count }}</span>
                </div>
              </button>
            </div>
          </div>
        </div>
      </div>

      <!-- Main content - Bookmarks grid -->
      <div 
        class="flex-1 overflow-y-auto transition-all duration-300"
        :class="{
          'ml-64': leftDrawerOpen,
          'mr-64': rightDrawerOpen,
          'ml-0': !leftDrawerOpen,
          'mr-0': !rightDrawerOpen
        }"
      >
        <div class="p-6">
          <InfiniteScroll
            :items="bookmarksStore.bookmarks || []"
            :is-loading="bookmarksStore.isLoading"
            :is-loading-more="bookmarksStore.isLoadingMore"
            :has-more="bookmarksStore.hasMore"
            @load-more="loadMore"
          >
            <template #default="{ items, isLoading, isLoadingMore }">
              <!-- Bookmarks grid -->
              <div v-if="!isLoading && items && items.length > 0" class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
                <div
                  v-for="bookmark in items"
                  :key="bookmark.id"
                  class="group bg-card border border-border/50 rounded-lg p-4 hover:shadow-lg transition-all duration-200 hover:scale-[1.02] cursor-pointer"
                >
                  <!-- Card header with favicon and actions -->
                  <div class="flex items-start justify-between mb-3">
                    <div class="flex items-center gap-2">
                      <img 
                        v-if="bookmark.favicon_url" 
                        :src="bookmark.favicon_url" 
                        :alt="bookmark.title"
                        class="w-5 h-5 rounded flex-shrink-0"
                        @error="handleFaviconError"
                      />
                      <div 
                        v-else
                        class="w-5 h-5 rounded bg-accent flex items-center justify-center flex-shrink-0"
                      >
                        <span class="text-xs text-accent-foreground">{{ getDomainInitial(bookmark.url) }}</span>
                      </div>
                      
                      <!-- Status indicators -->
                      <div class="flex items-center gap-1">
                        <span v-if="bookmark.is_favorite" class="text-yellow-500" title="收藏">⭐</span>
                        <span v-if="bookmark.is_read" class="text-green-500" title="已读">✓</span>
                        <span v-if="bookmark.is_archived" class="text-gray-500" title="归档">📁</span>
                      </div>
                    </div>
                    
                    <!-- Actions -->
                    <div class="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                      <button
                        @click.stop="toggleFavorite(bookmark)"
                        class="p-1.5 rounded hover:bg-accent transition-colors"
                        :title="bookmark.is_favorite ? '取消收藏' : '添加收藏'"
                      >
                        <span :class="bookmark.is_favorite ? 'text-yellow-500' : 'text-muted-foreground'">
                          {{ bookmark.is_favorite ? '⭐' : '☆' }}
                        </span>
                      </button>
                      <button
                        @click.stop="editBookmark(bookmark)"
                        class="p-1.5 rounded hover:bg-accent transition-colors text-muted-foreground"
                        title="编辑"
                      >
                        ✏️
                      </button>
                      <button
                        @click.stop="deleteBookmark(bookmark.id)"
                        class="p-1.5 rounded hover:bg-accent transition-colors text-red-500"
                        title="删除"
                      >
                        🗑️
                      </button>
                    </div>
                  </div>
                  
                  <!-- Title -->
                  <h3 
                    class="font-medium text-sm mb-2 line-clamp-2 hover:text-primary transition-colors"
                    @click="openBookmark(bookmark.url)"
                  >
                    {{ bookmark.title }}
                  </h3>
                  
                  <!-- URL -->
                  <p class="text-xs text-muted-foreground mb-2 truncate">{{ bookmark.url }}</p>
                  
                  <!-- Description -->
                  <p v-if="bookmark.description" class="text-xs text-muted-foreground mb-3 line-clamp-3">
                    {{ bookmark.description }}
                  </p>
                  
                  <!-- Tags -->
                  <div v-if="bookmark.tags && bookmark.tags.length > 0" class="flex flex-wrap gap-1 mb-3">
                    <span
                      v-for="tag in bookmark.tags"
                      :key="tag"
                      @click.stop="navigateToTag(tag)"
                      class="inline-flex items-center px-2 py-0.5 rounded-full text-xs bg-blue-100 text-blue-800 hover:bg-blue-200 cursor-pointer transition-colors"
                    >
                      {{ tag }}
                    </span>
                  </div>
                  
                  <!-- Meta info -->
                  <div class="flex items-center justify-between text-xs text-muted-foreground pt-3 border-t border-border/30">
                    <div class="flex items-center gap-3">
                      <span v-if="bookmark.visit_count > 0" class="flex items-center gap-1">
                        <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" />
                        </svg>
                        {{ bookmark.visit_count }}
                      </span>
                      <span v-if="bookmark.collection_name">{{ bookmark.collection_name }}</span>
                    </div>
                    <span>{{ formatDate(bookmark.created_at) }}</span>
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

      <!-- Right drawer - Tags -->
      <div
        :class="[
          'absolute right-0 top-0 h-full bg-background border-l border-border/40 transition-all duration-300 z-10',
          rightDrawerOpen ? 'w-64' : 'w-0 overflow-hidden'
        ]"
      >
        <div class="w-64 h-full flex flex-col">
          <div class="p-4 border-b border-border/40">
            <h3 class="font-semibold text-sm text-muted-foreground uppercase tracking-wider">标签</h3>
          </div>
          <div class="flex-1 overflow-y-auto p-2">
            <div class="space-y-1">
              <button
                v-for="tag in allTags"
                :key="tag.name"
                @click="navigateToTag(tag.name)"
                class="w-full text-left px-3 py-2 rounded-md text-sm hover:bg-accent/50 transition-colors"
              >
                <div class="flex items-center gap-2">
                  <span class="w-2 h-2 rounded-full bg-blue-500"></span>
                  <span class="truncate">{{ tag.name }}</span>
                  <span class="text-xs text-muted-foreground ml-auto">{{ tag.count }}</span>
                </div>
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, reactive, computed } from 'vue'
import { useRouter } from 'vue-router'
import { useBookmarksStore } from '@/stores/bookmarks'
import { useCollectionsStore } from '@/stores/collections'
import { EmptyState } from '@/components/ui/empty-state'
import { InfiniteScroll } from '@/components/ui/infinite-scroll'
import type { Bookmark, UpdateBookmarkRequest } from '@/types'

const router = useRouter()
const bookmarksStore = useBookmarksStore()
const collectionsStore = useCollectionsStore()

// Drawer states
const leftDrawerOpen = ref(true)
const rightDrawerOpen = ref(true)

// 筛选条件
const filters = reactive({
  collectionId: '',
  sortBy: 'created_at'
})

// 计算所有标签及使用次数
const allTags = computed(() => {
  const tagMap = new Map<string, number>()
  
  const bookmarks = bookmarksStore.bookmarks || []
  
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

// 切换左侧抽屉
const toggleLeftDrawer = () => {
  leftDrawerOpen.value = !leftDrawerOpen.value
}

// 切换右侧抽屉
const toggleRightDrawer = () => {
  rightDrawerOpen.value = !rightDrawerOpen.value
}

// 选择收藏夹
const selectCollection = (collectionId: string | null) => {
  filters.collectionId = collectionId || ''
  applyFilters()
}

// 导航到标签页面
const navigateToTag = (tagName: string) => {
  router.push({
    name: 'search',
    query: { tags: tagName }
  })
}

// 打开书签
const openBookmark = (url: string) => {
  window.open(url, '_blank')
}

// 获取域名首字母
const getDomainInitial = (url: string) => {
  try {
    const domain = new URL(url).hostname
    return domain.charAt(0).toUpperCase()
  } catch {
    return 'W'
  }
}

// 处理favicon加载错误
const handleFaviconError = (event: Event) => {
  const img = event.target as HTMLImageElement
  img.style.display = 'none'
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