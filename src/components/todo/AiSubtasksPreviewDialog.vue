<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { useI18n } from "vue-i18n";

const { t } = useI18n();

const props = defineProps<{
  modelValue: boolean;
  steps: string[];
  hasExisting: boolean;
  confirming?: boolean;
}>();

const emit = defineEmits<{
  "update:modelValue": [value: boolean];
  confirm: [titles: string[]];
}>();

interface StepDraft {
  selected: boolean;
  title: string;
}

const drafts = ref<StepDraft[]>([]);

const selectedCount = computed(() => drafts.value.filter((item) => item.selected).length);

const allSelected = computed({
  get: () => drafts.value.length > 0 && drafts.value.every((item) => item.selected),
  set: (value: boolean) => {
    drafts.value.forEach((item) => {
      item.selected = value;
    });
  },
});

watch(
  () => props.modelValue,
  (open) => {
    if (!open) return;
    drafts.value = props.steps.map((title) => ({ selected: true, title }));
  },
);

function onConfirm() {
  const titles = drafts.value
    .filter((item) => item.selected)
    .map((item) => item.title.trim())
    .filter(Boolean);
  if (!titles.length) return;
  emit("confirm", titles);
}
</script>

<template>
  <el-dialog
    :model-value="modelValue"
    :title="t('aiSubtask.previewTitle')"
    width="480px"
    destroy-on-close
    append-to-body
    align-center
    :z-index="4000"
    class="app-dialog ai-subtasks-preview"
    @update:model-value="emit('update:modelValue', $event)"
  >
    <p class="ai-subtasks-preview__desc">
      {{ hasExisting ? t("aiSubtask.previewDescAppend") : t("aiSubtask.previewDesc") }}
    </p>
    <p class="ai-subtasks-preview__hint">{{ t("aiSubtask.previewHint") }}</p>

    <div v-if="drafts.length" class="ai-subtasks-preview__toolbar">
      <el-checkbox v-model="allSelected">{{ t("aiSubtask.selectAll") }}</el-checkbox>
      <span class="ai-subtasks-preview__count">
        {{ t("aiSubtask.selectedCount", { count: selectedCount }) }}
      </span>
    </div>

    <ul v-if="drafts.length" class="ai-subtasks-preview__list">
      <li v-for="(item, index) in drafts" :key="index" class="ai-subtasks-preview__item">
        <el-checkbox v-model="item.selected" />
        <el-input v-model="item.title" :placeholder="t('aiSubtask.stepPlaceholder')" />
      </li>
    </ul>

    <template #footer>
      <el-button @click="emit('update:modelValue', false)" :disabled="confirming">
        {{ t("common.cancel") }}
      </el-button>
      <el-button
        type="primary"
        :disabled="selectedCount === 0"
        :loading="confirming"
        @click="onConfirm"
      >
        {{ t("aiSubtask.addSelected", { count: selectedCount }) }}
      </el-button>
    </template>
  </el-dialog>
</template>

<style scoped>
.ai-subtasks-preview__desc {
  margin: 0 0 6px;
  font-size: 13px;
  line-height: 1.5;
  color: var(--text-primary);
}

.ai-subtasks-preview__hint {
  margin: 0 0 14px;
  font-size: 12px;
  line-height: 1.5;
  color: var(--text-secondary);
}

.ai-subtasks-preview__toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  margin-bottom: 10px;
}

.ai-subtasks-preview__count {
  font-size: 12px;
  color: var(--text-secondary);
}

.ai-subtasks-preview__list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 8px;
  max-height: 320px;
  overflow: auto;
}

.ai-subtasks-preview__item {
  display: flex;
  align-items: center;
  gap: 8px;
}

.ai-subtasks-preview__item .el-input {
  flex: 1;
  min-width: 0;
}
</style>
