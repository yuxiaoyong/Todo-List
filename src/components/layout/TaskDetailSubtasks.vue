<script setup lang="ts">
import { computed, inject, onMounted, ref } from "vue";
import { useI18n } from "vue-i18n";
import { Delete, Loading, MagicStick } from "@element-plus/icons-vue";
import { ElMessage } from "element-plus";
import { aiGatewayApi, subtaskApi } from "../../api";
import { taskDetailPanelKey } from "../layout/taskDetailPanelContext";
import { useAiGatewayStore } from "../../stores/aiGateway";
import { htmlToPlainText } from "../../utils/htmlText";
import type { Subtask } from "../../types";
import AiSubtasksPreviewDialog from "../todo/AiSubtasksPreviewDialog.vue";

const props = defineProps<{
  todoId: number;
  subtasks: Subtask[];
  editable: boolean;
}>();

const emit = defineEmits<{
  change: [subtasks: Subtask[]];
}>();

const { t } = useI18n();
const aiStore = useAiGatewayStore();
const panelCtx = inject(taskDetailPanelKey, null);

const newTitle = ref("");
const adding = ref(false);
const togglingId = ref<number | null>(null);
const deletingId = ref<number | null>(null);
const parsing = ref(false);
const batchAdding = ref(false);
const previewOpen = ref(false);
const previewSteps = ref<string[]>([]);

onMounted(() => {
  if (!aiStore.ready) {
    void aiStore.load().catch(() => {
      // AI button stays hidden until enabled
    });
  }
});

const totalCount = computed(() => props.subtasks.length);
const completedCount = computed(() => props.subtasks.filter((item) => item.completed).length);
const progressPercent = computed(() => {
  if (!totalCount.value) return 0;
  return Math.round((completedCount.value / totalCount.value) * 100);
});

const taskTitle = computed(() => panelCtx?.form.value?.title.trim() ?? "");
const taskContent = computed(() => {
  const html = panelCtx?.form.value?.contentHtml ?? "";
  return htmlToPlainText(html);
});

const canDecompose = computed(
  () => aiStore.isActive && props.editable && Boolean(taskTitle.value) && !parsing.value,
);

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

async function decomposeWithAi() {
  if (!canDecompose.value) {
    if (!taskTitle.value) {
      ElMessage.warning(t("aiSubtask.titleRequired"));
    }
    return;
  }
  parsing.value = true;
  try {
    const existingSteps = props.subtasks.map((item) => item.title);
    const content = taskContent.value || undefined;
    const result = await aiGatewayApi.decomposeSubtasks({
      title: taskTitle.value,
      content,
      existingSteps,
    });
    previewSteps.value = result.steps;
    previewOpen.value = true;
  } catch (error) {
    console.error("ai decompose subtasks failed", error);
    const message =
      typeof error === "string"
        ? error
        : error instanceof Error
          ? error.message
          : t("aiSubtask.decomposeFailed");
    ElMessage.error(message);
  } finally {
    parsing.value = false;
  }
}

async function onConfirmSteps(titles: string[]) {
  if (!titles.length || batchAdding.value) return;
  batchAdding.value = true;
  try {
    const created: Subtask[] = [];
    for (const title of titles) {
      created.push(await subtaskApi.create(props.todoId, title));
    }
    updateList((list) => [...list, ...created]);
    previewOpen.value = false;
    ElMessage.success(t("aiSubtask.added", { count: created.length }));
  } catch (error) {
    console.error("batch create subtasks failed", error);
    ElMessage.error(t("subtask.createFailed"));
  } finally {
    batchAdding.value = false;
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
        <div class="subtask-title-group">
          <span class="subtask-title">{{ t("subtask.title") }}</span>
          <span v-if="totalCount" class="subtask-count">
            {{ t("subtask.progress", { done: completedCount, total: totalCount }) }}
          </span>
        </div>
        <button
          v-if="aiStore.isActive && editable"
          type="button"
          class="subtask-ai-btn"
          :disabled="!canDecompose"
          :title="t('aiSubtask.decomposeHint')"
          @click="decomposeWithAi"
        >
          <el-icon v-if="parsing" class="is-loading">
            <Loading />
          </el-icon>
          <el-icon v-else>
            <MagicStick />
          </el-icon>
          <span>{{ t("aiSubtask.decompose") }}</span>
        </button>
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

    <AiSubtasksPreviewDialog
      v-model="previewOpen"
      :steps="previewSteps"
      :has-existing="subtasks.length > 0"
      :confirming="batchAdding"
      @confirm="onConfirmSteps"
    />
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

.subtask-title-group {
  display: flex;
  align-items: baseline;
  gap: 8px;
  min-width: 0;
}

.subtask-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
}

.subtask-count {
  font-size: 12px;
  color: var(--text-secondary);
  white-space: nowrap;
}

.subtask-ai-btn {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  flex-shrink: 0;
  padding: 4px 10px;
  border: 1px solid color-mix(in srgb, var(--primary) 35%, var(--border-color));
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--primary);
  font-size: 12px;
  cursor: pointer;
  transition:
    background 0.15s,
    border-color 0.15s,
    color 0.15s;
}

.subtask-ai-btn:hover:not(:disabled) {
  background: var(--primary-light);
  border-color: color-mix(in srgb, var(--primary) 55%, var(--border-color));
  color: var(--primary-hover);
}

.subtask-ai-btn:disabled {
  opacity: 0.55;
  cursor: not-allowed;
}

.subtask-ai-btn .el-icon {
  font-size: 14px;
}

.subtask-progress {
  max-width: 100%;
}

.subtask-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 6px;
  overflow: auto;
  flex: 1;
  min-height: 0;
}

.subtask-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 8px;
  border-radius: var(--radius-sm);
  background: var(--surface-muted);
}

.subtask-item.is-done .subtask-input {
  text-decoration: line-through;
  color: var(--text-tertiary);
}

.subtask-input {
  flex: 1;
  min-width: 0;
  border: none;
  background: transparent;
  font-size: 13px;
  color: var(--text-primary);
  outline: none;
}

.subtask-input:read-only {
  cursor: default;
}

.subtask-delete {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  border: none;
  border-radius: 4px;
  background: transparent;
  color: var(--text-tertiary);
  cursor: pointer;
}

.subtask-delete:hover:not(:disabled) {
  color: #ff4d4f;
  background: color-mix(in srgb, #ff4d4f 12%, transparent);
}

.subtask-empty {
  margin: 8px 0 12px;
  font-size: 13px;
  color: var(--text-secondary);
}

.subtask-add {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 8px;
  flex-shrink: 0;
}

.subtask-add-checkbox {
  flex-shrink: 0;
}

.subtask-input--add::placeholder {
  color: var(--text-tertiary);
}
</style>
