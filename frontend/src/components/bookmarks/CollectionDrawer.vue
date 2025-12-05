<template>
  <div
    :class="[
      'absolute left-0 top-0 h-full bg-background border-r border-border/40 transition-all duration-300 z-10',
      isOpen ? 'w-64' : 'w-0 overflow-hidden'
    ]"
  >
    <div class="w-64 h-full flex flex-col">
      <!-- 标题栏 -->
      <div class="p-4 border-b border-border/40">
        <h3 class="font-semibold text-sm text-muted-foreground uppercase tracking-wider">收藏夹</h3>
      </div>

      <!-- 收藏夹列表 -->
      <div class="flex-1 overflow-y-auto p-2">
        <div class="space-y-1">
          <!-- 全部书签 -->
          <button
            @click="handleSelect(null)"
            :class="[
              'w-full text-left px-3 py-2 rounded-md text-sm transition-colors',
              !selectedId ? 'bg-accent text-accent-foreground' : 'hover:bg-accent/50'
            ]"
          >
            <div class="flex items-center gap-2">
              <span class="w-2 h-2 rounded-full bg-primary"></span>
              <span>全部书签</span>
            </div>
          </button>

          <!-- 收藏夹列表 -->
          <button
            v-for="collection in collections"
            :key="collection.id"
            @click="handleSelect(collection.id)"
            :class="[
              'w-full text-left px-3 py-2 rounded-md text-sm transition-colors',
              selectedId === collection.id ? 'bg-accent text-accent-foreground' : 'hover:bg-accent/50'
            ]"
          >
            <div class="flex items-center gap-2">
              <span
                class="w-2 h-2 rounded-full"
                :style="{ backgroundColor: collection.color }"
              ></span>
              <span class="truncate">{{ collection.name }}</span>
              <span class="text-xs text-muted-foreground ml-auto">{{ collection.bookmark_count }}</span>
            </div>
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { Collection } from '@/types'

// Props
interface Props {
  isOpen: boolean
  collections: Collection[]
  selectedId?: string | null
}

const props = withDefaults(defineProps<Props>(), {
  selectedId: null
})

// Emits
const emit = defineEmits<{
  select: [collectionId: string | null]
}>()

// 处理选择
const handleSelect = (collectionId: string | number | null) => {
  emit('select', collectionId ? String(collectionId) : null)
}
</script>
