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

        <!-- User actions -->
        <div class="flex items-center gap-2">
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
</template>

<script setup lang="ts">
import { Button } from '@/components/ui/button'
import { useAuthStore } from '@/stores/auth'
import { RouterLink, useRouter } from 'vue-router'

const authStore = useAuthStore()
const router = useRouter()

// 导航项配置
const navigationItems = [
  { to: '/bookmarks', label: '书签' },
  { to: '/collections', label: '收藏夹' },
  { to: '/tags', label: '标签' },
  { to: '/search', label: '搜索' }
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

const handleLogout = async () => {
  await authStore.logout()
  router.push('/auth/login')
}
</script>