<template>
  <div class="space-y-4">
    <!-- 引用管理头部 -->
    <div class="flex items-center justify-between">
      <h3 class="text-lg font-semibold">关联资源</h3>
      <button
        @click="showAddReference = true"
        class="px-3 py-1.5 text-sm bg-primary text-primary-foreground rounded-md hover:bg-primary/90 transition-colors"
      >
        + 添加关联
      </button>
    </div>

    <!-- 添加引用模态框 -->
    <div v-if="showAddReference" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div class="bg-background rounded-lg shadow-xl w-full max-w-md mx-4">
        <div class="p-6">
          <h4 class="text-lg font-semibold mb-4">添加关联资源</h4>

          <!-- 搜索输入 -->
          <div class="space-y-2 mb-4">
            <label class="text-sm font-medium">搜索资源</label>
            <input
              v-model="searchQuery"
              @input="searchResources"
              type="text"
              placeholder="输入标题或描述搜索..."
              class="w-full px-3 py-2 border border-input rounded-md bg-background"
            />
          </div>

          <!-- 搜索结果 -->
          <div v-if="searchResults.length > 0" class="max-h-60 overflow-y-auto border rounded-md">
            <div
              v-for="resource in searchResults"
              :key="resource.id"
              @click="addReference(resource.id)"
              class="p-3 border-b last:border-b-0 hover:bg-accent cursor-pointer transition-colors"
            >
              <div class="font-medium">{{ resource.title }}</div>
              <div class="text-xs text-muted-foreground truncate">
                {{ resource.type === 'link' ? resource.url : resource.type === 'note' ? '笔记' : resource.type === 'snippet' ? '代码片段' : '文件' }}
              </div>
            </div>
          </div>
          <div v-else-if="searchQuery && !isSearching" class="text-center py-4 text-muted-foreground">
            未找到匹配的资源
          </div>

          <!-- 操作按钮 -->
          <div class="flex justify-end gap-2 mt-6">
            <button
              @click="showAddReference = false"
              class="px-4 py-2 text-sm border border-input rounded-md hover:bg-accent transition-colors"
            >
              取消
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- 引用列表 -->
    <div v-if="references.length > 0" class="space-y-3">
      <div
        v-for="reference in references"
        :key="reference.id"
        class="flex items-center justify-between p-3 border rounded-md hover:bg-accent/50 transition-colors"
      >
        <div class="flex-1">
          <div class="font-medium">{{ reference.title }}</div>
          <div class="text-xs text-muted-foreground">
            <span class="inline-block px-2 py-0.5 rounded-full bg-gray-100 text-gray-700 mr-2">
              {{ getTypeLabel(reference.type) }}
            </span>
            {{ reference.type === 'link' ? reference.url : truncateContent(reference.description || '', 60) }}
          </div>
        </div>
        <button
          @click="removeReference(reference.id)"
          class="ml-2 p-1.5 text-red-500 hover:bg-red-50 rounded-md transition-colors"
          title="移除关联"
        >
          🗑️
        </button>
      </div>

      <!-- 加载更多 -->
      <div v-if="hasMore" class="text-center">
        <button
          @click="loadMore"
          :disabled="isLoadingMore"
          class="px-4 py-2 text-sm border border-input rounded-md hover:bg-accent transition-colors"
        >
          {{ isLoadingMore ? '加载中...' : '加载更多' }}
        </button>
      </div>
    </div>
    <div v-else class="text-center py-8 text-muted-foreground">
      暂无关联资源
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useApi } from '@/services/api'
import type { Resource, ResourceType } from '@/types'

const props = defineProps<{
  resourceId: number
}>()

const api = useApi()
const references = ref<Resource[]>([])
const searchResults = ref<Resource[]>([])
const searchQuery = ref('')
const showAddReference = ref(false)
const isLoading = ref(false)
const isLoadingMore = ref(false)
const isSearching = ref(false)
const limit = 20
const offset = ref(0)
const hasMore = ref(false)

// 获取类型标签
const getTypeLabel = (type: ResourceType): string => {
  const labels: Record<ResourceType, string> = {
    link: '链接',
    note: '笔记',
    snippet: '代码',
    file: '文件'
  }
  return labels[type] || '资源'
}

// 截断内容
const truncateContent = (content: string, maxLength: number): string => {
  if (content.length <= maxLength) return content
  return content.substring(0, maxLength) + '...'
}

// 加载引用
const loadReferences = async (reset = true) => {
  if (reset) {
    offset.value = 0
    isLoading.value = true
  } else {
    isLoadingMore.value = true
  }

  try {
    const response = await api.getResourceReferences(props.resourceId, {
      limit,
      offset: offset.value
    })

    if (response.success) {
      const newItems = response.data.items || []
      if (reset) {
        references.value = newItems
      } else {
        references.value = [...references.value, ...newItems]
      }

      hasMore.value = response.data.pagination?.has_next || false
      offset.value += limit
    }
  } catch (error) {
    console.error('加载引用失败:', error)
  } finally {
    isLoading.value = false
    isLoadingMore.value = false
  }
}

// 搜索资源
const searchResources = async () => {
  if (!searchQuery.value.trim()) {
    searchResults.value = []
    return
  }

  isSearching.value = true
  try {
    const response = await api.searchResources({
      q: searchQuery.value,
      limit: 10
    })

    if (response.success) {
      // 过滤掉当前资源本身
      searchResults.value = (response.data.items || []).filter(
        (resource: Resource) => resource.id !== props.resourceId
      )
    }
  } catch (error) {
    console.error('搜索资源失败:', error)
  } finally {
    isSearching.value = false
  }
}

// 添加引用
const addReference = async (targetId: number) => {
  try {
    await api.createResourceReference(props.resourceId, targetId)
    showAddReference.value = false
    searchQuery.value = ''
    searchResults.value = []
    loadReferences(true)
  } catch (error) {
    console.error('添加引用失败:', error)
  }
}

// 移除引用
const removeReference = async (targetId: number) => {
  if (!confirm('确定要移除这个关联吗？')) return

  try {
    await api.deleteResourceReference(props.resourceId, targetId)
    references.value = references.value.filter(ref => ref.id !== targetId)
  } catch (error) {
    console.error('移除引用失败:', error)
  }
}

// 加载更多
const loadMore = () => {
  loadReferences(false)
}

onMounted(() => {
  loadReferences(true)
})
</script>