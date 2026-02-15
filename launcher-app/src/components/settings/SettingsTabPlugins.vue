<script setup lang="ts">
/**
 * SettingsTabPlugins - Plugin management tab.
 * Lists installed plugins with enable/disable, uninstall, and preference editing.
 */
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { Switch, SwitchGroup, SwitchLabel, Listbox, ListboxButton, ListboxOptions, ListboxOption, ListboxLabel } from '@headlessui/vue';
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
  <div class="flex flex-col gap-6">
    <SettingSection title="Installed Plugins" id="plugins-list" icon="🧩">
      <!-- Loading -->
      <div v-if="loading" class="py-6 text-center text-sm text-text-secondary">
        Loading plugins...
      </div>

      <!-- Error -->
      <div v-else-if="error" class="py-6 text-center text-sm text-text-secondary">
        <p>Failed to load plugins: {{ error }}</p>
        <button class="mt-2 px-3.5 py-1.5 text-xs font-medium text-text-primary bg-bg-secondary border border-border-default rounded-md hover:bg-hover-bg transition-colors" @click="loadPlugins">Retry</button>
      </div>

      <!-- Empty -->
      <div v-else-if="plugins.length === 0" class="py-6 text-center text-sm text-text-secondary">
        <p class="mb-1">No plugins installed.</p>
        <p class="text-xs text-text-tertiary">
          Place plugin folders in the <code class="px-1 py-0.5 text-[11px] bg-bg-secondary rounded">plugins/</code> directory to get started.
        </p>
      </div>

      <!-- Plugin List -->
      <div v-else class="flex flex-col gap-2">
        <div
          v-for="plugin in plugins"
          :key="plugin.id"
          class="bg-bg-subtle border border-border-default rounded-lg p-3 transition-colors"
          :class="{ 'opacity-60': !plugin.enabled }"
        >
          <!-- Plugin header row -->
          <div class="flex items-center justify-between cursor-pointer" @click="toggleExpand(plugin.id)">
            <div class="flex items-center gap-2">
              <span v-if="plugin.icon" class="text-lg flex-shrink-0">{{ plugin.icon }}</span>
              <span v-else class="text-lg flex-shrink-0 text-text-tertiary">🧩</span>
              <div class="flex items-baseline gap-1.5">
                <span class="text-sm font-semibold text-text-primary">{{ plugin.title }}</span>
                <span class="text-[11px] text-text-tertiary">v{{ plugin.version }}</span>
              </div>
            </div>
            <div class="flex items-center gap-2">
              <SwitchGroup as="div" class="flex items-center" @click.stop>
                <Switch
                  :model-value="plugin.enabled"
                  @update:model-value="togglePlugin(plugin)"
                  :class="plugin.enabled ? 'bg-accent' : 'bg-bg-secondary border border-border-default'"
                  class="relative inline-flex h-5 w-9 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none focus-visible:ring-2 focus-visible:ring-accent focus-visible:ring-offset-2"
                >
                  <span class="sr-only">Enable plugin</span>
                  <span
                    aria-hidden="true"
                    :class="plugin.enabled ? 'translate-x-4' : 'translate-x-0'"
                    class="pointer-events-none inline-block h-4 w-4 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out"
                  />
                </Switch>
              </SwitchGroup>
              <svg
                class="text-text-tertiary opacity-50 transition-transform duration-200"
                :class="{ 'rotate-180': expandedPluginId === plugin.id }"
                width="14" height="14" viewBox="0 0 14 14" fill="none"
              >
                <path d="M3.5 5.25L7 8.75L10.5 5.25" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
              </svg>
            </div>
          </div>

          <!-- Plugin description -->
          <p v-if="plugin.description" class="text-xs text-text-secondary mt-1 ml-8 leading-snug">{{ plugin.description }}</p>

          <!-- Expanded detail section -->
          <div v-if="expandedPluginId === plugin.id" class="mt-3 pt-3 border-t border-border-default">
            <!-- Author / Installed at -->
            <div class="flex gap-4 mb-2">
              <span v-if="plugin.author" class="text-[11px] text-text-tertiary">Author: {{ plugin.author }}</span>
              <span class="text-[11px] text-text-tertiary">Installed: {{ new Date(plugin.installed_at).toLocaleDateString() }}</span>
            </div>

            <!-- Preferences form -->
            <div v-if="prefSchemas[plugin.id]?.length" class="my-3">
              <h4 class="text-[11px] font-semibold uppercase tracking-wider text-text-secondary mb-2">Preferences</h4>
              <div v-for="pref in prefSchemas[plugin.id]" :key="pref.name" class="mb-3 last:mb-0">
                <label class="block text-xs font-medium text-text-primary mb-1" :for="`pref-${plugin.id}-${pref.name}`">
                  {{ pref.title }}
                  <span v-if="pref.description" class="block text-[11px] font-normal text-text-tertiary mt-px">{{ pref.description }}</span>
                </label>

                <!-- Text / Password -->
                <input
                  v-if="pref.type === 'text' || pref.type === 'password'"
                  :id="`pref-${plugin.id}-${pref.name}`"
                  :type="pref.type"
                  :placeholder="pref.placeholder || ''"
                  :value="prefValues[plugin.id]?.[pref.name] ?? pref.default ?? ''"
                  class="w-full px-2.5 py-1.5 text-xs bg-bg-secondary border border-border-default rounded-md text-text-primary focus:outline-none focus:border-accent"
                  @change="(e) => updatePreference(plugin.id, pref.name, (e.target as HTMLInputElement).value)"
                />

                <!-- Number -->
                <input
                  v-else-if="pref.type === 'number'"
                  :id="`pref-${plugin.id}-${pref.name}`"
                  type="number"
                  :placeholder="pref.placeholder || ''"
                  :value="prefValues[plugin.id]?.[pref.name] ?? pref.default ?? ''"
                  class="w-28 px-2.5 py-1.5 text-xs bg-bg-secondary border border-border-default rounded-md text-text-primary focus:outline-none focus:border-accent"
                  @change="(e) => updatePreference(plugin.id, pref.name, (e.target as HTMLInputElement).value)"
                />

                <!-- Checkbox -->
                <label v-else-if="pref.type === 'checkbox'" class="flex items-center gap-2 text-xs text-text-primary cursor-pointer">
                  <input
                    :id="`pref-${plugin.id}-${pref.name}`"
                    type="checkbox"
                    :checked="(prefValues[plugin.id]?.[pref.name] ?? pref.default) === 'true'"
                    class="rounded border-border-default text-accent focus:ring-accent"
                    @change="(e) => updatePreference(plugin.id, pref.name, String((e.target as HTMLInputElement).checked))"
                  />
                  <span>{{ pref.title }}</span>
                </label>

                <!-- Select -->
                <div v-else-if="pref.type === 'select'" class="relative">
                  <Listbox 
                    :model-value="prefValues[plugin.id]?.[pref.name] ?? pref.default ?? ''"
                    @update:model-value="(val) => updatePreference(plugin.id, pref.name, val)"
                  >
                    <ListboxButton class="relative w-full cursor-pointer rounded-md bg-bg-secondary py-1.5 pl-2.5 pr-8 text-left text-xs border border-border-default focus:outline-none focus:border-accent">
                      <span class="block truncate text-text-primary">
                        {{ pref.options?.find(o => o.value === (prefValues[plugin.id]?.[pref.name] ?? pref.default))?.title || (prefValues[plugin.id]?.[pref.name] ?? pref.default) }}
                      </span>
                      <span class="pointer-events-none absolute inset-y-0 right-0 flex items-center pr-2">
                        <svg width="10" height="10" viewBox="0 0 12 12" fill="none" class="text-text-secondary">
                          <path d="M3 4.5L6 7.5L9 4.5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
                        </svg>
                      </span>
                    </ListboxButton>
                    <ListboxOptions class="absolute mt-1 max-h-40 w-full overflow-auto rounded-md bg-bg-elevated py-1 text-xs shadow-lg ring-1 ring-black/5 focus:outline-none z-10">
                      <ListboxOption
                        v-for="opt in pref.options"
                        :key="opt.value"
                        :value="opt.value"
                        as="template"
                        v-slot="{ active, selected }"
                      >
                        <li :class="[active ? 'bg-hover-bg text-text-primary' : 'text-text-primary', 'relative cursor-pointer select-none py-1.5 pl-2.5 pr-8']">
                          <span :class="[selected ? 'font-medium' : 'font-normal', 'block truncate']">{{ opt.title }}</span>
                          <span v-if="selected" class="absolute inset-y-0 right-0 flex items-center pr-2 text-accent">
                            <svg width="10" height="10" viewBox="0 0 12 12" fill="none">
                              <path d="M2.5 6L4.5 8L9.5 3.5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
                            </svg>
                          </span>
                        </li>
                      </ListboxOption>
                    </ListboxOptions>
                  </Listbox>
                </div>
              </div>
            </div>

            <!-- Action buttons -->
            <div class="flex gap-2 mt-3">
              <button class="px-3 py-1.5 text-xs font-medium text-text-primary bg-bg-secondary border border-border-default rounded-md hover:bg-hover-bg transition-colors" @click="viewLog(plugin.id)">
                {{ showLogFor === plugin.id ? 'Hide Log' : 'View Log' }}
              </button>
              <button class="px-3 py-1.5 text-xs font-medium text-red-500 bg-red-500/10 border border-red-500/20 rounded-md hover:bg-red-500/20 transition-colors" @click="uninstallPlugin(plugin.id)">
                Uninstall
              </button>
            </div>

            <!-- Log viewer -->
            <pre v-if="showLogFor === plugin.id" class="mt-2 p-2 text-[11px] font-mono bg-black/20 rounded-md text-text-secondary max-h-32 overflow-y-auto whitespace-pre-wrap break-all">{{ pluginLogs[plugin.id] }}</pre>
          </div>
        </div>
      </div>
    </SettingSection>
  </div>
</template>

<style scoped>
/* Scoped styles removed in favor of Tailwind CSS classes */
</style>
