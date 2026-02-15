<script setup lang="ts">
import { ref, computed } from 'vue';
import type { SearchResult } from '../types/search';
import { highlightParts } from '../composables/useHighlight';

interface Props {
  result: SearchResult;
  isSelected: boolean;
  highlightQuery?: string;
  index?: number;
}

const props = withDefaults(defineProps<Props>(), {
  highlightQuery: '',
  index: 0,
});

// Favicon state management
const faviconError = ref(false);

const hasFavicon = computed(() => {
  return props.result.type === 'bookmark' && props.result.icon && !faviconError.value;
});

const isPlugin = computed(() => props.result.type === 'plugin');

/** Plugin icon: prefer emoji, then url, then fallback to puzzle piece */
const pluginIconEmoji = computed(() => {
  if (!isPlugin.value) return null;
  const icon = props.result.icon;
  // If icon looks like an emoji (1-4 chars, non-ASCII), render as emoji
  if (icon && icon.length <= 4 && /[^\x00-\x7F]/.test(icon)) {
    return icon;
  }
  return null;
});

const pluginIconUrl = computed(() => {
  if (!isPlugin.value || pluginIconEmoji.value) return null;
  const icon = props.result.icon;
  if (icon && (icon.startsWith('http') || icon.startsWith('/'))) {
    return icon;
  }
  return null;
});

const faviconFallbackLetter = computed(() => {
  if (props.result.metadata?.domain) {
    return props.result.metadata.domain.charAt(0).toUpperCase();
  }
  if (props.result.url) {
    try {
      return new URL(props.result.url).hostname.charAt(0).toUpperCase();
    } catch { /* ignore */ }
  }
  return props.result.title.charAt(0).toUpperCase();
});

const handleFaviconError = () => {
  faviconError.value = true;
};

// Highlighted text parts (structured data, no v-html needed)
const titleParts = computed(() =>
  highlightParts(props.result.title, props.highlightQuery)
);

const subtitleParts = computed(() =>
  highlightParts(props.result.subtitle, props.highlightQuery)
);

const adaptiveMetadata = computed(() => {
  if (!props.result.metadata) return null;

  const parts: string[] = [];
  const md = props.result.metadata;

  if (md?.domain) parts.push(md.domain);
  if (md?.modified) parts.push(md.modified);
  if (md?.size) parts.push(md.size);

  return parts.length > 0 ? parts.join(' · ') : null;
});
</script>

<template>
  <div
    class="result-item"
    :class="{ 'result-item--selected': isSelected }"
    role="option"
    :aria-selected="isSelected"
    :aria-label="`${result.title}, ${result.type === 'bookmark' ? 'Bookmark' : result.type === 'file' ? 'File' : 'Plugin Result'}, ${result.subtitle}`"
  >
    <!-- Icon -->
    <div class="result-icon" aria-hidden="true">
      <!-- Plugin icons -->
      <div v-if="isPlugin && pluginIconEmoji" class="icon-plugin-emoji">
        {{ pluginIconEmoji }}
      </div>
      <img
        v-else-if="isPlugin && pluginIconUrl"
        :src="pluginIconUrl"
        class="favicon-img"
        alt=""
        loading="lazy"
      />
      <div v-else-if="isPlugin" class="icon-plugin" :class="{ 'icon-plugin--selected': isSelected }">
        <svg width="16" height="16" viewBox="0 0 16 16" fill="none" xmlns="http://www.w3.org/2000/svg">
          <path d="M9.5 1a1.5 1.5 0 0 1 1.5 1.5V4h1a2 2 0 0 1 2 2v1.5h-1.5a1.5 1.5 0 0 0 0 3H14V12a2 2 0 0 1-2 2H6v-1.5a1.5 1.5 0 0 0-3 0V14H2a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h1.5V2.5a1.5 1.5 0 0 1 3 0V4H9.5V2.5A1.5 1.5 0 0 1 9.5 1Z" fill="currentColor" opacity="0.7"/>
        </svg>
      </div>
      <!-- Bookmark icons -->
      <img
        v-else-if="hasFavicon"
        :src="result.icon"
        class="favicon-img"
        alt=""
        @error="handleFaviconError"
        loading="lazy"
      />
      <div
        v-else-if="result.type === 'bookmark'"
        class="icon-placeholder"
        :class="{ 'icon-placeholder--selected': isSelected }"
      >
        <span class="icon-letter">{{ faviconFallbackLetter }}</span>
      </div>
      <!-- File icons -->
      <div v-else class="icon-file" :class="{ 'icon-file--selected': isSelected }">
        <svg width="16" height="16" viewBox="0 0 16 16" fill="none" xmlns="http://www.w3.org/2000/svg">
          <path d="M4 1.5h5.172a2 2 0 0 1 1.414.586l2.828 2.828A2 2 0 0 1 14 6.328V12.5a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2v-9a2 2 0 0 1 2-2Z" fill="currentColor" opacity="0.7"/>
        </svg>
      </div>
    </div>

    <!-- Content -->
    <div class="result-body">
      <div class="result-title"><template v-for="(part, i) in titleParts" :key="i"><mark v-if="part.highlight" class="search-highlight">{{ part.text }}</mark><template v-else>{{ part.text }}</template></template></div>
      <div v-if="adaptiveMetadata" class="result-meta">{{ adaptiveMetadata }}</div>
      <div v-else class="result-subtitle"><template v-for="(part, i) in subtitleParts" :key="i"><mark v-if="part.highlight" class="search-highlight">{{ part.text }}</mark><template v-else>{{ part.text }}</template></template></div>
    </div>

    <!-- Plugin badge -->
    <span v-if="result.pluginBadge" class="plugin-badge" :class="{ 'plugin-badge--selected': isSelected }">
      {{ result.pluginBadge }}
    </span>

    <!-- Action hint -->
    <div v-if="isSelected" class="action-hint" aria-hidden="true">
      <svg width="12" height="12" viewBox="0 0 12 12" fill="none" xmlns="http://www.w3.org/2000/svg">
        <path d="M2.5 6.5h5m0 0L5 9m2.5-2.5L5 4" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round" transform="rotate(-45 6 6)"/>
      </svg>
    </div>

    <!-- Keyboard shortcuts hint -->
    <Transition name="hint-fade">
      <div v-if="isSelected" class="keyboard-hints" aria-hidden="true">
        <span class="hint-key">↵</span>
      </div>
    </Transition>
  </div>
</template>

<style scoped>
.result-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 10px;
  border-radius: 8px;
  cursor: pointer;
  color: var(--text-color);
  transition: background-color 0.1s ease;
  min-height: 40px;
}

.result-item:hover:not(.result-item--selected) {
  background-color: rgba(128, 128, 128, 0.08);
}

/* ===== Spotlight-style selected state: accent fill + white text ===== */
.result-item--selected {
  background: var(--accent-color, #007AFF);
  color: #ffffff;
  animation: select-pulse 0.2s ease-out;
}

@keyframes select-pulse {
  0% {
    transform: scale(1);
    box-shadow: none;
  }
  50% {
    transform: scale(1.005);
    box-shadow: 0 2px 8px rgba(var(--color-accent-rgb, 0, 122, 255), 0.3);
  }
  100% {
    transform: scale(1);
    box-shadow: none;
  }
}

.result-item--selected .result-title {
  color: #ffffff;
}

.result-item--selected .result-subtitle,
.result-item--selected .result-meta {
  color: rgba(255, 255, 255, 0.72);
}

.result-item--selected .action-hint {
  color: rgba(255, 255, 255, 0.6);
}

/* ===== Icon ===== */
.result-icon {
  flex-shrink: 0;
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  align-self: center;
  transition: transform 0.15s ease;
}

.result-item:hover .result-icon {
  transform: scale(1.05);
}

.favicon-img {
  width: 18px;
  height: 18px;
  border-radius: 4px;
  object-fit: contain;
}

.icon-placeholder {
  width: 28px;
  height: 28px;
  border-radius: 7px;
  background: rgba(128, 128, 128, 0.12);
  display: flex;
  align-items: center;
  justify-content: center;
}

.icon-placeholder--selected {
  background: rgba(255, 255, 255, 0.2);
}

.icon-letter {
  font-size: 11px;
  font-weight: 600;
  color: var(--color-text-secondary);
}

.result-item--selected .icon-letter {
  color: #ffffff;
}

.icon-file {
  color: var(--color-text-tertiary);
}

.icon-file--selected {
  color: rgba(255, 255, 255, 0.8);
}

/* ===== Content ===== */
.result-body {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 1px;
}

.result-title {
  font-size: 13px;
  font-weight: 500;
  white-space: nowrap;
  color: var(--color-text-primary, var(--text-color));
  overflow: hidden;
  text-overflow: ellipsis;
  line-height: 1.3;
}

.result-subtitle {
  font-size: 11px;
  font-weight: 400;
  color: var(--color-text-tertiary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  line-height: 1.3;
}

.result-meta {
  font-size: 11px;
  font-weight: 400;
  color: var(--color-text-tertiary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  line-height: 1.3;
  font-feature-settings: 'tnum';
}

/* Search highlight marks - Spotlight style */
.search-highlight {
  background: rgba(0, 122, 255, 0.15);
  color: inherit;
  border-radius: 3px;
  padding: 1px 3px;
  margin: 0 -1px;
}

.result-item--selected .search-highlight {
  background: rgba(255, 255, 255, 0.3);
  color: #ffffff;
  font-weight: 600;
}

/* ===== Action hint ===== */
.action-hint {
  flex-shrink: 0;
  color: rgba(255, 255, 255, 0.5);
  display: flex;
  align-items: center;
}

/* ===== Keyboard hints ===== */
.keyboard-hints {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  gap: 4px;
  margin-left: 4px;
}

.hint-key {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 18px;
  height: 18px;
  padding: 0 4px;
  font-size: 10px;
  font-weight: 500;
  color: rgba(255, 255, 255, 0.6);
  background: rgba(255, 255, 255, 0.12);
  border-radius: 4px;
  line-height: 1;
}

/* Hint fade transition */
.hint-fade-enter-active {
  transition: opacity 0.15s ease-out;
}

.hint-fade-leave-active {
  transition: opacity 0.1s ease-in;
}

.hint-fade-enter-from,
.hint-fade-leave-to {
  opacity: 0;
}

/* ===== Plugin icons ===== */
.icon-plugin-emoji {
  font-size: 18px;
  line-height: 1;
  display: flex;
  align-items: center;
  justify-content: center;
}

.icon-plugin {
  width: 28px;
  height: 28px;
  border-radius: 7px;
  background: rgba(128, 128, 128, 0.12);
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--color-text-tertiary);
}

.icon-plugin--selected {
  background: rgba(255, 255, 255, 0.2);
  color: rgba(255, 255, 255, 0.8);
}

/* ===== Plugin badge ===== */
.plugin-badge {
  flex-shrink: 0;
  font-size: 10px;
  font-weight: 500;
  padding: 1px 6px;
  border-radius: 4px;
  background: rgba(128, 128, 128, 0.12);
  color: var(--color-text-tertiary);
  line-height: 1.4;
}

.plugin-badge--selected {
  background: rgba(255, 255, 255, 0.2);
  color: rgba(255, 255, 255, 0.7);
}

/* Reduced motion */
@media (prefers-reduced-motion: reduce) {
  .result-item {
    transition: none;
  }
  .result-item--selected {
    animation: none;
  }
  .hint-fade-enter-active,
  .hint-fade-leave-active {
    transition: none;
  }
  .result-icon {
    transition: none;
  }
  .result-item:hover .result-icon {
    transform: none;
  }
}
</style>
