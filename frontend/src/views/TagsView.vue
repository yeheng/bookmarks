<template>
  <div class="container mx-auto px-4 py-8 h-screen flex flex-col">
    <!-- Page header -->
    <div class="mb-6 flex items-center justify-between flex-shrink-0">
      <div>
        <h1 class="text-2xl font-bold tracking-tight">标签</h1>
        <p class="text-muted-foreground">
          共 {{ tagsStore.tags?.length || 0 }} 个标签
        </p>
      </div>
      
      <Button @click="showCreateModal = true" class="flex items-center gap-2">
        <span>+</span>
        添加标签
      </Button>
    </div>

    <!-- Search and filter -->
    <div class="mb-6 flex-shrink-0">
      <Input
        v-model="searchQuery"
        placeholder="搜索标签..."
        class="max-w-md"
        @input="handleSearch"
      />
    </div>

    <!-- Infinite scroll container -->
    <div class="flex-1 min-h-0">
      <InfiniteScroll
        :items="filteredTags"
        :is-loading="tagsStore.isLoading"
        :is-loading-more="tagsStore.isLoadingMore"
        :has-more="tagsStore.hasMore && !searchQuery"
        @load-more="loadMore"
      >
        <template #default="{ items, isLoading, isLoadingMore }">
          <!-- Tags grid -->
          <div v-if="!isLoading && items.length > 0" class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-3">
            <div
              v-for="tag in items"
              :key="tag.id"
              class="group bg-card border border-border/50 rounded-lg p-3 hover:shadow-sm transition-all duration-200"
            >
              <div class="flex items-center justify-between mb-2">
                <div class="flex items-center gap-2">
                  <div 
                    class="w-4 h-4 rounded-full"
                    :style="{ backgroundColor: tag.color }"
                  />
                  <span 
                    class="font-medium truncate hover:text-primary cursor-pointer transition-colors"
                    @click="viewTagBookmarks(tag)"
                  >
                    {{ tag.name }}
                  </span>
                </div>
                
                <!-- Actions -->
                <div class="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                  <button
                    @click="editTag(tag)"
                    class="p-1 rounded hover:bg-accent transition-colors text-muted-foreground"
                    title="编辑"
                  >
                    ✏️
                  </button>
                  <button
                    @click="deleteTag(tag)"
                    class="p-1 rounded hover:bg-accent transition-colors text-red-500"
                    title="删除"
                  >
                    🗑️
                  </button>
                </div>
              </div>
              
              <!-- Usage count -->
              <div class="flex items-center justify-between text-sm text-muted-foreground">
                <span>{{ tag.usage_count || 0 }} 个书签</span>
                <span>{{ formatDate(tag.created_at) }}</span>
              </div>
              
              <!-- Description -->
              <div v-if="tag.description" class="mt-2">
                <p class="text-xs text-muted-foreground line-clamp-2">
                  {{ tag.description }}
                </p>
              </div>
            </div>
          </div>

          <!-- Loading state -->
          <div v-else-if="isLoading" class="flex justify-center py-12">
            <div class="text-center">
              <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary mx-auto mb-4"></div>
              <p class="text-muted-foreground">加载标签中...</p>
            </div>
          </div>

          <!-- Empty state -->
          <div v-else-if="searchQuery" class="flex justify-center py-12">
            <div class="text-center">
              <p class="text-muted-foreground">未找到匹配的标签</p>
              <Button @click="clearSearch" variant="outline" class="mt-4">
                清除搜索
              </Button>
            </div>
          </div>

          <!-- Empty state -->
          <div v-else class="flex justify-center py-12">
            <EmptyState
              title="暂无标签"
              description="为书签添加标签以便更好地分类和搜索"
              action-text="添加标签"
              icon-type="tag"
              @action="showCreateModal = true"
            />
          </div>
        </template>
      </InfiniteScroll>
    </div>

    <!-- Create/Edit Modal -->
    <div v-if="showCreateModal || editingTag" class="fixed inset-0 bg-black/50 flex items-center justify-center p-4 z-50">
      <div class="bg-card rounded-lg p-6 w-full max-w-md">
        <h2 class="text-lg font-semibold mb-4">
          {{ editingTag ? '编辑标签' : '添加标签' }}
        </h2>
        
        <form @submit.prevent="handleSubmit" class="space-y-4">
          <!-- Name -->
          <div>
            <Label for="name">名称</Label>
            <Input
              id="name"
              v-model="form.name"
              placeholder="标签名称"
              required
            />
          </div>
          
          <!-- Description -->
          <div>
            <Label for="description">描述</Label>
            <Textarea
              id="description"
              v-model="form.description"
              placeholder="标签描述（可选）"
              rows="3"
            />
          </div>
          
          <!-- Color -->
          <div>
            <Label for="color">颜色</Label>
            <div class="flex gap-2 mt-2">
              <button
                v-for="color in colorOptions"
                :key="color"
                type="button"
                class="w-8 h-8 rounded-full border-2 transition-all"
                :class="form.color === color ? 'border-foreground' : 'border-transparent'"
                :style="{ backgroundColor: color }"
                @click="form.color = color"
              />
            </div>
          </div>
          
          <!-- Actions -->
          <div class="flex gap-2 pt-4">
            <Button
              type="button"
              variant="outline"
              @click="closeModal"
              class="flex-1"
            >
              取消
            </Button>
            <Button
              type="submit"
              :disabled="isSubmitting"
              class="flex-1"
            >
              {{ isSubmitting ? '处理中...' : (editingTag ? '更新' : '添加') }}
            </Button>
          </div>
        </form>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted, watch } from 'vue'
import { useRouter } from 'vue-router'
import { useTagsStore } from '@/stores/tags'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Textarea } from '@/components/ui/textarea'
import { EmptyState } from '@/components/ui/empty-state'
import { InfiniteScroll } from '@/components/ui/infinite-scroll'
import type { Tag, CreateTagRequest, UpdateTagRequest } from '@/types'

const router = useRouter()
const tagsStore = useTagsStore()

// Modal state
const showCreateModal = ref(false)
const editingTag = ref<Tag | null>(null)
const isSubmitting = ref(false)

// Search state
const searchQuery = ref('')

// Form state
const form = reactive({
  name: '',
  description: '',
  color: '#3b82f6'
})

// Color options
const colorOptions = [
  '#3b82f6', '#ef4444', '#10b981', '#f59e0b', 
  '#8b5cf6', '#ec4899', '#6b7280', '#059669',
  '#14b8a6', '#f97316', '#84cc16', '#06b6d4'
]

// 过滤后的标签
const filteredTags = computed(() => {
  if (!searchQuery.value) {
    return tagsStore.tags
  }
  
  const query = searchQuery.value.toLowerCase()
  return tagsStore.tags.filter(tag => 
    tag.name.toLowerCase().includes(query) ||
    (tag.description && tag.description.toLowerCase().includes(query))
  )
})

// 处理搜索
let searchTimeout: NodeJS.Timeout
const handleSearch = () => {
  clearTimeout(searchTimeout)
  searchTimeout = setTimeout(() => {
    if (searchQuery.value) {
      tagsStore.fetchTags({ search: searchQuery.value }, true)
    } else {
      tagsStore.fetchTags({}, true)
    }
  }, 300)
}

// 清除搜索
const clearSearch = () => {
  searchQuery.value = ''
  tagsStore.fetchTags({}, true)
}

// 加载更多
const loadMore = async () => {
  if (!searchQuery.value) {
    await tagsStore.fetchTags({}, false)
  }
}

// 格式化日期
const formatDate = (timestamp: number) => {
  const date = new Date(timestamp * 1000)
  const now = new Date()
  const diff = now.getTime() - date.getTime()
  const days = Math.floor(diff / (1000 * 60 * 60 * 24))
  
  if (days === 0) return '今天'
  if (days === 1) return '昨天'
  if (days < 7) return `${days}天前`
  if (days < 30) return `${Math.floor(days / 7)}周前`
  return date.toLocaleDateString()
}

// 查看标签的书签
const viewTagBookmarks = (tag: Tag) => {
  router.push({
    name: 'bookmarks',
    query: { tag: tag.id.toString() }
  })
}

// 编辑标签
const editTag = (tag: Tag) => {
  editingTag.value = tag
  form.name = tag.name
  form.description = tag.description || ''
  form.color = tag.color
}

// 删除标签
const deleteTag = async (tag: Tag) => {
  if (tag.usage_count && tag.usage_count > 0) {
    if (!confirm(`标签"${tag.name}"被 ${tag.usage_count} 个书签使用，确定要删除吗？`)) {
      return
    }
  } else {
    if (!confirm(`确定要删除标签"${tag.name}"吗？`)) {
      return
    }
  }
  
  try {
    await tagsStore.deleteTag(tag.id)
  } catch (error) {
    console.error('删除标签失败:', error)
  }
}

// 关闭模态框
const closeModal = () => {
  showCreateModal.value = false
  editingTag.value = null
  form.name = ''
  form.description = ''
  form.color = '#3b82f6'
}

// 提交表单
const handleSubmit = async () => {
  isSubmitting.value = true
  
  try {
    if (editingTag.value) {
      const updateData: UpdateTagRequest = {
        name: form.name,
        description: form.description,
        color: form.color
      }
      await tagsStore.updateTag(editingTag.value.id, updateData)
    } else {
      const createData: CreateTagRequest = {
        name: form.name,
        description: form.description,
        color: form.color
      }
      await tagsStore.createTag(createData)
    }
    
    closeModal()
  } catch (error) {
    console.error('保存标签失败:', error)
  } finally {
    isSubmitting.value = false
  }
}

// 初始化
onMounted(() => {
  tagsStore.fetchTags({}, true)
})
</script>