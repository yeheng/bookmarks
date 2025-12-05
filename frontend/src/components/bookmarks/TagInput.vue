<template>
  <div class="space-y-2">
    <div
      ref="containerRef"
      class="flex flex-wrap gap-2 p-2 min-h-[42px] border border-input bg-background rounded-md ring-offset-background focus-within:ring-2 focus-within:ring-ring focus-within:ring-offset-2 transition-all duration-200"
      @click="focusInput"
    >
      <!-- 标签列表 -->
      <span
        v-for="(tag, index) in tags"
        :key="`${tag}-${index}`"
        class="group inline-flex items-center gap-1 px-2 py-1 bg-primary/10 text-primary rounded-md text-sm transition-all duration-200 hover:bg-primary/20"
      >
        {{ tag }}
        <button
          type="button"
          @click.stop="removeTag(index)"
          class="ml-1 opacity-60 hover:opacity-100 transition-opacity duration-200 focus:outline-none focus:ring-1 focus:ring-primary rounded-sm"
        >
          <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </span>

      <!-- 输入框 -->
      <input
        ref="inputRef"
        v-model="inputValue"
        type="text"
        :placeholder="tags.length === 0 ? placeholder : ''"
        :disabled="disabled || tags.length >= maxTags"
        class="flex-1 min-w-[120px] px-1 py-1 text-sm bg-transparent outline-none placeholder:text-muted-foreground disabled:cursor-not-allowed disabled:opacity-50 transition-colors duration-200"
        @keydown="handleKeydown"
        @input="handleInput"
        @blur="handleBlur"
      />
    </div>

    <!-- 标签计数和限制提示 -->
    <div class="flex items-center justify-between text-xs text-muted-foreground">
      <span>{{ tags.length }} / {{ maxTags }} 标签</span>
      <span v-if="tags.length >= maxTags" class="text-amber-600">已达到最大标签数量</span>
    </div>

    <!-- 建议标签 -->
    <Transition
      enter-active-class="transition duration-200 ease-out"
      enter-from-class="transform scale-95 opacity-0"
      enter-to-class="transform scale-100 opacity-100"
      leave-active-class="transition duration-75 ease-in"
      leave-from-class="transform scale-100 opacity-100"
      leave-to-class="transform scale-95 opacity-0"
    >
      <div
        v-if="suggestedTags.length > 0 && showSuggestions"
        class="z-10 w-full p-1 mt-1 bg-background border border-input rounded-md shadow-lg max-h-32 overflow-y-auto"
      >
        <div class="px-2 py-1 text-xs font-medium text-muted-foreground border-b border-border/50 mb-1">
          建议标签
        </div>
        <button
          v-for="(suggestedTag, index) in suggestedTags"
          :key="suggestedTag"
          type="button"
          :class="[
            'w-full px-2 py-1.5 text-left text-sm hover:bg-accent rounded transition-colors duration-150 flex items-center gap-2',
            highlightedIndex === index ? 'bg-accent' : '',
            suggestedTag.startsWith('+ 创建新标签') ? 'text-green-600 font-medium' : ''
          ]"
          @click="addSuggestedTag(suggestedTag)"
          @mouseenter="highlightedIndex = index"
        >
          <svg 
            v-if="suggestedTag.startsWith('+ 创建新标签')" 
            class="w-4 h-4" 
            fill="none" 
            stroke="currentColor" 
            viewBox="0 0 24 24"
          >
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
          </svg>
          {{ suggestedTag }}
        </button>
      </div>
    </Transition>
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
  'create-tag': [tagName: string]
}>()

// Refs
const containerRef = ref<HTMLElement>()
const inputRef = ref<HTMLInputElement>()
const inputValue = ref('')
const showSuggestions = ref(false)
const highlightedIndex = ref(-1)

// Computed
const tags = ref<string[]>([...props.modelValue])

// 防抖函数
const debounce = (func: Function, wait: number) => {
  let timeout: NodeJS.Timeout
  return function executedFunction(...args: any[]) {
    const later = () => {
      clearTimeout(timeout)
      func(...args)
    }
    clearTimeout(timeout)
    timeout = setTimeout(later, wait)
  }
}

// 获取建议标签（过滤掉已有的标签）
const suggestedTags = computed(() => {
  if (!inputValue.value.trim()) return []
  
  const existingTags = props.suggestions.filter(tag => 
    !tags.value.includes(tag) && 
    tag.toLowerCase().includes(inputValue.value.toLowerCase())
  ).slice(0, 5)
  
  // 如果输入的标签不存在于建议中，且不为空，则添加"创建新标签"选项
  if (inputValue.value.trim() && 
      !props.suggestions.includes(inputValue.value.trim()) && 
      !tags.value.includes(inputValue.value.trim())) {
    return [
      ...existingTags,
      `+ 创建新标签 "${inputValue.value.trim()}"`
    ]
  }
  
  return existingTags
})

// 监听 modelValue 变化
watch(() => props.modelValue, (newValue) => {
  tags.value = [...newValue]
})

// 监听建议变化，重置高亮索引
watch(() => suggestedTags.value.length, () => {
  highlightedIndex.value = -1
})

// 添加标签
const addTag = (tag: string) => {
  const trimmedTag = tag.trim()
  if (!trimmedTag) return
  
  // 检查是否已存在
  if (tags.value.includes(trimmedTag)) {
    inputValue.value = ''
    showSuggestions.value = false
    return
  }

  // 检查最大数量限制
  if (tags.value.length >= props.maxTags) {
    return
  }

  tags.value.push(trimmedTag)
  inputValue.value = ''
  showSuggestions.value = false
  highlightedIndex.value = -1
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
  // 如果是创建新标签的选项
  if (tag.startsWith('+ 创建新标签 "')) {
    const newTag = tag.replace('+ 创建新标签 "', '').replace('"', '')
    addTag(newTag)
    emit('create-tag', newTag)
  } else {
    addTag(tag)
  }
  focusInput()
}

// 处理键盘事件
const handleKeydown = (event: KeyboardEvent) => {
  switch (event.key) {
    case 'Enter':
      event.preventDefault()
      if (highlightedIndex.value >= 0 && suggestedTags.value[highlightedIndex.value]) {
        addSuggestedTag(suggestedTags.value[highlightedIndex.value])
      } else if (inputValue.value.trim()) {
        addTag(inputValue.value)
      }
      break
    
    case 'Tab':
      event.preventDefault()
      if (highlightedIndex.value >= 0 && suggestedTags.value[highlightedIndex.value]) {
        addSuggestedTag(suggestedTags.value[highlightedIndex.value])
      } else if (inputValue.value.trim()) {
        addTag(inputValue.value)
      }
      break
    
    case 'ArrowDown':
      event.preventDefault()
      if (suggestedTags.value.length > 0) {
        highlightedIndex.value = Math.min(highlightedIndex.value + 1, suggestedTags.value.length - 1)
      }
      break
    
    case 'ArrowUp':
      event.preventDefault()
      if (suggestedTags.value.length > 0) {
        highlightedIndex.value = Math.max(highlightedIndex.value - 1, -1)
      }
      break
    
    case 'Backspace':
      if (!inputValue.value && tags.value.length > 0) {
        removeTag(tags.value.length - 1)
      }
      break
    
    case 'Escape':
      event.preventDefault()
      showSuggestions.value = false
      highlightedIndex.value = -1
      break
  }
}

// 防抖处理输入
const debouncedInputHandler = debounce(() => {
  showSuggestions.value = inputValue.value.trim().length > 0 && suggestedTags.value.length > 0
  highlightedIndex.value = -1
}, 300)

// 处理输入
const handleInput = () => {
  debouncedInputHandler()
}

// 处理失焦
const handleBlur = () => {
  // 延迟隐藏建议，以便点击建议项
  setTimeout(() => {
    showSuggestions.value = false
    highlightedIndex.value = -1
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