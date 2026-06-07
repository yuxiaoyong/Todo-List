<script setup lang="ts">
import { computed, provide, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { ElMessage, ElMessageBox } from "element-plus";
import TaskDetailPanelBody from "./TaskDetailPanelBody.vue";
import { taskDetailPanelKey } from "./taskDetailPanelContext";
import { useCategoryStore } from "../../stores/category";
import { useKanbanColumnStore } from "../../stores/kanbanColumn";
import { useTagStore } from "../../stores/tag";
import { useTodoStore } from "../../stores/todo";
import { useUiStore } from "../../stores/ui";
import { attachmentApi, todoApi } from "../../api";
import { useAutoSave } from "../../composables/useAutoSave";
import { useUndoDelete } from "../../composables/useUndoDelete";
import { formatDateTimeCn } from "../../utils/formatDate";
import { isPanelAttachment } from "../../utils/attachmentTypes";
import { parseRecurrenceConfig } from "../../utils/recurrence";
import type { AttachmentInfo, Subtask, TodoDetail, UpdateTodoInput } from "../../types";

const props = defineProps<{
  standalone?: boolean;
  taskId?: number | null;
}>();

const { t } = useI18n();
const emit = defineEmits<{ refresh: [] }>();

const categoryStore = useCategoryStore();
const kanbanColumnStore = useKanbanColumnStore();
const tagStore = useTagStore();
const todoStore = useTodoStore();
const uiStore = useUiStore();

const form = ref<UpdateTodoInput | null>(null);
const detail = ref<TodoDetail | null>(null);
const loadingDetail = ref(false);
const loadError = ref(false);
const savingNow = ref(false);
const activeTab = ref<"info" | "reminder" | "subtasks" | "attachments">("info");
const attachments = ref<AttachmentInfo[]>([]);
const subtasks = ref<Subtask[]>([]);

const { saving, flush: flushAutoSave, resetSnapshot } = useAutoSave(form);
const { deleteWithUndo } = useUndoDelete(async () => emit("refresh"));

const visible = computed({
  get: () => (props.standalone ? !!props.taskId : uiStore.detailDrawerOpen),
  set: (v: boolean) => {
    if (!v) void closeDrawer();
  },
});

const categoryColor = computed(() => {
  if (!form.value?.categoryId) return "#1677ff";
  return categoryStore.categories.find((c) => c.id === form.value!.categoryId)?.color ?? "#1677ff";
});

const selectedTags = computed(() =>
  (form.value?.tagIds ?? [])
    .map((id) => tagStore.tags.find((t) => t.id === id))
    .filter(Boolean),
);

const priorityClass = computed(() => {
  const p = form.value?.priority ?? "low";
  if (p === "high") return "priority-high";
  if (p === "medium") return "priority-medium";
  return "priority-low";
});

const defaultAssignee = computed(() => t("taskDetail.assigneeDefault"));

const statusText = computed(() =>
  form.value?.completed ? t("status.completed") : t("status.inProgress"),
);
const assigneeText = computed(() => form.value?.assignee?.trim() || defaultAssignee.value);
const editable = computed(() => props.standalone || uiStore.viewMode !== "trash");
const tagPickerValue = ref<number | undefined>(undefined);

watch(activeTab, (tab, prev) => {
  if ((prev === "info" || prev === "reminder") && tab !== prev) {
    void flushAutoSave();
  }
});
const panelAttachments = computed(() =>
  attachments.value.filter((item) => isPanelAttachment(item, form.value?.contentHtml ?? "")),
);

async function loadDetail(id: number) {
  loadingDetail.value = true;
  loadError.value = false;
  try {
    await todoStore.fetchDetail(id);
    detail.value = todoStore.selectedTodo;
    if (!detail.value) {
      throw new Error("task not found");
    }
    const d = detail.value;
    form.value = {
      id: d.id,
      title: d.title,
      contentHtml: d.contentHtml || "",
      completed: d.completed,
      priority: d.priority,
      startDate: d.startDate,
      dueDate: d.dueDate,
      categoryId: d.categoryId,
      tagIds: [...d.tagIds],
      assignee: d.assignee || defaultAssignee.value,
      kanbanColumnId: d.kanbanColumnId ?? null,
      pinned: d.pinned,
      recurrenceJson: parseRecurrenceConfig(d.recurrenceJson),
    };
    resetSnapshot(form.value);
    subtasks.value = [...(d.subtasks ?? [])];
    tagPickerValue.value = undefined;
    activeTab.value = "info";
    void loadAttachments(id).catch((error) => {
      console.error("load attachments failed", error);
    });
  } catch (error) {
    console.error("load task detail failed", error);
    form.value = null;
    detail.value = null;
    attachments.value = [];
    subtasks.value = [];
    loadError.value = true;
  } finally {
    loadingDetail.value = false;
  }
}

async function loadAttachments(todoId: number) {
  attachments.value = await attachmentApi.list(todoId);
}

watch(
  () => (props.standalone ? props.taskId : null),
  async (id) => {
    if (!props.standalone) return;
    if (!id) {
      form.value = null;
      detail.value = null;
      attachments.value = [];
      subtasks.value = [];
      loadError.value = false;
      return;
    }
    await loadDetail(id);
  },
  { immediate: true },
);

watch(
  () => [uiStore.selectedTodoId, uiStore.detailDrawerOpen] as const,
  async ([id, open]) => {
    if (props.standalone) return;
    if (!open || !id) {
      form.value = null;
      detail.value = null;
      attachments.value = [];
      subtasks.value = [];
      return;
    }
    await loadDetail(id);
  },
);

async function closeStandaloneWindow() {
  try {
    await getCurrentWindow().close();
  } catch (error) {
    console.error("close task detail window failed", error);
    await getCurrentWindow().hide();
  }
}

async function closeDrawer() {
  if (props.standalone) {
    if (form.value) {
      void flushAutoSave(false);
    }
    form.value = null;
    detail.value = null;
    attachments.value = [];
    subtasks.value = [];
    tagPickerValue.value = undefined;
    await closeStandaloneWindow();
    return;
  }
  if (form.value) {
    await flushAutoSave(false);
  }
  uiStore.closeDetail();
  uiStore.selectTodo(null);
  todoStore.clearSelection();
  form.value = null;
  detail.value = null;
  subtasks.value = [];
  tagPickerValue.value = undefined;
}

function onSubtasksChange(list: Subtask[]) {
  subtasks.value = list;
  if (detail.value) {
    detail.value = { ...detail.value, subtasks: list };
  }
}

async function saveNow() {
  if (!form.value) return;
  savingNow.value = true;
  try {
    await flushAutoSave(false);
    await loadDetail(form.value.id);
    emit("refresh");
    ElMessage.success(t("taskDetail.saved"));
  } finally {
    savingNow.value = false;
  }
}

function onFieldBlur() {
  void flushAutoSave();
}

async function toggleCompleted() {
  if (!form.value || !editable.value) return;
  form.value.completed = !form.value.completed;
  await saveNow();
}

async function togglePin() {
  if (!form.value || !editable.value) return;
  const updated = await todoApi.togglePin(form.value.id);
  form.value.pinned = updated.pinned;
  detail.value = updated;
  resetSnapshot(form.value);
  emit("refresh");
}

async function onRestore() {
  if (!todoStore.selectedTodo) return;
  await todoApi.restore(todoStore.selectedTodo.id);
  ElMessage.success(t("task.restoreOk"));
  closeDrawer();
  emit("refresh");
}

async function onDelete() {
  if (!todoStore.selectedTodo) return;
  if (uiStore.viewMode === "trash") {
    await ElMessageBox.confirm(t("task.permanentDelete"), t("common.warning"), {
      type: "warning",
    });
    await todoApi.permanentDelete(todoStore.selectedTodo.id);
    closeDrawer();
    emit("refresh");
    return;
  }
  await deleteWithUndo(todoStore.selectedTodo);
  closeDrawer();
}

async function refreshAttachments() {
  if (form.value) await loadAttachments(form.value.id);
}

function mergeAttachments(items: AttachmentInfo[]) {
  if (!items.length) return;
  const existing = new Set(attachments.value.map((item) => item.id));
  const merged = [...attachments.value];
  for (const item of items) {
    if (!existing.has(item.id)) {
      merged.push(item);
      existing.add(item.id);
    }
  }
  attachments.value = merged;
}

function onAttachmentsUploaded(items: AttachmentInfo[]) {
  mergeAttachments(items);
  void refreshAttachments();
}

function removeTag(id: number) {
  if (!form.value) return;
  form.value.tagIds = form.value.tagIds.filter((t) => t !== id);
}

function onTagPickerChange(id: number | undefined) {
  if (!form.value || id == null) {
    tagPickerValue.value = undefined;
    return;
  }
  if (!form.value.tagIds.includes(id)) {
    form.value.tagIds = [...form.value.tagIds, id];
  }
  tagPickerValue.value = undefined;
}

provide(taskDetailPanelKey, {
  form,
  detail,
  editable,
  activeTab,
  tagPickerValue,
  saving,
  savingNow,
  categoryColor,
  selectedTags,
  priorityClass,
  statusText,
  assigneeText,
  panelAttachments,
  subtasks,
  onSubtasksChange,
  categoryStore,
  kanbanColumnStore,
  tagStore,
  t,
  togglePin,
  toggleCompleted,
  onFieldBlur,
  closeDrawer,
  saveNow,
  onRestore,
  onDelete,
  removeTag,
  onTagPickerChange,
  onAttachmentsUploaded,
  refreshAttachments,
  formatDateTimeCn,
});
</script>

<template>
  <div v-if="standalone" class="detail-shell detail-shell--standalone">
    <header class="standalone-chrome">
      <span class="standalone-title">{{ t("taskDetail.infoTab") }}</span>
      <el-button text @click="closeDrawer">{{ t("common.close") }}</el-button>
    </header>
    <div v-loading="loadingDetail" class="detail-shell-body">
      <div v-if="loadError" class="detail-load-error">
        <p>{{ t("taskDetail.loadFailed") }}</p>
        <el-button type="primary" @click="closeDrawer">{{ t("common.close") }}</el-button>
      </div>
      <TaskDetailPanelBody v-else />
    </div>
  </div>
  <el-drawer
    v-else
    v-model="visible"
    :with-header="false"
    size="800px"
    destroy-on-close
    class="task-drawer"
  >
    <div v-loading="loadingDetail" class="detail-shell">
      <div v-if="loadError" class="detail-load-error">
        <p>{{ t("taskDetail.loadFailed") }}</p>
        <el-button type="primary" @click="closeDrawer">{{ t("common.close") }}</el-button>
      </div>
      <TaskDetailPanelBody v-else />
    </div>
  </el-drawer>
</template>

<style scoped>
.detail-shell--standalone {
  height: 100vh;
}

.standalone-chrome {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 10px 16px;
  border-bottom: 1px solid var(--border-light);
  flex-shrink: 0;
  background: var(--panel-bg);
}

.standalone-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
}

.detail-shell-body {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
}

.detail-load-error {
  display: flex;
  flex: 1;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 16px;
  color: var(--text-secondary);
}

.detail-shell {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--panel-bg);
  color: var(--text-primary);
}
</style>
