<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { Calendar, CircleCheck, Folder, StarFilled } from "@element-plus/icons-vue";
import { useDraggable } from "vue-draggable-plus";
import type { SortableEvent } from "sortablejs";
import type { TodoSummary } from "../../types";
import { isTodoOverdue } from "../../utils/formatDate";

const props = defineProps<{
  modelValue: TodoSummary[];
  loading?: boolean;
  canDrag?: boolean;
  highlightId?: number | null;
}>();

const emit = defineEmits<{
  "update:modelValue": [TodoSummary[]];
  sortStart: [];
  sortEnd: [evt: SortableEvent];
  toggle: [todo: TodoSummary];
  view: [id: number];
}>();

const { t } = useI18n();
const sortableEl = ref<HTMLElement | null>(null);
let suppressCardClick = false;
const isDragging = ref(false);

const pinnedTodos = computed(() => props.modelValue.filter((todo) => todo.pinned));

const sortableTodos = computed({
  get: () => props.modelValue.filter((todo) => !todo.pinned),
  set: (unpinned) => {
    emit("update:modelValue", [...pinnedTodos.value, ...unpinned]);
  },
});

const priorityLabel = computed<Record<string, string>>(() => ({
  high: t("priority.high"),
  medium: t("priority.medium"),
  low: t("priority.none"),
}));

const dragOptions = computed(() => ({
  immediate: false,
  animation: 240,
  easing: "cubic-bezier(0.2, 0, 0, 1)",
  draggable: ".minimal-card--sortable",
  filter: ".no-drag",
  preventOnFilter: true,
  ghostClass: "minimal-card-ghost",
  chosenClass: "minimal-card-chosen",
  dragClass: "minimal-card-dragging",
  forceFallback: true,
  fallbackOnBody: true,
  swapThreshold: 0.65,
  disabled: !props.canDrag || sortableTodos.value.length < 2,
  onStart() {
    isDragging.value = true;
    document.body.style.userSelect = "none";
    document.body.style.webkitUserSelect = "none";
    emit("sortStart");
  },
  onEnd(evt: SortableEvent) {
    suppressCardClick = evt.oldIndex !== evt.newIndex;
    document.body.style.userSelect = "";
    document.body.style.webkitUserSelect = "";
    document
      .querySelectorAll("body > .sortable-fallback, body > .minimal-card-dragging")
      .forEach((node) => node.remove());
    emit("sortEnd", evt);
    isDragging.value = false;
    window.setTimeout(() => {
      suppressCardClick = false;
    }, 0);
  },
}));

const draggable = useDraggable(sortableEl, sortableTodos, dragOptions);

async function initDraggable() {
  await nextTick();
  if (!sortableEl.value) return;
  draggable.start(sortableEl.value);
  if (props.canDrag && sortableTodos.value.length >= 2) draggable.resume?.();
  else draggable.pause?.();
}

onMounted(() => {
  void initDraggable();
});

watch(
  () => sortableTodos.value.length,
  (len, prevLen) => {
    if (len > 0 && prevLen === 0) void initDraggable();
    if (len < 2) draggable.pause?.();
    else if (props.canDrag) draggable.resume?.();
  },
);

watch(
  () => props.canDrag,
  (enabled) => {
    if (enabled && sortableTodos.value.length >= 2) draggable.resume?.();
    else draggable.pause?.();
  },
);

onUnmounted(() => {
  document.body.style.userSelect = "";
  document.body.style.webkitUserSelect = "";
  draggable.destroy?.();
});

function formatCardDate(date?: string) {
  if (!date) return "";
  return date.replace("T", " ").slice(0, 16);
}

function cardDate(todo: TodoSummary) {
  return formatCardDate(todo.dueDate) || formatCardDate(todo.updatedAt);
}

function isDueDateOverdue(todo: TodoSummary) {
  return !!todo.dueDate && isTodoOverdue(todo);
}

function previewText(text: string) {
  const value = text.trim();
  if (!value) return "";
  return value.length > 96 ? `${value.slice(0, 96)}…` : value;
}

function priorityClass(priority: string) {
  if (priority === "high") return "pill-high";
  if (priority === "medium") return "pill-medium";
  return "pill-none";
}

function accentStyle(todo: TodoSummary) {
  if (todo.pinned) return { background: "var(--pin-color)" };
  return { background: todo.categoryColor || "var(--primary)" };
}

function cardClasses(todo: TodoSummary, sortable: boolean) {
  return {
    "minimal-card": true,
    "minimal-card--sortable": sortable,
    "minimal-card--pinned": todo.pinned,
    "minimal-card--done": todo.completed,
    "minimal-card--highlight": todo.id === props.highlightId,
  };
}

function onCardClick(todo: TodoSummary) {
  if (suppressCardClick) return;
  emit("view", todo.id);
}

function onToggle(todo: TodoSummary) {
  emit("toggle", todo);
}
</script>

<template>
  <div
    v-loading="loading"
    class="minimal-task-list"
    :class="{ 'is-dragging': isDragging, 'is-drag-disabled': !canDrag }"
  >
    <div class="minimal-task-list-body">
      <div v-if="!modelValue.length" class="minimal-empty">
        {{ t("task.noTasks") }}
      </div>

      <template v-else>
        <section v-if="pinnedTodos.length" class="minimal-pinned-zone">
          <div class="minimal-zone-label">
            <el-icon><StarFilled /></el-icon>
            {{ t("task.pin") }}
          </div>
          <article
            v-for="todo in pinnedTodos"
            :key="todo.id"
            :class="cardClasses(todo, false)"
            @click="onCardClick(todo)"
          >
            <div class="card-main">
              <button
                type="button"
                class="check-btn no-drag"
                :class="{ checked: todo.completed }"
                :aria-label="todo.completed ? t('status.completed') : t('status.inProgress')"
                @click.stop="onToggle(todo)"
              >
                <el-icon><CircleCheck /></el-icon>
              </button>

              <div class="card-content">
                <h3 class="card-title" :class="{ done: todo.completed }">{{ todo.title }}</h3>
                <p v-if="previewText(todo.contentText)" class="card-desc">
                  {{ previewText(todo.contentText) }}
                </p>

                <div class="card-meta">
                  <span class="meta-pill" :class="priorityClass(todo.priority)">
                    <span class="pill-dot" />
                    {{ priorityLabel[todo.priority] ?? t("priority.none") }}
                  </span>

                  <span v-if="todo.categoryName" class="meta-category" :title="todo.categoryName">
                    <span class="category-dot" :style="{ background: todo.categoryColor || '#1677ff' }" />
                    <el-icon class="category-icon"><Folder /></el-icon>
                  </span>

                  <span
                    v-if="cardDate(todo)"
                    class="meta-date"
                    :class="{ 'meta-date--overdue': isDueDateOverdue(todo) }"
                  >
                    <el-icon><Calendar /></el-icon>
                    {{ cardDate(todo) }}
                  </span>

                  <span
                    v-for="(name, index) in todo.tagNames.slice(0, 2)"
                    :key="`${todo.id}-${name}`"
                    class="meta-tag"
                    :style="{
                      color: todo.tagColors[index] || 'var(--primary)',
                      borderColor: todo.tagColors[index] || 'var(--tag-border)',
                    }"
                  >
                    #{{ name }}
                  </span>
                </div>
              </div>
            </div>

            <div class="card-accent" :style="accentStyle(todo)" />
          </article>
        </section>

        <div
          ref="sortableEl"
          class="minimal-sortable-zone"
          :class="{ 'minimal-sortable-zone--empty': !sortableTodos.length }"
        >
          <article
            v-for="todo in sortableTodos"
            :key="todo.id"
            :class="cardClasses(todo, true)"
            @click="onCardClick(todo)"
          >
            <div class="card-main">
              <button
                type="button"
                class="check-btn no-drag"
                :class="{ checked: todo.completed }"
                :aria-label="todo.completed ? t('status.completed') : t('status.inProgress')"
                @click.stop="onToggle(todo)"
              >
                <el-icon><CircleCheck /></el-icon>
              </button>

              <div class="card-content">
                <h3 class="card-title" :class="{ done: todo.completed }">{{ todo.title }}</h3>
                <p v-if="previewText(todo.contentText)" class="card-desc">
                  {{ previewText(todo.contentText) }}
                </p>

                <div class="card-meta">
                  <span class="meta-pill" :class="priorityClass(todo.priority)">
                    <span class="pill-dot" />
                    {{ priorityLabel[todo.priority] ?? t("priority.none") }}
                  </span>

                  <span v-if="todo.categoryName" class="meta-category" :title="todo.categoryName">
                    <span class="category-dot" :style="{ background: todo.categoryColor || '#1677ff' }" />
                    <el-icon class="category-icon"><Folder /></el-icon>
                  </span>

                  <span
                    v-if="cardDate(todo)"
                    class="meta-date"
                    :class="{ 'meta-date--overdue': isDueDateOverdue(todo) }"
                  >
                    <el-icon><Calendar /></el-icon>
                    {{ cardDate(todo) }}
                  </span>

                  <span
                    v-for="(name, index) in todo.tagNames.slice(0, 2)"
                    :key="`${todo.id}-${name}`"
                    class="meta-tag"
                    :style="{
                      color: todo.tagColors[index] || 'var(--primary)',
                      borderColor: todo.tagColors[index] || 'var(--tag-border)',
                    }"
                  >
                    #{{ name }}
                  </span>
                </div>
              </div>
            </div>

            <div class="card-accent" :style="accentStyle(todo)" />
          </article>
        </div>
      </template>
    </div>
  </div>
</template>

<style scoped>
.minimal-task-list {
  min-height: 120px;
}

.minimal-task-list.is-dragging,
.minimal-task-list.is-dragging * {
  user-select: none;
  -webkit-user-select: none;
}

.minimal-task-list-body {
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 0 14px 16px;
}

.minimal-pinned-zone,
.minimal-sortable-zone {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.minimal-pinned-zone + .minimal-sortable-zone {
  margin-top: 4px;
}

.minimal-zone-label {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 0 2px;
  font-size: 12px;
  font-weight: 600;
  color: var(--pin-color);
}

.minimal-zone-label :deep(.el-icon) {
  font-size: 13px;
}

.minimal-empty {
  padding: 48px 16px;
  text-align: center;
  color: var(--text-secondary);
  font-size: 14px;
}

.minimal-card {
  position: relative;
  background: var(--panel-bg);
  border: 1px solid var(--border-color);
  border-radius: 10px;
  box-shadow: var(--shadow-sm);
  overflow: hidden;
  cursor: pointer;
  transition: box-shadow 0.15s ease;
}

.minimal-task-list:not(.is-dragging) .minimal-card--sortable {
  transition: box-shadow 0.15s ease, transform 0.24s cubic-bezier(0.2, 0, 0, 1);
}

.minimal-task-list:not(.is-drag-disabled) .minimal-card--sortable {
  cursor: grab;
}

.minimal-card--pinned {
  background: color-mix(in srgb, var(--pin-row-bg) 55%, var(--panel-bg));
  cursor: pointer;
}

.minimal-card:hover {
  box-shadow: var(--shadow-card);
}

.minimal-card--done {
  opacity: 0.82;
}

.minimal-card--highlight {
  animation: minimal-card-highlight 2s ease-out;
}

@keyframes minimal-card-highlight {
  0% {
    box-shadow: 0 0 0 2px color-mix(in srgb, var(--primary) 35%, transparent);
  }
  100% {
    box-shadow: var(--shadow-sm);
  }
}

.minimal-card-chosen,
.minimal-card-dragging {
  cursor: grabbing;
}

.card-main {
  display: flex;
  gap: 10px;
  padding: 14px 14px 12px;
}

.check-btn {
  flex-shrink: 0;
  width: 22px;
  height: 22px;
  margin-top: 2px;
  border: 2px solid #91caff;
  border-radius: 50%;
  background: transparent;
  color: transparent;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 0;
  cursor: pointer;
  transition: background 0.15s, border-color 0.15s, color 0.15s;
}

.check-btn:hover {
  border-color: var(--primary);
}

.check-btn.checked {
  border-color: var(--primary);
  background: var(--primary);
  color: #fff;
}

.check-btn :deep(.el-icon) {
  font-size: 14px;
}

.card-content {
  min-width: 0;
  flex: 1;
}

.card-title {
  margin: 0;
  font-size: 15px;
  font-weight: 600;
  line-height: 1.45;
  color: var(--text-primary);
  word-break: break-word;
}

.card-title.done {
  color: var(--text-secondary);
  text-decoration: line-through;
  text-decoration-thickness: 2px;
  text-decoration-color: color-mix(in srgb, var(--text-secondary) 90%, var(--text-primary));
}

.minimal-card--done .card-desc {
  text-decoration: line-through;
  text-decoration-thickness: 1.5px;
  text-decoration-color: color-mix(in srgb, var(--text-secondary) 80%, transparent);
  color: var(--text-secondary);
}

.card-desc {
  margin: 6px 0 0;
  font-size: 13px;
  line-height: 1.5;
  color: #5b6b86;
  word-break: break-word;
}

.card-meta {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 8px;
  margin-top: 10px;
}

.meta-pill {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 2px 8px;
  border-radius: 999px;
  font-size: 12px;
  font-weight: 500;
  line-height: 18px;
}

.pill-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: currentColor;
  opacity: 0.9;
}

.pill-high {
  background: #fff1f0;
  color: #cf1322;
}

.pill-medium {
  background: #fffbe6;
  color: #d48806;
}

.pill-none {
  background: #f5f5f5;
  color: #8c8c8c;
}

.meta-category {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  color: var(--text-secondary);
}

.category-dot {
  width: 8px;
  height: 8px;
  border-radius: 2px;
  transform: rotate(45deg);
}

.category-icon {
  font-size: 14px;
}

.meta-date {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
  color: var(--text-secondary);
}

.meta-date--overdue {
  color: var(--priority-high-text);
  font-weight: 500;
}

.meta-date :deep(.el-icon) {
  font-size: 13px;
}

.meta-tag {
  display: inline-flex;
  align-items: center;
  padding: 1px 8px;
  border: 1px solid var(--tag-border);
  border-radius: 999px;
  font-size: 12px;
  background: transparent;
}

.card-accent {
  height: 3px;
  width: 100%;
}

:deep(.minimal-card-ghost) {
  opacity: 0.42;
  transform: scale(0.98);
}

:deep(.minimal-card-dragging) {
  opacity: 0.96;
  box-shadow: var(--shadow-card);
  transform: scale(1.02);
}
</style>
