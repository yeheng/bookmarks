<template>
  <InfiniteScroll
    :items="bookmarks"
    :is-loading="isLoading"
    :is-loading-more="isLoadingMore"
    :has-more="hasMore"
    @load-more="$emit('loadMore')"
  >
    <template #default="{ items, isLoading: loading }">
      <!-- 书签网格 -->
      <div v-if="!loading && items && items.length > 0" class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
        <div
          v-for="bookmark in items"
          :key="bookmark.id"
          class="group bg-card border border-border/50 rounded-lg p-4 hover:shadow-lg transition-all duration-200 hover:scale-[1.02] cursor-pointer"
        >
          <!-- 卡片头部：图标和操作按钮 -->
          <div class="flex items-start justify-between mb-3">
            <div class="flex items-center gap-2">
              <!-- Favicon -->
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

              <!-- 状态指示器 -->
              <div class="flex items-center gap-1">
                <span v-if="bookmark.is_favorite" class="text-yellow-500" title="收藏">⭐</span>
                <span v-if="bookmark.is_read" class="text-green-500" title="已读">✓</span>
                <span v-if="bookmark.is_archived" class="text-gray-500" title="归档">📁</span>
              </div>
            </div>

            <!-- 操作按钮 -->
            <div class="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
              <button
                @click.stop="$emit('toggleFavorite', bookmark)"
                class="p-1.5 rounded hover:bg-accent transition-colors"
                :title="bookmark.is_favorite ? '取消收藏' : '添加收藏'"
              >
                <span :class="bookmark.is_favorite ? 'text-yellow-500' : 'text-muted-foreground'">
                  {{ bookmark.is_favorite ? '⭐' : '☆' }}
                </span>
              </button>
              <button
                @click.stop="$emit('edit', bookmark)"
                class="p-1.5 rounded hover:bg-accent transition-colors text-muted-foreground"
                title="编辑"
              >
                ✏️
              </button>
              <button
                @click.stop="$emit('delete', bookmark.id)"
                class="p-1.5 rounded hover:bg-accent transition-colors text-red-500"
                title="删除"
              >
                🗑️
              </button>
            </div>
          </div>

          <!-- 标题 -->
          <h3
            class="font-medium text-sm mb-2 line-clamp-2 hover:text-primary transition-colors"
            @click="openBookmark(bookmark.url)"
          >
            {{ bookmark.title }}
          </h3>

          <!-- URL -->
          <p class="text-xs text-muted-foreground mb-2 truncate">{{ bookmark.url }}</p>

          <!-- 描述 -->
          <p v-if="bookmark.description" class="text-xs text-muted-foreground mb-3 line-clamp-3">
            {{ bookmark.description }}
          </p>

          <!-- 标签 -->
          <div v-if="bookmark.tags && bookmark.tags.length > 0" class="flex flex-wrap gap-1 mb-3">
            <span
              v-for="tag in bookmark.tags"
              :key="tag"
              @click.stop="$emit('tagClick', tag)"
              class="inline-flex items-center px-2 py-0.5 rounded-full text-xs bg-blue-100 text-blue-800 hover:bg-blue-200 cursor-pointer transition-colors"
            >
              {{ tag }}
            </span>
          </div>

          <!-- 元信息 -->
          <div class="flex items-center justify-between text-xs text-muted-foreground pt-3 border-t border-border/30">
            <div class="flex items-center gap-3">
              <span v-if="bookmark.collection_name">{{ bookmark.collection_name }}</span>
            </div>
            <span>{{ formatDate(bookmark.created_at) }}</span>
          </div>
        </div>
      </div>

      <!-- 加载状态 -->
      <div v-else-if="loading" class="flex justify-center py-12">
        <div class="text-center">
          <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary mx-auto mb-4"></div>
          <p class="text-muted-foreground">加载书签中...</p>
        </div>
      </div>

      <!-- 空状态 -->
      <div v-else class="flex justify-center py-12">
        <EmptyState
          title="暂无书签"
          description="使用顶部导航栏的 + 按钮添加第一个书签"
          action-text="添加书签"
          icon-type="bookmark"
          @action="$emit('addBookmark')"
        />
      </div>
    </template>
  </InfiniteScroll>
</template>

<script setup lang="ts">
import { EmptyState } from '@/components/ui/empty-state'
import { InfiniteScroll } from '@/components/ui/infinite-scroll'
import type { Bookmark } from '@/types'

// Props
interface Props {
  bookmarks: Bookmark[]
  isLoading: boolean
  isLoadingMore: boolean
  hasMore: boolean
}

defineProps<Props>()

// Emits
defineEmits<{
  loadMore: []
  toggleFavorite: [bookmark: Bookmark]
  edit: [bookmark: Bookmark]
  delete: [id: number]
  tagClick: [tagName: string]
  addBookmark: []
}>()

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

// 处理 favicon 加载错误
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
</script>
