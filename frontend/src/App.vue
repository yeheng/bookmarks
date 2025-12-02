<template>
  <div id="app" class="min-h-screen bg-background">
    <Navigation />
    <main class="flex-1">
      <RouterView v-slot="{ Component }">
        <Transition name="fade" mode="out-in">
          <component :is="Component" />
        </Transition>
      </RouterView>
    </main>
  </div>
</template>

<script setup lang="ts">
import Navigation from '@/components/Navigation.vue'
import { useAuthStore } from '@/stores/auth'
import { onMounted } from 'vue'
import { RouterView } from 'vue-router'

const authStore = useAuthStore()

onMounted(() => {
  authStore.initializeAuth()
})
</script>