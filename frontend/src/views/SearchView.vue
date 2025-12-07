<template>
  <div class="container mx-auto px-4 py-8">
    <!-- Page header -->
    <div class="mb-8">
      <div class="max-w-2xl mx-auto">
        <h1 class="text-3xl font-bold tracking-tight text-center mb-4">搜索</h1>
        <p class="text-muted-foreground text-center mb-8">在您的资源中快速查找内容</p>

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
              placeholder="搜索资源、收藏夹或标签（至少3个字符）..."
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
            {{ resourcesStore.resources?.length || 0 }} 个结果
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
      <div v-if="resourcesStore.resources?.length > 0" class="space-y-4">
        <div
          v-for="resource in searchResults"
          :key="resource.id"
          class="group bg-card border border-border/50 rounded-lg p-4 hover:shadow-sm transition-all duration-200"
        >
          <div class="flex items-start justify-between gap-4">
            <!-- Resource info -->
            <div class="flex-1 min-w-0">
              <div class="flex items-center gap-2 mb-1">
                <!-- Resource type indicator -->
                <div class="w-4 h-4 rounded flex items-center justify-center flex-shrink-0" :class="getTypeIconClass(resource.type)">
                  <span class="text-xs">{{ getTypeIcon(resource.type) }}</span>
                </div>
                <h3 class="font-medium truncate hover:text-primary cursor-pointer" @click="openResource(resource)">
                  {{ highlightText(resource.title, searchQuery) }}
                </h3>
                <!-- Status indicators -->
                <div class="flex items-center gap-1">
                  <span v-if="resource.is_favorite" class="text-yellow-500" title="收藏">⭐</span>
                  <span v-if="resource.is_read" class="text-green-500" title="已读">✓</span>
                  <span v-if="resource.is_archived" class="text-gray-500" title="归档">📁</span>
                  <span class="px-2 py-0.5 rounded-full bg-gray-100 text-gray-700 text-[10px]">{{ getTypeLabel(resource.type) }}</span>
                </div>
              </div>

              <!-- Resource-specific content -->
              <div class="text-sm text-muted-foreground mb-2">
                <div v-if="resource.type === 'link' && resource.url" class="truncate">
                  {{ resource.url }}
                </div>
                <div v-else-if="resource.type === 'note' && resource.content" class="line-clamp-2">
                  <span class="text-gray-500 text-xs">📝 笔记片段：</span>
                  {{ highlightText(truncateText(resource.content, 150), searchQuery) }}
                </div>
                <div v-else-if="resource.type === 'snippet' && resource.content" class="line-clamp-2 font-mono bg-gray-50 dark:bg-gray-800 p-1 rounded">
                  <span class="text-gray-500 text-xs">💻 代码片段：</span>
                  {{ highlightText(truncateText(resource.content, 120), searchQuery) }}
                </div>
                <div v-else-if="resource.type === 'file' && resource.source" class="truncate">
                  <span class="text-orange-500 text-xs">📄 文件：</span>
                  {{ resource.source }}
                  <span v-if="resource.mime_type" class="text-gray-400">({{ resource.mime_type }})</span>
                </div>
              </div>

              <p v-if="resource.description" class="text-sm text-muted-foreground mb-2 line-clamp-2">
                {{ highlightText(resource.description, searchQuery) }}
              </p>

              <!-- Tags -->
              <div v-if="resource.tags && resource.tags.length > 0" class="flex flex-wrap gap-1 mb-2">
                <span
                  v-for="tag in resource.tags"
                  :key="tag"
                  class="inline-flex items-center px-2 py-0.5 rounded-full text-xs bg-blue-100 text-blue-800 hover:bg-blue-200"
                >
                  {{ tag }}
                </span>
              </div>

              <!-- Meta info -->
              <div class="flex items-center gap-4 text-xs text-muted-foreground">
                <span>{{ formatDate(resource.created_at) }}</span>
                <span v-if="resource.visit_count > 0">{{ resource.visit_count }} 次访问</span>
                <span v-if="resource.collection_name">{{ resource.collection_name }}</span>
                <span v-if="resource.reference_count > 0">{{ resource.reference_count }} 个引用</span>
              </div>
            </div>

            <!-- Actions -->
            <div class="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
              <button
                @click="toggleFavorite(resource)"
                class="p-1.5 rounded hover:bg-accent transition-colors"
                :title="resource.is_favorite ? '取消收藏' : '添加收藏'"
              >
                <span :class="resource.is_favorite ? 'text-yellow-500' : 'text-muted-foreground'">
                  {{ resource.is_favorite ? '⭐' : '☆' }}
                </span>
              </button>
              <button
                @click="editResource(resource)"
                class="p-1.5 rounded hover:bg-accent transition-colors text-muted-foreground"
                title="编辑"
              >
                ✏️
              </button>
            </div>
          </div>
        </div>

        <!-- Load more -->
        <div v-if="resourcesStore.hasMore" class="text-center pt-4">
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
      <div v-else-if="searchQuery && !resourcesStore.isLoading && !searchError && (!resourcesStore.resources || resourcesStore.resources.length === 0)" class="text-center py-12">
        <div class="mx-auto h-16 w-16 rounded-full bg-muted flex items-center justify-center mb-4">
          <span class="text-2xl">🔍</span>
        </div>
        <h3 class="text-xl font-semibold mb-2">未找到结果</h3>
        <p class="text-muted-foreground mb-4">
          没有找到与 "{{ searchQuery }}" 相关的资源
        </p>
        <div class="space-y-2">
          <p class="text-sm text-muted-foreground">建议：</p>
          <ul class="text-sm text-muted-foreground space-y-1">
            <li>• 检查拼写是否正确</li>
            <li>• 尝试使用更通用的关键词</li>
            <li>• 减少筛选条件</li>
            <li>• 尝试搜索不同类型的资源（笔记、代码片段等）</li>
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
import { useResourcesStore } from '@/stores/resources'
import { useCollectionsStore } from '@/stores/collections'
import { useTagsStore } from '@/stores/tags'
import { useSearch } from '@/composables/useSearch'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import type { Resource, ResourceType, SearchFilters } from '@/types'

const resourcesStore = useResourcesStore()
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
    q: query, // 使用新的API参数名
    sort_by: filters.sortBy
  }

  if (filters.collectionId) {
    searchParams.collection_id = parseInt(filters.collectionId)
  }

  if (filters.tagId) {
    searchParams.tags = filters.tagId // 使用字符串而不是数组
  }

  // 执行搜索
  await resourcesStore.fetchResources(searchParams, true)
})

// 搜索结果和建议
const searchResults = computed(() => resourcesStore.resources || [])

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

// 转义正则表达式特殊字符
const escapeRegex = (str: string): string => {
  return str.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
}

// 转义 HTML 特殊字符
const escapeHtml = (str: string): string => {
  const htmlEscapeMap: Record<string, string> = {
    '&': '&amp;',
    '<': '&lt;',
    '>': '&gt;',
    '"': '&quot;',
    "'": '&#039;'
  }
  return str.replace(/[&<>"']/g, c => htmlEscapeMap[c] || c)
}

// 高亮搜索文本（XSS 安全）
const highlightText = (text: string, query: string) => {
  if (!query.trim()) return escapeHtml(text)

  const regex = new RegExp(`(${escapeRegex(query)})`, 'gi')
  return escapeHtml(text).replace(regex, '<mark class="bg-yellow-200 text-yellow-800">$1</mark>')
}

// 截断文本
const truncateText = (text: string, maxLength: number): string => {
  if (text.length <= maxLength) return text
  return text.substring(0, maxLength) + '...'
}

// 获取资源类型图标
const getTypeIcon = (type: ResourceType): string => {
  const icons: Record<ResourceType, string> = {
    link: '🔗',
    note: '📝',
    snippet: '💻',
    file: '📄'
  }
  return icons[type] || '📌'
}

// 获取类型图标样式类
const getTypeIconClass = (type: ResourceType): string => {
  const classes: Record<ResourceType, string> = {
    link: 'bg-blue-100 text-blue-700',
    note: 'bg-green-100 text-green-700',
    snippet: 'bg-purple-100 text-purple-700',
    file: 'bg-gray-100 text-gray-700'
  }
  return classes[type] || 'bg-accent text-accent-foreground'
}

// 获取类型标签
const getTypeLabel = (type: ResourceType): string => {
  const labels: Record<ResourceType, string> = {
    link: '链接',
    note: '笔记',
    snippet: '代码',
    file: '文件'
  }
  return labels[type] || '资源'
}

// 打开资源
const openResource = (resource: Resource) => {
  if (resource.type === 'link' && resource.url) {
    window.open(resource.url, '_blank')
  }
  // 对于非链接类型，暂不处理（可以扩展为打开详情页）
}

// 切换收藏状态
const toggleFavorite = async (resource: Resource) => {
  try {
    const updateData = { is_favorite: !resource.is_favorite }
    await resourcesStore.updateResource(resource.id, updateData)
  } catch (error) {
    console.error('切换收藏状态失败:', error)
  }
}

// 编辑资源
const editResource = (resource: Resource) => {
  // TODO: 实现编辑资源功能，可以打开编辑模态框
}

// 手动搜索处理函数（用于表单提交）
const handleSearch = async () => {
  await triggerSearch()
}

// 处理加载更多（结合store的分页逻辑）
const handleLoadMore = async () => {
  if (isLoadingMore.value || !resourcesStore.hasMore) return

  isLoadingMore.value = true

  try {
    const searchParams: any = {
      q: searchQuery.value.trim(), // 使用新的API参数名
      sort_by: filters.value.sortBy
    }

    if (filters.value.collectionId) {
      searchParams.collection_id = parseInt(filters.value.collectionId)
    }

    if (filters.value.tagId) {
      searchParams.tags = filters.value.tagId // 使用字符串而不是数组
    }

    await resourcesStore.fetchResources(searchParams, false)

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