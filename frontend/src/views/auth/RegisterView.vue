<template>
  <div class="min-h-screen flex items-center justify-center bg-background px-4">
    <div class="max-w-md w-full space-y-8">
      <!-- Header -->
      <div class="text-center">
        <h2 class="mt-4 text-2xl font-bold text-foreground">创建账号</h2>
        <p class="mt-1 text-sm text-muted-foreground">
          开始管理您的书签收藏
        </p>
      </div>

      <!-- Register Form -->
      <Card class="shadow-lg">
        <CardContent class="p-8">
          <div class="space-y-6">
            <form @submit.prevent="handleRegister" class="space-y-6">
              <!-- Username Field -->
              <div>
                <Label for="username" class="text-sm font-medium text-foreground mb-2">
                  用户名
                </Label>
                <Input
                  id="username"
                  v-model="form.username"
                  type="text"
                  placeholder="请输入用户名"
                  :disabled="isLoading"
                  required
                  :class="{ 'border-red-500': errors.username }"
                />
                <p v-if="errors.username" class="mt-1 text-sm text-red-600">
                  {{ errors.username }}
                </p>
              </div>

              <!-- Email Field -->
              <div>
                <Label for="email" class="text-sm font-medium text-foreground mb-2">
                  邮箱地址
                </Label>
                <Input
                  id="email"
                  v-model="form.email"
                  type="email"
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
                    placeholder="至少6位，包含大小写字母"
                    :disabled="isLoading"
                    required
                    class="pr-10"
                    :class="{ 'border-red-500': errors.password }"
                  />
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
                <!-- 密码强度提示 -->
                <div class="mt-2 space-y-1">
                  <p class="text-xs text-muted-foreground">密码需要包含：</p>
                  <div class="flex flex-wrap gap-2">
                    <span 
                      :class="[
                        'text-xs px-2 py-1 rounded',
                        passwordChecks.length ? 'bg-green-100 text-green-800' : 'bg-gray-100 text-gray-600'
                      ]"
                    >
                      至少6个字符
                    </span>
                    <span 
                      :class="[
                        'text-xs px-2 py-1 rounded',
                        passwordChecks.uppercase ? 'bg-green-100 text-green-800' : 'bg-gray-100 text-gray-600'
                      ]"
                    >
                      大写字母
                    </span>
                    <span 
                      :class="[
                        'text-xs px-2 py-1 rounded',
                        passwordChecks.lowercase ? 'bg-green-100 text-green-800' : 'bg-gray-100 text-gray-600'
                      ]"
                    >
                      小写字母
                    </span>
                  </div>
                </div>
              </div>

              <!-- Terms -->
              <div>
                <label class="flex items-start gap-2 cursor-pointer">
                  <Checkbox
                    id="terms"
                    v-model:checked="form.agreeTerms"
                    :disabled="isLoading"
                  />
                  <span class="text-sm text-muted-foreground leading-relaxed">
                    我已阅读并同意
                    <a href="#" class="text-primary hover:text-primary/80">服务条款</a>
                    和
                    <a href="#" class="text-primary hover:text-primary/80">隐私政策</a>
                  </span>
                </label>
                <p v-if="errors.terms" class="mt-1 text-sm text-red-600">
                  {{ errors.terms }}
                </p>
              </div>

              <!-- Error Message -->
              <Alert v-if="authStore.error" variant="destructive">
                {{ authStore.error }}
              </Alert>

              <!-- Submit Button -->
              <Button
                type="submit"
                :disabled="isLoading || !canSubmit"
                class="w-full"
              >
                <Spinner
                  v-if="isLoading"
                  class="mr-2 h-4 w-4 text-white"
                />
                {{ isLoading ? '注册中...' : '创建账号' }}
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

            <!-- Login Link -->
            <div class="mt-6 text-center">
              <p class="text-sm text-muted-foreground">
                已有账号？
                <router-link 
                  to="/auth/login" 
                  class="font-medium text-primary hover:text-primary/80"
                >
                  立即登录
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
import { validateEmail, validatePassword, validateUsername } from '@/utils/validation'
import { computed, reactive, ref } from 'vue'
import { useRouter } from 'vue-router'

const router = useRouter()
const authStore = useAuthStore()

const isLoading = ref(false)
const showPassword = ref(false)

const form = reactive({
  username: '',
  email: '',
  password: '',
  agreeTerms: false
})

const errors = reactive({
  username: '',
  email: '',
  password: '',
  terms: ''
})

// 密码强度检查
const passwordChecks = computed(() => ({
  length: form.password.length >= 6,
  uppercase: /[A-Z]/.test(form.password),
  lowercase: /[a-z]/.test(form.password)
}))

// 是否可以提交
const canSubmit = computed(() => {
  return form.username && 
         form.email && 
         form.password && 
         form.agreeTerms &&
         passwordChecks.value.length &&
         passwordChecks.value.uppercase &&
         passwordChecks.value.lowercase &&
         !errors.username && 
         !errors.email && 
         !errors.password
})

const validateForm = () => {
  errors.username = ''
  errors.email = ''
  errors.password = ''
  errors.terms = ''

  // 用户名验证
  if (!form.username || !form.username.trim()) {
    errors.username = '请输入用户名'
    return false
  }
  
  if (!validateUsername(form.username)) {
    errors.username = '用户名只能包含字母、数字和下划线，3-20个字符'
    return false
  }

  // 邮箱验证
  if (!form.email || !form.email.trim()) {
    errors.email = '请输入邮箱地址'
    return false
  }
  
  if (!validateEmail(form.email)) {
    errors.email = '请输入有效的邮箱地址'
    return false
  }

  // 密码验证
  if (!form.password) {
    errors.password = '请输入密码'
    return false
  }
  
  if (!validatePassword(form.password)) {
    errors.password = '密码不符合要求'
    return false
  }

  // 条款验证
  if (!form.agreeTerms) {
    errors.terms = '请同意服务条款和隐私政策'
    return false
  }

  return true
}

const handleRegister = async () => {
  if (!validateForm()) return

  isLoading.value = true
  
  try {
    await authStore.register({
      username: form.username,
      email: form.email,
      password: form.password
    })

    router.push('/')
  } catch (error) {
    console.error('Registration failed:', error)
  } finally {
    isLoading.value = false
  }
}
</script>