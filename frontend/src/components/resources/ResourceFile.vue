<template>
  <div class="text-xs text-muted-foreground mb-2">
    <div class="bg-orange-50 dark:bg-orange-900/20 border border-orange-200 dark:border-orange-800 rounded p-2">
      <div class="flex items-center justify-between mb-1">
        <span class="text-orange-600 dark:text-orange-400 text-[10px]">📄 文件</span>
        <div class="flex gap-1">
          <button
            v-if="resource.source"
            @click.stop="copyFilePath(resource.source)"
            class="text-orange-400 hover:text-orange-600 text-[10px] px-1 py-0.5 rounded hover:bg-orange-200 dark:hover:bg-orange-800 transition-colors"
            title="复制路径"
          >
            📋
          </button>
          <button
            v-if="resource.url"
            @click.stop="downloadFile(resource.url)"
            class="text-orange-400 hover:text-orange-600 text-[10px] px-1 py-0.5 rounded hover:bg-orange-200 dark:hover:bg-orange-800 transition-colors"
            title="下载文件"
          >
            ⬇️
          </button>
        </div>
      </div>
      <div class="font-medium truncate">
        {{ resource.source || resource.mime_type || '未知文件' }}
      </div>
      <div v-if="resource.mime_type" class="text-orange-500 text-[10px] mt-1">
        类型: {{ resource.mime_type }}
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { Resource } from '@/types'

defineProps<{
  resource: Resource
}>()

// 复制文件路径
const copyFilePath = async (filePath: string) => {
  try {
    await navigator.clipboard.writeText(filePath)
  } catch (err) {
    console.error('复制失败:', err)
  }
}

// 下载文件
const downloadFile = (url: string) => {
  try {
    const link = document.createElement('a')
    link.href = url
    link.download = '' // 让浏览器自动从 URL 推断文件名
    link.target = '_blank'
    document.body.appendChild(link)
    link.click()
    document.body.removeChild(link)
  } catch (err) {
    console.error('下载失败:', err)
    // 降级处理：直接在新标签页打开
    window.open(url, '_blank')
  }
}
</script>