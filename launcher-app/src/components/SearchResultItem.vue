<script setup lang="ts">
import type { SearchResult } from '../types/search';

interface Props {
  result: SearchResult;
  isSelected: boolean;
}

defineProps<Props>();
</script>

<template>
  <div
    class="result-item"
    :class="{ 'result-item--selected': isSelected }"
  >
    <div class="result-icon">
      <span v-if="result.type === 'bookmark'" class="icon">🔖</span>
      <span v-else class="icon">📄</span>
    </div>
    <div class="result-content">
      <div class="result-title">{{ result.title }}</div>
      <div class="result-subtitle">{{ result.subtitle }}</div>
    </div>
    <div class="result-badge">
      {{ result.type === 'bookmark' ? 'Bookmark' : 'File' }}
    </div>
  </div>
</template>

<style scoped>
.result-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
  border-radius: 6px;
  cursor: pointer;
  transition: all 100ms ease-in-out;
}

.result-item:hover {
  background-color: rgba(255, 255, 255, 0.05);
  transform: translateX(2px);
}

.result-item--selected {
  background-color: rgba(255, 107, 107, 0.15);
  transform: translateX(4px);
}

.result-icon {
  flex-shrink: 0;
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: rgba(255, 255, 255, 0.05);
  border-radius: 6px;
  transition: transform 100ms ease-in-out;
}

.result-item--selected .result-icon {
  transform: scale(1.05);
}

.icon {
  font-size: 18px;
}

.result-content {
  flex: 1;
  min-width: 0;
  overflow: hidden;
}

.result-title {
  font-size: 14px;
  font-weight: 500;
  color: #e0e0e0;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  transition: color 100ms ease-in-out;
}

.result-item--selected .result-title {
  color: #ffffff;
}

.result-subtitle {
  font-size: 12px;
  color: #9a9a9a;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  margin-top: 2px;
  transition: color 100ms ease-in-out;
}

.result-item--selected .result-subtitle {
  color: #b0b0b0;
}

.result-badge {
  flex-shrink: 0;
  font-size: 11px;
  color: #9a9a9a;
  padding: 2px 8px;
  background-color: rgba(255, 255, 255, 0.05);
  border-radius: 4px;
  transition: all 100ms ease-in-out;
}

.result-item--selected .result-badge {
  background-color: rgba(255, 107, 107, 0.2);
  color: #ff6b6b;
}

@media (prefers-color-scheme: light) {
  .result-title {
    color: #1a1a1a;
  }

  .result-item--selected .result-title {
    color: #000000;
  }

  .result-subtitle {
    color: #6a6a6a;
  }

  .result-item--selected .result-subtitle {
    color: #4a4a4a;
  }

  .result-badge {
    color: #6a6a6a;
  }

  .result-item:hover {
    background-color: rgba(0, 0, 0, 0.03);
  }

  .result-item--selected {
    background-color: rgba(255, 107, 107, 0.1);
  }

  .result-icon {
    background-color: rgba(0, 0, 0, 0.03);
  }

  .result-badge {
    background-color: rgba(0, 0, 0, 0.05);
  }

  .result-item--selected .result-badge {
    background-color: rgba(255, 107, 107, 0.15);
    color: #ff6b6b;
  }
}
</style>
