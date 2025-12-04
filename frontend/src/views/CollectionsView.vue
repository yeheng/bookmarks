<template>
  <div class="container mx-auto px-4 py-8">
    <!-- Page header -->
    <div class="mb-6 flex items-center justify-between">
      <div>
        <h1 class="text-2xl font-bold tracking-tight">收藏夹</h1>
        <p class="text-muted-foreground">
          共 {{ collectionsStore.collections.length }} 个收藏夹
        </p>
      </div>
      
      <Button @click="showCreateModal = true" class="flex items-center gap-2">
        <span>+</span>
        创建收藏夹
      </Button>
    </div>

    <!-- Collections grid -->
    <div v-if="!collectionsStore.isLoading && collectionsStore.collections.length > 0" class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
      <div
        v-for="collection in collectionsStore.collections"
        :key="collection.id"
        class="group bg-card border border-border/50 rounded-lg p-4 hover:shadow-sm transition-all duration-200 cursor-pointer"
        @click="openCollection(collection)"
      >
        <div class="flex items-start justify-between mb-3">
          <div class="flex items-center gap-2">
            <div 
              class="w-8 h-8 rounded-lg flex items-center justify-center text-white"
              :style="{ backgroundColor: collection.color }"
            >
              {{ getCollectionIcon(collection.icon) }}
            </div>
            <div>
              <h3 class="font-medium group-hover:text-primary transition-colors">
                {{ collection.name }}
              </h3>
              <p v-if="collection.description" class="text-sm text-muted-foreground line-clamp-1">
                {{ collection.description }}
              </p>
            </div>
          </div>
          
          <!-- Actions -->
          <div class="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
            <button
              @click.stop="editCollection(collection)"
              class="p-1.5 rounded hover:bg-accent transition-colors text-muted-foreground"
              title="编辑"
            >
              ✏️
            </button>
            <button
              @click.stop="deleteCollection(collection)"
              class="p-1.5 rounded hover:bg-accent transition-colors text-red-500"
              title="删除"
            >
              🗑️
            </button>
          </div>
        </div>
        
        <!-- Stats -->
        <div class="flex items-center justify-between text-sm text-muted-foreground">
          <span>{{ collection.bookmark_count || 0 }} 个书签</span>
          <span>{{ formatDate(collection.created_at) }}</span>
        </div>
        
        <!-- Recent bookmarks preview -->
        <div v-if="collection.recent_bookmarks && collection.recent_bookmarks.length > 0" class="mt-3 pt-3 border-t border-border/50">
          <div class="space-y-1">
            <div
              v-for="bookmark in collection.recent_bookmarks.slice(0, 3)"
              :key="bookmark.id"
              class="text-xs text-muted-foreground truncate hover:text-foreground cursor-pointer"
              @click.stop="openBookmark(bookmark.url)"
            >
              • {{ bookmark.title }}
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Loading state -->
    <div v-else-if="collectionsStore.isLoading" class="flex justify-center py-12">
      <div class="text-center">
        <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary mx-auto mb-4"></div>
        <p class="text-muted-foreground">加载收藏夹中...</p>
      </div>
    </div>

    <!-- Empty state -->
    <div v-else class="flex justify-center py-12">
      <EmptyState
        title="暂无收藏夹"
        description="创建收藏夹来按主题组织您的书签"
        action-text="创建收藏夹"
        icon-type="folder"
        @action="showCreateModal = true"
      />
    </div>

    <!-- Create/Edit Modal -->
    <div v-if="showCreateModal || editingCollection" class="fixed inset-0 bg-black/50 flex items-center justify-center p-4 z-50">
      <div class="bg-card rounded-lg p-6 w-full max-w-md">
        <h2 class="text-lg font-semibold mb-4">
          {{ editingCollection ? '编辑收藏夹' : '创建收藏夹' }}
        </h2>
        
        <form @submit.prevent="handleSubmit" class="space-y-4">
          <!-- Name -->
          <div>
            <Label for="name">名称</Label>
            <Input
              id="name"
              v-model="form.name"
              placeholder="收藏夹名称"
              required
            />
          </div>
          
          <!-- Description -->
          <div>
            <Label for="description">描述</Label>
            <Textarea
              id="description"
              v-model="form.description"
              placeholder="收藏夹描述（可选）"
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
                class="w-8 h-8 rounded-lg border-2 transition-all"
                :class="form.color === color ? 'border-foreground' : 'border-transparent'"
                :style="{ backgroundColor: color }"
                @click="form.color = color"
              />
            </div>
          </div>
          
          <!-- Icon -->
          <div>
            <Label for="icon">图标</Label>
            <div class="grid grid-cols-6 gap-2 mt-2">
              <button
                v-for="icon in iconOptions"
                :key="icon"
                type="button"
                class="w-10 h-10 rounded border border-border flex items-center justify-center hover:bg-accent transition-colors"
                :class="form.icon === icon ? 'border-primary bg-primary/10' : ''"
                @click="form.icon = icon"
              >
                {{ icon }}
              </button>
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
              {{ isSubmitting ? '处理中...' : (editingCollection ? '更新' : '创建') }}
            </Button>
          </div>
        </form>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useCollectionsStore } from '@/stores/collections'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Textarea } from '@/components/ui/textarea'
import { EmptyState } from '@/components/ui/empty-state'
import type { Collection, CreateCollectionRequest, UpdateCollectionRequest } from '@/types'

const router = useRouter()
const collectionsStore = useCollectionsStore()

// Modal state
const showCreateModal = ref(false)
const editingCollection = ref<Collection | null>(null)
const isSubmitting = ref(false)

// Form state
const form = reactive({
  name: '',
  description: '',
  color: '#3b82f6',
  icon: '📁'
})

// Color options
const colorOptions = [
  '#3b82f6', '#ef4444', '#10b981', '#f59e0b', 
  '#8b5cf6', '#ec4899', '#6b7280', '#059669'
]

// Icon options
const iconOptions = [
  '📁', '📂', '🗂️', '📋', '📝', '📚',
  '💼', '🎯', '🔖', '⭐', '🏷️', '📌'
]

// 获取收藏夹图标
const getCollectionIcon = (icon: string) => {
  return icon || '📁'
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

// 打开收藏夹
const openCollection = (collection: Collection) => {
  router.push({
    name: 'bookmarks',
    query: { collection: collection.id.toString() }
  })
}

// 打开书签
const openBookmark = (url: string) => {
  window.open(url, '_blank')
}

// 编辑收藏夹
const editCollection = (collection: Collection) => {
  editingCollection.value = collection
  form.name = collection.name
  form.description = collection.description || ''
  form.color = collection.color
  form.icon = collection.icon
}

// 删除收藏夹
const deleteCollection = async (collection: Collection) => {
  if (collection.bookmark_count && collection.bookmark_count > 0) {
    if (!confirm(`收藏夹"${collection.name}"包含 ${collection.bookmark_count} 个书签，确定要删除吗？`)) {
      return
    }
  } else {
    if (!confirm(`确定要删除收藏夹"${collection.name}"吗？`)) {
      return
    }
  }
  
  try {
    await collectionsStore.deleteCollection(collection.id)
  } catch (error) {
    console.error('删除收藏夹失败:', error)
  }
}

// 关闭模态框
const closeModal = () => {
  showCreateModal.value = false
  editingCollection.value = null
  form.name = ''
  form.description = ''
  form.color = '#3b82f6'
  form.icon = '📁'
}

// 提交表单
const handleSubmit = async () => {
  isSubmitting.value = true
  
  try {
    if (editingCollection.value) {
      const updateData: UpdateCollectionRequest = {
        name: form.name,
        description: form.description,
        color: form.color,
        icon: form.icon
      }
      await collectionsStore.updateCollection(editingCollection.value.id, updateData)
    } else {
      const createData: CreateCollectionRequest = {
        name: form.name,
        description: form.description,
        color: form.color,
        icon: form.icon
      }
      await collectionsStore.createCollection(createData)
    }
    
    closeModal()
  } catch (error) {
    console.error('保存收藏夹失败:', error)
  } finally {
    isSubmitting.value = false
  }
}

// 初始化
onMounted(() => {
  collectionsStore.fetchCollections()
})
</script>