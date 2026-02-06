<script setup lang="ts">
/**
 * SettingsTabData - Data management tab including import, export, stats, and danger zone
 */
import type { DataStats } from "../../composables/useAppSettings";
import SettingSection from "./SettingSection.vue";

defineProps<{
  stats: DataStats | null;
  importStatus: string;
}>();

const emit = defineEmits<{
  (e: 'import-bookmarks', browser: string): void;
  (e: 'export-settings'): void;
  (e: 'import-settings'): void;
  (e: 'reset-settings'): void;
}>();
</script>

<template>
  <div class="settings-tab-content">
    <SettingSection title="Import Bookmarks" id="import" icon="📥">
      <p class="section-description">Import bookmarks from your browsers</p>
      <div class="import-buttons" role="group" aria-label="Import from browser">
        <button class="btn btn--secondary" @click="emit('import-bookmarks', 'chrome')" aria-label="Import from Chrome">
          <span aria-hidden="true">🌐</span> Chrome
        </button>
        <button class="btn btn--secondary" @click="emit('import-bookmarks', 'firefox')" aria-label="Import from Firefox">
          <span aria-hidden="true">🦊</span> Firefox
        </button>
        <button class="btn btn--secondary" @click="emit('import-bookmarks', 'safari')" aria-label="Import from Safari">
          <span aria-hidden="true">🧭</span> Safari
        </button>
      </div>
    </SettingSection>

    <SettingSection title="Settings Backup" id="backup" icon="💾">
      <p class="section-description">Export or import your settings configuration</p>
      <div class="backup-buttons">
        <button class="btn btn--secondary" @click="emit('export-settings')">
          <span aria-hidden="true">📤</span> Export Settings
        </button>
        <button class="btn btn--secondary" @click="emit('import-settings')">
          <span aria-hidden="true">📥</span> Import Settings
        </button>
      </div>
    </SettingSection>

    <SettingSection v-if="stats" title="Statistics" id="stats" icon="📊">
      <div class="stats-grid" role="list">
        <div class="stat" role="listitem">
          <span class="stat-value">{{ stats.bookmarks_count }}</span>
          <span class="stat-label">Bookmarks</span>
        </div>
        <div class="stat" role="listitem">
          <span class="stat-value">{{ stats.files_count }}</span>
          <span class="stat-label">Files</span>
        </div>
        <div class="stat" role="listitem">
          <span class="stat-value">{{ stats.directories_count }}</span>
          <span class="stat-label">Directories</span>
        </div>
      </div>
    </SettingSection>

    <SettingSection title="Danger Zone" id="danger" icon="⚠️">
      <p class="section-description danger-text">These actions cannot be undone</p>
      <button class="btn btn--danger" @click="emit('reset-settings')">
        Reset All Settings
      </button>
    </SettingSection>

    <!-- Status Message -->
    <p v-if="importStatus" class="status-message" role="status" aria-live="polite">
      {{ importStatus }}
    </p>
  </div>
</template>

<style scoped>
.settings-tab-content {
  animation: fadeIn 0.15s ease;
}

@keyframes fadeIn {
  from { opacity: 0; }
  to { opacity: 1; }
}

.section-description {
  font-size: 13px;
  color: var(--color-text-secondary, #b3b3b3);
  margin-bottom: 12px;
}

.danger-text {
  color: var(--color-accent, #ff6b6b);
}

.import-buttons,
.backup-buttons {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 10px 16px;
  font-size: 13px;
  font-weight: 500;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.15s ease;
}

.btn--secondary {
  background: var(--color-bg-secondary, #3a3a3a);
  color: var(--color-text-primary, #e0e0e0);
}

.btn--secondary:hover {
  background: #4a4a4a;
}

.btn--danger {
  background: transparent;
  color: var(--color-accent, #ff6b6b);
  border: 1px solid var(--color-accent, #ff6b6b);
}

.btn--danger:hover {
  background: rgba(255, 107, 107, 0.1);
}

.btn:focus-visible {
  outline: 2px solid var(--color-accent, #ff6b6b);
  outline-offset: 2px;
}

.stats-grid {
  display: flex;
  gap: 12px;
  flex-wrap: wrap;
}

.stat {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 16px 20px;
  background: var(--color-bg-secondary, #262626);
  border-radius: 8px;
  min-width: 80px;
}

.stat-value {
  font-size: 24px;
  font-weight: 600;
  color: var(--color-accent, #ff6b6b);
}

.stat-label {
  font-size: 11px;
  color: var(--color-text-secondary, #b3b3b3);
  margin-top: 4px;
}

.status-message {
  margin-top: 16px;
  padding: 10px 14px;
  font-size: 13px;
  background: rgba(16, 185, 129, 0.1);
  border: 1px solid rgba(16, 185, 129, 0.3);
  border-radius: 6px;
  color: #10b981;
}

@media (prefers-color-scheme: light) {
  .btn--secondary {
    background: var(--color-bg-secondary, #e0e0e0);
    color: var(--color-text-primary, #1a1a1a);
  }
  .stat {
    background: var(--color-bg-secondary, #f5f5f5);
  }
}

@media (prefers-reduced-motion: reduce) {
  .settings-tab-content {
    animation: none;
  }
  .btn {
    transition: none;
  }
}
</style>
