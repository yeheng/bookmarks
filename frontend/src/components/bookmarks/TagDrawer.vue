<template>
  <div
    :class="[
      'absolute right-0 top-0 h-full bg-background border-l border-border/40 transition-all duration-300 z-10',
      isOpen ? 'w-64' : 'w-0 overflow-hidden'
    ]"
  >
    <div class="w-64 h-full flex flex-col">
      <!-- 标题栏 -->
      <div class="p-4 border-b border-border/40">
        <h3 class="font-semibold text-sm text-muted-foreground uppercase tracking-wider">标签</h3>
      </div>

      <!-- 标签列表 -->
      <div class="flex-1 overflow-y-auto p-2">
        <div class="space-y-1">
          <button
            v-for="tag in tags"
            :key="tag.name"
            @click="handleTagClick(tag.name)"
            :class="[
              'w-full text-left px-3 py-2 rounded-md text-sm transition-colors',
              selectedTag === tag.name 
                ? 'bg-accent text-accent-foreground' 
                : 'hover:bg-accent/50'
            ]"
          >
            <div class="flex items-center gap-2">
              <span class="w-2 h-2 rounded-full bg-blue-500"></span>
              <span class="truncate">{{ tag.name }}</span>
              <span class="text-xs text-muted-foreground ml-auto">{{ tag.count }}</span>
            </div>
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
// Props
interface TagInfo {
  name: string
  count: number
}

interface Props {
  isOpen: boolean
  tags: TagInfo[]
  selectedTag?: string
}

const props = withDefaults(defineProps<Props>(), {
  selectedTag: ''
})

// Emits
const emit = defineEmits<{
  tagClick: [tagName: string]
}>()

// 处理标签点击
const handleTagClick = (tagName: string) => {
  emit('tagClick', tagName)
}
</script>
