<template>
  <InfiniteScroll
    :items="resources"
    :is-loading="isLoading"
    :is-loading-more="isLoadingMore"
    :has-more="hasMore"
    @load-more="$emit('loadMore')"
  >
    <template #default="{ items, isLoading: loading }">
      <!-- 资源网格 -->
      <div v-if="!loading && items && items.length > 0" class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
        <div
          v-for="resource in items"
          :key="resource.id"
          class="group bg-card border border-border/50 rounded-lg p-4 hover:shadow-lg transition-all duration-200 hover:scale-[1.02] cursor-pointer"
          @click="handleResourceClick(resource)"
        >
          <!-- 卡片头部：类型图标和操作按钮 -->
          <div class="flex items-start justify-between mb-3">
            <div class="flex items-center gap-2">
              <!-- 类型图标 -->
              <div class="w-5 h-5 rounded flex items-center justify-center flex-shrink-0" :class="getTypeIconClass(resource.type)">
                <span class="text-xs">{{ getTypeIcon(resource.type) }}</span>
              </div>

              <!-- 状态指示器 -->
              <div class="flex items-center gap-1">
                <span v-if="resource.is_favorite" class="text-yellow-500" title="收藏">⭐</span>
                <span v-if="resource.is_read" class="text-green-500" title="已读">✓</span>
                <span v-if="resource.is_archived" class="text-gray-500" title="归档">📁</span>
              </div>
            </div>

            <!-- 操作按钮 -->
            <div class="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
              <button
                @click.stop="$emit('toggleFavorite', resource)"
                class="p-1.5 rounded hover:bg-accent transition-colors"
                :title="resource.is_favorite ? '取消收藏' : '添加收藏'"
              >
                <span :class="resource.is_favorite ? 'text-yellow-500' : 'text-muted-foreground'">
                  {{ resource.is_favorite ? '⭐' : '☆' }}
                </span>
              </button>
              <button
                @click.stop="$emit('edit', resource)"
                class="p-1.5 rounded hover:bg-accent transition-colors text-muted-foreground"
                title="编辑"
              >
                ✏️
              </button>
              <button
                @click.stop="$emit('delete', resource.id)"
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
          >
            {{ resource.title }}
          </h3>

          <!-- 内容预览 -->
          <div v-if="resource.type === 'link' && resource.url" class="text-xs text-muted-foreground mb-2 truncate">
            {{ resource.url }}
          </div>
          <div v-else-if="resource.type === 'note' && resource.content" class="text-xs text-muted-foreground mb-2 line-clamp-3">
            {{ truncateContent(resource.content, 100) }}
          </div>
          <div v-else-if="resource.type === 'snippet' && resource.content" class="text-xs text-muted-foreground mb-2">
            <div class="bg-gray-50 dark:bg-gray-800 border rounded p-2 font-mono leading-relaxed">
              <div class="flex items-start justify-between mb-1">
                <span class="text-gray-500 text-[10px]">💻 代码片段</span>
                <button
                  @click.stop="copyToClipboard(resource.content)"
                  class="text-gray-400 hover:text-gray-600 text-[10px] px-1 py-0.5 rounded hover:bg-gray-200 dark:hover:bg-gray-700 transition-colors"
                  title="复制代码"
                >
                  📋
                </button>
              </div>
              <pre class="whitespace-pre-wrap break-words line-clamp-3">{{ truncateContent(resource.content, 120) }}</pre>
              <div v-if="resource.content.length > 120" class="text-blue-600 text-[10px] mt-1">
                ... 点击查看更多
              </div>
            </div>
          </div>
          <div v-else-if="resource.type === 'file'" class="text-xs text-muted-foreground mb-2">
            <div class="bg-orange-50 dark:bg-orange-900/20 border border-orange-200 dark:border-orange-800 rounded p-2">
              <div class="flex items-center justify-between mb-1">
                <span class="text-orange-600 dark:text-orange-400 text-[10px]">📄 文件</span>
                <div class="flex gap-1">
                  <button
                    v-if="resource.source"
                    @click.stop="copyFilePath(resource.source)"
                    class="text-orange-400 hover:text-orange-600 text-[10px] px-1 py-0.5 rounded hover:bg-orange-200 dark:hover:bg-orange-800 transition-colors"
                    title="复制路径"
                  >
                    📋
                  </button>
                  <button
                    v-if="resource.url"
                    @click.stop="downloadFile(resource.url)"
                    class="text-orange-400 hover:text-orange-600 text-[10px] px-1 py-0.5 rounded hover:bg-orange-200 dark:hover:bg-orange-800 transition-colors"
                    title="下载文件"
                  >
                    ⬇️
                  </button>
                </div>
              </div>
              <div class="font-medium truncate">
                {{ resource.source || resource.mime_type || '未知文件' }}
              </div>
              <div v-if="resource.mime_type" class="text-orange-500 text-[10px] mt-1">
                类型: {{ resource.mime_type }}
              </div>
            </div>
          </div>

          <!-- 描述 -->
          <p v-if="resource.description" class="text-xs text-muted-foreground mb-3 line-clamp-3">
            {{ resource.description }}
          </p>

          <!-- 标签 -->
          <div v-if="resource.tags && resource.tags.length > 0" class="flex flex-wrap gap-1 mb-3">
            <span
              v-for="tag in resource.tags"
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
              <span v-if="resource.collection_name">{{ resource.collection_name }}</span>
              <span class="px-2 py-0.5 rounded-full bg-gray-100 text-gray-700">{{ getTypeLabel(resource.type) }}</span>
            </div>
            <span>{{ formatDate(resource.created_at) }}</span>
          </div>
        </div>
      </div>

      <!-- 加载状态 -->
      <div v-else-if="loading" class="flex justify-center py-12">
        <div class="text-center">
          <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary mx-auto mb-4"></div>
          <p class="text-muted-foreground">加载资源中...</p>
        </div>
      </div>

      <!-- 空状态 -->
      <div v-else class="flex justify-center py-12">
        <EmptyState
          title="暂无资源"
          description="使用顶部导航栏的 + 按钮添加第一个资源"
          action-text="添加资源"
          icon-type="bookmark"
          @action="$emit('addResource')"
        />
      </div>
    </template>
  </InfiniteScroll>
</template>

<script setup lang="ts">
import { EmptyState } from '@/components/ui/empty-state'
import { InfiniteScroll } from '@/components/ui/infinite-scroll'
import type { Resource, ResourceType } from '@/types'

// Props
interface Props {
  resources: Resource[]
  isLoading: boolean
  isLoadingMore: boolean
  hasMore: boolean
}

defineProps<Props>()

// Emits
const emit = defineEmits<{
  loadMore: []
  toggleFavorite: [resource: Resource]
  edit: [resource: Resource]
  delete: [id: number]
  tagClick: [tagName: string]
  addResource: []
}>()

// 处理资源点击
const handleResourceClick = (resource: Resource) => {
  if (resource.type === 'link' && resource.url) {
    window.open(resource.url, '_blank')
  } else {
    // 对于非链接资源，触发编辑操作
    emit('edit', resource)
  }
}

// 获取类型图标
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

// 截断内容
const truncateContent = (content: string, maxLength: number): string => {
  if (content.length <= maxLength) return content
  return content.substring(0, maxLength) + '...'
}

// 复制到剪贴板
const copyToClipboard = async (text: string) => {
  try {
    await navigator.clipboard.writeText(text)
    // 可以添加 toast 提示，这里简化处理
    console.log('代码已复制到剪贴板')
  } catch (err) {
    console.error('复制失败:', err)
    // 降级处理
    const textArea = document.createElement('textarea')
    textArea.value = text
    document.body.appendChild(textArea)
    textArea.select()
    document.execCommand('copy')
    document.body.removeChild(textArea)
  }
}

// 复制文件路径
const copyFilePath = async (filePath: string) => {
  try {
    await navigator.clipboard.writeText(filePath)
    console.log('文件路径已复制到剪贴板')
  } catch (err) {
    console.error('复制失败:', err)
  }
}

// 下载文件
const downloadFile = (url: string) => {
  try {
    const link = document.createElement('a')
    link.href = url
    link.download = '' // 让浏览器自动从 URL 推断文件名
    link.target = '_blank'
    document.body.appendChild(link)
    link.click()
    document.body.removeChild(link)
    console.log('文件下载已开始')
  } catch (err) {
    console.error('下载失败:', err)
    // 降级处理：直接在新标签页打开
    window.open(url, '_blank')
  }
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
