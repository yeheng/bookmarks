<template>
  <div class="container mx-auto px-4 py-8">
    <!-- 简化的标题区域 -->
    <div class="max-w-2xl mx-auto text-center mb-8">
      <h1 class="text-3xl font-bold tracking-tight mb-2">Bookmarks</h1>
      <p class="text-muted-foreground">
        使用 ⌘K 快速搜索，点击 + 添加书签
      </p>
    </div>

    <!-- 简化的统计信息 -->
    <div class="max-w-4xl mx-auto mb-8">
      <div class="grid grid-cols-3 gap-4 text-center">
        <div class="py-4">
          <p class="text-2xl font-bold">{{ bookmarksCount }}</p>
          <p class="text-sm text-muted-foreground">书签</p>
        </div>
        <div class="py-4">
          <p class="text-2xl font-bold">{{ collectionsCount }}</p>
          <p class="text-sm text-muted-foreground">收藏夹</p>
        </div>
        <div class="py-4">
          <p class="text-2xl font-bold">{{ tagsCount }}</p>
          <p class="text-sm text-muted-foreground">标签</p>
        </div>
      </div>
    </div>

    <!-- 最近书签（如果有）-->
    <div v-if="recentBookmarks.length > 0" class="max-w-4xl mx-auto mb-8">
      <h2 class="text-lg font-semibold mb-4">最近添加</h2>
      <div class="space-y-2">
        <div
          v-for="bookmark in recentBookmarks"
          :key="bookmark.id"
          class="flex items-center justify-between p-3 bg-card rounded-lg border border-border/50 hover:bg-accent transition-colors cursor-pointer"
          @click="openBookmark(bookmark.url)"
        >
          <div class="flex-1 min-w-0">
            <p class="font-medium truncate">{{ bookmark.title }}</p>
            <p class="text-sm text-muted-foreground truncate">{{ bookmark.url }}</p>
          </div>
          <div class="flex items-center gap-2">
            <span v-if="bookmark.tags" class="text-xs text-muted-foreground">
              {{ bookmark.tags.join(', ') }}
            </span>
          </div>
        </div>
      </div>
    </div>

    <!-- 空状态（简化版）-->
    <div v-else class="max-w-md mx-auto text-center py-12">
      <div class="mb-6">
        <svg class="w-16 h-16 mx-auto text-muted-foreground/50" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M5 5a2 2 0 012-2h10a2 2 0 012 2v16l-7-3.5L5 21V5z" />
        </svg>
      </div>
      <h3 class="text-lg font-medium mb-2">开始添加书签</h3>
      <p class="text-muted-foreground mb-6">
        使用顶部的 + 按钮添加第一个书签
      </p>
      <div class="text-sm text-muted-foreground">
        <p>快捷键提示：</p>
        <p>⌘K - 搜索书签</p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useBookmarksStore } from '@/stores/bookmarks'
import { useCollectionsStore } from '@/stores/collections'
import { useTagsStore } from '@/stores/tags'
import { apiService } from '@/services/api'
import type { Stats } from '@/types'

// Stores
const bookmarksStore = useBookmarksStore()
const collectionsStore = useCollectionsStore()
const tagsStore = useTagsStore()

// 统计数据
const stats = ref<Stats>({
  total_bookmarks: 0,
  total_collections: 0,
  total_tags: 0,
  recent_bookmarks: [],
  top_tags: [],
  total_visits: 0,
  favorite_bookmarks: 0,
  archived_bookmarks: 0
})

// 计算属性
const recentBookmarks = computed(() => stats.value.recent_bookmarks || [])
const bookmarksCount = computed(() => stats.value.total_bookmarks || 0)
const collectionsCount = computed(() => stats.value.total_collections || 0)
const tagsCount = computed(() => stats.value.total_tags || 0)

// 打开书签
const openBookmark = (url: string) => {
  window.open(url, '_blank')
}

// 加载统计数据
const loadStats = async () => {
  try {
    const statsData = await apiService.getStats()
    stats.value = statsData
  } catch (error) {
    console.error('加载统计数据失败:', error)
  }
}

// 加载基础数据
const loadBasicData = async () => {
  try {
    // 并行加载基础数据
    await Promise.all([
      collectionsStore.fetchCollections(),
      tagsStore.fetchTags(),
      bookmarksStore.fetchBookmarks({ limit: 5 }) // 只获取最近5个书签用于首页显示
    ])
  } catch (error) {
    console.error('加载基础数据失败:', error)
  }
}

// 加载所有数据
const loadData = async () => {
  await Promise.all([
    loadStats(),
    loadBasicData()
  ])
}

onMounted(() => {
  loadData()
})
</script>