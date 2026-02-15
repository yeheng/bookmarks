<script setup lang="ts">
/**
 * SettingsTabGeneral - General settings tab including startup, updates, and search settings
 */
import type { AppSettings } from "../../types/settings";
import SettingToggle from "./SettingToggle.vue";
import SettingNumber from "./SettingNumber.vue";
import SettingSection from "./SettingSection.vue";

defineProps<{
  settings: AppSettings;
}>();
</script>

<template>
  <div class="animate-in fade-in duration-150">
    <SettingSection title="Startup" id="startup" icon="🚀">
      <SettingToggle
        v-model="settings.general.launch_at_startup"
        label="Launch at Startup"
        description="Automatically start the app when you log in"
      />
      <SettingToggle
        v-model="settings.general.hide_dock_icon"
        label="Hide Dock Icon"
        description="Hide the app icon from the dock (macOS only)"
      />
    </SettingSection>

    <SettingSection title="Updates" id="updates" icon="🔄">
      <SettingToggle
        v-model="settings.general.check_updates"
        label="Check for Updates"
        description="Automatically check for new versions"
      />
    </SettingSection>

    <SettingSection title="Search" id="search-settings" icon="🔍">
      <SettingNumber
        v-model="settings.search.max_results"
        label="Max Results"
        description="Maximum number of search results to display"
        :min="5"
        :max="50"
        width="small"
      />
      <SettingToggle
        v-model="settings.search.show_bookmarks"
        label="Show Bookmarks"
        description="Include bookmarks in search results"
      />
      <SettingToggle
        v-model="settings.search.show_files"
        label="Show Files"
        description="Include files in search results"
      />
      <SettingToggle
        v-model="settings.search.fuzzy_matching"
        label="Fuzzy Matching"
        description="Enable fuzzy search for more flexible matching"
      />
    </SettingSection>
  </div>
</template>

