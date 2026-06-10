<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { Search, Plus, Delete } from "@element-plus/icons-vue";
import { ElMessage, ElMessageBox } from "element-plus";
import type { SortableEvent } from "sortablejs";
import NewTaskDialog from "../todo/NewTaskDialog.vue";
import QuickInputBar from "../todo/QuickInputBar.vue";
import KanbanView from "./KanbanView.vue";
import GanttView from "./GanttView.vue";
import DraggableTaskList from "./DraggableTaskList.vue";
import { useTodoStore } from "../../stores/todo";
import { useCategoryStore } from "../../stores/category";
import { useTagStore } from "../../stores/tag";
import { useUiStore } from "../../stores/ui";
import { todoApi } from "../../api";
import { buildBucketSortPositions } from "../../utils/kanban";
import { matchesTimeFilter } from "../../utils/timeFilter";
import {
  buildTodoSort,
  parseTodoSort,
  sortTodos,
  TODO_SORT_PROP_MAP,
  type TodoSortField,
} from "../../utils/sortTodos";
import { useUndoDelete } from "../../composables/useUndoDelete";
import type { TodoSummary } from "../../types";

const { t } = useI18n();
const emit = defineEmits<{ refresh: [] }>();

const todoStore = useTodoStore();
const categoryStore = useCategoryStore();
const tagStore = useTagStore();
const uiStore = useUiStore();
const { deleteWithUndo } = useUndoDelete(async () => {
  emit("refresh");
});

const sortableTodos = ref<TodoSummary[]>([]);
const listReordering = ref(false);

const filteredTodos = computed(() =>
  todoStore.todos.filter((t) => matchesTimeFilter(t, uiStore.timeFilter)),
);

const parsedSort = computed(() => parseTodoSort(uiStore.todoSort));

const displayTodos = computed(() => {
  const { field, direction } = parsedSort.value;
  return sortTodos(filteredTodos.value, field, direction);
});

const kanbanTodos = computed(() => filteredTodos.value);
const ganttTodos = computed(() => filteredTodos.value);
const isTrashMode = computed(() => uiStore.viewMode === "trash");
const canDrag = computed(() => parsedSort.value.field === "default" && !isTrashMode.value);

watch(
  displayTodos,
  (value) => {
    if (listReordering.value) return;
    sortableTodos.value = [...value];
  },
  { immediate: true },
);

async function refresh() {
  await todoStore.fetchList(uiStore.buildFilter());
}

let searchTimer: ReturnType<typeof setTimeout> | null = null;
watch(
  () => uiStore.searchQuery,
  () => {
    if (searchTimer) clearTimeout(searchTimer);
    searchTimer = setTimeout(() => refresh(), 300);
  },
);

async function onToggle(todo: TodoSummary) {
  await todoApi.toggleComplete(todo.id);
  emit("refresh");
}

async function onTogglePin(todo: TodoSummary) {
  await todoApi.togglePin(todo.id);
  emit("refresh");
}

function onView(id: number) {
  uiStore.openDetail(id);
}

function onEdit(id: number) {
  uiStore.openDetail(id);
}

async function updateTodoInline(
  todo: TodoSummary,
  patch: {
    title?: string;
    priority?: string;
    dueDate?: string | null;
    categoryId?: number | null;
    tagIds?: number[];
  },
) {
  const detail = await todoApi.get(todo.id);
  await todoApi.update({
    id: todo.id,
    title: patch.title ?? todo.title,
    contentHtml: detail.contentHtml,
    completed: todo.completed,
    priority: patch.priority ?? todo.priority,
    dueDate: patch.dueDate !== undefined ? patch.dueDate : todo.dueDate,
    categoryId: patch.categoryId !== undefined ? patch.categoryId : todo.categoryId,
    tagIds: patch.tagIds ?? todo.tagIds,
    sortOrder: todo.sortOrder,
    pinned: todo.pinned,
    assignee: todo.assignee,
    kanbanColumnId: todo.kanbanColumnId,
    quiet: true,
  });
  emit("refresh");
}

async function onTitleUpdate(todo: TodoSummary, title: string) {
  await updateTodoInline(todo, { title });
}

async function onPriorityUpdate(todo: TodoSummary, priority: string) {
  await updateTodoInline(todo, { priority });
}

async function onDueDateUpdate(todo: TodoSummary, dueDate: string | null) {
  await updateTodoInline(todo, { dueDate });
}

async function onTagsUpdate(todo: TodoSummary, tagIds: number[]) {
  await updateTodoInline(todo, { tagIds });
}

async function onCategoryUpdate(todo: TodoSummary, categoryId: number | null) {
  await updateTodoInline(todo, { categoryId });
}

async function onRestore(todo: TodoSummary) {
  await todoApi.restore(todo.id);
  ElMessage.success(t("task.restoreOk"));
  emit("refresh");
}

async function onDelete(todo: TodoSummary) {
  if (isTrashMode.value) {
    await ElMessageBox.confirm(t("task.permanentDelete"), t("common.warning"), {
      type: "warning",
    });
    await todoApi.permanentDelete(todo.id);
    emit("refresh");
    return;
  }
  await deleteWithUndo(todo);
}

async function onEmptyTrash() {
  if (!displayTodos.value.length) return;
  await ElMessageBox.confirm(t("task.emptyTrashConfirm"), t("task.emptyTrashTitle"), {
    type: "warning",
    confirmButtonText: t("task.emptyTrash"),
  });
  await todoApi.emptyTrash();
  ElMessage.success(t("task.emptyTrashOk"));
  emit("refresh");
}

function onQuickCreated(openDetail: boolean) {
  if (!openDetail) {
    window.setTimeout(() => todoStore.setHighlight(null), 2000);
  }
}

function onListSortStart() {
  listReordering.value = true;
}

async function onListSortEnd(evt: SortableEvent) {
  if (evt.oldIndex === evt.newIndex || evt.oldIndex == null || evt.newIndex == null) {
    listReordering.value = false;
    return;
  }

  try {
    const items = buildBucketSortPositions(sortableTodos.value);
    if (items.length) {
      await todoApi.reorderPositions(items);
    }
    emit("refresh");
  } catch (error) {
    console.error("list reorder failed", error);
    sortableTodos.value = [...displayTodos.value];
    ElMessage.error(t("kanban.reorderFailed"));
  } finally {
    listReordering.value = false;
  }
}

function onPriorityFilterChange(v: string | null) {
  uiStore.setPriorityFilter(v);
  refresh();
}

function onCompletedFilterChange(v: boolean | null) {
  uiStore.setCompletedFilter(v);
  refresh();
}

function onHeaderSortChange(prop: string) {
  const field = TODO_SORT_PROP_MAP[prop];
  if (!field) return;

  const { field: currentField, direction: currentDirection } = parsedSort.value;
  if (currentField === field) {
    const nextDirection = currentDirection === "asc" ? "desc" : "asc";
    uiStore.setTodoSort(buildTodoSort(field as TodoSortField, nextDirection));
    return;
  }
  uiStore.setTodoSort(buildTodoSort(field as TodoSortField, "asc"));
}

defineExpose({ refresh });

onMounted(() => {
  void refresh();
});
</script>

<template>
  <section class="main-area">
    <div class="toolbar">
      <div class="toolbar-left">
        <el-input
          v-model="uiStore.searchQuery"
          class="search-input"
          :prefix-icon="Search"
          :placeholder="t('task.searchPlaceholder')"
          clearable
        />
        <el-radio-group
          :model-value="uiStore.taskViewMode"
          size="small"
          class="view-switch"
          :disabled="uiStore.viewMode === 'trash'"
          @change="uiStore.setTaskViewMode"
        >
          <el-radio-button value="list">{{ t("task.listView") }}</el-radio-button>
          <el-radio-button value="kanban">{{ t("task.kanbanView") }}</el-radio-button>
          <el-radio-button value="gantt">{{ t("task.ganttView") }}</el-radio-button>
        </el-radio-group>
      </div>
      <div class="toolbar-right">
        <el-select
          :model-value="uiStore.priorityFilter"
          :placeholder="t('priority.all')"
          clearable
          class="filter-select"
          @change="onPriorityFilterChange"
        >
          <el-option :label="t('priority.high')" value="high" />
          <el-option :label="t('priority.medium')" value="medium" />
          <el-option :label="t('priority.low')" value="low" />
        </el-select>
        <el-select
          :model-value="uiStore.completedFilter"
          :placeholder="t('status.all')"
          class="filter-select"
          @change="onCompletedFilterChange"
        >
          <el-option :label="t('common.all')" :value="null" />
          <el-option :label="t('status.incomplete')" :value="false" />
          <el-option :label="t('status.completed')" :value="true" />
        </el-select>
        <el-button
          v-if="isTrashMode"
          type="danger"
          :icon="Delete"
          :disabled="!displayTodos.length"
          @click="onEmptyTrash"
        >
          {{ t("task.emptyTrash") }}
        </el-button>
        <el-button v-else type="primary" :icon="Plus" @click="uiStore.openNewTaskDialog()">
          {{ t("task.newTask") }}
        </el-button>
      </div>
    </div>

    <QuickInputBar
      v-if="!isTrashMode"
      @refresh="emit('refresh')"
      @created="onQuickCreated"
    />

    <KanbanView
      v-if="uiStore.taskViewMode === 'kanban'"
      :todos="kanbanTodos"
      :loading="todoStore.loading"
      @refresh="emit('refresh')"
      @view="onView"
      @toggle="onToggle"
      @delete="onDelete"
    />

    <GanttView
      v-else-if="uiStore.taskViewMode === 'gantt'"
      :todos="ganttTodos"
      :loading="todoStore.loading"
      @refresh="emit('refresh')"
      @view="onView"
    />

    <div v-else class="table-wrap">
      <DraggableTaskList
        v-model="sortableTodos"
        :categories="categoryStore.categories"
        :tags="tagStore.tags"
        :loading="todoStore.loading"
        :is-trash-mode="isTrashMode"
        :can-drag="canDrag"
        :sort-field="parsedSort.field"
        :sort-direction="parsedSort.direction"
        :highlight-id="todoStore.highlightId"
        @sort-start="onListSortStart"
        @sort-end="onListSortEnd"
        @sort-change="onHeaderSortChange"
        @toggle="onToggle"
        @toggle-pin="onTogglePin"
        @view="onView"
        @edit="onEdit"
        @title-update="onTitleUpdate"
        @priority-update="onPriorityUpdate"
        @due-date-update="onDueDateUpdate"
        @tags-update="onTagsUpdate"
        @category-update="onCategoryUpdate"
        @delete="onDelete"
        @restore="onRestore"
      />
    </div>

    <NewTaskDialog
      :model-value="uiStore.newTaskDialogOpen"
      @update:model-value="(v) => { if (!v) uiStore.closeNewTaskDialog(); }"
      @created="emit('refresh')"
    />
  </section>
</template>

<style scoped>
.toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  padding: 16px 20px;
  background: var(--panel-bg);
  border-bottom: 1px solid var(--border-light);
  flex-shrink: 0;
}

.toolbar-left {
  display: flex;
  align-items: center;
  gap: 12px;
  flex: 1;
  min-width: 0;
}

.search-input {
  max-width: 280px;
}

.view-switch {
  flex-shrink: 0;
}

.toolbar-right {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-shrink: 0;
}

.filter-select {
  width: 120px;
}

.table-wrap {
  flex: 1;
  min-height: 0;
  overflow: auto;
  background: var(--panel-bg);
}
</style>
