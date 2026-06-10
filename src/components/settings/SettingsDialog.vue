<script setup lang="ts">

import { computed, ref } from "vue";

import { useI18n } from "vue-i18n";

import { ElMessage } from "element-plus";

import { useThemeStore, type ThemeMode } from "../../stores/theme";
import { useWindowOpacityStore } from "../../stores/windowOpacity";

import { useLocaleStore } from "../../stores/locale";
import type { AppLocale } from "../../i18n";

import { useShortcutStore } from "../../stores/shortcut";

import {

  CUSTOMIZABLE_SHORTCUT_DEFS,

  STATIC_SHORTCUT_DEFS,

  isSameShortcut,

  staticShortcutI18nKey,

  type CustomizableShortcutId,

  type ShortcutBinding,

} from "../../utils/shortcuts";

import ShortcutKeyInput from "./ShortcutKeyInput.vue";

import SettingsDataSection from "./SettingsDataSection.vue";
import SettingsNotificationSection from "./SettingsNotificationSection.vue";
import SettingsEmailGatewaySection from "./SettingsEmailGatewaySection.vue";
import SettingsAiSection from "./SettingsAiSection.vue";



type SettingsTab = "appearance" | "language" | "shortcuts" | "notifications" | "ai" | "email" | "data";



const { t } = useI18n();

const visible = defineModel<boolean>({ default: false });



const themeStore = useThemeStore();
const windowOpacityStore = useWindowOpacityStore();

const localeStore = useLocaleStore();

const shortcutStore = useShortcutStore();



const activeTab = ref<SettingsTab>("appearance");

const shortcutDrafts = ref<Record<CustomizableShortcutId, ShortcutBinding>>({

  quickCapture: shortcutStore.getShortcut("quickCapture"),

  toggleMain: shortcutStore.getShortcut("toggleMain"),

});

const savingShortcutId = ref<CustomizableShortcutId | null>(null);



const themeMode = computed({

  get: () => themeStore.mode,

  set: (value: ThemeMode) => {

    void themeStore.setMode(value);

  },

});

const windowOpacityPercent = computed(() => windowOpacityStore.opacityPercent);

function onOpacityInput(value: number) {

  void windowOpacityStore.setOpacityLive(value / 100);

}

function onOpacityChange(value: number) {

  void windowOpacityStore.setOpacity(value / 100);

}



const appLocale = computed({

  get: () => localeStore.locale,

  set: (value: AppLocale) => {

    void localeStore.setLocale(value);

  },

});



const dataSectionRef = ref<InstanceType<typeof SettingsDataSection> | null>(null);



const navItems = computed(() => [

  { id: "appearance" as const, label: t("settings.appearance") },

  { id: "language" as const, label: t("settings.language") },

  { id: "shortcuts" as const, label: t("settings.shortcuts") },

  { id: "notifications" as const, label: t("settings.notifications") },

  { id: "ai" as const, label: t("settings.ai") },

  { id: "email" as const, label: t("settings.email") },

  { id: "data" as const, label: t("settings.data") },

]);



const themeOptions = computed(() => [

  { value: "light" as const, label: t("settings.themeLight"), hint: t("settings.themeLightHint") },

  { value: "dark" as const, label: t("settings.themeDark"), hint: t("settings.themeDarkHint") },

  { value: "system" as const, label: t("settings.themeSystem"), hint: t("settings.themeSystemHint") },

]);



const localeOptions = computed(() => [

  { value: "zh-CN" as const, label: t("locale.zhCN") },

  { value: "en" as const, label: t("locale.en") },

]);



const customizableShortcuts = computed(() =>

  CUSTOMIZABLE_SHORTCUT_DEFS.map((item) => ({

    ...item,

    label: t(`shortcuts.${item.id}.label`),

    description: t(`shortcuts.${item.id}.description`),

  })),

);



const staticShortcuts = computed(() =>

  STATIC_SHORTCUT_DEFS.map((item) => {

    const key = staticShortcutI18nKey(item.id);

    return {

      ...item,

      label: t(`shortcuts.${key}.label`),

      description: t(`shortcuts.${key}.description`),

    };

  }),

);



function onDialogOpen() {

  activeTab.value = "appearance";

  syncShortcutDrafts();

  void dataSectionRef.value?.reload();

}



function syncShortcutDrafts() {

  for (const item of CUSTOMIZABLE_SHORTCUT_DEFS) {

    shortcutDrafts.value[item.id] = shortcutStore.getShortcut(item.id);

  }

}



function isShortcutDefault(id: CustomizableShortcutId) {

  const item = CUSTOMIZABLE_SHORTCUT_DEFS.find((entry) => entry.id === id);

  if (!item) return true;

  return isSameShortcut(shortcutDrafts.value[id], item.default);

}



async function saveShortcut(id: CustomizableShortcutId) {

  const draft = shortcutDrafts.value[id];

  if (isSameShortcut(draft, shortcutStore.getShortcut(id))) return;

  savingShortcutId.value = id;

  try {

    await shortcutStore.setShortcut(id, draft);

    ElMessage.success(t("settings.shortcutUpdated"));

  } catch (error) {

    console.error("save shortcut failed", error);

    syncShortcutDrafts();

    ElMessage.error(t("settings.shortcutSaveFailed"));

  } finally {

    savingShortcutId.value = null;

  }

}



async function resetShortcut(id: CustomizableShortcutId) {

  savingShortcutId.value = id;

  try {

    await shortcutStore.resetShortcut(id);

    syncShortcutDrafts();

    ElMessage.success(t("settings.shortcutResetOk"));

  } catch (error) {

    console.error("reset shortcut failed", error);

    ElMessage.error(t("settings.shortcutResetFailed"));

  } finally {

    savingShortcutId.value = null;

  }

}

</script>



<template>

  <el-dialog
    v-model="visible"
    :title="t('settings.title')"
    width="790px"
    destroy-on-close
    append-to-body
    align-center
    :z-index="4000"
    class="settings-dialog app-dialog"
    @open="onDialogOpen"
  >

    <div class="settings-layout">

      <nav class="settings-nav">

        <button

          v-for="item in navItems"

          :key="item.id"

          type="button"

          class="settings-nav-item"

          :class="{ active: activeTab === item.id }"

          @click="activeTab = item.id"

        >

          {{ item.label }}

        </button>

      </nav>



      <div class="settings-panel">

        <section v-show="activeTab === 'appearance'" class="settings-section">

          <h3 class="settings-section-title">{{ t("settings.themeTitle") }}</h3>

          <p class="settings-section-desc">{{ t("settings.themeDesc") }}</p>

          <el-radio-group v-model="themeMode" class="theme-options">

            <label

              v-for="option in themeOptions"

              :key="option.value"

              class="theme-option"

              :class="{ active: themeMode === option.value }"

            >

              <el-radio :value="option.value" class="theme-option-radio">

                <span class="theme-option-label">{{ option.label }}</span>

                <span class="theme-option-hint">{{ option.hint }}</span>

              </el-radio>

            </label>

          </el-radio-group>

          <h3 class="settings-section-title settings-subsection-title">{{ t("settings.opacityTitle") }}</h3>

          <p class="settings-section-desc">{{ t("settings.opacityDesc") }}</p>

          <div class="opacity-control">

            <el-slider

              :model-value="windowOpacityPercent"

              :min="50"

              :max="100"

              :step="5"

              :format-tooltip="(value: number) => t('settings.opacityValue', { value })"

              @input="onOpacityInput"

              @change="onOpacityChange"

            />

            <span class="opacity-value">{{ t("settings.opacityValue", { value: windowOpacityPercent }) }}</span>

          </div>

        </section>



        <section v-show="activeTab === 'language'" class="settings-section">

          <h3 class="settings-section-title">{{ t("locale.label") }}</h3>

          <p class="settings-section-desc">{{ t("locale.desc") }}</p>

          <el-radio-group v-model="appLocale" class="locale-options">

            <label

              v-for="option in localeOptions"

              :key="option.value"

              class="locale-option"

              :class="{ active: appLocale === option.value }"

            >

              <el-radio :value="option.value">{{ option.label }}</el-radio>

            </label>

          </el-radio-group>

        </section>



        <section v-show="activeTab === 'shortcuts'" class="settings-section">

          <h3 class="settings-section-title">{{ t("settings.shortcutsTitle") }}</h3>

          <p class="settings-section-desc">{{ t("settings.shortcutsDesc") }}</p>



          <div

            v-for="item in customizableShortcuts"

            :key="item.id"

            class="shortcut-row customizable"

          >

            <div class="shortcut-info">

              <span class="shortcut-label">{{ item.label }}</span>

              <span class="shortcut-desc">{{ item.description }}</span>

            </div>

            <div class="shortcut-actions">

              <ShortcutKeyInput

                v-model="shortcutDrafts[item.id]"

                @update:model-value="saveShortcut(item.id)"

              />

              <el-button

                size="small"

                text

                :disabled="savingShortcutId !== null || isShortcutDefault(item.id)"

                @click="resetShortcut(item.id)"

              >

                {{ t("settings.resetDefault") }}

              </el-button>

            </div>

          </div>



          <div v-for="item in staticShortcuts" :key="item.id" class="shortcut-row">

            <div class="shortcut-info">

              <span class="shortcut-label">{{ item.label }}</span>

              <span class="shortcut-desc">{{ item.description }}</span>

            </div>

            <kbd class="shortcut-kbd">{{ item.keys }}</kbd>

          </div>

        </section>



        <SettingsNotificationSection v-show="activeTab === 'notifications'" />

        <SettingsAiSection v-show="activeTab === 'ai'" />

        <SettingsEmailGatewaySection v-show="activeTab === 'email'" />

        <SettingsDataSection v-show="activeTab === 'data'" ref="dataSectionRef" />

      </div>

    </div>

  </el-dialog>

</template>



<style scoped>

.settings-layout {

  display: flex;

  height: 100%;

  min-height: 0;

  margin: -8px -4px -12px;

}



.settings-nav {

  display: flex;

  flex-direction: column;

  gap: 4px;

  width: 132px;

  flex-shrink: 0;

  padding: 12px 10px;

  border-right: 1px solid var(--border-color);

  background: var(--surface-subtle);

}



.settings-nav-item {

  display: block;

  width: 100%;

  padding: 10px 12px;

  border: none;

  border-radius: var(--radius-sm);

  background: transparent;

  color: var(--text-primary);

  font-size: 14px;

  text-align: left;

  cursor: pointer;

  transition: background 0.15s, color 0.15s;

}



.settings-nav-item:hover {

  background: var(--nav-hover);

}



.settings-nav-item.active {

  background: var(--nav-active);

  color: var(--primary);

  font-weight: 500;

}



.settings-panel {

  flex: 1;

  min-width: 0;

  padding: 16px 20px 20px;

  overflow-y: auto;

}



.settings-section {

  padding: 0;

}



.settings-section-title {

  margin: 0 0 6px;

  font-size: 15px;

  font-weight: 600;

  color: var(--text-primary);

}



.settings-section-desc {

  margin: 0 0 16px;

  font-size: 13px;

  color: var(--text-secondary);

}



.theme-options {

  display: flex;

  flex-direction: column;

  gap: 10px;

  width: 100%;

  align-items: stretch;

}



.theme-option {

  display: block;

  border: 1px solid var(--border-color);

  border-radius: var(--radius);

  background: var(--surface-muted);

  cursor: pointer;

  transition: border-color 0.15s, background 0.15s, box-shadow 0.15s;

}



.theme-option:hover {

  border-color: var(--primary);

}



.theme-option.active {

  border-color: var(--primary);

  background: var(--primary-light);

  box-shadow: 0 0 0 1px color-mix(in srgb, var(--primary) 18%, transparent);

}



.theme-option-radio {

  display: flex;

  width: 100%;

  height: auto;

  margin: 0;

  padding: 12px 14px;

  white-space: normal;

}



.theme-option-radio :deep(.el-radio__label) {

  display: flex;

  flex-direction: column;

  gap: 4px;

  padding-left: 8px;

  white-space: normal;

}



.theme-option-label {

  font-size: 14px;

  font-weight: 500;

  color: var(--text-primary);

}



.theme-option-hint {

  font-size: 12px;

  color: var(--text-secondary);

  line-height: 1.4;

}

.settings-subsection-title {

  margin-top: 24px;

}

.opacity-control {

  display: flex;

  align-items: center;

  gap: 16px;

}

.opacity-control :deep(.el-slider) {

  flex: 1;

}

.opacity-value {

  min-width: 44px;

  font-size: 14px;

  font-weight: 500;

  color: var(--text-primary);

  text-align: right;

}



.locale-options {

  display: flex;

  gap: 10px;

  flex-wrap: wrap;

}



.locale-option {

  display: inline-flex;

  align-items: center;

  padding: 10px 14px;

  border: 1px solid var(--border-color);

  border-radius: var(--radius);

  background: var(--surface-muted);

  cursor: pointer;

  transition: border-color 0.15s, background 0.15s;

}



.locale-option:hover {

  border-color: var(--primary);

}



.locale-option.active {

  border-color: var(--primary);

  background: var(--primary-light);

}



.shortcut-row {

  display: flex;

  align-items: center;

  justify-content: space-between;

  gap: 12px;

  padding: 10px 0;

}



.shortcut-row + .shortcut-row {

  border-top: 1px solid color-mix(in srgb, var(--border-color) 60%, transparent);

}



.shortcut-row.customizable {

  padding-bottom: 14px;

}



.shortcut-info {

  display: flex;

  flex-direction: column;

  gap: 2px;

  min-width: 0;

  flex: 1;

}



.shortcut-label {

  font-size: 14px;

  font-weight: 500;

  color: var(--text-primary);

}



.shortcut-desc {

  font-size: 12px;

  color: var(--text-secondary);

}



.shortcut-actions {

  display: flex;

  align-items: center;

  gap: 4px;

  flex-shrink: 0;

}



.shortcut-kbd {

  display: inline-block;

  padding: 4px 8px;

  font-size: 12px;

  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;

  color: var(--text-primary);

  background: var(--surface-muted);

  border: 1px solid var(--border-color);

  border-radius: 4px;

  white-space: nowrap;

  flex-shrink: 0;

}

</style>



<style>

.settings-dialog.el-dialog,
.el-dialog.settings-dialog {
  height: 720px;
  max-height: 720px;
  display: flex;
  flex-direction: column;
}

.settings-dialog .el-dialog__header {
  flex-shrink: 0;
}

.settings-dialog .el-dialog__body {

  flex: 1;

  min-height: 0;

  overflow: hidden;

  padding-top: 8px;

  padding-bottom: 12px;

}

</style>

