<script setup lang="ts">
interface Props {
  count?: number;
  animated?: boolean;
}

withDefaults(defineProps<Props>(), {
  count: 3,
  animated: true,
});
</script>

<template>
  <div class="flex flex-col py-0.5" role="status" aria-label="Loading results" aria-busy="true">
    <div
      v-for="i in count"
      :key="i"
      class="flex items-center gap-2.5 px-2.5 py-1.5 min-h-[36px]"
    >
      <div class="w-6 h-6 rounded-md bg-white/10 shrink-0" :class="{ 'animate-pulse': animated }"></div>
      <div class="flex-1 flex flex-col gap-1">
        <div class="h-3 rounded-sm bg-white/10" :class="{ 'animate-pulse': animated }" :style="{ width: `${55 + i * 10}%` }"></div>
        <div class="h-2.5 rounded-sm bg-white/5" :class="{ 'animate-pulse': animated }" :style="{ width: `${70 + i * 5}%` }"></div>
      </div>
    </div>
  </div>
</template>

