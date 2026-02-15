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
    class="welcome-screen"
    role="dialog"
    aria-modal="true"
    aria-labelledby="welcome-title"
    aria-describedby="welcome-subtitle"
  >
    <div class="welcome-overlay" @click="skipTutorial" aria-hidden="true"></div>

    <div class="welcome-content">
      <!-- Header -->
      <div class="welcome-header">
        <h1 id="welcome-title" class="welcome-title">Welcome to Bookmark Launcher! 🚀</h1>
        <p id="welcome-subtitle" class="welcome-subtitle">Let's get you started with a quick tour</p>
        <p class="keyboard-hint" aria-live="polite">
          Use ← → arrow keys to navigate, Enter to continue, Escape to skip
        </p>
      </div>

      <!-- Feature Showcase -->
      <div class="feature-showcase" role="region" aria-label="Feature tour">
        <TransitionGroup name="slide-fade">
          <div
            v-for="(feature, index) in features"
            v-show="index === currentStep"
            :key="index"
            class="feature-card"
            role="tabpanel"
            :id="`feature-panel-${index}`"
            :aria-labelledby="`feature-title-${index}`"
          >
            <div class="feature-icon" aria-hidden="true">{{ feature.icon }}</div>
            <h2 :id="`feature-title-${index}`" class="feature-title">{{ feature.title }}</h2>
            <p class="feature-description">{{ feature.description }}</p>
            <div class="feature-shortcut">
              <kbd aria-label="Keyboard shortcut">{{ feature.shortcut }}</kbd>
            </div>
          </div>
        </TransitionGroup>
      </div>

      <!-- Progress Indicators -->
      <div class="progress-dots" role="tablist" aria-label="Tutorial steps">
        <button
          v-for="(feature, index) in features"
          :key="index"
          class="progress-dot"
          :class="{ active: index === currentStep }"
          @click="currentStep = index"
          role="tab"
          :aria-selected="index === currentStep"
          :aria-label="`Step ${index + 1} of ${features.length}: ${feature.title}`"
          :aria-controls="`feature-panel-${index}`"
        ></button>
      </div>

      <!-- Screen reader status -->
      <div class="visually-hidden" aria-live="polite" aria-atomic="true">
        Step {{ currentStep + 1 }} of {{ features.length }}: {{ features[currentStep].title }}
      </div>

      <!-- Actions -->
      <div class="welcome-actions">
        <button
          v-if="currentStep > 0"
          @click="previousStep"
          class="btn-secondary"
          aria-label="Go to previous step"
        >
          ← Previous
        </button>
        <div v-else class="btn-spacer"></div>

        <button
          @click="skipTutorial"
          class="btn-link"
          aria-label="Skip the tutorial"
        >
          Skip Tutorial
        </button>

        <button
          v-if="currentStep < features.length - 1"
          @click="nextStep"
          class="btn-primary"
          aria-label="Go to next step"
        >
          Next →
        </button>
        <button
          v-else
          @click="completeOnboarding"
          class="btn-primary btn-complete"
          aria-label="Complete tutorial and start using the app"
        >
          Get Started ✓
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.welcome-screen {
  position: fixed;
  inset: 0;
  z-index: 9999;
  display: flex;
  align-items: center;
  justify-content: center;
  animation: fadeIn 0.3s ease-out;
}

.welcome-overlay {
  position: absolute;
  inset: 0;
  background: rgba(0, 0, 0, 0.85);
  backdrop-filter: blur(8px);
}

.welcome-content {
  position: relative;
  z-index: 10000;
  width: 90%;
  max-width: 600px;
  background: var(--bg-color);
  border-radius: 16px;
  padding: 48px;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.5);
  animation: slideInUp 0.4s cubic-bezier(0.68, -0.55, 0.265, 1.55);
}

/* Header */
.welcome-header {
  text-align: center;
  margin-bottom: 40px;
}

.welcome-title {
  font-size: 32px;
  font-weight: 700;
  color: var(--text-color);
  margin-bottom: 8px;
}

.welcome-subtitle {
  font-size: 16px;
  color: var(--secondary-text);
}

.keyboard-hint {
  font-size: 12px;
  color: var(--color-text-tertiary, #a3a3a3);
  margin-top: 12px;
  opacity: 0.8;
}

/* Visually hidden for screen readers */
.visually-hidden {
  position: absolute;
  width: 1px;
  height: 1px;
  padding: 0;
  margin: -1px;
  overflow: hidden;
  clip: rect(0, 0, 0, 0);
  white-space: nowrap;
  border-width: 0;
}

/* Feature Showcase */
.feature-showcase {
  position: relative;
  min-height: 280px;
  margin-bottom: 32px;
}

.feature-card {
  text-align: center;
  position: absolute;
  width: 100%;
  top: 0;
  left: 0;
}

.feature-icon {
  font-size: 72px;
  margin-bottom: 24px;
  line-height: 1;
}

.feature-title {
  font-size: 24px;
  font-weight: 600;
  color: var(--text-color);
  margin-bottom: 12px;
}

.feature-description {
  font-size: 16px;
  color: var(--secondary-text);
  line-height: 1.6;
  margin-bottom: 20px;
  max-width: 400px;
  margin-left: auto;
  margin-right: auto;
}

.feature-shortcut {
  margin-top: 16px;
}

.feature-shortcut kbd {
  display: inline-block;
  padding: 8px 16px;
  background: rgba(255, 255, 255, 0.1);
  border: 1px solid rgba(255, 255, 255, 0.2);
  border-radius: 8px;
  font-family: monospace;
  font-size: 14px;
  color: var(--accent-color);
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}

/* Progress Dots */
.progress-dots {
  display: flex;
  justify-content: center;
  gap: 8px;
  margin-bottom: 32px;
}

.progress-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: rgba(255, 255, 255, 0.3);
  border: none;
  cursor: pointer;
  transition: all 0.3s ease;
  padding: 0;
}

.progress-dot:hover {
  background: rgba(255, 255, 255, 0.5);
  transform: scale(1.2);
}

.progress-dot.active {
  background: var(--accent-color);
  width: 24px;
  border-radius: 4px;
}

/* Actions */
.welcome-actions {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 16px;
}

.btn-spacer {
  width: 100px;
}

button {
  padding: 12px 24px;
  border-radius: 8px;
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.15s ease;
  border: none;
}

.btn-primary {
  background: var(--accent-color);
  color: white;
  min-width: 120px;
}

.btn-primary:hover {
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(255, 107, 107, 0.4);
}

.btn-primary.btn-complete {
  background: linear-gradient(135deg, #10b981 0%, #059669 100%);
}

.btn-secondary {
  background: rgba(255, 255, 255, 0.1);
  color: var(--text-color);
  border: 1px solid rgba(255, 255, 255, 0.2);
  min-width: 100px;
}

.btn-secondary:hover {
  background: rgba(255, 255, 255, 0.15);
}

.btn-link {
  background: transparent;
  color: var(--secondary-text);
  text-decoration: underline;
}

.btn-link:hover {
  color: var(--text-color);
}

/* Animations */
@keyframes fadeIn {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}

@keyframes slideInUp {
  from {
    opacity: 0;
    transform: translateY(40px) scale(0.95);
  }
  to {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
}

/* Slide transitions for feature cards */
.slide-fade-enter-active,
.slide-fade-leave-active {
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.slide-fade-enter-from {
  opacity: 0;
  transform: translateX(30px);
}

.slide-fade-leave-to {
  opacity: 0;
  transform: translateX(-30px);
}

/* Accessibility */
@media (prefers-reduced-motion: reduce) {
  *,
  *::before,
  *::after {
    animation-duration: 0.01ms !important;
    transition-duration: 0.01ms !important;
  }
}

/* Responsive */
@media (max-width: 640px) {
  .welcome-content {
    padding: 32px 24px;
  }

  .welcome-title {
    font-size: 24px;
  }

  .feature-icon {
    font-size: 56px;
  }

  .feature-title {
    font-size: 20px;
  }
}
</style>
