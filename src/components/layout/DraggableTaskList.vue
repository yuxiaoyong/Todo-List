<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, provide, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { useDraggable } from "vue-draggable-plus";
import type { SortableEvent } from "sortablejs";
import TaskListColumnCells from "./TaskListColumnCells.vue";
import TaskListRowCells from "./TaskListRowCells.vue";
import TaskListColumnSettings from "./TaskListColumnSettings.vue";
import { TASK_LIST_CELL_CTX } from "./taskListCellContext";
import { useTaskListColumnStore } from "../../stores/taskListColumns";
import {
  buildZoneGridTemplate,
  getVisibleColumns,
  splitVisibleColumns,
  tableMinWidthPx,
  TASK_LIST_SETTINGS_WIDTH_PX,
  zoneWidthPx,
  type TaskListColumnItem,
  type TaskListColumnZone,
} from "../../utils/taskListColumns";
import { formatDateTimeCn } from "../../utils/formatDate";
import { formatRecurrenceSummary } from "../../utils/recurrence";
import type { TodoSortField } from "../../utils/sortTodos";
import type { Category, KanbanColumn, Tag, TodoSummary } from "../../types";

const props = withDefaults(
  defineProps<{
    modelValue: TodoSummary[];
    categories?: Category[];
    tags?: Tag[];
    kanbanColumns?: KanbanColumn[];
    loading?: boolean;
    isTrashMode?: boolean;
    canDrag?: boolean;
    sortField?: TodoSortField;
    sortDirection?: "asc" | "desc";
    highlightId?: number | null;
    minimal?: boolean;
  }>(),
  {
    categories: () => [],
    tags: () => [],
    kanbanColumns: () => [],
  },
);

const emit = defineEmits<{
  "update:modelValue": [TodoSummary[]];
  sortChange: [prop: string];
  sortStart: [];
  sortEnd: [evt: SortableEvent];
  toggle: [todo: TodoSummary];
  togglePin: [todo: TodoSummary];
  view: [id: number];
  edit: [id: number];
  titleUpdate: [todo: TodoSummary, title: string];
  priorityUpdate: [todo: TodoSummary, priority: string];
  dueDateUpdate: [todo: TodoSummary, dueDate: string | null];
  tagsUpdate: [todo: TodoSummary, tagIds: number[]];
  categoryUpdate: [todo: TodoSummary, categoryId: number | null];
  startDateUpdate: [todo: TodoSummary, startDate: string | null];
  kanbanColumnUpdate: [todo: TodoSummary, kanbanColumnId: number | null];
  delete: [todo: TodoSummary];
  restore: [todo: TodoSummary];
}>();

const { t, locale } = useI18n();
const columnStore = useTaskListColumnStore();
const scrollEl = ref<HTMLElement | null>(null);
const bodyEl = ref<HTMLElement | null>(null);
const scrollContainerWidth = ref(0);
const scrollShadow = ref({ left: false, right: false });
const titleOverflowMap = ref<Record<number, boolean>>({});
const editingTitleId = ref<number | null>(null);
const titleDrafts = ref<Record<number, string>>({});
const duePopoverId = ref<number | null>(null);
const startPopoverId = ref<number | null>(null);
const tagPopoverId = ref<number | null>(null);
const tagDrafts = ref<Record<number, number[]>>({});
const tagDraftSnapshots = ref<Record<number, number[]>>({});
let suppressRowClick = false;
let titleEditEscaping = false;
let dragStarted = false;
const isDragging = ref(false);

const list = computed({
  get: () => props.modelValue,
  set: (value) => emit("update:modelValue", value),
});

const priorityLabel = computed<Record<string, string>>(() => ({
  high: t("priority.high"),
  medium: t("priority.medium"),
  low: t("priority.low"),
}));

const displayColumns = computed(() =>
  getVisibleColumns(columnStore.columns, {
    minimal: props.minimal,
    isTrashMode: props.isTrashMode,
  }),
);

const isTrash = computed(() => props.isTrashMode ?? false);

const columnZones = computed(() => splitVisibleColumns(displayColumns.value));
const leftColumns = computed(() => columnZones.value.left);
const scrollColumns = computed(() => columnZones.value.scroll);
const rightColumns = computed(() => columnZones.value.right);

let scrollResizeObserver: ResizeObserver | null = null;

function zoneStyle(columns: TaskListColumnItem[], zone: TaskListColumnZone) {
  if (!columns.length) return { display: "none" };

  const minWidth = zoneWidthPx(columns, isTrash.value, zone);

  if (zone === "left" && columns.some((column) => column.id === "title")) {
    return {
      gridTemplateColumns: buildZoneGridTemplate(columns, isTrash.value, "title"),
      flex: "1 1 0",
      minWidth: `${minWidth}px`,
    };
  }

  if (zone === "scroll") {
    const flexColumnId = columns[columns.length - 1]!.id;
    return {
      gridTemplateColumns: buildZoneGridTemplate(columns, isTrash.value, flexColumnId),
      flex: "1 1 0",
      minWidth: `${minWidth}px`,
    };
  }

  return {
    gridTemplateColumns: buildZoneGridTemplate(columns, isTrash.value),
    flex: "0 0 auto",
    width: `${minWidth}px`,
  };
}

const leftZoneStyle = computed(() => zoneStyle(leftColumns.value, "left"));
const scrollZoneStyle = computed(() => zoneStyle(scrollColumns.value, "scroll"));
const rightZoneStyle = computed(() => zoneStyle(rightColumns.value, "right"));

const tableMinWidth = computed(() =>
  tableMinWidthPx(
    leftColumns.value,
    scrollColumns.value,
    rightColumns.value,
    isTrash.value,
  ),
);

const layoutWidth = computed(() =>
  Math.max(scrollContainerWidth.value, tableMinWidth.value),
);

const scrollInnerStyle = computed(() => ({
  width: layoutWidth.value > 0 ? `${layoutWidth.value}px` : "100%",
}));

function updateScrollMetrics() {
  const el = scrollEl.value;
  if (!el) return;
  scrollContainerWidth.value = el.clientWidth;
  scrollShadow.value = {
    left: el.scrollLeft > 1,
    right: el.scrollLeft + el.clientWidth < el.scrollWidth - 1,
  };
}

function setupScrollObserver() {
  scrollResizeObserver?.disconnect();
  scrollResizeObserver = null;
  const el = scrollEl.value;
  if (!el || typeof ResizeObserver === "undefined") {
    updateScrollMetrics();
    return;
  }
  scrollResizeObserver = new ResizeObserver(() => updateScrollMetrics());
  scrollResizeObserver.observe(el);
  updateScrollMetrics();
}

const minimalGridStyle = computed(() => ({
  gridTemplateColumns: buildZoneGridTemplate(displayColumns.value, isTrash.value),
}));

function onBodyScroll() {
  updateScrollMetrics();
}

const dragOptions = computed(() => ({
  immediate: false,
  animation: 200,
  easing: "cubic-bezier(0.2, 0, 0, 1)",
  draggable: ".task-list-row-bar",
  filter: ".no-drag, .title-input",
  preventOnFilter: false,
  delay: 120,
  ghostClass: "list-drag-ghost",
  chosenClass: "list-drag-chosen",
  dragClass: "list-drag-dragging",
  forceFallback: true,
  fallbackOnBody: true,
  disabled: !props.canDrag,
  onStart() {
    dragStarted = true;
    isDragging.value = true;
    document.body.style.userSelect = "none";
    document.body.style.webkitUserSelect = "none";
    emit("sortStart");
  },
  onEnd(evt: SortableEvent) {
    suppressRowClick = dragStarted || evt.oldIndex !== evt.newIndex;
    document.body.style.userSelect = "";
    document.body.style.webkitUserSelect = "";
    document
      .querySelectorAll("body > .sortable-fallback, body > .list-drag-dragging")
      .forEach((node) => node.remove());
    emit("sortEnd", evt);
    isDragging.value = false;
    window.setTimeout(() => {
      suppressRowClick = false;
      dragStarted = false;
    }, 200);
  },
}));

const draggable = useDraggable(bodyEl, list, dragOptions);

async function initDraggable() {
  await nextTick();
  if (!bodyEl.value) return;
  draggable.start(bodyEl.value);
  if (props.canDrag) {
    draggable.resume?.();
  } else {
    draggable.pause?.();
  }
}

onMounted(() => {
  void columnStore.init();
  void initDraggable();
  void nextTick(() => setupScrollObserver());
});

watch(displayColumns, () => {
  void nextTick(() => updateScrollMetrics());
});

watch(
  () => props.modelValue.length,
  (len, prevLen) => {
    if (len > 0 && prevLen === 0) {
      void initDraggable();
    }
  },
);

watch(
  () => props.canDrag,
  (enabled) => {
    if (enabled) draggable.resume?.();
    else draggable.pause?.();
  },
);

watch(
  () => props.modelValue,
  () => {
    titleDrafts.value = {};
    editingTitleId.value = null;
    duePopoverId.value = null;
    startPopoverId.value = null;
    tagPopoverId.value = null;
    tagDrafts.value = {};
    tagDraftSnapshots.value = {};
  },
);

onUnmounted(() => {
  scrollResizeObserver?.disconnect();
  scrollResizeObserver = null;
  document.body.style.userSelect = "";
  document.body.style.webkitUserSelect = "";
  draggable.destroy?.();
});

function formatDueDate(date?: string) {
  if (!date) return t("common.dash");
  return date.length <= 10 ? date : date.slice(0, 16).replace("T", " ");
}

function priorityClass(priority: string) {
  if (priority === "high") return "priority-high";
  if (priority === "medium") return "priority-medium";
  return "priority-none";
}

function rowClasses(todo: TodoSummary) {
  const classes = ["task-list-row-bar"];
  if (props.isTrashMode) classes.push("task-list-row-bar--trash");
  if (todo.completed) classes.push("row-done");
  if (todo.pinned) classes.push("row-pinned");
  if (todo.id === props.highlightId) classes.push("row-highlight");
  if (isTitleEditing(todo.id)) classes.push("row-title-editing");
  return classes;
}

function sortIndicator(prop: string) {
  const fieldMap: Record<string, TodoSortField> = {
    title: "title",
    priority: "priority",
    startDate: "startDate",
    dueDate: "dueDate",
    createdAt: "createdAt",
    updatedAt: "updatedAt",
    assignee: "assignee",
  };
  if (props.sortField !== fieldMap[prop]) return "";
  return props.sortDirection === "asc" ? "↑" : "↓";
}

function onHeaderClick(prop: string) {
  emit("sortChange", prop);
}

function onRowClick(todo: TodoSummary, event: MouseEvent) {
  if (suppressRowClick || isDragging.value || editingTitleId.value === todo.id) return;
  const target = event.target as HTMLElement;
  if (target.closest(".no-drag, .title-input, .title-edit-btn, .col-priority, .col-status, .col-start, .col-due, .col-category, .col-tags, .col-kanban")) return;
  emit("view", todo.id);
}

function onPriorityChange(todo: TodoSummary, priority: string) {
  if (priority === todo.priority) return;
  emit("priorityUpdate", todo, priority);
}

function pillStyleFromColor(color: string) {
  return {
    color,
    borderColor: `color-mix(in srgb, ${color} 28%, transparent)`,
    backgroundColor: `color-mix(in srgb, ${color} 12%, var(--panel-bg))`,
  };
}

function categoryPillStyle(todo: TodoSummary) {
  const color = todo.categoryColor;
  if (!color) return undefined;
  return pillStyleFromColor(color);
}

function onCategoryChange(todo: TodoSummary, command: number | "none") {
  const next = command === "none" ? null : command;
  const current = todo.categoryId ?? null;
  if (next === current) return;
  emit("categoryUpdate", todo, next);
}

function dueDateValue(todo: TodoSummary) {
  if (!todo.dueDate) return undefined;
  return todo.dueDate.length > 10 ? todo.dueDate.slice(0, 10) : todo.dueDate;
}

function onDuePopoverVisible(todoId: number, visible: boolean) {
  duePopoverId.value = visible ? todoId : null;
}

function onDueDateChange(todo: TodoSummary, value: string | undefined) {
  const next = value ?? null;
  const current = dueDateValue(todo) ?? null;
  duePopoverId.value = null;
  if (next === current) return;
  emit("dueDateUpdate", todo, next);
}

function startDateValue(todo: TodoSummary) {
  if (!todo.startDate) return undefined;
  return todo.startDate.length > 10 ? todo.startDate.slice(0, 10) : todo.startDate;
}

function onStartPopoverVisible(todoId: number, visible: boolean) {
  startPopoverId.value = visible ? todoId : null;
}

function onStartDateChange(todo: TodoSummary, value: string | undefined) {
  const next = value ?? null;
  const current = startDateValue(todo) ?? null;
  startPopoverId.value = null;
  if (next === current) return;
  emit("startDateUpdate", todo, next);
}

function statusLabel(todo: TodoSummary) {
  return todo.completed ? t("status.completed") : t("status.incomplete");
}

function assigneeLabel(todo: TodoSummary) {
  const value = todo.assignee?.trim();
  return value || t("taskDetail.assigneeDefault");
}

function kanbanPillStyle(todo: TodoSummary) {
  const color = todo.kanbanColumnColor;
  if (!color) return undefined;
  return pillStyleFromColor(color);
}

function onKanbanColumnChange(todo: TodoSummary, command: number | "none") {
  const next = command === "none" ? null : command;
  const current = todo.kanbanColumnId ?? null;
  if (next === current) return;
  emit("kanbanColumnUpdate", todo, next);
}

function recurrenceLabel(todo: TodoSummary) {
  if (!todo.recurrenceJson?.enabled) return t("common.dash");
  return formatRecurrenceSummary(
    todo.recurrenceJson,
    { startDate: todo.startDate, dueDate: todo.dueDate },
    t,
    locale.value,
  );
}

function sameTagIds(a: number[], b: number[]) {
  if (a.length !== b.length) return false;
  const left = [...a].sort((x, y) => x - y);
  const right = [...b].sort((x, y) => x - y);
  return left.every((value, index) => value === right[index]);
}

function tagPillStyle(todo: TodoSummary, index: number) {
  const color = todo.tagColors[index];
  if (!color) return undefined;
  return pillStyleFromColor(color);
}

function onTagPopoverVisible(todo: TodoSummary, visible: boolean) {
  if (visible) {
    tagPopoverId.value = todo.id;
    tagDrafts.value[todo.id] = [...todo.tagIds];
    tagDraftSnapshots.value[todo.id] = [...todo.tagIds];
    return;
  }
  if (tagPopoverId.value === todo.id) {
    flushTagEdit(todo);
    tagPopoverId.value = null;
  }
}

function flushTagEdit(todo: TodoSummary) {
  const draft = tagDrafts.value[todo.id];
  const snapshot = tagDraftSnapshots.value[todo.id];
  if (!draft || !snapshot) return;
  delete tagDrafts.value[todo.id];
  delete tagDraftSnapshots.value[todo.id];
  if (sameTagIds(draft, snapshot)) return;
  emit("tagsUpdate", todo, [...draft]);
}

function isTitleEditing(todoId: number) {
  return editingTitleId.value === todoId;
}

function displayTitle(todo: TodoSummary) {
  return titleDrafts.value[todo.id] ?? todo.title;
}

function startTitleEdit(todo: TodoSummary) {
  if (props.isTrashMode) return;
  editingTitleId.value = todo.id;
  titleDrafts.value[todo.id] = todo.title;
  void nextTick(() => {
    if (!isTitleEditing(todo.id)) return;
    const input = bodyEl.value?.querySelector(
      `[data-title-input="${todo.id}"]`,
    ) as HTMLInputElement | null;
    input?.focus();
    input?.select();
  });
}

function cancelTitleEdit() {
  if (editingTitleId.value == null) return;
  delete titleDrafts.value[editingTitleId.value];
  editingTitleId.value = null;
}

function commitTitleEdit(todo: TodoSummary) {
  if (!isTitleEditing(todo.id)) return;
  const value = (titleDrafts.value[todo.id] ?? todo.title).trim();
  delete titleDrafts.value[todo.id];
  editingTitleId.value = null;
  if (!value || value === todo.title) return;
  emit("titleUpdate", todo, value);
}

function onTitleInput(todo: TodoSummary, event: Event) {
  titleDrafts.value[todo.id] = (event.target as HTMLInputElement).value;
}

function onTitleKeydown(todo: TodoSummary, event: KeyboardEvent) {
  if (event.key === "Enter") {
    event.preventDefault();
    commitTitleEdit(todo);
  } else if (event.key === "Escape") {
    event.preventDefault();
    titleEditEscaping = true;
    cancelTitleEdit();
  }
}

function onTitleBlur() {
  if (titleEditEscaping) {
    titleEditEscaping = false;
    return;
  }
  cancelTitleEdit();
}

function checkTitleOverflow(event: MouseEvent, id: number) {
  const el = event.currentTarget as HTMLElement;
  titleOverflowMap.value[id] = el.scrollWidth > el.clientWidth;
}

provide(TASK_LIST_CELL_CTX, {
  isTrashMode: computed(() => isTrash.value),
  minimal: computed(() => props.minimal ?? false),
  categories: computed(() => props.categories),
  tags: computed(() => props.tags),
  kanbanColumns: computed(() => props.kanbanColumns),
  sortField: computed(() => props.sortField),
  sortDirection: computed(() => props.sortDirection),
  highlightId: computed(() => props.highlightId ?? null),
  duePopoverId,
  startPopoverId,
  tagPopoverId,
  tagDrafts,
  titleOverflowMap,
  editingTitleId,
  titleDrafts,
  priorityLabel,
  onHeaderClick,
  sortIndicator,
  onRowClick,
  onToggle: (todo) => emit("toggle", todo),
  onTogglePin: (todo) => emit("togglePin", todo),
  onView: (id) => emit("view", id),
  onEdit: (id) => emit("edit", id),
  onDelete: (todo) => emit("delete", todo),
  onRestore: (todo) => emit("restore", todo),
  onTitleUpdate: (todo, title) => emit("titleUpdate", todo, title),
  onPriorityUpdate: (todo, priority) => emit("priorityUpdate", todo, priority),
  onDueDateUpdate: (todo, dueDate) => emit("dueDateUpdate", todo, dueDate),
  onStartDateUpdate: (todo, startDate) => emit("startDateUpdate", todo, startDate),
  onTagsUpdate: (todo, tagIds) => emit("tagsUpdate", todo, tagIds),
  onCategoryUpdate: (todo, categoryId) => emit("categoryUpdate", todo, categoryId),
  onKanbanColumnUpdate: (todo, kanbanColumnId) => emit("kanbanColumnUpdate", todo, kanbanColumnId),
  formatDueDate,
  dueDateValue,
  startDateValue,
  onDuePopoverVisible,
  onStartPopoverVisible,
  onDueDateChange,
  onStartDateChange,
  onTagPopoverVisible,
  onPriorityChange,
  onCategoryChange,
  onKanbanColumnChange,
  priorityClass,
  categoryPillStyle,
  kanbanPillStyle,
  tagPillStyle,
  statusLabel,
  assigneeLabel,
  recurrenceLabel,
  formatDateTimeCn,
  isTitleEditing,
  displayTitle,
  startTitleEdit,
  onTitleInput,
  onTitleKeydown,
  onTitleBlur,
  checkTitleOverflow,
});
</script>

<template>
  <div
    v-loading="loading"
    class="task-list"
    :class="{
      'is-drag-disabled': !canDrag,
      'is-dragging': isDragging,
      'task-list--minimal': minimal,
    }"
  >
    <template v-if="minimal">
      <div class="task-list-header-bar task-list-header-bar--minimal">
        <div class="task-list-header task-list-header--minimal" :style="minimalGridStyle">
          <TaskListColumnCells :columns="displayColumns" />
        </div>
      </div>
      <div ref="bodyEl" class="task-list-body">
        <div v-if="!modelValue.length" class="empty-state">
          {{ isTrashMode ? t("task.trashEmpty") : t("task.noTasks") }}
        </div>
        <div
          v-for="todo in modelValue"
          v-else
          :key="todo.id"
          :class="rowClasses(todo)"
          :data-todo-id="todo.id"
          @click="onRowClick(todo, $event)"
        >
          <div class="task-list-row task-list-row--minimal" :style="minimalGridStyle">
            <TaskListRowCells :columns="displayColumns" :todo="todo" />
          </div>
        </div>
      </div>
    </template>

    <div
      v-else
      class="task-list-frame task-list-frame--has-settings"
      :style="{
        '--list-settings-width': `${TASK_LIST_SETTINGS_WIDTH_PX}px`,
      }"
    >
      <div ref="scrollEl" class="task-list-scroll" @scroll="onBodyScroll">
        <div class="task-list-scroll-inner" :style="scrollInnerStyle">
          <div class="task-list-header-bar task-list-header-bar--has-settings">
            <div class="task-list-header-row">
              <div
                v-if="leftColumns.length"
                class="task-list-zone task-list-zone--left task-list-zone--header"
                :class="{ 'is-shadow': scrollShadow.left }"
                :style="leftZoneStyle"
              >
                <TaskListColumnCells :columns="leftColumns" />
              </div>
              <div
                v-if="scrollColumns.length"
                class="task-list-zone task-list-zone--scroll task-list-zone--header"
                :style="scrollZoneStyle"
              >
                <TaskListColumnCells :columns="scrollColumns" />
              </div>
              <div
                v-if="rightColumns.length"
                class="task-list-zone task-list-zone--right task-list-zone--header"
                :class="{ 'is-shadow': scrollShadow.right }"
                :style="rightZoneStyle"
              >
                <TaskListColumnCells :columns="rightColumns" />
              </div>
            </div>
            <div
              class="col-settings"
              :class="{ 'is-shadow': scrollShadow.right }"
            >
              <TaskListColumnSettings />
            </div>
          </div>

          <div ref="bodyEl" class="task-list-body">
            <div v-if="!modelValue.length" class="empty-state">
              {{ isTrashMode ? t("task.trashEmpty") : t("task.noTasks") }}
            </div>
            <div
              v-for="todo in modelValue"
              v-else
              :key="todo.id"
              :class="rowClasses(todo)"
              :data-todo-id="todo.id"
              @click="onRowClick(todo, $event)"
            >
              <div class="task-list-row">
                <div
                  v-if="leftColumns.length"
                  class="task-list-zone task-list-zone--left"
                  :class="{ 'is-shadow': scrollShadow.left }"
                  :style="leftZoneStyle"
                >
                  <TaskListRowCells :columns="leftColumns" :todo="todo" />
                </div>
                <div
                  v-if="scrollColumns.length"
                  class="task-list-zone task-list-zone--scroll"
                  :style="scrollZoneStyle"
                >
                  <TaskListRowCells :columns="scrollColumns" :todo="todo" />
                </div>
                <div
                  v-if="rightColumns.length"
                  class="task-list-zone task-list-zone--right"
                  :class="{ 'is-shadow': scrollShadow.right }"
                  :style="rightZoneStyle"
                >
                  <TaskListRowCells :columns="rightColumns" :todo="todo" />
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>


<style scoped>
.task-list {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-height: 0;
  width: 100%;
}

.task-list.is-dragging,
.task-list.is-dragging * {
  user-select: none;
  -webkit-user-select: none;
}

.list-drag-chosen,
.list-drag-dragging,
.list-drag-ghost {
  user-select: none;
  -webkit-user-select: none;
}

.task-list-frame {
  position: relative;
  flex: 1;
  min-height: 0;
  min-width: 0;
  display: flex;
  flex-direction: column;
}

.task-list-header-bar {
  position: sticky;
  top: 0;
  z-index: 4;
  display: grid;
  grid-template-areas: "header-main";
  align-items: stretch;
  width: 100%;
  min-height: 44px;
  flex-shrink: 0;
  border-bottom: 1px solid var(--border-light);
  background: var(--surface-muted);
  color: var(--text-secondary);
  font-size: 13px;
  font-weight: 500;
}

.task-list-frame--has-settings {
  --list-settings-width: 28px;
}

.task-list-header-row {
  grid-area: header-main;
  display: flex;
  align-items: stretch;
  width: 100%;
  min-width: 0;
  height: 44px;
  text-align: left;
}

.col-settings {
  grid-area: header-main;
  position: sticky;
  right: 20px;
  justify-self: end;
  align-self: center;
  flex-shrink: 0;
  width: var(--list-settings-width);
  height: 44px;
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 5;
  background: var(--surface-muted);
  pointer-events: auto;
}

.task-list-zone--right {
  padding-left: 0;
  padding-right: 12px;
  right: 0;
}

.task-list-scroll {
  flex: 1;
  min-height: 0;
  min-width: 0;
  overflow: auto;
  background: var(--panel-bg);
}

.task-list-scroll-inner {
  display: flex;
  flex-direction: column;
  min-height: 100%;
  box-sizing: border-box;
}

.task-list-zone {
  display: grid;
  align-items: center;
  column-gap: 8px;
  box-sizing: border-box;
}

.task-list-zone--left {
  padding-left: 12px;
  padding-right: 0;
}

.task-list-zone--scroll {
  padding: 0;
}

.task-list-zone--header {
  height: 44px;
  justify-items: start;
}

.task-list-zone--left,
.task-list-zone--right {
  position: sticky;
  z-index: 3;
}

.task-list-row-bar .task-list-zone--left,
.task-list-row-bar .task-list-zone--right {
  z-index: 2;
}

.task-list-row-bar .task-list-zone--scroll {
  z-index: 1;
}

.task-list-header-row .task-list-zone--left,
.task-list-header-row .task-list-zone--right,
.task-list-header-row .task-list-zone--scroll {
  background: var(--surface-muted);
}

.task-list-zone--left {
  left: 0;
}

.task-list-header-row .col-settings.is-shadow {
  box-shadow: -4px 0 8px -4px color-mix(in srgb, var(--text-primary) 16%, transparent);
}

.task-list-zone--left.is-shadow {
  box-shadow: 4px 0 8px -4px color-mix(in srgb, var(--text-primary) 16%, transparent);
}

.task-list-zone--right.is-shadow {
  box-shadow: -4px 0 8px -4px color-mix(in srgb, var(--text-primary) 16%, transparent);
}

.task-list-header {
  flex: 1;
  min-width: 0;
  display: grid;
  align-items: center;
  justify-items: start;
  column-gap: 8px;
  padding: 0 12px;
  height: 44px;
  color: var(--text-secondary);
  font-size: 13px;
  font-weight: 500;
  text-align: left;
}

.task-list-header--minimal,
.task-list-row--minimal {
  width: 100%;
}

.task-list-row {
  display: flex;
  align-items: stretch;
  width: 100%;
  min-width: 0;
  min-height: 52px;
}

.task-list-row-bar {
  width: 100%;
  border-bottom: 1px solid var(--border-light);
  background: var(--panel-bg);
  cursor: pointer;
  transition: background 0.15s;
  box-sizing: border-box;
}

.task-list-row-bar .task-list-zone {
  background: var(--panel-bg);
}

.task-list-row-bar:nth-child(even) {
  background: var(--table-stripe);
}

.task-list-row-bar:nth-child(even) .task-list-zone {
  background: var(--table-stripe);
}

.task-list-row-bar:hover {
  background: var(--nav-hover);
}

.task-list-row-bar:hover .task-list-zone {
  background: var(--nav-hover);
}

.task-list-row-bar.row-pinned {
  background: var(--pin-row-bg);
}

.task-list-row-bar.row-pinned .task-list-zone {
  background: var(--pin-row-bg);
}

.task-list-row-bar.row-title-editing {
  background: var(--nav-hover);
}

.task-list-row-bar.row-title-editing .task-list-zone {
  background: var(--nav-hover);
}

.task-list-row-bar.row-highlight,
.task-list-row-bar.row-highlight .task-list-zone {
  animation: row-highlight-fade 2s ease-out forwards;
}

.sortable {
  border: none;
  background: transparent;
  padding: 0;
  text-align: inherit;
  color: inherit;
  font: inherit;
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  gap: 4px;
}

.task-list :deep(.sortable) {
  border: none;
  background: transparent;
  padding: 0;
  text-align: inherit;
  color: inherit;
  font: inherit;
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  gap: 4px;
  max-width: 100%;
  min-width: 0;
}

.sortable:hover,
.task-list :deep(.sortable:hover) {
  color: var(--primary);
}

.sort-mark,
.task-list :deep(.sort-mark) {
  color: var(--primary);
  font-size: 12px;
  flex-shrink: 0;
}

.task-list-body {
  display: flex;
  flex-direction: column;
  min-height: 48px;
  width: 100%;
}

.task-list-body .empty-state {
  padding: 64px 24px;
  text-align: center;
  color: var(--text-secondary);
}

.task-list:not(.is-drag-disabled) .task-list-row-bar {
  cursor: grab;
}

.task-list-row-bar.list-drag-chosen,
.task-list-row-bar.list-drag-dragging {
  cursor: grabbing;
}

.task-list :deep(.task-title) {
  display: block;
  width: 100%;
  font-weight: 500;
  color: var(--text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.task-list :deep(.task-title.done) {
  text-decoration: line-through;
  color: var(--text-secondary);
}

.task-list-row-bar.row-done :deep(.task-title),
.task-list-row-bar.row-done :deep(.title-input) {
  opacity: 0.72;
}

@keyframes row-highlight-fade {
  0% {
    background: color-mix(in srgb, var(--primary) 18%, var(--panel-bg));
  }
  100% {
    background: var(--panel-bg);
  }
}

.task-list-row-bar:nth-child(even).row-highlight,
.task-list-row-bar:nth-child(even).row-highlight .task-list-zone {
  animation-name: row-highlight-fade-even;
}

@keyframes row-highlight-fade-even {
  0% {
    background: color-mix(in srgb, var(--primary) 18%, var(--table-stripe));
  }
  100% {
    background: var(--table-stripe);
  }
}

.pin-btn,
.task-list :deep(.pin-btn) {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border: none;
  outline: none;
  box-shadow: none;
  background: transparent;
  color: var(--text-tertiary);
  cursor: pointer;
  border-radius: 6px;
  padding: 0;
  opacity: 0;
  transition: opacity 0.15s, color 0.15s, background 0.15s;
  -webkit-appearance: none;
  appearance: none;
}

.task-list :deep(.pin-btn:focus),
.task-list :deep(.pin-btn:focus-visible),
.task-list :deep(.pin-btn:active) {
  border: none;
  outline: none;
  box-shadow: none;
}

.task-list-row-bar:hover :deep(.pin-btn),
.task-list :deep(.pin-btn.active) {
  opacity: 1;
}

.task-list :deep(.pin-btn:hover),
.task-list :deep(.pin-btn.active) {
  color: var(--pin-color);
  background: transparent;
}

.col-title {
  min-width: 0;
  overflow: hidden;
}

.task-list :deep(.col-title) {
  display: flex;
  align-items: center;
  width: 100%;
  min-width: 0;
  padding-right: 20px;
  box-sizing: border-box;
  overflow: hidden;
}

.task-list :deep(.title-cell) {
  display: flex;
  align-items: center;
  gap: 4px;
  width: 100%;
  min-width: 0;
}

.col-title :deep(.el-tooltip__trigger) {
  display: block;
  flex: 1 1 auto;
  min-width: 0;
}

.task-list :deep(.title-edit-btn) {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  width: 24px;
  height: 24px;
  margin-left: auto;
  border: none;
  outline: none;
  box-shadow: none;
  background: transparent;
  color: var(--text-tertiary);
  cursor: pointer;
  border-radius: 6px;
  padding: 0;
  opacity: 0;
  transition: opacity 0.15s, color 0.15s, background 0.15s;
  -webkit-appearance: none;
  appearance: none;
}

.task-list :deep(.col-title:hover .title-edit-btn),
.task-list-row-bar.row-title-editing :deep(.title-edit-btn) {
  opacity: 1;
}

.task-list :deep(.title-edit-btn:hover) {
  color: var(--primary);
  background: var(--nav-hover);
}

.task-list :deep(.title-input) {
  display: block;
  width: 100%;
  box-sizing: border-box;
  border: 1px solid var(--border-color);
  border-radius: 6px;
  background: var(--panel-bg);
  padding: 4px 8px;
  font-size: 14px;
  font-weight: 500;
  color: var(--text-primary);
  outline: none;
}

.task-list :deep(.title-input:focus) {
  border-color: var(--primary);
}

.task-list :deep(.title-input.done) {
  text-decoration: line-through;
  color: var(--text-secondary);
}

.col-priority {
  display: flex;
  align-items: center;
}

.col-priority :deep(.el-dropdown) {
  display: flex;
  max-width: 100%;
}

.priority-pill {
  display: inline-block;
  padding: 2px 10px;
  border-radius: 12px;
  font-size: 12px;
  font-weight: 500;
}

.priority-cell-trigger {
  cursor: pointer;
  transition: opacity 0.15s, box-shadow 0.15s;
}

.priority-cell-trigger:hover {
  box-shadow: 0 0 0 1px color-mix(in srgb, var(--text-tertiary) 35%, transparent);
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

.col-due {
  display: flex;
  align-items: center;
}

.col-start {
  display: flex;
  align-items: center;
}

.col-status,
.col-assignee,
.col-created,
.col-updated,
.col-recurrence {
  display: flex;
  align-items: center;
  min-width: 0;
}

.status-pill {
  display: inline-block;
  padding: 2px 10px;
  border-radius: 12px;
  font-size: 12px;
  font-weight: 500;
}

.status-pill--done {
  background: color-mix(in srgb, var(--primary) 12%, transparent);
  color: var(--primary);
}

.status-pill--open {
  background: var(--surface-muted);
  color: var(--text-secondary);
}

.meta-text {
  font-size: 13px;
  color: var(--text-secondary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.meta-text--ellipsis {
  display: block;
  max-width: 100%;
}

.col-kanban {
  display: flex;
  align-items: center;
  min-width: 0;
}

.col-kanban :deep(.el-dropdown) {
  display: flex;
  max-width: 100%;
}

.due-cell-trigger {
  display: inline-flex;
  align-items: center;
  max-width: 100%;
  cursor: pointer;
  border-radius: 6px;
  padding: 2px 4px;
  transition: box-shadow 0.15s, background 0.15s;
}

.due-cell-trigger:hover {
  background: var(--nav-hover);
  box-shadow: 0 0 0 1px color-mix(in srgb, var(--text-tertiary) 35%, transparent);
}

.due-date {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  font-size: 13px;
  color: var(--text-secondary);
}

.due-empty {
  color: var(--text-tertiary);
  font-size: 13px;
}

.due-clear-btn {
  display: block;
  width: 100%;
  margin-top: 8px;
  padding: 6px 0;
  border: none;
  border-top: 1px solid var(--border-light);
  background: transparent;
  color: var(--text-secondary);
  font-size: 13px;
  cursor: pointer;
  text-align: center;
}

.due-clear-btn:hover {
  color: var(--primary);
}

.col-category {
  display: flex;
  align-items: center;
  min-width: 0;
}

.col-category :deep(.el-dropdown) {
  display: flex;
  max-width: 100%;
}

.task-list :deep(.category-pill),
.task-list :deep(.tag-pill) {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  max-width: 100%;
  height: 22px;
  padding: 0 9px;
  border-radius: var(--el-tag-border-radius, 4px);
  border: 1px solid var(--el-tag-border-color, var(--tag-border));
  background-color: var(--el-tag-bg-color, var(--tag-bg));
  color: var(--el-tag-text-color, var(--tag-text));
  font-size: 12px;
  line-height: 1;
  font-weight: 400;
  box-sizing: border-box;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  vertical-align: middle;
  transition: opacity 0.15s, border-color 0.15s, background-color 0.15s;
}

.task-list :deep(.category-pill--empty) {
  color: var(--el-text-color-placeholder, var(--text-tertiary));
  background-color: var(--el-fill-color-blank, var(--panel-bg));
  border-color: var(--el-border-color-lighter, var(--border-light));
}

.task-list :deep(.category-cell-trigger) {
  cursor: pointer;
}

.task-list :deep(.category-cell-trigger:hover),
.task-list :deep(.tag-cell-trigger:hover .tag-pill) {
  opacity: 0.88;
}

.col-tags {
  display: flex;
  align-items: center;
  min-width: 0;
}

.task-list :deep(.tag-cell-trigger) {
  display: inline-flex;
  align-items: center;
  max-width: 100%;
  min-width: 0;
  cursor: pointer;
}

.task-list :deep(.tag-list) {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
  min-width: 0;
}

.row-actions {
  display: flex;
  justify-content: center;
  gap: 2px;
  width: 100%;
}

.task-list-row-bar :deep(.col-actions) {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 100%;
  min-width: 0;
}

.task-list-row-bar :deep(.row-actions) {
  display: flex;
  justify-content: center;
  gap: 2px;
  width: 100%;
}

:deep(.list-drag-ghost) {
  opacity: 0.45;
  background: color-mix(in srgb, var(--primary) 10%, var(--panel-bg));
  box-shadow: var(--shadow-card);
}

:deep(.list-drag-dragging) {
  opacity: 0.92;
  box-shadow: var(--shadow-card);
}
</style>
