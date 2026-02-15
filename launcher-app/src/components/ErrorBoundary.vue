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
  <div class="error-boundary">
    <!-- Error Fallback UI -->
    <div v-if="hasError" class="error-fallback" role="alert" aria-live="assertive">
      <div class="error-content">
        <div class="error-icon-wrapper">
          <span class="error-icon" role="img" aria-label="Error">💥</span>
        </div>
        <h2 class="error-title">{{ displayTitle }}</h2>
        <p class="error-description">{{ displayDescription }}</p>
        <p v-if="error" class="error-details">
          {{ error.message }}
        </p>
        <div class="error-actions">
          <button
            class="error-btn error-btn--primary"
            @click="handleReset"
            aria-label="Try again"
          >
            <span class="btn-icon">🔄</span>
            Try Again
          </button>
          <button
            class="error-btn error-btn--secondary"
            @click="handleReload"
            aria-label="Reload application"
          >
            <span class="btn-icon">↻</span>
            Reload App
          </button>
        </div>
      </div>
    </div>

    <!-- Normal Content -->
    <slot v-else></slot>
  </div>
</template>

<style scoped>
.error-boundary {
  width: 100%;
  height: 100%;
}

.error-fallback {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 24px;
  background: var(--bg-color, rgba(26, 26, 26, 0.95));
}

.error-content {
  text-align: center;
  max-width: 320px;
}

.error-icon-wrapper {
  margin-bottom: 20px;
}

.error-icon {
  font-size: 64px;
  filter: drop-shadow(0 4px 8px rgba(0, 0, 0, 0.3));
}

.error-title {
  font-size: 18px;
  font-weight: 600;
  color: var(--text-color, #e0e0e0);
  margin-bottom: 8px;
}

.error-description {
  font-size: 14px;
  color: var(--secondary-text, #9a9a9a);
  margin-bottom: 12px;
  line-height: 1.5;
}

.error-details {
  font-size: 12px;
  color: var(--secondary-text, #9a9a9a);
  background: rgba(255, 107, 107, 0.1);
  border: 1px solid rgba(255, 107, 107, 0.2);
  border-radius: 6px;
  padding: 12px;
  margin-bottom: 20px;
  font-family: ui-monospace, SFMono-Regular, "SF Mono", Consolas, monospace;
  word-break: break-word;
  max-height: 100px;
  overflow-y: auto;
}

.error-actions {
  display: flex;
  gap: 12px;
  justify-content: center;
}

.error-btn {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 10px 16px;
  font-size: 14px;
  font-weight: 500;
  border: none;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.15s ease;
}

.error-btn:active {
  transform: scale(0.98);
}

.error-btn--primary {
  background: var(--accent-color, #ff6b6b);
  color: #fff;
}

.error-btn--primary:hover {
  background: #ff5252;
  box-shadow: 0 4px 12px rgba(255, 107, 107, 0.3);
}

.error-btn--secondary {
  background: rgba(128, 128, 128, 0.15);
  color: var(--text-color, #e0e0e0);
  border: 1px solid var(--border-color, #3a3a3a);
}

.error-btn--secondary:hover {
  background: rgba(128, 128, 128, 0.25);
  border-color: var(--secondary-text, #9a9a9a);
}

.btn-icon {
  font-size: 14px;
}
</style>
