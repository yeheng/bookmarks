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
  <div class="skeleton-container" role="status" aria-label="Loading results" aria-busy="true">
    <div
      v-for="i in count"
      :key="i"
      class="skeleton-item"
      :class="{ 'skeleton-item--animated': animated }"
    >
      <div class="skeleton-icon"></div>
      <div class="skeleton-content">
        <div class="skeleton-title" :style="{ width: `${55 + i * 10}%` }"></div>
        <div class="skeleton-subtitle" :style="{ width: `${70 + i * 5}%` }"></div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.skeleton-container {
  display: flex;
  flex-direction: column;
  padding: 2px 0;
}

.skeleton-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 6px 10px;
  min-height: 36px;
}

.skeleton-icon {
  width: 24px;
  height: 24px;
  border-radius: 6px;
  background: rgba(128, 128, 128, 0.08);
  flex-shrink: 0;
}

.skeleton-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.skeleton-title {
  height: 12px;
  border-radius: 3px;
  background: rgba(128, 128, 128, 0.1);
}

.skeleton-subtitle {
  height: 10px;
  border-radius: 3px;
  background: rgba(128, 128, 128, 0.06);
}

/* Shimmer animation */
.skeleton-item--animated .skeleton-icon,
.skeleton-item--animated .skeleton-title,
.skeleton-item--animated .skeleton-subtitle {
  background-image: linear-gradient(
    90deg,
    transparent 0%,
    rgba(128, 128, 128, 0.06) 50%,
    transparent 100%
  );
  background-size: 200% 100%;
  animation: skeleton-shimmer 1.5s ease-in-out infinite;
}

.skeleton-item:nth-child(1) * { animation-delay: 0s; }
.skeleton-item:nth-child(2) * { animation-delay: 0.1s; }
.skeleton-item:nth-child(3) * { animation-delay: 0.2s; }

@keyframes skeleton-shimmer {
  0% { background-position: 200% 0; }
  100% { background-position: -200% 0; }
}

/* Reduced motion */
@media (prefers-reduced-motion: reduce) {
  .skeleton-item--animated .skeleton-icon,
  .skeleton-item--animated .skeleton-title,
  .skeleton-item--animated .skeleton-subtitle {
    animation: none;
    background-image: none;
  }
}
</style>
