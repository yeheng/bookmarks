<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';

interface Emits {
  (e: 'complete'): void;
  (e: 'skip'): void;
}

const emit = defineEmits<Emits>();

const currentStep = ref(0);

const features = [
  {
    icon: '🔍',
    title: 'Lightning-Fast Search',
    description: 'Search through your bookmarks and files instantly with fuzzy matching',
    shortcut: 'Cmd+F1',
  },
  {
    icon: '⚡',
    title: 'Frecency Ranking',
    description: 'Results are ranked by frequency and recency - the more you use them, the higher they appear',
    shortcut: '↑↓ to navigate',
  },
  {
    icon: '⌨️',
    title: 'Keyboard-First',
    description: 'Navigate everything with your keyboard. No mouse needed!',
    shortcut: '⏎ to open',
  },
  {
    icon: '🎨',
    title: 'Customizable',
    description: 'Adjust themes, shortcuts, and preferences to match your workflow',
    shortcut: 'Cmd+, for settings',
  },
];

const nextStep = () => {
  if (currentStep.value < features.length - 1) {
    currentStep.value++;
  } else {
    emit('complete');
  }
};

const previousStep = () => {
  if (currentStep.value > 0) {
    currentStep.value--;
  }
};

const skipTutorial = () => {
  emit('skip');
};

const completeOnboarding = () => {
  emit('complete');
};

// Global keyboard navigation
const handleGlobalKeydown = (e: KeyboardEvent) => {
  switch (e.key) {
    case 'ArrowLeft':
      previousStep();
      break;
    case 'ArrowRight':
      nextStep();
      break;
    case 'Escape':
      skipTutorial();
      break;
    case 'Enter':
      if (currentStep.value === features.length - 1) {
        completeOnboarding();
      } else {
        nextStep();
      }
      break;
  }
};

onMounted(() => {
  document.addEventListener('keydown', handleGlobalKeydown);
});

onUnmounted(() => {
  document.removeEventListener('keydown', handleGlobalKeydown);
});
</script>

<template>
  <div
    class="fixed inset-0 z-[9999] flex items-center justify-center animate-in fade-in duration-300"
    role="dialog"
    aria-modal="true"
    aria-labelledby="welcome-title"
    aria-describedby="welcome-subtitle"
  >
    <div class="absolute inset-0 bg-black/85 backdrop-blur-sm" @click="skipTutorial" aria-hidden="true"></div>

    <div class="relative z-[10000] w-[90%] max-w-[600px] bg-bg-primary rounded-2xl p-12 shadow-2xl animate-in slide-in-from-bottom-10 duration-400 ease-[cubic-bezier(0.68,-0.55,0.265,1.55)]">
      <!-- Header -->
      <div class="text-center mb-10">
        <h1 id="welcome-title" class="text-3xl font-bold text-text-primary mb-2">Welcome to Bookmark Launcher! 🚀</h1>
        <p id="welcome-subtitle" class="text-base text-text-secondary">Let's get you started with a quick tour</p>
        <p class="text-xs text-text-tertiary mt-3 opacity-80" aria-live="polite">
          Use ← → arrow keys to navigate, Enter to continue, Escape to skip
        </p>
      </div>

      <!-- Feature Showcase -->
      <div class="relative min-h-[280px] mb-8" role="region" aria-label="Feature tour">
        <TransitionGroup 
          enter-active-class="transition-all duration-300 ease-[cubic-bezier(0.4,0,0.2,1)]"
          enter-from-class="opacity-0 translate-x-8"
          leave-active-class="transition-all duration-300 ease-[cubic-bezier(0.4,0,0.2,1)] absolute w-full top-0 left-0"
          leave-to-class="opacity-0 -translate-x-8"
        >
          <div
            v-for="(feature, index) in features"
            v-show="index === currentStep"
            :key="index"
            class="text-center w-full"
            role="tabpanel"
            :id="`feature-panel-${index}`"
            :aria-labelledby="`feature-title-${index}`"
          >
            <div class="text-7xl mb-6 leading-none" aria-hidden="true">{{ feature.icon }}</div>
            <h2 :id="`feature-title-${index}`" class="text-2xl font-semibold text-text-primary mb-3">{{ feature.title }}</h2>
            <p class="text-base text-text-secondary leading-relaxed mb-5 max-w-[400px] mx-auto">{{ feature.description }}</p>
            <div class="mt-4">
              <kbd class="inline-block px-4 py-2 bg-white/10 border border-white/20 rounded-lg font-mono text-sm text-accent shadow-sm" aria-label="Keyboard shortcut">{{ feature.shortcut }}</kbd>
            </div>
          </div>
        </TransitionGroup>
      </div>

      <!-- Progress Indicators -->
      <div class="flex justify-center gap-2 mb-8" role="tablist" aria-label="Tutorial steps">
        <button
          v-for="(feature, index) in features"
          :key="index"
          class="w-2 h-2 rounded-full bg-white/30 border-none cursor-pointer transition-all duration-300 p-0 hover:bg-white/50 hover:scale-125"
          :class="{ 'bg-accent w-6 rounded': index === currentStep }"
          @click="currentStep = index"
          role="tab"
          :aria-selected="index === currentStep"
          :aria-label="`Step ${index + 1} of ${features.length}: ${feature.title}`"
          :aria-controls="`feature-panel-${index}`"
        ></button>
      </div>

      <!-- Screen reader status -->
      <div class="sr-only" aria-live="polite" aria-atomic="true">
        Step {{ currentStep + 1 }} of {{ features.length }}: {{ features[currentStep].title }}
      </div>

      <!-- Actions -->
      <div class="flex justify-between items-center gap-4">
        <button
          v-if="currentStep > 0"
          @click="previousStep"
          class="px-6 py-3 rounded-lg text-sm font-semibold cursor-pointer transition-all duration-150 border-none bg-white/10 text-text-primary border border-white/20 min-w-[100px] hover:bg-white/15"
          aria-label="Go to previous step"
        >
          ← Previous
        </button>
        <div v-else class="w-[100px]"></div>

        <button
          @click="skipTutorial"
          class="bg-transparent text-text-secondary underline hover:text-text-primary text-sm cursor-pointer border-none"
          aria-label="Skip the tutorial"
        >
          Skip Tutorial
        </button>

        <button
          v-if="currentStep < features.length - 1"
          @click="nextStep"
          class="px-6 py-3 rounded-lg text-sm font-semibold cursor-pointer transition-all duration-150 border-none bg-accent text-white min-w-[120px] hover:-translate-y-px hover:shadow-lg"
          aria-label="Go to next step"
        >
          Next →
        </button>
        <button
          v-else
          @click="completeOnboarding"
          class="px-6 py-3 rounded-lg text-sm font-semibold cursor-pointer transition-all duration-150 border-none bg-gradient-to-br from-emerald-500 to-emerald-700 text-white min-w-[120px] hover:-translate-y-px hover:shadow-lg"
          aria-label="Complete tutorial and start using the app"
        >
          Get Started ✓
        </button>
      </div>
    </div>
  </div>
</template>

