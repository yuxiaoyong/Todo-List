<script setup lang="ts">
import { computed, ref } from "vue";
import { useI18n } from "vue-i18n";
import {
  captureShortcutFromEvent,
  formatShortcut,
  type ShortcutBinding,
} from "../../utils/shortcuts";

const { t } = useI18n();
const model = defineModel<ShortcutBinding>({ required: true });

const recording = ref(false);
const draft = ref<ShortcutBinding | null>(null);

const displayText = computed(() => {
  if (recording.value) {
    return draft.value ? formatShortcut(draft.value) : t("settings.pressShortcut");
  }
  return formatShortcut(model.value);
});

function startRecording() {
  recording.value = true;
  draft.value = null;
  window.addEventListener("keydown", onKeyDown, true);
}

function stopRecording(apply: boolean) {
  window.removeEventListener("keydown", onKeyDown, true);
  if (apply && draft.value) {
    model.value = draft.value;
  }
  recording.value = false;
  draft.value = null;
}

function onKeyDown(event: KeyboardEvent) {
  event.preventDefault();
  event.stopPropagation();
  if (event.key === "Escape") {
    stopRecording(false);
    return;
  }
  const captured = captureShortcutFromEvent(event);
  if (!captured) return;
  draft.value = captured;
  stopRecording(true);
}
</script>

<template>
  <button
    type="button"
    class="shortcut-key-input"
    :class="{ recording }"
    @click="startRecording"
  >
    <kbd>{{ displayText }}</kbd>
    <span v-if="recording" class="shortcut-key-hint">{{ t("settings.escCancel") }}</span>
  </button>
</template>

<style scoped>
.shortcut-key-input {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 4px 8px;
  border: 1px dashed var(--border-color);
  border-radius: 4px;
  background: var(--surface-muted);
  cursor: pointer;
  font: inherit;
}

.shortcut-key-input.recording {
  border-color: var(--primary);
  background: var(--primary-light);
}

.shortcut-key-input kbd {
  font-size: 12px;
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
  color: var(--text-primary);
}

.shortcut-key-hint {
  font-size: 11px;
  color: var(--text-secondary);
}
</style>
