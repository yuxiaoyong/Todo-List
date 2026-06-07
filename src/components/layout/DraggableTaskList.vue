<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import {
  Calendar,
  Delete,
  Edit,
  RefreshLeft,
  Star,
  StarFilled,
  View,
} from "@element-plus/icons-vue";
import { useDraggable } from "vue-draggable-plus";
import type { SortableEvent } from "sortablejs";
import type { TodoSortField } from "../../utils/sortTodos";
import type { Category, Tag, TodoSummary } from "../../types";

const props = withDefaults(
  defineProps<{
    modelValue: TodoSummary[];
    categories?: Category[];
    tags?: Tag[];
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
  delete: [todo: TodoSummary];
  restore: [todo: TodoSummary];
}>();

const { t } = useI18n();
const bodyEl = ref<HTMLElement | null>(null);
const titleOverflowMap = ref<Record<number, boolean>>({});
const editingTitleId = ref<number | null>(null);
const titleDrafts = ref<Record<number, string>>({});
const duePopoverId = ref<number | null>(null);
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

const dragOptions = computed(() => ({
  immediate: false,
  animation: 200,
  easing: "cubic-bezier(0.2, 0, 0, 1)",
  draggable: ".task-list-row",
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
  void initDraggable();
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
    tagPopoverId.value = null;
    tagDrafts.value = {};
    tagDraftSnapshots.value = {};
  },
);

onUnmounted(() => {
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
  const classes = ["task-list-row"];
  if (props.isTrashMode) classes.push("task-list-row--trash");
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
    dueDate: "dueDate",
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
  if (target.closest(".no-drag, .title-input, .title-edit-btn, .col-priority, .col-due, .col-category, .col-tags")) return;
  emit("view", todo.id);
}

function onPriorityChange(todo: TodoSummary, priority: string) {
  if (priority === todo.priority) return;
  emit("priorityUpdate", todo, priority);
}

function categoryPillStyle(todo: TodoSummary) {
  const color = todo.categoryColor;
  if (!color) return undefined;
  return {
    color,
    borderColor: color,
    background: `color-mix(in srgb, ${color} 12%, transparent)`,
  };
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

function sameTagIds(a: number[], b: number[]) {
  if (a.length !== b.length) return false;
  const left = [...a].sort((x, y) => x - y);
  const right = [...b].sort((x, y) => x - y);
  return left.every((value, index) => value === right[index]);
}

function tagPillStyle(todo: TodoSummary, index: number) {
  const color = todo.tagColors[index];
  if (!color) return undefined;
  return {
    color,
    borderColor: color,
    background: `color-mix(in srgb, ${color} 12%, transparent)`,
  };
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
    <div
      class="task-list-header"
      :class="{ 'task-list-header--trash': isTrashMode }"
    >
      <span v-if="!isTrashMode" class="col-pin" />
      <span class="col-check" />
      <button type="button" class="col-title sortable" @click="onHeaderClick('title')">
        {{ t("task.taskName") }}
        <span class="sort-mark">{{ sortIndicator("title") }}</span>
      </button>
      <template v-if="!minimal">
        <button type="button" class="col-priority sortable" @click="onHeaderClick('priority')">
          {{ t("task.priority") }}
          <span class="sort-mark">{{ sortIndicator("priority") }}</span>
        </button>
        <button type="button" class="col-due sortable" @click="onHeaderClick('dueDate')">
          {{ t("task.dueDate") }}
          <span class="sort-mark">{{ sortIndicator("dueDate") }}</span>
        </button>
        <span class="col-category">{{ t("task.category") }}</span>
        <span class="col-tags">{{ t("task.tags") }}</span>
        <span class="col-actions">{{ t("task.actions") }}</span>
      </template>
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
        <div v-if="!isTrashMode" class="col-pin">
          <button
            type="button"
            class="pin-btn no-drag"
            :class="{ active: todo.pinned }"
            :title="todo.pinned ? t('task.unpin') : t('task.pin')"
            @click.stop="emit('togglePin', todo)"
          >
            <el-icon>
              <StarFilled v-if="todo.pinned" />
              <Star v-else />
            </el-icon>
          </button>
        </div>

        <div class="col-check">
          <el-checkbox
            class="no-drag"
            :model-value="todo.completed"
            :disabled="isTrashMode"
            @click.stop
            @change="emit('toggle', todo)"
          />
        </div>

        <div class="col-title">
          <input
            v-if="!isTrashMode && isTitleEditing(todo.id)"
            :data-title-input="todo.id"
            class="title-input no-drag"
            :class="{ done: todo.completed }"
            :value="displayTitle(todo)"
            :placeholder="t('taskDetail.titlePlaceholder')"
            @input="onTitleInput(todo, $event)"
            @keydown="onTitleKeydown(todo, $event)"
            @blur="onTitleBlur()"
            @click.stop
            @dblclick.stop
            @mousedown.stop
            @pointerdown.stop
          />
          <div v-else class="title-cell">
            <el-tooltip
              :content="todo.title"
              placement="top"
              :show-after="300"
              teleported
              :disabled="!titleOverflowMap[todo.id]"
            >
              <span
                class="task-title"
                :class="{ done: todo.completed }"
                @mouseenter="checkTitleOverflow($event, todo.id)"
              >
                {{ todo.title }}
              </span>
            </el-tooltip>
            <button
              v-if="!isTrashMode"
              type="button"
              class="title-edit-btn no-drag"
              :title="t('common.edit')"
              @click.stop="startTitleEdit(todo)"
            >
              <el-icon><Edit /></el-icon>
            </button>
          </div>
        </div>

        <template v-if="!minimal">
          <div class="col-priority no-drag" @click.stop>
            <el-dropdown
              v-if="!isTrashMode"
              trigger="click"
              teleported
              @command="(value: string) => onPriorityChange(todo, value)"
            >
              <span
                class="priority-pill priority-cell-trigger"
                :class="priorityClass(todo.priority)"
                :title="t('task.changePriority')"
              >
                {{ priorityLabel[todo.priority] ?? t("priority.none") }}
              </span>
              <template #dropdown>
                <el-dropdown-menu>
                  <el-dropdown-item command="high">
                    {{ t("priority.highOption") }}
                  </el-dropdown-item>
                  <el-dropdown-item command="medium">
                    {{ t("priority.mediumOption") }}
                  </el-dropdown-item>
                  <el-dropdown-item command="low">
                    {{ t("priority.low") }}
                  </el-dropdown-item>
                </el-dropdown-menu>
              </template>
            </el-dropdown>
            <span v-else class="priority-pill" :class="priorityClass(todo.priority)">
              {{ priorityLabel[todo.priority] ?? t("priority.none") }}
            </span>
          </div>

          <div class="col-due no-drag" @click.stop>
            <el-popover
              v-if="!isTrashMode"
              :visible="duePopoverId === todo.id"
              placement="bottom-start"
              :width="380"
              trigger="click"
              teleported
              popper-class="due-date-popover"
              @update:visible="(visible: boolean) => onDuePopoverVisible(todo.id, visible)"
            >
              <template #reference>
                <span class="due-cell-trigger" :title="t('task.changeDueDate')">
                  <span v-if="todo.dueDate" class="due-date">
                    <el-icon><Calendar /></el-icon>
                    {{ formatDueDate(todo.dueDate) }}
                  </span>
                  <span v-else class="due-empty">{{ t("common.dash") }}</span>
                </span>
              </template>
              <el-date-picker-panel
                :model-value="dueDateValue(todo)"
                type="date"
                value-format="YYYY-MM-DD"
                :border="false"
                @update:model-value="(value: string | undefined) => onDueDateChange(todo, value)"
              />
              <button
                v-if="todo.dueDate"
                type="button"
                class="due-clear-btn"
                @click="onDueDateChange(todo, undefined)"
              >
                {{ t("task.clearDueDate") }}
              </button>
            </el-popover>
            <template v-else>
              <span v-if="todo.dueDate" class="due-date">
                <el-icon><Calendar /></el-icon>
                {{ formatDueDate(todo.dueDate) }}
              </span>
              <span v-else class="due-empty">{{ t("common.dash") }}</span>
            </template>
          </div>

          <div class="col-category no-drag" @click.stop>
            <el-dropdown
              v-if="!isTrashMode"
              trigger="click"
              teleported
              @command="(value: number | 'none') => onCategoryChange(todo, value)"
            >
              <span
                class="category-pill category-cell-trigger"
                :class="{ 'category-pill--empty': !todo.categoryName }"
                :style="categoryPillStyle(todo)"
                :title="t('task.changeCategory')"
              >
                {{ todo.categoryName ?? t("common.dash") }}
              </span>
              <template #dropdown>
                <el-dropdown-menu>
                  <el-dropdown-item command="none">
                    {{ t("common.none") }}
                  </el-dropdown-item>
                  <el-dropdown-item
                    v-for="category in categories"
                    :key="category.id"
                    :command="category.id"
                  >
                    {{ category.name }}
                  </el-dropdown-item>
                </el-dropdown-menu>
              </template>
            </el-dropdown>
            <span
              v-else
              class="category-pill"
              :class="{ 'category-pill--empty': !todo.categoryName }"
              :style="categoryPillStyle(todo)"
            >
              {{ todo.categoryName ?? t("common.dash") }}
            </span>
          </div>

          <div class="col-tags no-drag" @click.stop>
            <el-popover
              v-if="!isTrashMode"
              :visible="tagPopoverId === todo.id"
              placement="bottom-start"
              :width="300"
              trigger="click"
              teleported
              popper-class="tag-picker-popover"
              @update:visible="(visible: boolean) => onTagPopoverVisible(todo, visible)"
            >
              <template #reference>
                <span class="tag-cell-trigger" :title="t('task.changeTags')">
                  <span v-if="todo.tagNames.length" class="tag-list">
                    <span
                      v-for="(name, index) in todo.tagNames"
                      :key="`${todo.id}-${name}`"
                      class="tag-pill"
                      :style="tagPillStyle(todo, index)"
                    >
                      #{{ name }}
                    </span>
                  </span>
                  <span v-else class="due-empty">{{ t("common.dash") }}</span>
                </span>
              </template>
              <el-select
                v-model="tagDrafts[todo.id]"
                multiple
                filterable
                collapse-tags
                collapse-tags-tooltip
                :teleported="false"
                :placeholder="t('task.selectTags')"
                class="tag-inline-select"
                style="width: 100%"
              >
                <el-option
                  v-for="tag in tags"
                  :key="tag.id"
                  :label="tag.name"
                  :value="tag.id"
                />
              </el-select>
            </el-popover>
            <template v-else>
              <div v-if="todo.tagNames.length" class="tag-list">
                <span
                  v-for="(name, index) in todo.tagNames"
                  :key="`${todo.id}-${name}`"
                  class="tag-pill"
                  :style="tagPillStyle(todo, index)"
                >
                  #{{ name }}
                </span>
              </div>
              <span v-else class="due-empty">{{ t("common.dash") }}</span>
            </template>
          </div>

          <div class="col-actions no-drag" @click.stop>
            <div class="row-actions">
              <template v-if="isTrashMode">
                <el-button
                  link
                  type="primary"
                  :icon="RefreshLeft"
                  :title="t('common.restore')"
                  @click="emit('restore', todo)"
                />
                <el-button
                  link
                  type="danger"
                  :icon="Delete"
                  :title="t('common.delete')"
                  @click="emit('delete', todo)"
                />
              </template>
              <template v-else>
                <el-button
                  link
                  type="primary"
                  :icon="View"
                  :title="t('common.view')"
                  @click="emit('view', todo.id)"
                />
                <el-button
                  link
                  type="primary"
                  :icon="Edit"
                  :title="t('common.edit')"
                  @click="emit('edit', todo.id)"
                />
                <el-button
                  link
                  type="danger"
                  :icon="Delete"
                  :title="t('common.delete')"
                  @click="emit('delete', todo)"
                />
              </template>
            </div>
          </div>
        </template>
      </div>
    </div>
  </div>
</template>

<style scoped>
.task-list {
  display: flex;
  flex-direction: column;
  min-height: 0;
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

.task-list-header,
.task-list-row {
  display: grid;
  grid-template-columns: 40px 48px minmax(220px, 1.6fr) 108px 110px minmax(120px, 0.8fr) minmax(160px, 1fr) 120px;
  align-items: center;
  column-gap: 8px;
  padding: 0 12px;
}

.task-list-header--trash,
.task-list-row--trash {
  grid-template-columns: 48px minmax(220px, 1.6fr) 108px 110px minmax(120px, 0.8fr) minmax(160px, 1fr) 140px;
}

.task-list--minimal .task-list-header,
.task-list--minimal .task-list-row {
  grid-template-columns: 40px 48px minmax(0, 1fr);
}

.task-list-header {
  height: 44px;
  border-bottom: 1px solid var(--border-light);
  background: var(--surface-muted);
  color: var(--text-secondary);
  font-size: 13px;
  font-weight: 500;
  flex-shrink: 0;
  position: sticky;
  top: 0;
  z-index: 2;
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

.sortable:hover {
  color: var(--primary);
}

.sort-mark {
  color: var(--primary);
  font-size: 12px;
}

.task-list-body {
  display: flex;
  flex-direction: column;
  min-height: 48px;
}

.task-list-body .empty-state {
  padding: 64px 24px;
  text-align: center;
  color: var(--text-secondary);
}

.task-list:not(.is-drag-disabled) .task-list-row {
  cursor: grab;
}

.task-list-row.list-drag-chosen,
.task-list-row.list-drag-dragging {
  cursor: grabbing;
}

.task-list-row {
  min-height: 52px;
  border-bottom: 1px solid var(--border-light);
  cursor: pointer;
  transition: background 0.15s;
}

.task-list-row:nth-child(even) {
  background: var(--table-stripe);
}

.task-list-row:hover {
  background: var(--nav-hover);
}

.task-list-row.row-done {
  opacity: 0.72;
}

.task-list-row.row-pinned {
  background: var(--pin-row-bg);
}

.task-list-row.row-highlight {
  animation: row-highlight-fade 2s ease-out;
}

@keyframes row-highlight-fade {
  0% {
    background: color-mix(in srgb, var(--primary) 18%, var(--panel-bg));
  }
  100% {
    background: transparent;
  }
}

.pin-btn {
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
  padding: 0;
  opacity: 0;
  transition: opacity 0.15s, color 0.15s, background 0.15s;
}

.task-list-row:hover .pin-btn,
.pin-btn.active {
  opacity: 1;
}

.pin-btn:hover,
.pin-btn.active {
  color: var(--pin-color);
  background: var(--pin-bg);
}

.col-title {
  min-width: 0;
  overflow: hidden;
}

.title-cell {
  display: flex;
  align-items: center;
  gap: 4px;
  min-width: 0;
}

.col-title :deep(.el-tooltip__trigger) {
  display: block;
  flex: 1;
  min-width: 0;
}

.task-title {
  display: block;
  width: 100%;
  font-weight: 500;
  color: var(--text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.task-title.done {
  text-decoration: line-through;
  color: var(--text-secondary);
}

.title-edit-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  width: 24px;
  height: 24px;
  border: none;
  background: transparent;
  color: var(--text-tertiary);
  cursor: pointer;
  border-radius: 6px;
  padding: 0;
  opacity: 0;
  transition: opacity 0.15s, color 0.15s, background 0.15s;
}

.col-title:hover .title-edit-btn,
.task-list-row.row-title-editing .title-edit-btn {
  opacity: 1;
}

.title-edit-btn:hover {
  color: var(--primary);
  background: var(--nav-hover);
}

.title-input {
  width: 100%;
  border: 1px solid var(--border-color);
  border-radius: 6px;
  background: var(--panel-bg);
  padding: 4px 8px;
  font-size: 14px;
  font-weight: 500;
  color: var(--text-primary);
  outline: none;
}

.title-input:focus {
  border-color: var(--primary);
}

.title-input.done {
  text-decoration: line-through;
  color: var(--text-secondary);
}

.task-list-row.row-title-editing {
  background: var(--nav-hover);
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

.category-pill {
  display: inline-block;
  max-width: 100%;
  padding: 2px 10px;
  border-radius: 12px;
  font-size: 12px;
  font-weight: 500;
  color: var(--tag-text);
  background: var(--tag-bg);
  border: 1px solid var(--tag-border);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.category-pill--empty {
  color: var(--text-tertiary);
  background: transparent;
  border-color: var(--border-light);
}

.category-cell-trigger {
  cursor: pointer;
  transition: box-shadow 0.15s;
}

.category-cell-trigger:hover {
  box-shadow: 0 0 0 1px color-mix(in srgb, var(--text-tertiary) 35%, transparent);
}

.col-tags {
  display: flex;
  align-items: center;
  min-width: 0;
}

.tag-cell-trigger {
  display: inline-flex;
  align-items: center;
  max-width: 100%;
  min-width: 0;
  cursor: pointer;
  border-radius: 6px;
  padding: 2px 4px;
  transition: box-shadow 0.15s, background 0.15s;
}

.tag-cell-trigger:hover {
  background: var(--nav-hover);
  box-shadow: 0 0 0 1px color-mix(in srgb, var(--text-tertiary) 35%, transparent);
}

.tag-list {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  min-width: 0;
}

.tag-pill {
  display: inline-block;
  padding: 1px 8px;
  border-radius: 12px;
  font-size: 12px;
  color: var(--tag-text);
  background: var(--tag-bg);
  border: 1px solid var(--tag-border);
}

.row-actions {
  display: flex;
  justify-content: center;
  gap: 2px;
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
