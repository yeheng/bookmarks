<template>
  <div
    v-if="isOpen"
    class="fixed inset-0 z-50 flex items-center justify-center"
    @click="handleBackdropClick"
  >
    <!-- 背景遮罩 -->
    <div class="absolute inset-0 bg-black/50 backdrop-blur-sm" />

    <!-- 模态框内容 -->
    <div
      class="relative w-full max-w-2xl max-h-[90vh] mx-4 bg-background rounded-lg shadow-xl overflow-hidden"
      @click.stop
    >
      <!-- 标题栏 -->
      <div class="flex items-center justify-between p-6 border-b">
        <h2 class="text-lg font-semibold">
          {{ isEditMode ? '编辑书签' : '添加书签' }}
        </h2>
        <button
          type="button"
          @click="handleClose"
          class="p-2 rounded-md hover:bg-accent transition-colors"
        >
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <!-- 表单内容 -->
      <div class="p-6 overflow-y-auto max-h-[calc(90vh-80px)]">
        <BookmarkForm
          :bookmark="bookmark"
          :collections="collections"
          :is-submitting="isSubmitting"
          @submit="handleSubmit"
          @cancel="handleClose"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, watch } from 'vue'
import { BookmarkForm } from '@/components/bookmarks'
import type { Bookmark, Collection, CreateBookmarkRequest, UpdateBookmarkRequest } from '@/types'

// Props
interface Props {
  isOpen: boolean
  bookmark?: Bookmark
  collections: Collection[]
  isSubmitting: boolean
}

const props = withDefaults(defineProps<Props>(), {
  bookmark: undefined
})

// Emits
const emit = defineEmits<{
  close: []
  submit: [data: CreateBookmarkRequest | UpdateBookmarkRequest]
}>()

// 是否为编辑模式
const isEditMode = computed(() => !!props.bookmark)

// 处理背景点击
const handleBackdropClick = () => {
  handleClose()
}

// 处理关闭
const handleClose = () => {
  if (!props.isSubmitting) {
    emit('close')
  }
}

// 处理提交
const handleSubmit = (data: CreateBookmarkRequest | UpdateBookmarkRequest) => {
  emit('submit', data)
}

// 监听模态框打开状态，聚焦到第一个输入框
watch(() => props.isOpen, (isOpen) => {
  if (isOpen) {
    nextTick(() => {
      const firstInput = document.querySelector('#title') as HTMLInputElement
      firstInput?.focus()
    })
  }
})

// 处理 ESC 键关闭
watch(() => props.isOpen, (isOpen) => {
  if (isOpen) {
    const handleEsc = (event: KeyboardEvent) => {
      if (event.key === 'Escape' && !props.isSubmitting) {
        handleClose()
      }
    }
    document.addEventListener('keydown', handleEsc)
    
    // 清理事件监听器
    return () => {
      document.removeEventListener('keydown', handleEsc)
    }
  }
})
</script>