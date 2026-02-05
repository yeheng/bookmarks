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
      :style="{ animationDelay: `${i * 0.1}s` }"
    >
      <div class="skeleton-icon"></div>
      <div class="skeleton-content">
        <div class="skeleton-title"></div>
        <div class="skeleton-subtitle"></div>
      </div>
      <div class="skeleton-badge"></div>
    </div>
  </div>
</template>

<style scoped>
.skeleton-container {
  display: flex;
  flex-direction: column;
  padding: 8px 0;
}

.skeleton-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 0 16px;
  height: var(--item-height, 44px);
}

.skeleton-icon {
  width: 24px;
  height: 24px;
  border-radius: 6px;
  background: linear-gradient(
    90deg,
    rgba(128, 128, 128, 0.1) 0%,
    rgba(128, 128, 128, 0.2) 50%,
    rgba(128, 128, 128, 0.1) 100%
  );
  background-size: 200% 100%;
}

.skeleton-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.skeleton-title {
  height: 14px;
  width: 60%;
  border-radius: 4px;
  background: linear-gradient(
    90deg,
    rgba(128, 128, 128, 0.1) 0%,
    rgba(128, 128, 128, 0.2) 50%,
    rgba(128, 128, 128, 0.1) 100%
  );
  background-size: 200% 100%;
}

.skeleton-subtitle {
  height: 12px;
  width: 80%;
  border-radius: 4px;
  background: linear-gradient(
    90deg,
    rgba(128, 128, 128, 0.08) 0%,
    rgba(128, 128, 128, 0.15) 50%,
    rgba(128, 128, 128, 0.08) 100%
  );
  background-size: 200% 100%;
}

.skeleton-badge {
  width: 60px;
  height: 20px;
  border-radius: 4px;
  background: linear-gradient(
    90deg,
    rgba(128, 128, 128, 0.08) 0%,
    rgba(128, 128, 128, 0.15) 50%,
    rgba(128, 128, 128, 0.08) 100%
  );
  background-size: 200% 100%;
}

/* Animation */
.skeleton-item--animated .skeleton-icon,
.skeleton-item--animated .skeleton-title,
.skeleton-item--animated .skeleton-subtitle,
.skeleton-item--animated .skeleton-badge {
  animation: skeleton-shimmer 1.5s ease-in-out infinite;
}

@keyframes skeleton-shimmer {
  0% {
    background-position: 200% 0;
  }
  100% {
    background-position: -200% 0;
  }
}

/* Staggered animation for each item */
.skeleton-item:nth-child(1) .skeleton-icon,
.skeleton-item:nth-child(1) .skeleton-title,
.skeleton-item:nth-child(1) .skeleton-subtitle,
.skeleton-item:nth-child(1) .skeleton-badge {
  animation-delay: 0s;
}

.skeleton-item:nth-child(2) .skeleton-icon,
.skeleton-item:nth-child(2) .skeleton-title,
.skeleton-item:nth-child(2) .skeleton-subtitle,
.skeleton-item:nth-child(2) .skeleton-badge {
  animation-delay: 0.15s;
}

.skeleton-item:nth-child(3) .skeleton-icon,
.skeleton-item:nth-child(3) .skeleton-title,
.skeleton-item:nth-child(3) .skeleton-subtitle,
.skeleton-item:nth-child(3) .skeleton-badge {
  animation-delay: 0.3s;
}
</style>
