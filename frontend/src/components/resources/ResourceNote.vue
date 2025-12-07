<template>
  <div class="text-xs text-muted-foreground mb-2 line-clamp-3">
    {{ truncateContent(resource.content, 100) }}
  </div>
</template>

<script setup lang="ts">
import type { Resource } from '@/types'

defineProps<{
  resource: Resource
}>()

// 截断内容
const truncateContent = (content: string, maxLength: number): string => {
  if (!content) return ''
  if (content.length <= maxLength) return content
  return content.substring(0, maxLength) + '...'
}
</script>