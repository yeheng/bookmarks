<template>
  <div class="container mx-auto px-4 py-8">
    <!-- Page header -->
    <div class="mb-8">
      <div class="flex items-center justify-between mb-4">
        <div>
          <h1 class="text-3xl font-bold tracking-tight">收藏夹</h1>
          <p class="text-muted-foreground mt-2">按主题整理您的书签</p>
        </div>
      </div>
    </div>

    <!-- Content area -->
    <div class="bg-card rounded-xl border border-border/50">
      <div class="p-8">
        <EmptyState
          title="暂无收藏夹"
          description="创建收藏夹来按主题组织您的书签"
          action-text="创建收藏夹"
          icon-type="folder"
          @action="handleCreateCollection"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { EmptyState } from '@/components/ui/empty-state'

const handleCreateCollection = () => {
  // 创建收藏夹逻辑
}
</script>