<template>
  <div class="min-h-screen flex items-center justify-center bg-background px-4">
    <div class="max-w-md w-full space-y-8">
      <!-- Header -->
      <div class="text-center">
        <h2 class="mt-4 text-2xl font-bold text-foreground">欢迎回来</h2>
        <p class="mt-1 text-sm text-muted-foreground">
          登录到您的书签管理空间
        </p>
      </div>

      <!-- Login Form -->
      <Card class="shadow-lg">
        <CardContent class="p-8">
          <div class="space-y-6">
            <form @submit.prevent="handleLogin" class="space-y-6">
              <!-- Email Field -->
              <div>
                <Label for="email" class="text-sm font-medium text-foreground mb-2">
                  邮箱地址
                </Label>
                <Input
                  id="email"
                  v-model="form.email"
                  type="text"
                  placeholder="your@email.com"
                  :disabled="isLoading"
                  required
                  :class="{ 'border-red-500': errors.email }"
                />
                <p v-if="errors.email" class="mt-1 text-sm text-red-600">
                  {{ errors.email }}
                </p>
              </div>

              <!-- Password Field -->
              <div>
                <Label for="password" class="text-sm font-medium text-foreground mb-2">
                  密码
                </Label>
                <div class="relative">
                  <Input
                    id="password"
                    v-model="form.password"
                    :type="showPassword ? 'text' : 'password'"
                    placeholder="••••••••"
                    :disabled="isLoading"
                    required
                    class="pr-10"
                    :class="{ 'border-red-500': errors.password }"
                  />
                  <!-- 密码显示/隐藏按钮 - 使用 ghost variant 避免背景色干扰 -->
                  <Button
                    type="button"
                    variant="ghost"
                    size="icon-sm"
                    @click="showPassword = !showPassword"
                    class="absolute inset-y-0 right-0 h-full px-3"
                    :disabled="isLoading"
                  >
                    <Eye
                      v-if="showPassword"
                      class="h-5 w-5 text-muted-foreground"
                    />
                    <EyeOff
                      v-else
                      class="h-5 w-5 text-muted-foreground"
                    />
                  </Button>
                </div>
                <p v-if="errors.password" class="mt-1 text-sm text-red-600">
                  {{ errors.password }}
                </p>
              </div>

              <!-- Remember Me & Forgot Password -->
              <div class="flex items-center justify-between">
                <!-- 使用 Checkbox UI 组件保持一致的视觉风格 -->
                <label class="flex items-center gap-2 cursor-pointer">
                  <Checkbox
                    id="remember"
                    v-model:checked="form.remember"
                    :disabled="isLoading"
                  />
                  <Label
                    for="remember"
                    class="text-sm font-normal cursor-pointer"
                  >
                    记住我
                  </Label>
                </label>
                <div class="text-sm">
                  <a href="#" class="font-medium text-primary hover:text-primary/80">
                    忘记密码？
                  </a>
                </div>
              </div>

              <!-- Error Message -->
              <Alert v-if="authStore.error" variant="destructive">
                {{ authStore.error }}
              </Alert>

              <!-- Submit Button -->
              <Button
                type="submit"
                :disabled="isLoading"
                class="w-full"
              >
                <Spinner
                  v-if="isLoading"
                  class="mr-2 h-4 w-4 text-white"
                />
                {{ isLoading ? '登录中...' : '登录' }}
              </Button>
            </form>

            <!-- Divider -->
            <div class="mt-6 relative">
              <div class="absolute inset-0 flex items-center">
                <div class="w-full border-t border-border"></div>
              </div>
              <div class="relative flex justify-center text-sm">
                <span class="px-2 bg-card text-muted-foreground">或者</span>
              </div>
            </div>

            <!-- Register Link -->
            <div class="mt-6 text-center">
              <p class="text-sm text-muted-foreground">
                还没有账号？
                <router-link 
                  to="/auth/register" 
                  class="font-medium text-primary hover:text-primary/80"
                >
                  立即注册
                </router-link>
              </p>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  </div>
</template>

<script setup lang="ts">
import { Eye, EyeOff, Spinner } from '@/components/icons'
import { Alert } from '@/components/ui/alert'
import { Button } from '@/components/ui/button'
import { Card, CardContent } from '@/components/ui/card'
import { Checkbox } from '@/components/ui/checkbox'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { useAuthStore } from '@/stores/auth'
import { validatePassword } from '@/utils/validation'
import { reactive, ref } from 'vue'
import { useRouter } from 'vue-router'

const router = useRouter()
const authStore = useAuthStore()

const isLoading = ref(false)
const showPassword = ref(false)

const form = reactive({
  email: '',
  password: '',
  remember: false
})

const errors = reactive({
  email: '',
  password: ''
})

const validateForm = () => {
  errors.email = ''
  errors.password = ''

  console.log('Validating form:', { email: form.email, password: form.password })

  if (!form.email || !form.email.trim()) {
    console.log('Email validation failed')
    errors.email = '请输入邮箱地址'
    return false
  }

  if (!form.password) {
    console.log('Password validation failed')
    errors.password = '请输入密码'
    return false
  }

  if (!validatePassword(form.password)) {
    errors.password = '密码至少需要6个字符'
    return false
  }

  console.log('Form validation passed')
  return true
}

const handleLogin = async () => {
  if (!validateForm()) return

  isLoading.value = true
  
  try {
    await authStore.login({
      email: form.email,
      password: form.password
    })

    if (form.remember) {
      localStorage.setItem('auth_token', authStore.token || '')
    }

    router.push('/')
  } catch (error) {
    console.error('Login failed:', error)
  } finally {
    isLoading.value = false
  }
}
</script>