<template>
  <div class="text-xs text-muted-foreground mb-2 truncate">
    {{ resource.url }}
  </div>
</template>

<script setup lang="ts">
import type { Resource } from '@/types'

defineProps<{
  resource: Resource
}>()
</script>