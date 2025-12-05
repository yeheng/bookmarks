<template>
  <nav class="sticky top-0 z-50 w-full border-b border-border/40 bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
    <div class="container mx-auto px-4">
      <div class="flex h-14 items-center justify-between">
        <!-- Logo and navigation -->
        <div class="flex items-center gap-6">
          <RouterLink to="/" class="flex items-center gap-2">
            <div class="h-6 w-6 rounded-md bg-primary flex items-center justify-center">
              <span class="text-xs font-medium text-primary-foreground">B</span>
            </div>
            <span class="text-lg font-semibold tracking-tight">Bookmarks</span>
          </RouterLink>

          <!-- Desktop navigation -->
          <nav class="hidden md:flex items-center gap-1">
            <RouterLink
              v-for="item in navigationItems"
              :key="item.to"
              :to="item.to"
              class="inline-flex items-center justify-center whitespace-nowrap rounded-md px-3 py-1.5 text-sm font-medium transition-colors hover:bg-accent hover:text-accent-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
              :class="{
                'bg-accent text-accent-foreground': $route.path === item.to,
                'text-muted-foreground': $route.path !== item.to
              }"
            >
              {{ item.label }}
            </RouterLink>
          </nav>
        </div>

        <!-- Center: Search and Quick Add -->
        <div class="flex-1 max-w-md mx-6">
          <div class="relative">
            <div class="absolute inset-y-0 left-0 flex items-center pl-3 pointer-events-none">
              <svg class="w-4 h-4 text-muted-foreground" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
              </svg>
            </div>
            <input
              ref="searchInput"
              v-model="searchQuery"
              type="text"
              placeholder="搜索书签... (⌘K)"
              class="w-full h-9 pl-10 pr-4 text-sm bg-background border border-input rounded-md focus:outline-none focus:ring-2 focus:ring-ring focus:border-transparent transition-all"
              @input="handleSearchInput"
              @keydown="handleSearchKeydown"
              @focus="showSearchResults = true"
              @blur="hideSearchResults"
            />
            <!-- Search results dropdown -->
            <div
              v-if="showSearchResults && (searchQuery || (searchResults && searchResults.length > 0))"
              class="absolute top-full left-0 right-0 mt-1 bg-background border border-input rounded-md shadow-lg max-h-80 overflow-y-auto z-50"
            >
              <div v-if="isSearching" class="p-4 text-center text-muted-foreground">
                搜索中...
              </div>
              <div v-else-if="(!searchResults || searchResults.length === 0) && searchQuery" class="p-4 text-center text-muted-foreground">
                未找到相关书签
              </div>
              <div v-else>
                <div
                  v-for="result in (searchResults || [])"
                  :key="result?.id"
                  class="p-3 hover:bg-accent cursor-pointer border-b last:border-b-0"
                  @click="goToBookmark(result)"
                >
                  <div class="font-medium text-sm">{{ result?.title || '无标题' }}</div>
                  <div class="text-xs text-muted-foreground truncate">{{ result?.url || '' }}</div>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- Right: Quick Add and User actions -->
        <div class="flex items-center gap-2">
          <Button
            v-if="authStore.isAuthenticated"
            variant="outline"
            size="sm"
            @click="showAddBookmarkModal = true"
            class="h-9"
          >
            <svg class="w-4 h-4 mr-1" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
            </svg>
            添加
          </Button>
          
          <template v-if="authStore.isAuthenticated">
            <div class="hidden sm:flex items-center gap-2">
              <div class="h-8 w-8 rounded-full bg-primary/10 flex items-center justify-center">
                <span class="text-xs font-medium text-primary">{{ getUserInitials() }}</span>
              </div>
              <span class="text-sm font-medium">{{ authStore.user?.username }}</span>
            </div>
            <Button variant="ghost" size="sm" @click="handleLogout" class="h-9">
              退出
            </Button>
          </template>
          <template v-else>
            <RouterLink to="/auth/login">
              <Button variant="ghost" size="sm" class="h-9">
                登录
              </Button>
            </RouterLink>
            <RouterLink to="/auth/register">
              <Button size="sm" class="h-9">
                注册
              </Button>
            </RouterLink>
          </template>
        </div>
      </div>
    </div>
  </nav>

  <!-- Add Bookmark Modal -->
  <div v-if="showAddBookmarkModal" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
    <div class="bg-background rounded-lg p-6 w-full max-w-md">
      <h3 class="text-lg font-semibold mb-4">添加书签</h3>
      <div class="space-y-4">
        <div>
          <label class="text-sm font-medium">标题</label>
          <input
            v-model="newBookmark.title"
            type="text"
            class="w-full mt-1 px-3 py-2 border border-input rounded-md focus:outline-none focus:ring-2 focus:ring-ring"
            placeholder="输入书签标题"
          />
        </div>
        <div>
          <label class="text-sm font-medium">URL</label>
          <input
            v-model="newBookmark.url"
            type="url"
            class="w-full mt-1 px-3 py-2 border border-input rounded-md focus:outline-none focus:ring-2 focus:ring-ring"
            placeholder="https://example.com"
          />
        </div>
        <div>
          <label class="text-sm font-medium">描述（可选）</label>
          <textarea
            v-model="newBookmark.description"
            class="w-full mt-1 px-3 py-2 border border-input rounded-md focus:outline-none focus:ring-2 focus:ring-ring"
            rows="3"
            placeholder="输入书签描述"
          />
        </div>
      </div>
      <div class="flex justify-end gap-2 mt-6">
        <Button variant="outline" @click="showAddBookmarkModal = false">
          取消
        </Button>
        <Button @click="handleAddBookmark" :disabled="!newBookmark.title || !newBookmark.url">
          添加
        </Button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { Button } from '@/components/ui/button'
import { useAuthStore } from '@/stores/auth'
import { onMounted, onUnmounted, ref } from 'vue'
import { RouterLink, useRouter } from 'vue-router'

const authStore = useAuthStore()
const router = useRouter()

// 搜索相关状态
const searchQuery = ref('')
const searchResults = ref<any[]>([])
const showSearchResults = ref(false)
const isSearching = ref(false)
const searchInput = ref<HTMLInputElement>()

// 添加书签相关状态
const showAddBookmarkModal = ref(false)
const newBookmark = ref({
  title: '',
  url: '',
  description: ''
})

// 导航项配置（简化导航，聚焦书签功能）
const navigationItems = [
  { to: '/bookmarks', label: '书签' }
]

// 获取用户首字母（用于头像显示）
const getUserInitials = () => {
  const username = authStore.user?.username || ''
  if (!username) return 'U'

  // 取前两个字符的首字母
  const parts = username.split(' ').filter(Boolean)
  if (parts.length >= 2) {
    return (parts[0][0] + parts[1][0]).toUpperCase()
  }
  return username.substring(0, 2).toUpperCase()
}

// 搜索功能
const handleSearchInput = async () => {
  if (!searchQuery.value.trim()) {
    searchResults.value = []
    return
  }

  isSearching.value = true
  try {
    // 直接调用API，避免影响store状态
    const { apiService } = await import('@/services/api')
    const response = await apiService.search({
      q: searchQuery.value.trim(),
      limit: 5
    })
    
    // API返回格式: {data: {items: [...], pagination: {...}}, success: true}
    const items = response.data?.items || []
    searchResults.value = items
  } catch (error) {
    console.error('搜索失败:', error)
    searchResults.value = []
  } finally {
    isSearching.value = false
  }
}

const handleSearchKeydown = (event: KeyboardEvent) => {
  if (event.key === 'Enter' && searchResults.value.length > 0) {
    goToBookmark(searchResults.value[0])
  }
}

const goToBookmark = (bookmark: any) => {
  showSearchResults.value = false
  searchQuery.value = ''
  // 直接打开书签URL，而不是跳转到详情页
  if (bookmark && bookmark.url) {
    window.open(bookmark.url, '_blank')
  }
}

const hideSearchResults = () => {
  setTimeout(() => {
    showSearchResults.value = false
  }, 200)
}

// 添加书签功能
const handleAddBookmark = async () => {
  if (!newBookmark.value.title || !newBookmark.value.url) return

  try {
    // 调用真实的添加书签API
    const { useBookmarksStore } = await import('@/stores/bookmarks')
    const bookmarksStore = useBookmarksStore()
    
    await bookmarksStore.createBookmark({
      title: newBookmark.value.title,
      url: newBookmark.value.url,
      description: newBookmark.value.description
    })
    
    // 重置表单
    newBookmark.value = {
      title: '',
      url: '',
      description: ''
    }
    showAddBookmarkModal.value = false
    
    // 显示成功提示
    // 这里可以添加toast通知
  } catch (error) {
    console.error('添加书签失败:', error)
  }
}

// 键盘快捷键支持
const handleKeydown = (event: KeyboardEvent) => {
  // Cmd/Ctrl + K 聚焦搜索框
  if ((event.metaKey || event.ctrlKey) && event.key === 'k') {
    event.preventDefault()
    searchInput.value?.focus()
  }
  
  // Escape 关闭搜索结果
  if (event.key === 'Escape') {
    showSearchResults.value = false
    showAddBookmarkModal.value = false
  }
}

const handleLogout = async () => {
  await authStore.logout()
  router.push('/auth/login')
}

// 生命周期
onMounted(() => {
  document.addEventListener('keydown', handleKeydown)
})

onUnmounted(() => {
  document.removeEventListener('keydown', handleKeydown)
})
</script>