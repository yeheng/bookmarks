<template>
  <div class="container mx-auto px-4 py-8">
    <div class="text-center">
      <h1 class="text-6xl font-bold mb-4">404</h1>
      <h2 class="text-2xl font-semibold mb-4">Page Not Found</h2>
      <p class="text-muted-foreground mb-8">The page you're looking for doesn't exist.</p>
      <RouterLink to="/" class="text-primary hover:underline">
        Go back home
      </RouterLink>
    </div>
  </div>
</template>

<script setup lang="ts">
import { RouterLink } from 'vue-router'
</script>