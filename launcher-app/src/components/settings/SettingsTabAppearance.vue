<script setup lang="ts">
/**
 * SettingsTabAppearance - Appearance settings tab including theme, colors, and layout
 */
import type { AppSettings } from "../../types/settings";
import { themeOptions } from "../../composables/useAppSettings";
import SettingSelect from "./SettingSelect.vue";
import SettingNumber from "./SettingNumber.vue";
import SettingColor from "./SettingColor.vue";
import SettingSection from "./SettingSection.vue";

defineProps<{
  settings: AppSettings;
}>();
</script>

<template>
  <div class="settings-tab-content">
    <SettingSection title="Theme" id="theme" icon="🎨">
      <SettingSelect
        v-model="settings.theme.mode"
        label="Theme Mode"
        description="Choose your preferred color scheme"
        :options="themeOptions"
      />
    </SettingSection>

    <SettingSection title="Colors" id="colors" icon="🖌️">
      <div class="color-grid">
        <SettingColor
          v-model="settings.theme.accent_color"
          label="Accent"
          default-color="#ff6b6b"
        />
        <SettingColor
          v-model="settings.theme.bg_color"
          label="Background"
          :optional="true"
          default-color="#1a1a1a"
        />
        <SettingColor
          v-model="settings.theme.text_color"
          label="Text"
          :optional="true"
          default-color="#e0e0e0"
        />
        <SettingColor
          v-model="settings.theme.selection_bg_color"
          label="Selection"
          :optional="true"
          :default-color="settings.theme.accent_color"
        />
      </div>
    </SettingSection>

    <SettingSection title="Layout" id="layout" icon="📐">
      <div class="layout-grid">
        <SettingNumber
          v-model="settings.theme.font_size"
          label="Font Size"
          :min="10"
          :max="24"
          width="small"
        />
        <SettingNumber
          v-model="settings.theme.border_radius"
          label="Border Radius"
          :min="0"
          :max="24"
          width="small"
        />
      </div>
      <div class="layout-grid">
        <SettingNumber
          v-model="settings.theme.window_width"
          label="Window Width"
          :min="400"
          :max="1200"
          :step="10"
          width="medium"
        />
        <SettingNumber
          v-model="settings.theme.window_height"
          label="Window Height"
          :min="200"
          :max="900"
          :step="10"
          width="medium"
        />
      </div>
      <div class="layout-grid">
        <SettingNumber
          v-model="settings.theme.input_height"
          label="Input Height"
          :min="30"
          :max="80"
          width="small"
        />
        <SettingNumber
          v-model="settings.theme.item_height"
          label="Item Height"
          :min="20"
          :max="60"
          width="small"
        />
      </div>
    </SettingSection>
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

.color-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 8px;
}

.layout-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 16px;
  margin-bottom: 8px;
}

@media (prefers-reduced-motion: reduce) {
  .settings-tab-content {
    animation: none;
  }
}
</style>
