<template>
  <form @submit.prevent="handleSubmit" class="space-y-6">
    <!-- 资源类型 -->
    <div class="space-y-2">
      <Label for="type">资源类型 *</Label>
      <div class="flex flex-wrap gap-2">
        <button
          v-for="typeOption in typeOptions"
          :key="typeOption.value"
          type="button"
          @click="form.type = typeOption.value"
          class="px-3 py-2 rounded-md border transition-all"
          :class="form.type === typeOption.value
            ? 'bg-primary text-primary-foreground border-primary'
            : 'bg-background hover:bg-accent border-input'"
          :disabled="isSubmitting"
        >
          <span class="mr-2">{{ typeOption.icon }}</span>
          {{ typeOption.label }}
        </button>
      </div>
    </div>

    <!-- 标题 -->
    <div class="space-y-2">
      <Label for="title">标题 *</Label>
      <Input
        id="title"
        v-model="form.title"
        type="text"
        placeholder="输入资源标题"
        required
        :disabled="isSubmitting"
      />
    </div>

    <!-- URL (仅链接类型显示) -->
    <div v-if="form.type === 'link'" class="space-y-2">
      <Label for="url">URL *</Label>
      <Input
        id="url"
        v-model="form.url"
        type="url"
        placeholder="https://example.com"
        required
        :disabled="isSubmitting"
      />
      <p v-if="errors.url" class="text-sm text-destructive">{{ errors.url }}</p>
    </div>

    <!-- 内容 (笔记和代码片段类型显示) -->
    <div v-if="form.type === 'note' || form.type === 'snippet'" class="space-y-2">
      <Label for="content">内容 *</Label>
      <Textarea
        id="content"
        v-model="form.content"
        :placeholder="form.type === 'note' ? '输入笔记内容...' : '输入代码片段...'"
        :rows="form.type === 'note' ? 6 : 10"
        :class="form.type === 'snippet' ? 'font-mono' : ''"
        :disabled="isSubmitting"
      />
      <p v-if="errors.content" class="text-sm text-destructive">{{ errors.content }}</p>
    </div>

    <!-- 来源 (文件类型显示) -->
    <div v-if="form.type === 'file'" class="space-y-2">
      <Label for="source">来源</Label>
      <Input
        id="source"
        v-model="form.source"
        type="text"
        placeholder="文件来源或路径"
        :disabled="isSubmitting"
      />
      <p v-if="errors.source" class="text-sm text-destructive">{{ errors.source }}</p>
    </div>

    <!-- 描述 -->
    <div class="space-y-2">
      <Label for="description">描述</Label>
      <Textarea
        id="description"
        v-model="form.description"
        placeholder="输入书签描述（可选）"
        rows="3"
        :disabled="isSubmitting"
      />
    </div>

    <!-- 收藏夹 -->
    <CollectionSelector
      v-model="form.collection_id"
      :collections="collections"
      :disabled="isSubmitting || operationStatus.creatingCollection"
      @collection-created="handleCollectionCreated"
    />
    <p v-if="errors.collection" class="text-sm text-destructive">{{ errors.collection }}</p>
    <p v-if="operationStatus.creatingCollection" class="text-sm text-blue-600">正在创建收藏夹...</p>

    <!-- 标签 -->
    <div class="space-y-2">
      <Label for="tags">标签</Label>
      <TagInput
        v-model="form.tags"
        placeholder="输入标签后按回车添加"
        :disabled="isSubmitting || operationStatus.creatingTag"
        :suggestions="availableTags"
        @create-tag="handleCreateTag"
      />
      <p v-if="errors.tags" class="text-sm text-destructive">{{ errors.tags }}</p>
      <p v-if="operationStatus.creatingTag" class="text-sm text-blue-600">正在创建标签...</p>
    </div>

    <!-- 选项 -->
    <div class="space-y-4">
      <div class="flex items-center space-x-2">
        <Checkbox
          id="is_favorite"
          v-model:checked="form.is_favorite"
          :disabled="isSubmitting"
        />
        <Label for="is_favorite" class="text-sm font-normal">
          收藏
        </Label>
      </div>

      <div class="flex items-center space-x-2">
        <Checkbox
          id="is_private"
          v-model:checked="form.is_private"
          :disabled="isSubmitting"
        />
        <Label for="is_private" class="text-sm font-normal">
          私密
        </Label>
      </div>

      <div class="flex items-center space-x-2">
        <Checkbox
          id="is_read"
          v-model:checked="form.is_read"
          :disabled="isSubmitting"
        />
        <Label for="is_read" class="text-sm font-normal">
          已读
        </Label>
      </div>

      <div class="flex items-center space-x-2">
        <Checkbox
          id="is_archived"
          v-model:checked="form.is_archived"
          :disabled="isSubmitting"
        />
        <Label for="is_archived" class="text-sm font-normal">
          归档
        </Label>
      </div>
    </div>

    <!-- 高级选项 -->
    <div class="space-y-4">
      <div class="space-y-2">
        <Label for="reading_time">阅读时间（分钟）</Label>
        <Input
          id="reading_time"
          v-model.number="form.reading_time"
          type="number"
          min="0"
          placeholder="预计阅读时间"
          :disabled="isSubmitting"
        />
      </div>

      <div class="space-y-2">
        <Label for="difficulty_level">难度等级</Label>
        <select
          id="difficulty_level"
          v-model="form.difficulty_level"
          class="w-full h-10 px-3 py-2 text-sm border border-input bg-background rounded-md ring-offset-background file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50"
          :disabled="isSubmitting"
        >
          <option value="">未设置</option>
          <option value="1">1 - 简单</option>
          <option value="2">2 - 较简单</option>
          <option value="3">3 - 中等</option>
          <option value="4">4 - 较难</option>
          <option value="5">5 - 困难</option>
        </select>
      </div>
    </div>

    <!-- 引用管理（仅在编辑模式显示） -->
    <div v-if="isEditMode && props.resource" class="space-y-4 pt-4 border-t">
      <ResourceReferences :resource-id="props.resource.id" />
    </div>

    <!-- 操作按钮 -->
    <div class="flex justify-end space-x-2 pt-4 border-t">
      <Button
        type="button"
        variant="outline"
        @click="$emit('cancel')"
        :disabled="isSubmitting"
      >
        取消
      </Button>
      <Button
        type="submit"
        :disabled="isSubmitting || !isFormValid"
      >
        <Spinner v-if="isSubmitting" class="mr-2 h-4 w-4" />
        {{ isEditMode ? '更新' : '创建' }}
      </Button>
    </div>
  </form>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, watch } from 'vue'
import { Button } from '@/components/ui/button'
import { Checkbox } from '@/components/ui/checkbox'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Textarea } from '@/components/ui/textarea'
import { Spinner } from '@/components/icons'
import { CollectionSelector, TagInput, ResourceReferences } from '@/components/bookmarks'
import { useTagsStore } from '@/stores/tags'
import type { Resource, ResourceType, Collection, CreateResourceRequest, UpdateResourceRequest } from '@/types'

// Props
interface Props {
  resource?: Resource
  collections: Collection[]
  isSubmitting: boolean
}

const props = withDefaults(defineProps<Props>(), {
  resource: undefined
})

// Emits
const emit = defineEmits<{
  submit: [data: CreateResourceRequest | UpdateResourceRequest]
  cancel: []
}>()

// Stores
const tagsStore = useTagsStore()

// 类型选项
const typeOptions = [
  { value: 'link' as ResourceType, label: '链接', icon: '🔗' },
  { value: 'note' as ResourceType, label: '笔记', icon: '📝' },
  { value: 'snippet' as ResourceType, label: '代码片段', icon: '💻' },
  { value: 'file' as ResourceType, label: '文件', icon: '📄' }
]

// 计算可用标签
const availableTags = computed(() => {
  return tagsStore.tags?.map(tag => tag.name) || []
})

// 是否为编辑模式
const isEditMode = computed(() => !!props.resource)

// 表单数据
const form = reactive({
  type: 'link' as ResourceType,
  title: '',
  url: '',
  content: '',
  source: '',
  description: '',
  collection_id: undefined as number | undefined,
  tags: [] as string[],
  is_favorite: false,
  is_private: false,
  is_read: false,
  is_archived: false,
  reading_time: undefined as number | undefined,
  difficulty_level: undefined as number | undefined
})

// 表单验证错误
const errors = reactive({
  url: '',
  content: '',
  source: '',
  tags: '',
  collection: ''
})

// 操作状态
const operationStatus = reactive({
  creatingTag: false,
  creatingCollection: false,
  lastError: ''
})

// 监听 resource 变化，初始化表单
watch(() => props.resource, (resource) => {
  if (resource) {
    form.type = resource.type
    form.title = resource.title
    form.url = resource.url || ''
    form.content = resource.content || ''
    form.source = resource.source || ''
    form.description = resource.description || ''
    form.collection_id = resource.collection_id
    form.tags = Array.isArray(resource.tags) ? [...resource.tags] : []
    form.is_favorite = resource.is_favorite
    form.is_private = resource.is_private
    form.is_read = resource.is_read
    form.is_archived = resource.is_archived
    form.reading_time = resource.reading_time
    form.difficulty_level = resource.difficulty_level
  } else {
    resetForm()
  }
}, { immediate: true })

// 重置表单
const resetForm = () => {
  form.type = 'link'
  form.title = ''
  form.url = ''
  form.content = ''
  form.source = ''
  form.description = ''
  form.collection_id = undefined
  form.tags = []
  form.is_favorite = false
  form.is_private = false
  form.is_read = false
  form.is_archived = false
  form.reading_time = undefined
  form.difficulty_level = undefined

  // 重置错误状态
  errors.url = ''
  errors.content = ''
  errors.source = ''
  errors.tags = ''
  errors.collection = ''

  // 重置操作状态
  operationStatus.creatingTag = false
  operationStatus.creatingCollection = false
  operationStatus.lastError = ''
}

// 验证 URL
const validateUrl = (url: string) => {
  if (!url) return true
  try {
    new URL(url)
    return true
  } catch {
    return false
  }
}

// 表单是否有效
const isFormValid = computed(() => {
  const titleValid = form.title.trim() !== ''

  // 根据类型验证必填字段
  let typeValid = true
  if (form.type === 'link') {
    typeValid = form.url.trim() !== '' && validateUrl(form.url) && !errors.url
  } else if (form.type === 'note' || form.type === 'snippet') {
    typeValid = form.content.trim() !== '' && !errors.content
  }

  return titleValid && typeValid
})

// 实时验证 URL
watch(() => form.url, (newUrl) => {
  if (newUrl && !validateUrl(newUrl)) {
    errors.url = '请输入有效的 URL'
  } else {
    errors.url = ''
  }
})

// 实时验证内容
watch(() => form.content, (newContent) => {
  if (form.type === 'note' || form.type === 'snippet') {
    if (!newContent.trim()) {
      errors.content = '内容不能为空'
    } else {
      errors.content = ''
    }
  }
})

// 处理创建新标签
const handleCreateTag = async (tagName: string) => {
  operationStatus.creatingTag = true
  errors.tags = ''
  
  try {
    // 创建标签时使用默认颜色
    const newTag = await tagsStore.createTag({ 
      name: tagName,
      color: '#3B82F6' // 默认蓝色
    })
    
    // 标签创建成功后，确保标签在可用列表中
    if (!availableTags.value.includes(tagName)) {
      await tagsStore.fetchTags()
    }
    
    console.log(`标签 "${tagName}" 创建成功`)
  } catch (error: any) {
    console.error('创建标签失败:', error)
    errors.tags = `创建标签失败: ${error.message || '未知错误'}`
    
    // 移除刚添加的标签，因为创建失败
    const tagIndex = form.tags.indexOf(tagName)
    if (tagIndex > -1) {
      form.tags.splice(tagIndex, 1)
    }
    
    // 不重新抛出错误，让用户继续使用表单
  } finally {
    operationStatus.creatingTag = false
  }
}

// 处理收藏夹创建成功
const handleCollectionCreated = (collection: Collection) => {
  console.log('新收藏夹已创建:', collection)
  operationStatus.creatingCollection = false
  errors.collection = ''
  
  // 确保新创建的收藏夹在表单中被选中
  // CollectionSelector 组件已经处理了选择逻辑，这里只需要清理状态
}

// 初始化标签数据
const initializeTags = async () => {
  if (tagsStore.tags.length === 0) {
    try {
      await tagsStore.fetchTags()
    } catch (error) {
      console.error('加载标签失败:', error)
    }
  }
}

// 组件挂载时初始化
onMounted(() => {
  initializeTags()
})

// 处理提交
const handleSubmit = () => {
  if (!isFormValid.value) return

  let submitData: any

  if (isEditMode.value && props.resource) {
    // 编辑模式：只发送有变化的字段
    submitData = {}

    if (form.type !== props.resource.type) {
      submitData.type = form.type
    }
    if (form.title.trim() !== props.resource.title) {
      submitData.title = form.title.trim()
    }

    // URL 处理
    const url = form.url.trim() || undefined
    if (url !== props.resource.url) {
      submitData.url = url
    }

    // 内容处理
    const content = form.content.trim() || undefined
    if (content !== props.resource.content) {
      submitData.content = content
    }

    // 来源处理
    const source = form.source.trim() || undefined
    if (source !== props.resource.source) {
      submitData.source = source
    }

    const description = form.description.trim() || undefined
    if (description !== props.resource.description) {
      submitData.description = description
    }

    if (form.collection_id !== props.resource.collection_id) {
      if (form.collection_id) {
        submitData.collection_id = form.collection_id
      } else if (props.resource.collection_id) {
        submitData.clear_collection_id = true
      }
    }

    // 比较标签数组
    const currentTags = [...form.tags].sort()
    const originalTags = Array.isArray(props.resource.tags) ? [...props.resource.tags].sort() : []
    if (JSON.stringify(currentTags) !== JSON.stringify(originalTags)) {
      submitData.tags = form.tags.length > 0 ? form.tags : []
    }

    if (form.is_favorite !== props.resource.is_favorite) {
      submitData.is_favorite = form.is_favorite
    }
    if (form.is_private !== props.resource.is_private) {
      submitData.is_private = form.is_private
    }
    if (form.is_read !== props.resource.is_read) {
      submitData.is_read = form.is_read
    }
    if (form.is_archived !== props.resource.is_archived) {
      submitData.is_archived = form.is_archived
    }
    if (form.reading_time !== props.resource.reading_time) {
      submitData.reading_time = form.reading_time || undefined
    }
    if (form.difficulty_level !== props.resource.difficulty_level) {
      submitData.difficulty_level = form.difficulty_level || undefined
    }

    // 如果没有字段变化，至少发送一个字段以避免空提交错误
    if (Object.keys(submitData).length === 0) {
      // 发送一个不会改变数据的字段，但满足API要求
      submitData.title = props.resource.title
    }

    console.log('编辑模式提交数据:', submitData)
  } else {
    // 创建模式：发送所有字段
    submitData = {
      type: form.type,
      title: form.title.trim(),
      url: form.type === 'link' ? form.url.trim() : undefined,
      content: (form.type === 'note' || form.type === 'snippet') ? form.content.trim() : undefined,
      source: form.type === 'file' ? form.source.trim() : undefined,
      description: form.description.trim() || undefined,
      collection_id: form.collection_id || undefined,
      tags: form.tags.length > 0 ? form.tags : undefined,
      is_favorite: form.is_favorite,
      is_private: form.is_private,
      is_read: form.is_read,
      is_archived: form.is_archived,
      reading_time: form.reading_time || undefined,
      difficulty_level: form.difficulty_level || undefined
    }

    console.log('创建模式提交数据:', submitData)
  }

  emit('submit', submitData)
}
</script>