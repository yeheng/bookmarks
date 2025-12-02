<template>
  <div class="container mx-auto px-4 py-8">
    <!-- Page header -->
    <div class="mb-8">
      <div class="flex items-center justify-between mb-4">
        <div>
          <h1 class="text-3xl font-bold tracking-tight">标签</h1>
          <p class="text-muted-foreground mt-2">使用标签分类您的书签</p>
        </div>
      </div>
    </div>

    <!-- Content area -->
    <div class="bg-card rounded-xl border border-border/50">
      <div class="p-8">
        <EmptyState
          title="暂无标签"
          description="为书签添加标签以便更好地分类和搜索"
          action-text="添加标签"
          icon-type="tag"
          @action="handleAddTag"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { EmptyState } from '@/components/ui/empty-state'

const handleAddTag = () => {
  // 添加标签逻辑
}
</script>