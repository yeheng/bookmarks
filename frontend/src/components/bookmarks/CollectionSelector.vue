<template>
  <div class="space-y-2">
    <Label for="collection">收藏夹</Label>
    
    <!-- 快速创建模式 -->
    <div v-if="showQuickCreate" class="space-y-3">
      <div class="flex items-center space-x-2">
        <Input
          ref="quickCreateInputRef"
          v-model="quickCreateName"
          type="text"
          placeholder="输入收藏夹名称，按回车创建"
          :disabled="isCreating"
          class="flex-1"
          @keydown.enter="handleQuickCreate"
          @keydown.escape="cancelQuickCreate"
          @blur="handleQuickCreateBlur"
        />
        <Button
          type="button"
          size="sm"
          @click="handleQuickCreate"
          :disabled="!quickCreateName.trim() || isCreating"
        >
          <Spinner v-if="isCreating" class="mr-2 h-4 w-4" />
          创建
        </Button>
        <Button
          type="button"
          variant="outline"
          size="sm"
          @click="cancelQuickCreate"
          :disabled="isCreating"
        >
          取消
        </Button>
      </div>
      
      <!-- 快速颜色选择 -->
      <div class="flex items-center space-x-2">
        <Label class="text-xs text-muted-foreground">颜色:</Label>
        <div class="flex space-x-1">
          <button
            v-for="color in presetColors"
            :key="color"
            type="button"
            :class="[
              'w-6 h-6 rounded-full border-2 transition-all duration-200',
              quickCreateColor === color ? 'border-primary scale-110' : 'border-transparent hover:scale-105'
            ]"
            :style="{ backgroundColor: color }"
            @click="quickCreateColor = color"
          />
          <input
            v-model="quickCreateColor"
            type="color"
            class="w-6 h-6 rounded cursor-pointer"
            :disabled="isCreating"
          />
        </div>
      </div>
    </div>

    <!-- 标准选择模式 -->
    <div v-else class="relative">
      <select
        id="collection"
        v-model="selectedCollectionId"
        class="w-full h-10 px-3 py-2 text-sm border border-input bg-background rounded-md ring-offset-background file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50 transition-all duration-200"
        :disabled="disabled || isCreating"
        @change="handleCollectionChange"
      >
        <option value="">无收藏夹</option>
        <option
          v-for="collection in collections"
          :key="collection.id"
          :value="collection.id"
        >
          {{ collection.name }}
        </option>
        <option value="create-new" class="text-green-600 font-medium">
          + 创建新收藏夹...
        </option>
        <option value="quick-create" class="text-blue-600 font-medium">
          + 快速创建...
        </option>
      </select>
    </div>

    <!-- 高级创建表单 -->
    <Transition
      enter-active-class="transition duration-200 ease-out"
      enter-from-class="transform scale-95 opacity-0"
      enter-to-class="transform scale-100 opacity-100"
      leave-active-class="transition duration-75 ease-in"
      leave-from-class="transform scale-100 opacity-100"
      leave-to-class="transform scale-95 opacity-0"
    >
      <div
        v-if="showAdvancedCreate"
        class="p-4 border border-input rounded-md bg-accent/20 space-y-3"
      >
        <div class="flex items-center justify-between">
          <h4 class="text-sm font-medium">高级创建选项</h4>
          <Button
            type="button"
            variant="ghost"
            size="sm"
            @click="showAdvancedCreate = false; showQuickCreate = true"
          >
            切换到快速创建
          </Button>
        </div>

        <div class="space-y-2">
          <Label for="new-collection-name">收藏夹名称 *</Label>
          <Input
            id="new-collection-name"
            ref="advancedCreateInputRef"
            v-model="newCollectionForm.name"
            type="text"
            placeholder="输入收藏夹名称"
            :disabled="isCreating"
            @keydown.enter="handleAdvancedCreate"
          />
        </div>

        <div class="space-y-2">
          <Label for="new-collection-description">描述</Label>
          <Textarea
            id="new-collection-description"
            v-model="newCollectionForm.description"
            placeholder="输入收藏夹描述（可选）"
            rows="2"
            :disabled="isCreating"
          />
        </div>

        <div class="space-y-2">
          <Label for="new-collection-color">颜色</Label>
          <div class="flex items-center space-x-2">
            <div class="flex space-x-1">
              <button
                v-for="color in presetColors"
                :key="color"
                type="button"
                :class="[
                  'w-8 h-8 rounded-full border-2 transition-all duration-200',
                  newCollectionForm.color === color ? 'border-primary scale-110' : 'border-transparent hover:scale-105'
                ]"
                :style="{ backgroundColor: color }"
                @click="newCollectionForm.color = color"
              />
            </div>
            <input
              v-model="newCollectionForm.color"
              type="color"
              class="w-10 h-10 border border-input rounded cursor-pointer"
              :disabled="isCreating"
            />
            <Input
              v-model="newCollectionForm.color"
              type="text"
              placeholder="#3B82F6"
              class="w-24"
              :disabled="isCreating"
            />
          </div>
        </div>

        <div class="flex justify-end space-x-2">
          <Button
            type="button"
            variant="outline"
            size="sm"
            @click="cancelAdvancedCreate"
            :disabled="isCreating"
          >
            取消
          </Button>
          <Button
            type="button"
            size="sm"
            @click="handleAdvancedCreate"
            :disabled="!isNewCollectionValid || isCreating"
          >
            <Spinner v-if="isCreating" class="mr-2 h-4 w-4" />
            创建
          </Button>
        </div>
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, reactive, ref, watch } from 'vue'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Textarea } from '@/components/ui/textarea'
import { Spinner } from '@/components/icons'
import { useCollectionsStore } from '@/stores/collections'
import type { Collection } from '@/types'

// Props
interface Props {
  modelValue?: number | null
  collections: Collection[]
  disabled?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  modelValue: null,
  disabled: false
})

// Emits
const emit = defineEmits<{
  'update:modelValue': [value: number | null]
  'collection-created': [collection: Collection]
}>()

// Stores
const collectionsStore = useCollectionsStore()

// 预设颜色
const presetColors = [
  '#3B82F6', '#EF4444', '#10B981', '#F59E0B', '#8B5CF6',
  '#EC4899', '#14B8A6', '#F97316', '#6366F1', '#84CC16'
]

// Refs
const selectedCollectionId = ref<number | '' | 'create-new' | 'quick-create'>('')
const showQuickCreate = ref(false)
const showAdvancedCreate = ref(false)
const isCreating = ref(false)
const quickCreateInputRef = ref<HTMLInputElement>()
const advancedCreateInputRef = ref<HTMLInputElement>()

// 快速创建表单
const quickCreateName = ref('')
const quickCreateColor = ref('#3B82F6')

// 高级创建表单
const newCollectionForm = reactive({
  name: '',
  description: '',
  color: '#3B82F6'
})

// 监听 modelValue 变化
watch(() => props.modelValue, (newValue) => {
  selectedCollectionId.value = newValue || ''
  if (selectedCollectionId.value === 'create-new' || selectedCollectionId.value === 'quick-create') {
    selectedCollectionId.value = ''
  }
}, { immediate: true })

// 监听集合列表变化，如果当前选中的集合不存在了，清空选择
watch(() => props.collections, (newCollections) => {
  if (selectedCollectionId.value && 
      typeof selectedCollectionId.value === 'number' &&
      !newCollections.find(c => c.id === selectedCollectionId.value)) {
    selectedCollectionId.value = ''
    emit('update:modelValue', null)
  }
})

// 验证新收藏夹表单
const isNewCollectionValid = computed(() => {
  return newCollectionForm.name.trim() !== '' && 
         newCollectionForm.color.trim() !== ''
})

// 处理收藏夹选择变化
const handleCollectionChange = () => {
  if (selectedCollectionId.value === 'create-new') {
    showAdvancedCreate.value = true
    showQuickCreate.value = false
    resetNewCollectionForm()
    // 聚焦到名称输入框
    nextTick(() => {
      advancedCreateInputRef.value?.focus()
    })
  } else if (selectedCollectionId.value === 'quick-create') {
    showQuickCreate.value = true
    showAdvancedCreate.value = false
    resetQuickCreateForm()
    // 聚焦到名称输入框
    nextTick(() => {
      quickCreateInputRef.value?.focus()
    })
  } else {
    showQuickCreate.value = false
    showAdvancedCreate.value = false
    emit('update:modelValue', selectedCollectionId.value || null)
  }
}

// 重置快速创建表单
const resetQuickCreateForm = () => {
  quickCreateName.value = ''
  quickCreateColor.value = '#3B82F6'
}

// 重置高级创建表单
const resetNewCollectionForm = () => {
  newCollectionForm.name = ''
  newCollectionForm.description = ''
  newCollectionForm.color = '#3B82F6'
}

// 取消快速创建
const cancelQuickCreate = () => {
  showQuickCreate.value = false
  selectedCollectionId.value = props.modelValue || ''
  resetQuickCreateForm()
}

// 取消高级创建
const cancelAdvancedCreate = () => {
  showAdvancedCreate.value = false
  selectedCollectionId.value = props.modelValue || ''
  resetNewCollectionForm()
}

// 处理快速创建失焦
const handleQuickCreateBlur = () => {
  setTimeout(() => {
    if (!isCreating.value && !quickCreateInputRef.value?.matches(':focus')) {
      cancelQuickCreate()
    }
  }, 200)
}

// 快速创建收藏夹
const handleQuickCreate = async () => {
  if (!quickCreateName.value.trim() || isCreating.value) return

  isCreating.value = true
  
  try {
    const newCollection = await collectionsStore.createCollection({
      name: quickCreateName.value.trim(),
      color: quickCreateColor.value
    })

    // 选择新创建的收藏夹
    selectedCollectionId.value = newCollection.id
    emit('update:modelValue', newCollection.id)
    emit('collection-created', newCollection)
    
    // 重置表单
    showQuickCreate.value = false
    resetQuickCreateForm()
  } catch (error: any) {
    console.error('创建收藏夹失败:', error)
    // 可以在这里添加错误提示，比如通过 emit 发送错误事件
    // emit('collection-error', error.message)
  } finally {
    isCreating.value = false
  }
}

// 高级创建收藏夹
const handleAdvancedCreate = async () => {
  if (!isNewCollectionValid.value || isCreating.value) return

  isCreating.value = true
  
  try {
    const newCollection = await collectionsStore.createCollection({
      name: newCollectionForm.name.trim(),
      description: newCollectionForm.description.trim() || undefined,
      color: newCollectionForm.color
    })

    // 选择新创建的收藏夹
    selectedCollectionId.value = newCollection.id
    emit('update:modelValue', newCollection.id)
    emit('collection-created', newCollection)
    
    // 重置表单
    showAdvancedCreate.value = false
    resetNewCollectionForm()
  } catch (error: any) {
    console.error('创建收藏夹失败:', error)
    // 可以在这里添加错误提示
  } finally {
    isCreating.value = false
  }
}
</script>