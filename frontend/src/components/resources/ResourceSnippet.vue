<template>
  <div class="text-xs text-muted-foreground mb-2">
    <div class="bg-gray-50 dark:bg-gray-800 border rounded p-2 font-mono leading-relaxed">
      <div class="flex items-start justify-between mb-1">
        <span class="text-gray-500 text-[10px]">💻 代码片段</span>
        <button
          @click.stop="copyToClipboard(resource.content)"
          class="text-gray-400 hover:text-gray-600 text-[10px] px-1 py-0.5 rounded hover:bg-gray-200 dark:hover:bg-gray-700 transition-colors"
          title="复制代码"
        >
          📋
        </button>
      </div>
      <pre class="whitespace-pre-wrap break-words line-clamp-3">{{ truncateContent(resource.content, 120) }}</pre>
      <div v-if="resource.content.length > 120" class="text-blue-600 text-[10px] mt-1">
        ... 点击查看更多
      </div>
    </div>
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

// 复制到剪贴板
const copyToClipboard = async (text: string) => {
  try {
    await navigator.clipboard.writeText(text)
  } catch (err) {
    console.error('复制失败:', err)
    // 降级处理
    const textArea = document.createElement('textarea')
    textArea.value = text
    document.body.appendChild(textArea)
    textArea.select()
    document.execCommand('copy')
    document.body.removeChild(textArea)
  }
}
</script>