<template>
  <div class="space-y-2">
    <div
      ref="containerRef"
      class="flex flex-wrap gap-2 p-2 min-h-[42px] border border-input bg-background rounded-md ring-offset-background focus-within:ring-2 focus-within:ring-ring focus-within:ring-offset-2"
      @click="focusInput"
    >
      <!-- 标签列表 -->
      <span
        v-for="(tag, index) in tags"
        :key="`${tag}-${index}`"
        class="inline-flex items-center gap-1 px-2 py-1 bg-blue-100 text-blue-800 rounded-md text-sm"
      >
        {{ tag }}
        <button
          type="button"
          @click.stop="removeTag(index)"
          class="ml-1 text-blue-600 hover:text-blue-800 focus:outline-none"
        >
          ×
        </button>
      </span>

      <!-- 输入框 -->
      <input
        ref="inputRef"
        v-model="inputValue"
        type="text"
        :placeholder="placeholder"
        :disabled="disabled"
        class="flex-1 min-w-[120px] px-1 py-1 text-sm bg-transparent outline-none placeholder:text-muted-foreground disabled:cursor-not-allowed disabled:opacity-50"
        @keydown="handleKeydown"
        @input="handleInput"
        @blur="handleBlur"
      />
    </div>

    <!-- 建议标签 -->
    <div
      v-if="suggestedTags.length > 0 && showSuggestions"
      class="z-10 w-full p-1 mt-1 bg-background border border-input rounded-md shadow-lg max-h-32 overflow-y-auto"
    >
      <button
        v-for="suggestedTag in suggestedTags"
        :key="suggestedTag"
        type="button"
        class="w-full px-2 py-1 text-left text-sm hover:bg-accent rounded"
        @click="addSuggestedTag(suggestedTag)"
      >
        {{ suggestedTag }}
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue'

// Props
interface Props {
  modelValue: string[]
  placeholder?: string
  disabled?: boolean
  suggestions?: string[]
  maxTags?: number
}

const props = withDefaults(defineProps<Props>(), {
  placeholder: '输入标签后按回车添加',
  disabled: false,
  suggestions: () => [],
  maxTags: 20
})

// Emits
const emit = defineEmits<{
  'update:modelValue': [tags: string[]]
}>()

// Refs
const containerRef = ref<HTMLElement>()
const inputRef = ref<HTMLInputElement>()
const inputValue = ref('')
const showSuggestions = ref(false)

// Computed
const tags = ref<string[]>([...props.modelValue])

// 获取建议标签（过滤掉已有的标签）
const suggestedTags = computed(() => {
  return props.suggestions.filter(tag => 
    !tags.value.includes(tag) && 
    tag.toLowerCase().includes(inputValue.value.toLowerCase())
  ).slice(0, 5)
})

// 监听 modelValue 变化
watch(() => props.modelValue, (newValue) => {
  tags.value = [...newValue]
})

// 添加标签
const addTag = (tag: string) => {
  const trimmedTag = tag.trim()
  if (!trimmedTag) return
  
  // 检查是否已存在
  if (tags.value.includes(trimmedTag)) {
    inputValue.value = ''
    return
  }

  // 检查最大数量限制
  if (tags.value.length >= props.maxTags) {
    return
  }

  tags.value.push(trimmedTag)
  inputValue.value = ''
  showSuggestions.value = false
  emitUpdate()
}

// 移除标签
const removeTag = (index: number) => {
  tags.value.splice(index, 1)
  emitUpdate()
  focusInput()
}

// 添加建议标签
const addSuggestedTag = (tag: string) => {
  addTag(tag)
  focusInput()
}

// 处理键盘事件
const handleKeydown = (event: KeyboardEvent) => {
  switch (event.key) {
    case 'Enter':
    case 'Tab':
      event.preventDefault()
      if (inputValue.value.trim()) {
        addTag(inputValue.value)
      } else if (event.key === 'Tab') {
        showSuggestions.value = false
      }
      break
    
    case 'Backspace':
      if (!inputValue.value && tags.value.length > 0) {
        removeTag(tags.value.length - 1)
      }
      break
    
    case 'Escape':
      showSuggestions.value = false
      break
  }
}

// 处理输入
const handleInput = () => {
  showSuggestions.value = inputValue.value.trim().length > 0 && suggestedTags.value.length > 0
}

// 处理失焦
const handleBlur = () => {
  // 延迟隐藏建议，以便点击建议项
  setTimeout(() => {
    showSuggestions.value = false
    if (inputValue.value.trim()) {
      addTag(inputValue.value)
    }
  }, 200)
}

// 聚焦输入框
const focusInput = () => {
  nextTick(() => {
    inputRef.value?.focus()
  })
}

// 发送更新事件
const emitUpdate = () => {
  emit('update:modelValue', [...tags.value])
}
</script>