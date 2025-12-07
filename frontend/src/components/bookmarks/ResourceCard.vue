<template>
  <div
    class="group bg-card border border-border/50 rounded-lg p-4 hover:shadow-lg transition-all duration-200 hover:scale-[1.02] cursor-pointer"
    @click="handleResourceClick"
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

    <!-- 动态内容组件 -->
    <component :is="resourceComponent" :resource="resource" />

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
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { Resource, ResourceType } from '@/types'
import { ResourceLink, ResourceNote, ResourceSnippet, ResourceFile } from '@/components/resources'

const props = defineProps<{
  resource: Resource
}>()

const emit = defineEmits<{
  toggleFavorite: [resource: Resource]
  edit: [resource: Resource]
  delete: [id: number]
  tagClick: [tagName: string]
}>()

// 动态组件映射
const resourceComponents = {
  link: ResourceLink,
  note: ResourceNote,
  snippet: ResourceSnippet,
  file: ResourceFile
}

// 计算当前资源对应的组件
const resourceComponent = computed(() => {
  return resourceComponents[props.resource.type] || ResourceNote
})

// 处理资源点击
const handleResourceClick = () => {
  if (props.resource.type === 'link' && props.resource.url) {
    window.open(props.resource.url, '_blank')
  } else {
    // 对于非链接资源，触发编辑操作
    emit('edit', props.resource)
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
