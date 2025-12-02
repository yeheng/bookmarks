<template>
  <div class="text-center py-12">
    <div class="mb-6">
      <svg 
        class="w-16 h-16 mx-auto text-muted-foreground/50" 
        xmlns="http://www.w3.org/2000/svg" 
        fill="none" 
        viewBox="0 0 24 24" 
        stroke="currentColor"
      >
        <path 
          stroke-linecap="round" 
          stroke-linejoin="round" 
          stroke-width="1.5" 
          :d="iconPath"
        />
      </svg>
    </div>
    <h3 class="text-lg font-medium mb-2">{{ title }}</h3>
    <p class="text-muted-foreground mb-6">{{ description }}</p>
    <Button v-if="actionText" @click="$emit('action')">
      <svg class="w-4 h-4 mr-1" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
      </svg>
      {{ actionText }}
    </Button>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { Button } from '@/components/ui/button'

interface Props {
  title: string
  description: string
  actionText?: string
  iconType?: 'bookmark' | 'folder' | 'tag' | 'search'
}

const props = withDefaults(defineProps<Props>(), {
  iconType: 'bookmark'
})

defineEmits<{
  action: []
}>()

// 根据图标类型返回对应的SVG路径
const iconPath = computed(() => {
  switch (props.iconType) {
    case 'bookmark':
      return 'M5 5a2 2 0 012-2h10a2 2 0 012 2v16l-7-3.5L5 21V5z'
    case 'folder':
      return 'M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z'
    case 'tag':
      return 'M7 7h.01M7 3h5c.512 0 1.024.195 1.414.586l7 7a2 2 0 010 2.828l-7 7a2 2 0 01-2.828 0l-7-7A1.994 1.994 0 013 12V7a4 4 0 014-4z'
    case 'search':
      return 'M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z'
    default:
      return 'M5 5a2 2 0 012-2h10a2 2 0 012 2v16l-7-3.5L5 21V5z'
  }
})
</script>