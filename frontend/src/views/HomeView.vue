<template>
  <div class="container mx-auto px-4 py-12">
    <!-- Hero section -->
    <div class="max-w-3xl mx-auto text-center mb-16">
      <h1 class="text-4xl font-bold tracking-tight mb-4">Bookmarks</h1>
      <p class="text-lg text-muted-foreground">
        简洁高效的书签管理工具，助您整理和发现网络资源
      </p>
    </div>

    <!-- Stats overview -->
    <div class="grid grid-cols-1 md:grid-cols-3 gap-6 mb-12">
      <div class="bg-card rounded-xl p-6 border border-border/50">
        <div class="flex items-center justify-between mb-4">
          <h3 class="text-lg font-semibold">书签总数</h3>
          <div class="h-10 w-10 rounded-lg bg-primary/10 flex items-center justify-center">
            <span class="text-sm font-medium text-primary">📚</span>
          </div>
        </div>
        <p class="text-3xl font-bold mb-2">0</p>
        <p class="text-sm text-muted-foreground">尚未添加书签</p>
      </div>

      <div class="bg-card rounded-xl p-6 border border-border/50">
        <div class="flex items-center justify-between mb-4">
          <h3 class="text-lg font-semibold">收藏夹</h3>
          <div class="h-10 w-10 rounded-lg bg-primary/10 flex items-center justify-center">
            <span class="text-sm font-medium text-primary">📁</span>
          </div>
        </div>
        <p class="text-3xl font-bold mb-2">0</p>
        <p class="text-sm text-muted-foreground">尚未创建收藏夹</p>
      </div>

      <div class="bg-card rounded-xl p-6 border border-border/50">
        <div class="flex items-center justify-between mb-4">
          <h3 class="text-lg font-semibold">标签</h3>
          <div class="h-10 w-10 rounded-lg bg-primary/10 flex items-center justify-center">
            <span class="text-sm font-medium text-primary">🏷️</span>
          </div>
        </div>
        <p class="text-3xl font-bold mb-2">0</p>
        <p class="text-sm text-muted-foreground">尚未添加标签</p>
      </div>
    </div>

    <!-- Quick actions -->
    <div class="max-w-2xl mx-auto">
      <h2 class="text-2xl font-semibold mb-6 text-center">快速开始</h2>
      <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
        <RouterLink to="/bookmarks" class="block">
          <div class="bg-card rounded-xl p-6 border border-border/50 hover:border-primary/50 transition-colors">
            <div class="flex items-center gap-3 mb-3">
              <div class="h-8 w-8 rounded-md bg-primary/10 flex items-center justify-center">
                <span class="text-sm font-medium text-primary">+</span>
              </div>
              <h3 class="text-lg font-medium">添加书签</h3>
            </div>
            <p class="text-sm text-muted-foreground">保存您喜欢的网页链接</p>
          </div>
        </RouterLink>

        <RouterLink to="/collections" class="block">
          <div class="bg-card rounded-xl p-6 border border-border/50 hover:border-primary/50 transition-colors">
            <div class="flex items-center gap-3 mb-3">
              <div class="h-8 w-8 rounded-md bg-primary/10 flex items-center justify-center">
                <span class="text-sm font-medium text-primary">📂</span>
              </div>
              <h3 class="text-lg font-medium">创建收藏夹</h3>
            </div>
            <p class="text-sm text-muted-foreground">按主题整理您的书签</p>
          </div>
        </RouterLink>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { RouterLink } from 'vue-router'
// Home page logic will be implemented later
</script>