<script setup lang="ts">
import { ref, computed, onErrorCaptured } from 'vue';

interface Props {
  fallbackTitle?: string;
  fallbackDescription?: string;
}

const props = withDefaults(defineProps<Props>(), {
  fallbackTitle: 'Something went wrong',
  fallbackDescription: 'An unexpected error occurred. Please try again.',
});

// Use props in computed for fallback display
const displayTitle = computed(() => props.fallbackTitle);
const displayDescription = computed(() => props.fallbackDescription);

const emit = defineEmits<{
  (e: 'error', error: Error): void;
}>();

const error = ref<Error | null>(null);
const hasError = ref(false);

// Capture errors from child components
onErrorCaptured((err: Error, _instance, info) => {
  console.error('ErrorBoundary caught error:', err, info);
  error.value = err;
  hasError.value = true;
  emit('error', err);
  // Return false to stop error propagation
  return false;
});

const handleReset = () => {
  error.value = null;
  hasError.value = false;
};

const handleReload = () => {
  window.location.reload();
};

defineExpose({
  reset: handleReset,
  hasError,
});
</script>

<template>
  <div class="w-full h-full">
    <!-- Error Fallback UI -->
    <div v-if="hasError" class="w-full h-full flex items-center justify-center p-6 bg-bg-primary/95" role="alert" aria-live="assertive">
      <div class="text-center max-w-xs">
        <div class="mb-5">
          <span class="text-6xl drop-shadow-lg" role="img" aria-label="Error">💥</span>
        </div>
        <h2 class="text-lg font-semibold text-text-primary mb-2">{{ displayTitle }}</h2>
        <p class="text-sm text-text-secondary mb-3 leading-relaxed">{{ displayDescription }}</p>
        <p v-if="error" class="text-xs text-text-secondary bg-red-500/10 border border-red-500/20 rounded-md p-3 mb-5 font-mono break-words max-h-[100px] overflow-y-auto">
          {{ error.message }}
        </p>
        <div class="flex gap-3 justify-center">
          <button
            class="inline-flex items-center gap-2 px-4 py-2.5 text-sm font-medium border-none rounded-lg cursor-pointer transition-all duration-150 active:scale-95 bg-accent text-white hover:bg-red-500 hover:shadow-lg"
            @click="handleReset"
            aria-label="Try again"
          >
            <span class="text-sm">🔄</span>
            Try Again
          </button>
          <button
            class="inline-flex items-center gap-2 px-4 py-2.5 text-sm font-medium rounded-lg cursor-pointer transition-all duration-150 active:scale-95 bg-white/15 text-text-primary border border-border-default hover:bg-white/25 hover:border-text-secondary"
            @click="handleReload"
            aria-label="Reload application"
          >
            <span class="text-sm">↻</span>
            Reload App
          </button>
        </div>
      </div>
    </div>

    <!-- Normal Content -->
    <slot v-else></slot>
  </div>
</template>

