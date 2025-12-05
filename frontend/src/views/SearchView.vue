<template>
  <div class="container mx-auto px-4 py-8">
    <!-- Page header -->
    <div class="mb-8">
      <div class="max-w-2xl mx-auto">
        <h1 class="text-3xl font-bold tracking-tight text-center mb-4">搜索</h1>
        <p class="text-muted-foreground text-center mb-8">在您的书签中快速查找内容</p>

        <!-- Search form -->
        <form @submit.prevent="handleSearch" class="space-y-4">
          <!-- Main search input -->
          <div class="relative">
            <div class="absolute left-3 top-1/2 transform -translate-y-1/2">
              <span class="text-muted-foreground">🔍</span>
            </div>
            <Input
              v-model="searchQuery"
              type="text"
              placeholder="搜索书签、收藏夹或标签（至少3个字符）..."
              class="pl-10 pr-4 py-3 h-12 text-base"
              autofocus
              @input="handleInput"
            />
            <!-- 输入状态指示 -->
            <div v-if="queryStatus.message && !canSearch" class="mt-2 text-xs"
                 :class="queryStatus.type === 'warning' ? 'text-orange-600' : 'text-muted-foreground'">
              {{ queryStatus.message }}
            </div>
            <div v-else-if="isTyping && canSearch" class="mt-2 text-xs text-muted-foreground">
              正在输入...
            </div>
          </div>

          <!-- Filters -->
          <div class="flex flex-wrap gap-3">
            <!-- Collection filter -->
            <select
              :value="filters.collectionId"
              @change="updateFilter('collectionId', ($event.target as HTMLSelectElement).value)"
              class="px-3 py-2 text-sm border border-border rounded-md bg-background"
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

            <!-- Tag filter -->
            <select
              :value="filters.tagId"
              @change="updateFilter('tagId', ($event.target as HTMLSelectElement).value)"
              class="px-3 py-2 text-sm border border-border rounded-md bg-background"
            >
              <option value="">所有标签</option>
              <option
                v-for="tag in tagsStore.tags"
                :key="tag.id"
                :value="tag.id"
              >
                {{ tag.name }}
              </option>
            </select>

            <!-- Sort options -->
            <select
              :value="filters.sortBy"
              @change="updateFilter('sortBy', ($event.target as HTMLSelectElement).value)"
              class="px-3 py-2 text-sm border border-border rounded-md bg-background"
            >
              <option value="relevance">相关度</option>
              <option value="created_at">最新创建</option>
              <option value="updated_at">最近更新</option>
              <option value="visit_count">访问次数</option>
            </select>

            <!-- Search button -->
            <Button
              type="submit"
              :disabled="!canSearch || isSearching"
              class="px-6"
            >
              {{ isSearching ? '搜索中...' : '搜索' }}
            </Button>
          </div>
        </form>

        <!-- Search suggestions -->
        <div v-if="searchSuggestions.length > 0 && !hasSearched" class="mt-4">
          <p class="text-sm text-muted-foreground mb-2">热门搜索：</p>
          <div class="flex flex-wrap gap-2">
            <button
              v-for="suggestion in searchSuggestions"
              :key="suggestion"
              @click="applySuggestion(suggestion)"
              class="px-3 py-1 text-sm bg-muted hover:bg-muted/80 rounded-full transition-colors"
            >
              {{ suggestion }}
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- Search results -->
    <div v-if="hasSearched" class="max-w-4xl mx-auto">
      <!-- Error display -->
      <div v-if="searchError" class="mb-6 bg-destructive/10 border border-destructive/20 rounded-lg p-4">
        <div class="flex items-center gap-3">
          <span class="text-destructive text-lg">⚠️</span>
          <div>
            <h3 class="font-medium text-destructive">搜索失败</h3>
            <p class="text-sm text-destructive/80 mt-1">{{ searchError }}</p>
            <button
              @click="retrySearch"
              class="mt-2 text-sm text-destructive hover:text-destructive/80 underline"
            >
              重试搜索
            </button>
          </div>
        </div>
      </div>
      
      <!-- Results header -->
      <div v-else-if="!isSearching" class="mb-6 flex items-center justify-between">
        <div>
          <h2 class="text-xl font-semibold">
            搜索结果
            <span v-if="searchQuery" class="text-muted-foreground font-normal">
              - "{{ searchQuery }}"
            </span>
          </h2>
          <p class="text-sm text-muted-foreground">
            {{ bookmarksStore.bookmarks.length }} 个结果
            <span v-if="searchTime">({{ searchTime }}秒)</span>
          </p>
        </div>
        
        <Button
          v-if="searchQuery"
          @click="clearSearch"
          variant="outline"
          size="sm"
        >
          清除搜索
        </Button>
      </div>

      <!-- Results list -->
      <div v-if="bookmarksStore.bookmarks.length > 0" class="space-y-4">
        <div
          v-for="bookmark in searchResults"
          :key="bookmark.id"
          class="group bg-card border border-border/50 rounded-lg p-4 hover:shadow-sm transition-all duration-200"
        >
          <div class="flex items-start justify-between gap-4">
            <!-- Bookmark info -->
            <div class="flex-1 min-w-0">
              <div class="flex items-center gap-2 mb-1">
                <h3 class="font-medium truncate hover:text-primary cursor-pointer" @click="openBookmark(bookmark.url)">
                  {{ highlightText(bookmark.title, searchQuery) }}
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
                {{ highlightText(bookmark.description, searchQuery) }}
              </p>
              
              <!-- Tags -->
              <div v-if="bookmark.tags && bookmark.tags.length > 0" class="flex flex-wrap gap-1 mb-2">
                <span
                  v-for="tag in bookmark.tags"
                  :key="tag.id"
                  class="inline-flex items-center px-2 py-0.5 rounded-full text-xs"
                  :style="{ backgroundColor: tag.color + '20', color: tag.color }"
                >
                  {{ tag.name }}
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
            </div>
          </div>
        </div>
        
        <!-- Load more -->
        <div v-if="bookmarksStore.hasMore" class="text-center pt-4">
          <button
            @click="handleLoadMore"
            :disabled="isLoadingMore"
            class="px-4 py-2 text-sm border border-border rounded-md hover:bg-accent transition-colors disabled:opacity-50"
          >
            {{ isLoadingMore ? '加载中...' : '加载更多' }}
          </button>
        </div>
      </div>

      <!-- Loading state -->
      <div v-if="isSearching" class="text-center py-12">
        <div class="mx-auto h-16 w-16 rounded-full bg-primary/10 flex items-center justify-center mb-4">
          <div class="animate-spin h-8 w-8 border-2 border-primary border-t-transparent rounded-full"></div>
        </div>
        <h3 class="text-xl font-semibold mb-2">正在搜索</h3>
        <p class="text-muted-foreground">
          正在查找与 "{{ searchQuery }}" 相关的书签...
        </p>
      </div>

      <!-- No results -->
      <div v-else-if="searchQuery && !bookmarksStore.isLoading && !searchError && bookmarksStore.bookmarks.length === 0" class="text-center py-12">
        <div class="mx-auto h-16 w-16 rounded-full bg-muted flex items-center justify-center mb-4">
          <span class="text-2xl">🔍</span>
        </div>
        <h3 class="text-xl font-semibold mb-2">未找到结果</h3>
        <p class="text-muted-foreground mb-4">
          没有找到与 "{{ searchQuery }}" 相关的书签
        </p>
        <div class="space-y-2">
          <p class="text-sm text-muted-foreground">建议：</p>
          <ul class="text-sm text-muted-foreground space-y-1">
            <li>• 检查拼写是否正确</li>
            <li>• 尝试使用更通用的关键词</li>
            <li>• 减少筛选条件</li>
          </ul>
        </div>
      </div>
    </div>

    <!-- Initial state -->
    <div v-else class="max-w-4xl mx-auto">
      <div class="bg-card rounded-xl border border-border/50">
        <div class="p-8">
          <div class="text-center py-12">
            <div class="mx-auto h-16 w-16 rounded-full bg-primary/10 flex items-center justify-center mb-4">
              <span class="text-2xl">🔍</span>
            </div>
            <h3 class="text-xl font-semibold mb-2">开始搜索</h3>
            <p class="text-muted-foreground">输入关键词查找您的书签内容</p>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted } from 'vue'
import { useBookmarksStore } from '@/stores/bookmarks'
import { useCollectionsStore } from '@/stores/collections'
import { useTagsStore } from '@/stores/tags'
import { useSearch } from '@/composables/useSearch'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import type { Bookmark, SearchFilters } from '@/types'

const bookmarksStore = useBookmarksStore()
const collectionsStore = useCollectionsStore()
const tagsStore = useTagsStore()

// 使用搜索组合式函数
const {
  searchQuery,
  isSearching,
  hasSearched,
  searchTime,
  searchError,
  isTyping,
  isLoadingMore,
  filters,
  queryStatus,
  canSearch,
  handleInput,
  triggerSearch,
  retrySearch,
  clearSearch,
  updateFilter,
  applySuggestion,
  cleanup
} = useSearch(async (query: string, filters: SearchFilters) => {
  // 构建搜索参数
  const searchParams: any = {
    search: query,
    sort_by: filters.sortBy
  }

  if (filters.collectionId) {
    searchParams.collection_id = parseInt(filters.collectionId)
  }

  if (filters.tagId) {
    searchParams.tags = [parseInt(filters.tagId)]
  }

  // 执行搜索
  await bookmarksStore.searchBookmarks(searchParams, true)
})

// 搜索结果和建议
const searchResults = computed(() => bookmarksStore.bookmarks)

const searchSuggestions = computed(() => {
  const allTags = tagsStore.tags.map(tag => tag.name)
  const popularTerms = ['Vue.js', 'JavaScript', 'React', 'CSS', 'TypeScript']
  return [...allTags.slice(0, 5), ...popularTerms].slice(0, 8)
})

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

// 高亮搜索文本
const highlightText = (text: string, query: string) => {
  if (!query.trim()) return text

  const regex = new RegExp(`(${query})`, 'gi')
  return text.replace(regex, '<mark class="bg-yellow-200 text-yellow-800">$1</mark>')
}

// 打开书签
const openBookmark = (url: string) => {
  window.open(url, '_blank')
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

// 手动搜索处理函数（用于表单提交）
const handleSearch = async () => {
  await triggerSearch()
}

// 处理加载更多（结合store的分页逻辑）
const handleLoadMore = async () => {
  if (isLoadingMore.value || !bookmarksStore.hasMore) return

  isLoadingMore.value = true

  try {
    const searchParams: any = {
      search: searchQuery.value.trim(),
      sort_by: filters.value.sortBy
    }

    if (filters.value.collectionId) {
      searchParams.collection_id = parseInt(filters.value.collectionId)
    }

    if (filters.value.tagId) {
      searchParams.tags = [parseInt(filters.value.tagId)]
    }

    await bookmarksStore.searchBookmarks(searchParams, false)

  } catch (error) {
    console.error('加载更多结果失败:', error)
  } finally {
    isLoadingMore.value = false
  }
}

// 组件初始化和清理
onMounted(async () => {
  await Promise.all([
    collectionsStore.fetchCollections(),
    tagsStore.fetchTags()
  ])
})

onUnmounted(() => {
  // 清理防抖定时器
  cleanup()
})
</script>