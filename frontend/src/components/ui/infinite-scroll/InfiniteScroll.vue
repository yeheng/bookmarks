<template>
  <div 
    ref="containerRef" 
    class="infinite-scroll-container"
    @scroll="handleScroll"
  >
    <slot :items="safeItems" :isLoading="isLoading" :isLoadingMore="isLoadingMore" />
    
    <!-- 加载更多指示器 -->
    <div 
      v-if="hasMore && !isLoading" 
      ref="loadTriggerRef"
      class="load-trigger"
      style="height: 1px; width: 100%;"
    />
    
    <!-- 加载更多状态 -->
    <div v-if="isLoadingMore" class="flex justify-center py-4">
      <div class="animate-spin rounded-full h-6 w-6 border-b-2 border-primary"></div>
    </div>
    
    <!-- 没有更多数据提示 -->
    <div v-if="!hasMore && safeItems.length > 0" class="text-center py-4 text-muted-foreground text-sm">
      已加载全部数据
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, nextTick, computed } from 'vue'

interface Props {
  items: any[]
  isLoading: boolean
  isLoadingMore: boolean
  hasMore: boolean
  threshold?: number
  debounceMs?: number // 防抖延迟时间（毫秒）
}

const props = withDefaults(defineProps<Props>(), {
  threshold: 200,
  debounceMs: 300
})

// 确保 items 始终是一个数组
const safeItems = computed(() => props.items || [])

const emit = defineEmits<{
  loadMore: []
}>()

const containerRef = ref<HTMLElement>()
const loadTriggerRef = ref<HTMLElement>()
const observer = ref<IntersectionObserver | null>(null)

// 本地加载状态 - 防止在 emit 后立即重复触发
const localLoading = ref(false)
let debounceTimer: ReturnType<typeof setTimeout> | null = null

// 防抖的加载触发函数
const triggerLoadMore = () => {
  // 双重检查：props 状态 + 本地状态
  if (props.isLoading || props.isLoadingMore || !props.hasMore || localLoading.value) {
    return
  }

  // 清除之前的防抖计时器
  if (debounceTimer) {
    clearTimeout(debounceTimer)
  }

  // 设置防抖
  debounceTimer = setTimeout(() => {
    // 再次检查状态（防抖期间可能状态已改变）
    if (props.isLoading || props.isLoadingMore || !props.hasMore || localLoading.value) {
      return
    }

    // 设置本地加载标志
    localLoading.value = true

    // 触发加载事件
    emit('loadMore')

    // 安全地重置本地加载标志（给父组件足够时间更新 props）
    setTimeout(() => {
      localLoading.value = false
    }, 500)
  }, props.debounceMs)
}

const handleScroll = () => {
  if (!containerRef.value) {
    return
  }

  const { scrollTop, scrollHeight, clientHeight } = containerRef.value

  // 当滚动到距离底部 threshold 像素时触发加载
  if (scrollHeight - scrollTop - clientHeight < props.threshold) {
    triggerLoadMore()
  }
}

// 使用 Intersection Observer 替代滚动事件监听，性能更好
const setupIntersectionObserver = () => {
  if (!loadTriggerRef.value) return

  observer.value = new IntersectionObserver(
    (entries) => {
      entries.forEach((entry) => {
        if (entry.isIntersecting) {
          triggerLoadMore()
        }
      })
    },
    {
      root: containerRef.value,
      rootMargin: `${props.threshold}px`,
      threshold: 0.1
    }
  )

  observer.value.observe(loadTriggerRef.value)
}

onMounted(() => {
  nextTick(() => {
    setupIntersectionObserver()
  })
})

onUnmounted(() => {
  // 清理防抖计时器
  if (debounceTimer) {
    clearTimeout(debounceTimer)
  }

  // 断开 Intersection Observer
  if (observer.value) {
    observer.value.disconnect()
  }
})
</script>

<style scoped>
.infinite-scroll-container {
  height: 100%;
  overflow-y: auto;
}
</style>