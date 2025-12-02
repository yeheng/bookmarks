<template>
  <div class="container mx-auto px-4 py-8">
    <!-- Page header -->
    <div class="mb-8">
      <div class="flex items-center justify-between mb-4">
        <div>
          <h1 class="text-3xl font-bold tracking-tight">标签</h1>
          <p class="text-muted-foreground mt-2">使用标签分类您的书签</p>
        </div>
        <Button>
          <span class="mr-2">+</span>
          添加标签
        </Button>
      </div>
    </div>

    <!-- Content area -->
    <div class="bg-card rounded-xl border border-border/50">
      <div class="p-8">
        <div class="text-center py-12">
          <div class="mx-auto h-16 w-16 rounded-full bg-primary/10 flex items-center justify-center mb-4">
            <span class="text-2xl">🏷️</span>
          </div>
          <h3 class="text-xl font-semibold mb-2">暂无标签</h3>
          <p class="text-muted-foreground mb-6">开始添加您的第一个标签</p>
          <Button>
            <span class="mr-2">+</span>
            添加标签
          </Button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { Button } from '@/components/ui/button'
// Tags page logic will be implemented later
</script>