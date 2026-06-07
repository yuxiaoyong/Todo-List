<script setup lang="ts">
import { nextTick, reactive, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { Calendar, Delete, Edit, Plus, Rank, Star, StarFilled } from "@element-plus/icons-vue";
import { ElMessage, ElMessageBox } from "element-plus";
import { useDraggable } from "vue-draggable-plus";
import KanbanTaskBucket from "./KanbanTaskBucket.vue";
import type { SortableEvent } from "sortablejs";
import { todoApi } from "../../api";
import { useKanbanColumnStore } from "../../stores/kanbanColumn";
import { useTodoStore } from "../../stores/todo";
import { useUiStore } from "../../stores/ui";
import {
  buildBucketSortPositions,
  groupTodosByKanban,
  nextKanbanColor,
} from "../../utils/kanban";
import type { KanbanColumn, TodoSummary } from "../../types";

const props = defineProps<{
  todos: TodoSummary[];
  loading?: boolean;
}>();

const emit = defineEmits<{
  refresh: [];
  view: [id: number];
  toggle: [todo: TodoSummary];
  delete: [todo: TodoSummary];
}>();

const { t } = useI18n();
const kanbanColumnStore = useKanbanColumnStore();
const todoStore = useTodoStore();
const uiStore = useUiStore();

const columnOrder = ref<KanbanColumn[]>([]);
const buckets = ref<Record<number, TodoSummary[]>>({});
const unassigned = ref<TodoSummary[]>([]);
const dragOverTarget = ref<string | null>(null);
const moving = ref(false);
const columnReordering = ref(false);
const taskReordering = ref(false);

const columnDialogOpen = ref(false);
const editingColumnId = ref<number | null>(null);
const columnForm = reactive({
  name: "",
  subtitle: "",
  color: "#1677ff",
});

type BucketKey = "unassigned" | number;

const dragSourceBucket = ref<BucketKey | null>(null);
const showTaskBuckets = ref(true);
const bucketsRenderKey = ref(0);

const columnsEl = ref<HTMLElement | null>(null);

useDraggable(columnsEl, columnOrder, {
  animation: 220,
  direction: "horizontal",
  handle: ".column-header-drag-area",
  draggable: ".kanban-column-unit",
  filter: ".no-drag",
  preventOnFilter: false,
  ghostClass: "column-sort-ghost",
  chosenClass: "column-sort-chosen",
  dragClass: "column-sort-drag",
  forceFallback: true,
  fallbackOnBody: true,
  easing: "cubic-bezier(0.2, 0, 0, 1)",
  onStart() {
    columnReordering.value = true;
  },
  onEnd() {
    columnReordering.value = false;
    void persistColumnOrder();
  },
});

function syncColumnOrder() {
  if (columnReordering.value) return;
  columnOrder.value = [...kanbanColumnStore.columns];
}

function syncTodoBuckets() {
  const grouped = groupTodosByKanban(props.todos, columnOrder.value);
  buckets.value = { ...grouped.buckets };
  unassigned.value = [...grouped.unassigned];
}

function syncFromProps() {
  syncColumnOrder();
  syncTodoBuckets();
}

watch(
  () => kanbanColumnStore.columns,
  () => syncColumnOrder(),
  { immediate: true, deep: true },
);

watch(
  () => props.todos,
  () => {
    if (moving.value || taskReordering.value) return;
    syncTodoBuckets();
  },
  { immediate: true, deep: true },
);

function formatDueDate(date?: string) {
  if (!date) return "";
  return date.length <= 10 ? date : date.slice(0, 10);
}

function priorityClass(priority: string) {
  if (priority === "high") return "priority-high";
  if (priority === "medium") return "priority-medium";
  return "priority-none";
}

function parseTodoId(evt: SortableEvent): number | null {
  const id = Number((evt.item as HTMLElement).dataset.todoId);
  if (!Number.isFinite(id) || id <= 0) return null;
  return id;
}

function parseBucketKeyString(key: string): BucketKey | null {
  if (key === "unassigned") return "unassigned";
  const id = Number(key);
  return Number.isFinite(id) && id > 0 ? id : null;
}

function parseBucketKey(el: HTMLElement): BucketKey | null {
  const direct = parseBucketKeyString(el.dataset.bucket ?? "");
  if (direct !== null) return direct;
  return null;
}

function resolveBucketKey(el: HTMLElement | null): BucketKey | null {
  let node: HTMLElement | null = el;
  while (node) {
    const key = parseBucketKey(node);
    if (key !== null) return key;
    node = node.parentElement;
  }
  return null;
}

function getBucketByKey(key: BucketKey): TodoSummary[] {
  if (key === "unassigned") return unassigned.value;
  return buckets.value[key] ?? [];
}

function onBucketSortStart(bucketKey: string) {
  dragSourceBucket.value = parseBucketKeyString(bucketKey);
  taskReordering.value = true;
  dragOverTarget.value = bucketKey;
}

function updateBucket(columnId: number, list: TodoSummary[]) {
  buckets.value = { ...buckets.value, [columnId]: list };
}

function cleanupDragFallbackNodes() {
  document
    .querySelectorAll("body > .sortable-fallback, body > .dragging-card")
    .forEach((node) => node.remove());
}

async function forceRemountBuckets() {
  await todoStore.fetchList(uiStore.buildFilter());
  syncFromProps();
  showTaskBuckets.value = false;
  await nextTick();
  cleanupDragFallbackNodes();
  bucketsRenderKey.value += 1;
  showTaskBuckets.value = true;
  await nextTick();
}

async function handleTaskSortEnd(evt: SortableEvent, sourceBucketKey: string) {
  dragOverTarget.value = null;

  const fromKey =
    dragSourceBucket.value ?? parseBucketKeyString(sourceBucketKey);
  const toKey = resolveBucketKey(evt.to as HTMLElement) ?? fromKey;
  const id = parseTodoId(evt);

  dragSourceBucket.value = null;

  if (!id || fromKey === null || toKey === null) {
    taskReordering.value = false;
    return;
  }
  if (evt.from === evt.to && evt.oldIndex === evt.newIndex) {
    taskReordering.value = false;
    return;
  }
  if (moving.value) {
    taskReordering.value = false;
    return;
  }

  await nextTick();

  moving.value = true;
  try {
    const keysToPersist = new Set<BucketKey>();

    if (fromKey !== toKey) {
      const todo = props.todos.find((item) => item.id === id);
      if (!todo) {
        syncFromProps();
        return;
      }
      if (toKey === "unassigned") {
        if (todo.kanbanColumnId) {
          await todoApi.setKanbanColumn(id, null);
        }
      } else if (todo.kanbanColumnId !== toKey) {
        await todoApi.setKanbanColumn(id, toKey);
      }
      keysToPersist.add(fromKey);
      keysToPersist.add(toKey);
    } else {
      keysToPersist.add(toKey);
    }

    const items = [...keysToPersist].flatMap((key) =>
      buildBucketSortPositions(getBucketByKey(key)),
    );
    if (items.length) {
      await todoApi.reorderPositions(items);
    }

    await forceRemountBuckets();
  } catch (error) {
    console.error("task drag persist failed", error);
    syncFromProps();
    ElMessage.error(fromKey !== toKey ? t("kanban.moveFailed") : t("kanban.reorderFailed"));
  } finally {
    moving.value = false;
    taskReordering.value = false;
  }
}

function openCreateColumn() {
  editingColumnId.value = null;
  columnForm.name = "";
  columnForm.subtitle = "";
  columnForm.color = nextKanbanColor(kanbanColumnStore.columns);
  columnDialogOpen.value = true;
}

function openEditColumn(column: KanbanColumn) {
  editingColumnId.value = column.id;
  columnForm.name = column.name;
  columnForm.subtitle = column.subtitle ?? "";
  columnForm.color = column.color;
  columnDialogOpen.value = true;
}

async function saveColumn() {
  const name = columnForm.name.trim();
  if (!name) {
    ElMessage.warning(t("kanban.columnNameRequired"));
    return;
  }
  const subtitle = columnForm.subtitle.trim() || null;
  try {
    if (editingColumnId.value) {
      await kanbanColumnStore.update(
        editingColumnId.value,
        name,
        columnForm.color,
        subtitle,
      );
    } else {
      await kanbanColumnStore.create(name, columnForm.color, subtitle ?? undefined);
    }
    columnDialogOpen.value = false;
    emit("refresh");
  } catch (error) {
    console.error("save column failed", error);
    ElMessage.error(t("kanban.saveColumnFailed"));
  }
}

async function removeColumn(column: KanbanColumn) {
  await ElMessageBox.confirm(
    t("kanban.deleteColumnConfirm", { name: column.name }),
    t("common.hint"),
    { type: "warning" },
  );
  try {
    await kanbanColumnStore.remove(column.id);
    emit("refresh");
  } catch (error) {
    console.error("delete column failed", error);
    ElMessage.error(t("kanban.deleteColumnFailed"));
  }
}

function getColumnAccentStyle(column: KanbanColumn) {
  return { "--column-accent": column.color };
}

async function persistColumnOrder() {
  const ids = columnOrder.value.map((column) => column.id);
  const previous = kanbanColumnStore.columns.map((column) => column.id);
  if (ids.join(",") === previous.join(",")) return;

  try {
    await kanbanColumnStore.reorder(ids);
  } catch (error) {
    console.error("reorder columns failed", error);
    syncColumnOrder();
    ElMessage.error(t("kanban.reorderColumnFailed"));
  }
}

function onDelete(todo: TodoSummary) {
  emit("delete", todo);
}

async function onTogglePin(todo: TodoSummary) {
  await todoApi.togglePin(todo.id);
  emit("refresh");
}
</script>

<template>
  <div
    v-loading="loading || kanbanColumnStore.loading"
    class="kanban-view"
    :class="{ 'is-task-reordering': taskReordering }"
  >
    <div class="kanban-board">
      <section
        class="kanban-column inbox-column"
        :class="{ 'is-drag-over': dragOverTarget === 'unassigned' }"
      >
        <header class="column-header">
          <div>
            <div class="column-title">{{ t("kanban.unassigned") }}</div>
            <div class="column-subtitle">{{ t("kanban.unassignedHint") }}</div>
          </div>
          <span class="column-count">{{ unassigned.length }}</span>
        </header>
        <KanbanTaskBucket
          v-if="showTaskBuckets"
          :key="`unassigned-${bucketsRenderKey}`"
          v-model="unassigned"
          bucket-key="unassigned"
          :disabled="columnReordering"
          @sort-start="onBucketSortStart"
          @sort-end="handleTaskSortEnd"
        >
          <template #default="{ todo }">
            <div
              class="task-card"
              :class="{ done: todo.completed, pinned: todo.pinned }"
              :data-todo-id="todo.id"
              @click="emit('view', todo.id)"
            >
              <div class="task-card-top">
                <el-checkbox
                  class="no-drag"
                  :model-value="todo.completed"
                  @click.stop
                  @change="emit('toggle', todo)"
                />
                <span class="task-title" :class="{ struck: todo.completed }">{{ todo.title }}</span>
                <button
                  type="button"
                  class="task-pin no-drag"
                  :class="{ active: todo.pinned }"
                  :title="todo.pinned ? t('task.unpin') : t('task.pin')"
                  @click.stop="onTogglePin(todo)"
                >
                  <el-icon>
                    <StarFilled v-if="todo.pinned" />
                    <Star v-else />
                  </el-icon>
                </button>
              </div>
            </div>
          </template>
        </KanbanTaskBucket>
        <div v-if="!unassigned.length" class="column-empty">{{ t("kanban.unassignedEmpty") }}</div>
      </section>

      <div ref="columnsEl" class="kanban-custom-columns">
        <div
          v-for="column in columnOrder"
          :key="column.id"
          class="kanban-column-unit"
          :class="{ 'is-drag-over': dragOverTarget === String(column.id) }"
          :style="getColumnAccentStyle(column)"
        >
          <header class="column-header">
            <div class="column-header-drag-area" :title="t('kanban.dragSort')">
              <el-icon class="column-drag-icon"><Rank /></el-icon>
              <div class="column-header-text">
                <div class="column-title">{{ column.name }}</div>
                <div v-if="column.subtitle" class="column-subtitle">{{ column.subtitle }}</div>
              </div>
            </div>
            <div class="column-header-actions no-drag">
              <span class="column-count">{{ buckets[column.id]?.length ?? 0 }}</span>
              <button
                type="button"
                class="column-action"
                :title="t('kanban.editGroup')"
                @click.stop="openEditColumn(column)"
              >
                <el-icon><Edit /></el-icon>
              </button>
              <button
                type="button"
                class="column-action"
                :title="t('kanban.deleteGroup')"
                @click.stop="removeColumn(column)"
              >
                <el-icon><Delete /></el-icon>
              </button>
            </div>
          </header>

          <div class="column-body-wrap">
            <KanbanTaskBucket
              v-if="showTaskBuckets"
              :key="`${column.id}-${bucketsRenderKey}`"
              :model-value="buckets[column.id] ?? []"
              :bucket-key="String(column.id)"
              :disabled="columnReordering"
              @update:model-value="(list) => updateBucket(column.id, list)"
              @sort-start="onBucketSortStart"
              @sort-end="handleTaskSortEnd"
            >
              <template #default="{ todo }">
                <div
                  class="task-card"
                  :class="{ done: todo.completed, pinned: todo.pinned }"
                  :data-todo-id="todo.id"
                  @click="emit('view', todo.id)"
                >
                  <div class="task-card-top">
                    <el-checkbox
                      class="no-drag"
                      :model-value="todo.completed"
                      @click.stop
                      @change="emit('toggle', todo)"
                    />
                    <span class="task-title" :class="{ struck: todo.completed }">{{ todo.title }}</span>
                    <button
                      type="button"
                      class="task-pin no-drag"
                      :class="{ active: todo.pinned }"
                      :title="todo.pinned ? t('task.unpin') : t('task.pin')"
                      @click.stop="onTogglePin(todo)"
                    >
                      <el-icon>
                        <StarFilled v-if="todo.pinned" />
                        <Star v-else />
                      </el-icon>
                    </button>
                    <button type="button" class="task-delete no-drag" :title="t('common.delete')" @click.stop="onDelete(todo)">
                      <el-icon><Delete /></el-icon>
                    </button>
                  </div>
                  <div class="task-card-meta">
                    <span class="priority-pill" :class="priorityClass(todo.priority)">
                      {{
                        todo.priority === "high"
                          ? t("priority.high")
                          : todo.priority === "medium"
                            ? t("priority.medium")
                            : t("priority.low")
                      }}
                    </span>
                    <span v-if="todo.dueDate" class="due-date">
                      <el-icon><Calendar /></el-icon>
                      {{ formatDueDate(todo.dueDate) }}
                    </span>
                    <span v-if="todo.categoryName" class="category-name">{{ todo.categoryName }}</span>
                  </div>
                </div>
              </template>
            </KanbanTaskBucket>
            <div v-if="!buckets[column.id]?.length" class="column-empty">{{ t("kanban.columnEmpty") }}</div>
          </div>
        </div>
      </div>

      <button type="button" class="add-column-card no-drag" @click="openCreateColumn">
        <el-icon><Plus /></el-icon>
        <span>{{ t("kanban.addColumn") }}</span>
      </button>
    </div>

    <el-dialog
      v-model="columnDialogOpen"
      :title="editingColumnId ? t('kanban.editColumn') : t('kanban.addColumn')"
      width="420px"
      destroy-on-close
    >
      <el-form label-width="72px">
        <el-form-item :label="t('kanban.name')" required>
          <el-input v-model="columnForm.name" :placeholder="t('kanban.namePlaceholder')" maxlength="30" />
        </el-form-item>
        <el-form-item :label="t('kanban.description')">
          <el-input
            v-model="columnForm.subtitle"
            :placeholder="t('kanban.descPlaceholder')"
            maxlength="40"
          />
        </el-form-item>
        <el-form-item :label="t('kanban.color')">
          <el-color-picker v-model="columnForm.color" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="columnDialogOpen = false">{{ t("common.cancel") }}</el-button>
        <el-button type="primary" @click="saveColumn">{{ t("common.save") }}</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<style scoped>
.kanban-view {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  padding: 16px;
  background: var(--panel-bg);
}

.kanban-view.is-task-reordering {
  user-select: none;
  -webkit-user-select: none;
}

.kanban-board {
  display: flex;
  align-items: stretch;
  gap: 12px;
  flex: 1;
  min-height: 0;
  max-height: calc(100% - 10px);
  overflow-x: auto;
  overflow-y: hidden;
}

.kanban-custom-columns {
  display: flex;
  align-items: stretch;
  gap: 12px;
  min-height: 0;
}

.kanban-column-unit {
  position: relative;
  display: flex;
  flex-direction: column;
  width: 300px;
  min-width: 300px;
  min-height: 0;
  border: 1px solid var(--border-light);
  border-top: 3px solid var(--column-accent, var(--border-color));
  border-radius: var(--radius);
  background: color-mix(in srgb, var(--column-accent, var(--primary)) 8%, var(--panel-bg));
  box-sizing: border-box;
}

.kanban-column-unit.is-drag-over {
  border-color: var(--primary);
  box-shadow: 0 0 0 2px color-mix(in srgb, var(--primary) 18%, transparent);
}

.inbox-column {
  --column-accent: #8c8c8c;
  position: relative;
  display: flex;
  flex-direction: column;
  width: 300px;
  min-width: 300px;
  min-height: 0;
  background: color-mix(in srgb, var(--column-accent) 8%, var(--panel-bg));
  border: 1px dashed var(--border-color);
  border-top: 3px solid var(--column-accent);
  border-radius: var(--radius);
}

.inbox-column.is-drag-over {
  border-color: var(--primary);
  box-shadow: 0 0 0 2px color-mix(in srgb, var(--primary) 18%, transparent);
}

.column-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 8px;
  flex-shrink: 0;
  padding: 8px 8px 8px 10px;
  border-bottom: 1px solid var(--border-light);
  background: color-mix(in srgb, var(--surface-elevated) 92%, transparent);
  border-radius: var(--radius) var(--radius) 0 0;
}

.column-header-drag-area {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  flex: 1;
  min-width: 0;
  padding: 4px 6px;
  border-radius: var(--radius-sm);
  cursor: grab;
  user-select: none;
  touch-action: none;
  transition: background 0.15s;
}

.column-header-drag-area:hover {
  background: var(--primary-light);
}

.column-header-drag-area:active {
  cursor: grabbing;
}

.column-drag-icon {
  margin-top: 2px;
  color: var(--text-tertiary);
  flex-shrink: 0;
}

.column-header-text {
  min-width: 0;
}

.column-header-actions {
  display: flex;
  align-items: center;
  gap: 4px;
  flex-shrink: 0;
}

.column-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
}

.column-subtitle {
  margin-top: 2px;
  font-size: 12px;
  color: var(--text-secondary);
}

.column-count {
  min-width: 22px;
  height: 22px;
  padding: 0 6px;
  border-radius: 11px;
  background: var(--surface-elevated);
  border: 1px solid var(--border-light);
  color: var(--text-secondary);
  font-size: 12px;
  line-height: 22px;
  text-align: center;
}

.column-action {
  border: none;
  background: transparent;
  color: var(--text-tertiary);
  cursor: pointer;
  padding: 2px;
}

.column-action:hover {
  color: var(--text-secondary);
}

.column-body-wrap {
  position: relative;
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
}

:deep(.column-body) {
  flex: 1;
  min-height: 0;
  overflow-x: hidden;
  overflow-y: auto;
  padding: 10px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

:deep(.column-body.is-empty) {
  min-height: 0;
}

.column-empty {
  position: absolute;
  inset: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-tertiary);
  font-size: 13px;
  pointer-events: none;
}

.add-column-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
  width: 180px;
  min-width: 180px;
  min-height: 0;
  align-self: stretch;
  border: 1px dashed var(--border-color);
  border-radius: var(--radius);
  background: transparent;
  color: var(--text-secondary);
  cursor: pointer;
  transition: border-color 0.15s, color 0.15s, background 0.15s;
}

.add-column-card:hover {
  border-color: var(--primary);
  color: var(--primary);
  background: var(--primary-light);
}

.task-card {
  min-width: 0;
  padding: 10px 12px;
  border-radius: var(--radius-sm);
  background: var(--surface-elevated);
  border: 1px solid var(--border-light);
  box-shadow: var(--shadow-sm);
  cursor: grab;
  user-select: none;
  -webkit-user-select: none;
  touch-action: none;
  transition: opacity 0.15s, box-shadow 0.15s;
}

.task-card * {
  user-select: none;
  -webkit-user-select: none;
}

.kanban-view.is-task-reordering .task-card {
  cursor: grabbing;
}

.task-card:hover {
  border-color: var(--border-color);
}

.task-card.done {
  opacity: 0.72;
}

:deep(.column-body .drag-ghost) {
  opacity: 0.45;
  background: color-mix(in srgb, var(--primary) 10%, var(--surface-elevated));
  border: 1px dashed var(--primary);
  box-shadow: none;
}

:deep(.column-body .drag-chosen) {
  box-shadow: var(--shadow-card);
}

:deep(.column-body .dragging-card) {
  opacity: 0.92;
  box-shadow: var(--shadow-card);
}

:deep(.column-sort-ghost) {
  opacity: 0.45;
}

:deep(.column-sort-chosen) {
  box-shadow: var(--shadow-card);
}

:deep(.column-sort-drag) {
  opacity: 0.92;
}

.task-card-top {
  display: flex;
  align-items: flex-start;
  gap: 8px;
}

.task-title {
  flex: 1;
  min-width: 0;
  font-size: 14px;
  font-weight: 500;
  color: var(--text-primary);
  line-height: 1.4;
  word-break: break-word;
  overflow-wrap: anywhere;
}

.task-title.struck {
  text-decoration: line-through;
  color: var(--text-secondary);
}

.task-pin,
.task-delete {
  border: none;
  background: transparent;
  color: var(--text-tertiary);
  cursor: pointer;
  padding: 0;
  opacity: 0;
  transition: opacity 0.15s, color 0.15s;
}

.task-pin.active {
  opacity: 1;
  color: var(--pin-color);
}

.task-card:hover .task-pin,
.task-card:hover .task-delete,
.task-card.pinned .task-pin {
  opacity: 1;
}

.task-pin:hover {
  color: var(--pin-color);
}

.task-delete:hover {
  color: var(--priority-high-text);
}

.task-card.pinned {
  background: var(--pin-row-bg);
}

.task-card-meta {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 8px;
  margin-top: 8px;
  font-size: 12px;
}

.priority-pill {
  display: inline-block;
  padding: 1px 8px;
  border-radius: 10px;
  font-size: 12px;
}

.priority-high {
  background: var(--priority-high-bg);
  color: var(--priority-high-text);
}

.priority-medium {
  background: var(--priority-medium-bg);
  color: var(--priority-medium-text);
}

.priority-none {
  background: var(--priority-low-bg);
  color: var(--priority-low-text);
}

.due-date {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  color: var(--text-secondary);
}

.category-name {
  color: var(--text-secondary);
}
</style>
