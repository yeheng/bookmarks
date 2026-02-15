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
    class="flex items-center gap-2.5 px-2.5 py-2 rounded-lg cursor-pointer text-text-primary transition-colors duration-100 min-h-[40px] hover:bg-white/10 group motion-reduce:transition-none"
    :class="{ 'bg-accent text-white hover:bg-accent animate-[select-pulse_0.2s_ease-out] motion-reduce:animate-none': isSelected }"
    role="option"
    :aria-selected="isSelected"
    :aria-label="`${result.title}, ${result.type === 'bookmark' ? 'Bookmark' : result.type === 'file' ? 'File' : 'Plugin Result'}, ${result.subtitle}`"
    :data-selected="isSelected"
  >
    <!-- Icon -->
    <div class="shrink-0 w-8 h-8 flex items-center justify-center self-center transition-transform duration-150 group-hover:scale-105 motion-reduce:transition-none motion-reduce:transform-none" aria-hidden="true">
      <!-- Plugin icons -->
      <div v-if="isPlugin && pluginIconEmoji" class="text-lg leading-none flex items-center justify-center">
        {{ pluginIconEmoji }}
      </div>
      <img
        v-else-if="isPlugin && pluginIconUrl"
        :src="pluginIconUrl"
        class="w-[18px] h-[18px] rounded object-contain"
        alt=""
        loading="lazy"
      />
      <div v-else-if="isPlugin" class="w-7 h-7 rounded-[7px] bg-white/10 flex items-center justify-center text-text-tertiary group-data-[selected=true]:bg-white/20 group-data-[selected=true]:text-white/80">
        <svg width="16" height="16" viewBox="0 0 16 16" fill="none" xmlns="http://www.w3.org/2000/svg">
          <path d="M9.5 1a1.5 1.5 0 0 1 1.5 1.5V4h1a2 2 0 0 1 2 2v1.5h-1.5a1.5 1.5 0 0 0 0 3H14V12a2 2 0 0 1-2 2H6v-1.5a1.5 1.5 0 0 0-3 0V14H2a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h1.5V2.5a1.5 1.5 0 0 1 3 0V4H9.5V2.5A1.5 1.5 0 0 1 9.5 1Z" fill="currentColor" opacity="0.7"/>
        </svg>
      </div>
      <!-- Bookmark icons -->
      <img
        v-else-if="hasFavicon"
        :src="result.icon"
        class="w-[18px] h-[18px] rounded object-contain"
        alt=""
        @error="handleFaviconError"
        loading="lazy"
      />
      <div
        v-else-if="result.type === 'bookmark'"
        class="w-7 h-7 rounded-[7px] bg-white/10 flex items-center justify-center group-data-[selected=true]:bg-white/20"
      >
        <span class="text-[11px] font-semibold text-text-secondary group-data-[selected=true]:text-white">{{ faviconFallbackLetter }}</span>
      </div>
      <!-- File icons -->
      <div v-else class="text-text-tertiary group-data-[selected=true]:text-white/80">
        <svg width="16" height="16" viewBox="0 0 16 16" fill="none" xmlns="http://www.w3.org/2000/svg">
          <path d="M4 1.5h5.172a2 2 0 0 1 1.414.586l2.828 2.828A2 2 0 0 1 14 6.328V12.5a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2v-9a2 2 0 0 1 2-2Z" fill="currentColor" opacity="0.7"/>
        </svg>
      </div>
    </div>

    <!-- Content -->
    <div class="flex-1 min-w-0 flex flex-col gap-[1px]">
      <div class="text-[13px] font-medium whitespace-nowrap text-text-primary overflow-hidden text-ellipsis leading-tight group-data-[selected=true]:text-white">
        <template v-for="(part, i) in titleParts" :key="i"><mark v-if="part.highlight" class="bg-accent/15 text-inherit rounded-[3px] px-[3px] mx-[-1px] group-data-[selected=true]:bg-white/30 group-data-[selected=true]:text-white group-data-[selected=true]:font-semibold">{{ part.text }}</mark><template v-else>{{ part.text }}</template></template>
      </div>
      <div v-if="adaptiveMetadata" class="text-[11px] font-normal text-text-tertiary whitespace-nowrap overflow-hidden text-ellipsis leading-tight tabular-nums group-data-[selected=true]:text-white/70">{{ adaptiveMetadata }}</div>
      <div v-else class="text-[11px] font-normal text-text-tertiary whitespace-nowrap overflow-hidden text-ellipsis leading-tight group-data-[selected=true]:text-white/70">
        <template v-for="(part, i) in subtitleParts" :key="i"><mark v-if="part.highlight" class="bg-accent/15 text-inherit rounded-[3px] px-[3px] mx-[-1px] group-data-[selected=true]:bg-white/30 group-data-[selected=true]:text-white group-data-[selected=true]:font-semibold">{{ part.text }}</mark><template v-else>{{ part.text }}</template></template>
      </div>
    </div>

    <!-- Plugin badge -->
    <span v-if="result.pluginBadge" class="shrink-0 text-[10px] font-medium px-1.5 py-[1px] rounded bg-white/10 text-text-tertiary leading-tight group-data-[selected=true]:bg-white/20 group-data-[selected=true]:text-white/70">
      {{ result.pluginBadge }}
    </span>

    <!-- Action hint -->
    <div v-if="isSelected" class="shrink-0 text-white/50 flex items-center" aria-hidden="true">
      <svg width="12" height="12" viewBox="0 0 12 12" fill="none" xmlns="http://www.w3.org/2000/svg">
        <path d="M2.5 6.5h5m0 0L5 9m2.5-2.5L5 4" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round" transform="rotate(-45 6 6)"/>
      </svg>
    </div>

    <!-- Keyboard shortcuts hint -->
    <Transition 
      enter-active-class="transition-opacity duration-150 ease-out"
      enter-from-class="opacity-0"
      leave-active-class="transition-opacity duration-100 ease-in"
      leave-to-class="opacity-0"
    >
      <div v-if="isSelected" class="shrink-0 flex items-center gap-1 ml-1 motion-reduce:transition-none" aria-hidden="true">
        <span class="inline-flex items-center justify-center min-w-[18px] h-[18px] px-1 text-[10px] font-medium text-white/60 bg-white/10 rounded leading-none">↵</span>
      </div>
    </Transition>
  </div>
</template>

<style scoped>
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
</style>

