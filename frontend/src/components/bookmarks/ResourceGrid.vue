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
        <ResourceCard
          v-for="resource in items"
          :key="resource.id"
          :resource="resource"
          @toggle-favorite="$emit('toggleFavorite', $event)"
          @edit="$emit('edit', $event)"
          @delete="$emit('delete', $event)"
          @tag-click="$emit('tagClick', $event)"
        />
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
import ResourceCard from './ResourceCard.vue'
import type { Resource } from '@/types'

// Props
interface Props {
  resources: Resource[]
  isLoading: boolean
  isLoadingMore: boolean
  hasMore: boolean
}

defineProps<Props>()

// Emits
defineEmits<{
  loadMore: []
  toggleFavorite: [resource: Resource]
  edit: [resource: Resource]
  delete: [id: number]
  tagClick: [tagName: string]
  addResource: []
}>()
</script>
