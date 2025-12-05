import { apiService } from '@/services/api'
import type { LoginRequest, RegisterRequest, User } from '@/types'
import { defineStore } from 'pinia'
import { computed, ref } from 'vue'

export const useAuthStore = defineStore('auth', () => {
  const user = ref<User | null>(null)
  const token = ref<string | null>(null)
  const isLoading = ref(false)
  const error = ref<string | null>(null)

  const isAuthenticated = computed(() => !!token.value && !!user.value)

  const initializeAuth = async () => {
    const savedToken = localStorage.getItem('auth_token')
    
    if (savedToken) {
      token.value = savedToken
      apiService.setToken(savedToken)
      
      try {
        // Verify token validity by fetching current user
        await fetchCurrentUser()
      } catch (error) {
        // If token is invalid, clear it and don't redirect to login immediately
        // Let the router guard handle the redirect
        console.warn('Token validation failed during initialization:', error)
      }
    } else {
      console.log('initializeAuth: no token found in localStorage')
    }
  }

  const login = async (credentials: LoginRequest): Promise<void> => {
    isLoading.value = true
    error.value = null
    
    try {
      const response = await apiService.login(credentials)
      user.value = response.user
      token.value = response.token
      // API service already handles localStorage
    } catch (err: any) {
      error.value = err.message || 'Login failed'
      throw err
    } finally {
      isLoading.value = false
    }
  }

  const register = async (userData: RegisterRequest): Promise<void> => {
    isLoading.value = true
    error.value = null
    
    try {
      const response = await apiService.register(userData)
      user.value = response.user
      token.value = response.token
      // API service already handles localStorage
    } catch (err: any) {
      error.value = err.message || 'Registration failed'
      throw err
    } finally {
      isLoading.value = false
    }
  }

  const logout = async (): Promise<void> => {
    isLoading.value = true
    error.value = null
    
    try {
      await apiService.logout()
    } catch (err: any) {
      error.value = err.message || 'Logout failed'
    } finally {
      user.value = null
      token.value = null
      // API service already handles localStorage cleanup
      apiService.setToken(null)
      isLoading.value = false
    }
  }

  const fetchCurrentUser = async (): Promise<void> => {
    if (!token.value) return
    
    try {
      const response = await apiService.getCurrentUser()
      user.value = response.data
    } catch (err: any) {
      // Token might be invalid, clear auth state
      user.value = null
      token.value = null
      // API service already handles localStorage cleanup
      apiService.setToken(null)
      error.value = err.message || 'Failed to fetch user'
      throw err // Re-throw to let caller handle the error
    }
  }

  return {
    user,
    token,
    isLoading,
    error,
    isAuthenticated,
    initializeAuth,
    login,
    register,
    logout,
    fetchCurrentUser
  }
})