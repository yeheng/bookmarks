<script setup lang="ts">
/**
 * SettingsTabPlugins - Plugin management tab.
 * Lists installed plugins with enable/disable, uninstall, and preference editing.
 */
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { PluginInfo, PluginPreferenceSchema } from '../../types/plugin';
import SettingSection from './SettingSection.vue';

const plugins = ref<PluginInfo[]>([]);
const loading = ref(true);
const error = ref<string | null>(null);
const expandedPluginId = ref<string | null>(null);
const prefSchemas = ref<Record<string, PluginPreferenceSchema[]>>({});
const prefValues = ref<Record<string, Record<string, string>>>({});
const pluginLogs = ref<Record<string, string>>({});
const showLogFor = ref<string | null>(null);

async function loadPlugins() {
  loading.value = true;
  error.value = null;
  try {
    plugins.value = await invoke<PluginInfo[]>('list_plugins');
  } catch (err) {
    error.value = err instanceof Error ? err.message : String(err);
  } finally {
    loading.value = false;
  }
}

async function togglePlugin(plugin: PluginInfo) {
  try {
    if (plugin.enabled) {
      await invoke('disable_plugin', { pluginId: plugin.id });
    } else {
      await invoke('enable_plugin', { pluginId: plugin.id });
    }
    plugin.enabled = !plugin.enabled;
  } catch (err) {
    console.error('Failed to toggle plugin:', err);
  }
}

async function uninstallPlugin(pluginId: string) {
  if (!confirm(`Are you sure you want to uninstall this plugin?`)) return;
  try {
    await invoke('uninstall_plugin', { pluginId });
    plugins.value = plugins.value.filter(p => p.id !== pluginId);
  } catch (err) {
    console.error('Failed to uninstall plugin:', err);
  }
}

async function toggleExpand(pluginId: string) {
  if (expandedPluginId.value === pluginId) {
    expandedPluginId.value = null;
    return;
  }
  expandedPluginId.value = pluginId;

  // Load preferences schema + current values
  if (!prefSchemas.value[pluginId]) {
    try {
      const [schema, values] = await Promise.all([
        invoke<PluginPreferenceSchema[]>('get_plugin_manifest_preferences', { pluginId }),
        invoke<Record<string, string>>('get_plugin_preferences', { pluginId }),
      ]);
      prefSchemas.value[pluginId] = schema;
      prefValues.value[pluginId] = values;
    } catch (err) {
      console.error('Failed to load plugin preferences:', err);
    }
  }
}

async function updatePreference(pluginId: string, key: string, value: string) {
  try {
    await invoke('set_plugin_preference', { pluginId, key, value });
    if (prefValues.value[pluginId]) {
      prefValues.value[pluginId][key] = value;
    }
  } catch (err) {
    console.error('Failed to set preference:', err);
  }
}

async function viewLog(pluginId: string) {
  if (showLogFor.value === pluginId) {
    showLogFor.value = null;
    return;
  }
  try {
    const log = await invoke<string>('get_plugin_log', { pluginId });
    pluginLogs.value[pluginId] = log || '(No logs)';
    showLogFor.value = pluginId;
  } catch (err) {
    pluginLogs.value[pluginId] = `Error loading log: ${err}`;
    showLogFor.value = pluginId;
  }
}

onMounted(loadPlugins);
</script>

<template>
  <div class="settings-tab-content">
    <SettingSection title="Installed Plugins" id="plugins-list" icon="🧩">
      <!-- Loading -->
      <div v-if="loading" class="plugins-loading">
        Loading plugins...
      </div>

      <!-- Error -->
      <div v-else-if="error" class="plugins-error">
        <p>Failed to load plugins: {{ error }}</p>
        <button class="btn-retry" @click="loadPlugins">Retry</button>
      </div>

      <!-- Empty -->
      <div v-else-if="plugins.length === 0" class="plugins-empty">
        <p class="plugins-empty__text">No plugins installed.</p>
        <p class="plugins-empty__hint">
          Place plugin folders in the <code>plugins/</code> directory to get started.
        </p>
      </div>

      <!-- Plugin List -->
      <div v-else class="plugins-list">
        <div
          v-for="plugin in plugins"
          :key="plugin.id"
          class="plugin-card"
          :class="{ 'plugin-card--disabled': !plugin.enabled }"
        >
          <!-- Plugin header row -->
          <div class="plugin-header" @click="toggleExpand(plugin.id)">
            <div class="plugin-info">
              <span v-if="plugin.icon" class="plugin-icon">{{ plugin.icon }}</span>
              <span v-else class="plugin-icon plugin-icon--default">🧩</span>
              <div class="plugin-meta">
                <span class="plugin-name">{{ plugin.title }}</span>
                <span class="plugin-version">v{{ plugin.version }}</span>
              </div>
            </div>
            <div class="plugin-actions">
              <label class="toggle-switch" @click.stop>
                <input
                  type="checkbox"
                  :checked="plugin.enabled"
                  @change="togglePlugin(plugin)"
                />
                <span class="toggle-slider"></span>
              </label>
              <svg
                class="expand-chevron"
                :class="{ 'expand-chevron--open': expandedPluginId === plugin.id }"
                width="14" height="14" viewBox="0 0 14 14" fill="none"
              >
                <path d="M3.5 5.25L7 8.75L10.5 5.25" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
              </svg>
            </div>
          </div>

          <!-- Plugin description -->
          <p v-if="plugin.description" class="plugin-description">{{ plugin.description }}</p>

          <!-- Expanded detail section -->
          <div v-if="expandedPluginId === plugin.id" class="plugin-detail">
            <!-- Author / Installed at -->
            <div class="plugin-detail-row">
              <span v-if="plugin.author" class="detail-label">Author: {{ plugin.author }}</span>
              <span class="detail-label">Installed: {{ new Date(plugin.installed_at).toLocaleDateString() }}</span>
            </div>

            <!-- Preferences form -->
            <div v-if="prefSchemas[plugin.id]?.length" class="plugin-preferences">
              <h4 class="pref-title">Preferences</h4>
              <div v-for="pref in prefSchemas[plugin.id]" :key="pref.name" class="pref-field">
                <label class="pref-label" :for="`pref-${plugin.id}-${pref.name}`">
                  {{ pref.title }}
                  <span v-if="pref.description" class="pref-desc">{{ pref.description }}</span>
                </label>

                <!-- Text / Password -->
                <input
                  v-if="pref.type === 'text' || pref.type === 'password'"
                  :id="`pref-${plugin.id}-${pref.name}`"
                  :type="pref.type"
                  :placeholder="pref.placeholder || ''"
                  :value="prefValues[plugin.id]?.[pref.name] ?? pref.default ?? ''"
                  class="pref-input"
                  @change="(e) => updatePreference(plugin.id, pref.name, (e.target as HTMLInputElement).value)"
                />

                <!-- Number -->
                <input
                  v-else-if="pref.type === 'number'"
                  :id="`pref-${plugin.id}-${pref.name}`"
                  type="number"
                  :placeholder="pref.placeholder || ''"
                  :value="prefValues[plugin.id]?.[pref.name] ?? pref.default ?? ''"
                  class="pref-input pref-input--number"
                  @change="(e) => updatePreference(plugin.id, pref.name, (e.target as HTMLInputElement).value)"
                />

                <!-- Checkbox -->
                <label v-else-if="pref.type === 'checkbox'" class="pref-checkbox">
                  <input
                    :id="`pref-${plugin.id}-${pref.name}`"
                    type="checkbox"
                    :checked="(prefValues[plugin.id]?.[pref.name] ?? pref.default) === 'true'"
                    @change="(e) => updatePreference(plugin.id, pref.name, String((e.target as HTMLInputElement).checked))"
                  />
                  <span>{{ pref.title }}</span>
                </label>

                <!-- Select -->
                <select
                  v-else-if="pref.type === 'select'"
                  :id="`pref-${plugin.id}-${pref.name}`"
                  class="pref-select"
                  :value="prefValues[plugin.id]?.[pref.name] ?? pref.default ?? ''"
                  @change="(e) => updatePreference(plugin.id, pref.name, (e.target as HTMLSelectElement).value)"
                >
                  <option v-for="opt in pref.options" :key="opt.value" :value="opt.value">{{ opt.title }}</option>
                </select>
              </div>
            </div>

            <!-- Action buttons -->
            <div class="plugin-actions-row">
              <button class="btn-secondary" @click="viewLog(plugin.id)">
                {{ showLogFor === plugin.id ? 'Hide Log' : 'View Log' }}
              </button>
              <button class="btn-danger" @click="uninstallPlugin(plugin.id)">
                Uninstall
              </button>
            </div>

            <!-- Log viewer -->
            <pre v-if="showLogFor === plugin.id" class="plugin-log">{{ pluginLogs[plugin.id] }}</pre>
          </div>
        </div>
      </div>
    </SettingSection>
  </div>
</template>

<style scoped>
.settings-tab-content {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.plugins-loading,
.plugins-error,
.plugins-empty {
  padding: 24px 0;
  text-align: center;
  color: var(--color-text-secondary);
  font-size: 13px;
}

.plugins-empty__hint {
  margin-top: 4px;
  font-size: 12px;
  color: var(--color-text-tertiary);
}

.plugins-empty__hint code {
  background: rgba(128, 128, 128, 0.15);
  padding: 1px 4px;
  border-radius: 3px;
  font-size: 11px;
}

.btn-retry {
  margin-top: 8px;
  padding: 5px 14px;
  font-size: 12px;
  font-weight: 500;
  color: var(--text-color);
  background: rgba(128, 128, 128, 0.1);
  border: 0.5px solid rgba(128, 128, 128, 0.2);
  border-radius: 6px;
  cursor: pointer;
}

/* ── Plugin Card ── */
.plugins-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.plugin-card {
  background: rgba(128, 128, 128, 0.06);
  border: 0.5px solid rgba(128, 128, 128, 0.15);
  border-radius: 8px;
  padding: 10px 12px;
  transition: background 0.15s ease;
}

.plugin-card--disabled {
  opacity: 0.55;
}

.plugin-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  cursor: pointer;
}

.plugin-info {
  display: flex;
  align-items: center;
  gap: 8px;
}

.plugin-icon {
  font-size: 18px;
  flex-shrink: 0;
}

.plugin-meta {
  display: flex;
  align-items: baseline;
  gap: 6px;
}

.plugin-name {
  font-size: 13px;
  font-weight: 600;
  color: var(--color-text-primary, var(--text-color));
}

.plugin-version {
  font-size: 11px;
  color: var(--color-text-tertiary);
}

.plugin-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.plugin-description {
  font-size: 12px;
  color: var(--color-text-secondary);
  margin: 4px 0 0 30px;
  line-height: 1.4;
}

/* ── Toggle Switch ── */
.toggle-switch {
  position: relative;
  display: inline-block;
  width: 36px;
  height: 20px;
  flex-shrink: 0;
}

.toggle-switch input {
  opacity: 0;
  width: 0;
  height: 0;
}

.toggle-slider {
  position: absolute;
  cursor: pointer;
  inset: 0;
  background-color: rgba(128, 128, 128, 0.3);
  border-radius: 20px;
  transition: background-color 0.2s ease;
}

.toggle-slider::before {
  content: '';
  position: absolute;
  height: 16px;
  width: 16px;
  left: 2px;
  bottom: 2px;
  background-color: white;
  border-radius: 50%;
  transition: transform 0.2s ease;
}

.toggle-switch input:checked + .toggle-slider {
  background-color: var(--accent-color, #007AFF);
}

.toggle-switch input:checked + .toggle-slider::before {
  transform: translateX(16px);
}

/* ── Expand chevron ── */
.expand-chevron {
  color: var(--color-text-tertiary);
  opacity: 0.5;
  transition: transform 0.2s ease;
}

.expand-chevron--open {
  transform: rotate(180deg);
}

/* ── Plugin Detail ── */
.plugin-detail {
  margin-top: 10px;
  padding-top: 10px;
  border-top: 0.5px solid rgba(128, 128, 128, 0.15);
}

.plugin-detail-row {
  display: flex;
  gap: 16px;
  margin-bottom: 8px;
}

.detail-label {
  font-size: 11px;
  color: var(--color-text-tertiary);
}

/* ── Preferences ── */
.plugin-preferences {
  margin: 10px 0;
}

.pref-title {
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: var(--color-text-secondary);
  margin-bottom: 8px;
}

.pref-field {
  margin-bottom: 10px;
}

.pref-label {
  display: block;
  font-size: 13px;
  font-weight: 500;
  color: var(--color-text-primary, var(--text-color));
  margin-bottom: 4px;
}

.pref-desc {
  display: block;
  font-size: 11px;
  font-weight: 400;
  color: var(--color-text-tertiary);
  margin-top: 1px;
}

.pref-input,
.pref-select {
  width: 100%;
  padding: 6px 10px;
  font-size: 13px;
  background: var(--color-bg-secondary, rgba(128, 128, 128, 0.1));
  border: 0.5px solid rgba(128, 128, 128, 0.2);
  border-radius: 6px;
  color: var(--color-text-primary, var(--text-color));
  outline: none;
}

.pref-input:focus,
.pref-select:focus {
  border-color: var(--accent-color, #007AFF);
}

.pref-input--number {
  width: 120px;
}

.pref-checkbox {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  color: var(--color-text-primary, var(--text-color));
  cursor: pointer;
}

/* ── Action buttons ── */
.plugin-actions-row {
  display: flex;
  gap: 8px;
  margin-top: 10px;
}

.btn-secondary {
  padding: 5px 12px;
  font-size: 12px;
  font-weight: 500;
  color: var(--color-text-primary, var(--text-color));
  background: rgba(128, 128, 128, 0.1);
  border: 0.5px solid rgba(128, 128, 128, 0.2);
  border-radius: 6px;
  cursor: pointer;
  transition: background 0.15s ease;
}

.btn-secondary:hover {
  background: rgba(128, 128, 128, 0.2);
}

.btn-danger {
  padding: 5px 12px;
  font-size: 12px;
  font-weight: 500;
  color: #ef4444;
  background: rgba(239, 68, 68, 0.08);
  border: 0.5px solid rgba(239, 68, 68, 0.2);
  border-radius: 6px;
  cursor: pointer;
  transition: background 0.15s ease;
}

.btn-danger:hover {
  background: rgba(239, 68, 68, 0.15);
}

/* ── Log viewer ── */
.plugin-log {
  margin-top: 8px;
  padding: 8px;
  font-size: 11px;
  font-family: 'SF Mono', 'Fira Code', monospace;
  background: rgba(0, 0, 0, 0.2);
  border-radius: 6px;
  color: var(--color-text-secondary);
  max-height: 120px;
  overflow-y: auto;
  white-space: pre-wrap;
  word-break: break-all;
}

@media (prefers-reduced-motion: reduce) {
  .toggle-slider,
  .toggle-slider::before,
  .expand-chevron,
  .plugin-card,
  .btn-secondary,
  .btn-danger {
    transition: none;
  }
}
</style>
