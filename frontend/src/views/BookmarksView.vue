<template>
  <div class="container mx-auto px-4 py-8">
    <!-- Page header -->
    <div class="mb-8">
      <div class="flex items-center justify-between mb-4">
        <div>
          <h1 class="text-3xl font-bold tracking-tight">书签</h1>
          <p class="text-muted-foreground mt-2">管理您的网页书签</p>
        </div>
      </div>
    </div>

    <!-- Content area -->
    <div class="bg-card rounded-xl border border-border/50">
      <div class="p-8">
        <EmptyState
          title="暂无书签"
          description="使用顶部导航栏的 + 按钮添加第一个书签"
          action-text="添加书签"
          icon-type="bookmark"
          @action="handleAddBookmark"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { EmptyState } from '@/components/ui/empty-state'

const handleAddBookmark = () => {
  // 触发全局添加书签功能
  // 这里可以通过事件总线或者状态管理来触发导航栏的添加书签模态框
}
</script>