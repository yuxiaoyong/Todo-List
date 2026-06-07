<script setup lang="ts">
import { computed, ref } from "vue";
import { useI18n } from "vue-i18n";
import { Delete } from "@element-plus/icons-vue";
import { ElMessage } from "element-plus";
import { subtaskApi } from "../../api";
import type { Subtask } from "../../types";

const props = defineProps<{
  todoId: number;
  subtasks: Subtask[];
  editable: boolean;
}>();

const emit = defineEmits<{
  change: [subtasks: Subtask[]];
}>();

const { t } = useI18n();
const newTitle = ref("");
const adding = ref(false);
const togglingId = ref<number | null>(null);
const deletingId = ref<number | null>(null);

const totalCount = computed(() => props.subtasks.length);
const completedCount = computed(() => props.subtasks.filter((item) => item.completed).length);
const progressPercent = computed(() => {
  if (!totalCount.value) return 0;
  return Math.round((completedCount.value / totalCount.value) * 100);
});

function updateList(mutator: (list: Subtask[]) => Subtask[]) {
  emit("change", mutator([...props.subtasks]));
}

async function onToggle(subtask: Subtask) {
  if (!props.editable || togglingId.value != null) return;
  togglingId.value = subtask.id;
  try {
    const updated = await subtaskApi.toggle(subtask.id);
    updateList((list) => list.map((item) => (item.id === updated.id ? updated : item)));
  } catch (error) {
    console.error("toggle subtask failed", error);
    ElMessage.error(t("subtask.toggleFailed"));
  } finally {
    togglingId.value = null;
  }
}

async function onDelete(subtask: Subtask) {
  if (!props.editable || deletingId.value != null) return;
  deletingId.value = subtask.id;
  try {
    await subtaskApi.delete(subtask.id);
    updateList((list) => list.filter((item) => item.id !== subtask.id));
  } catch (error) {
    console.error("delete subtask failed", error);
    ElMessage.error(t("subtask.deleteFailed"));
  } finally {
    deletingId.value = null;
  }
}

async function onTitleBlur(subtask: Subtask, value: string) {
  const trimmed = value.trim();
  if (!trimmed || trimmed === subtask.title) return;
  try {
    const updated = await subtaskApi.update(subtask.id, trimmed);
    updateList((list) => list.map((item) => (item.id === updated.id ? updated : item)));
  } catch (error) {
    console.error("update subtask failed", error);
    ElMessage.error(t("subtask.updateFailed"));
  }
}

async function addSubtask() {
  const title = newTitle.value.trim();
  if (!title || !props.editable || adding.value) return;
  adding.value = true;
  try {
    const created = await subtaskApi.create(props.todoId, title);
    updateList((list) => [...list, created]);
    newTitle.value = "";
  } catch (error) {
    console.error("create subtask failed", error);
    ElMessage.error(t("subtask.createFailed"));
  } finally {
    adding.value = false;
  }
}

function onNewKeydown(event: KeyboardEvent) {
  if (event.key === "Enter") {
    event.preventDefault();
    void addSubtask();
  }
}
</script>

<template>
  <section class="subtask-section">
    <div class="subtask-header">
      <div class="subtask-title-row">
        <span class="subtask-title">{{ t("subtask.title") }}</span>
        <span v-if="totalCount" class="subtask-count">
          {{ t("subtask.progress", { done: completedCount, total: totalCount }) }}
        </span>
      </div>
      <el-progress
        v-if="totalCount"
        :percentage="progressPercent"
        :stroke-width="8"
        :show-text="false"
        class="subtask-progress"
      />
    </div>

    <ul v-if="subtasks.length" class="subtask-list">
      <li
        v-for="subtask in subtasks"
        :key="subtask.id"
        class="subtask-item"
        :class="{ 'is-done': subtask.completed }"
      >
        <el-checkbox
          :model-value="subtask.completed"
          :disabled="!editable || togglingId === subtask.id"
          @click.stop="onToggle(subtask)"
        />
        <input
          class="subtask-input"
          :value="subtask.title"
          :readonly="!editable"
          :disabled="!editable"
          @change="onTitleBlur(subtask, ($event.target as HTMLInputElement).value)"
        />
        <button
          v-if="editable"
          type="button"
          class="subtask-delete"
          :disabled="deletingId === subtask.id"
          :title="t('common.delete')"
          @click="onDelete(subtask)"
        >
          <el-icon><Delete /></el-icon>
        </button>
      </li>
    </ul>

    <p v-else class="subtask-empty">{{ t("subtask.empty") }}</p>

    <div v-if="editable" class="subtask-add">
      <el-checkbox disabled class="subtask-add-checkbox" />
      <input
        v-model="newTitle"
        class="subtask-input subtask-input--add"
        :placeholder="t('subtask.addPlaceholder')"
        :disabled="adding"
        @keydown="onNewKeydown"
      />
      <el-button
        type="primary"
        link
        :disabled="!newTitle.trim() || adding"
        :loading="adding"
        @click="addSubtask"
      >
        {{ t("subtask.add") }}
      </el-button>
    </div>
  </section>
</template>

<style scoped>
.subtask-section {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  padding: 4px 0;
}

.subtask-header {
  margin-bottom: 12px;
}

.subtask-title-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  margin-bottom: 8px;
}

.subtask-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-primary);
}

.subtask-count {
  font-size: 12px;
  color: var(--text-secondary);
  flex-shrink: 0;
}

.subtask-progress {
  width: 100%;
}

.subtask-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 8px;
  flex: 1;
  min-height: 0;
  overflow-y: auto;
}

.subtask-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 8px;
  border-radius: 8px;
  background: var(--panel-bg);
}

.subtask-item.is-done .subtask-input {
  color: var(--text-secondary);
  text-decoration: line-through;
}

.subtask-input {
  flex: 1;
  min-width: 0;
  border: none;
  outline: none;
  background: transparent;
  font-size: 14px;
  color: var(--text-primary);
  padding: 4px 0;
}

.subtask-input:read-only {
  cursor: default;
}

.subtask-delete {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border: none;
  background: transparent;
  color: var(--text-tertiary);
  cursor: pointer;
  border-radius: 6px;
  flex-shrink: 0;
}

.subtask-delete:hover:not(:disabled) {
  color: var(--el-color-danger);
  background: var(--surface-subtle);
}

.subtask-empty {
  margin: 0 0 12px;
  font-size: 13px;
  color: var(--text-secondary);
}

.subtask-add {
  display: flex;
  align-items: center;
  gap: 8px;
  padding-top: 4px;
}

.subtask-add-checkbox {
  flex-shrink: 0;
  opacity: 0.35;
}

.subtask-input--add::placeholder {
  color: var(--text-tertiary);
}
</style>
