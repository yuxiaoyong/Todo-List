<script setup lang="ts">
import {
  computed,
  nextTick,
  onBeforeUnmount,
  onMounted,
  ref,
  shallowRef,
  watch,
} from "vue";
import { useI18n } from "vue-i18n";
import Gantt from "frappe-gantt";
import { todoApi } from "../../api";
import {
  ganttDatesToApi,
  ganttLocaleFromAppLocale,
  todosToGanttTasks,
} from "../../utils/ganttTasks";
import {
  applyTodayMarker,
  scrollToToday,
  type GanttChartInstance,
  type GanttViewMode,
} from "../../utils/ganttToday";
import type { TodoSummary } from "../../types";

const props = defineProps<{
  todos: TodoSummary[];
  loading?: boolean;
}>();

const emit = defineEmits<{
  refresh: [];
  view: [id: number];
}>();

const { locale, t } = useI18n();

const containerRef = ref<HTMLElement | null>(null);
const ganttInstance = shallowRef<Gantt | null>(null);
const savingDates = ref(false);
const viewMode = ref<GanttViewMode>("Week");

const ganttTasks = computed(() => todosToGanttTasks(props.todos));
const ganttLanguage = computed(() => ganttLocaleFromAppLocale(locale.value));

function asChart(gantt: Gantt): GanttChartInstance {
  return gantt as unknown as GanttChartInstance;
}

function buildOptions() {
  return {
    view_mode: viewMode.value,
    bar_height: 28,
    padding: 16,
    language: ganttLanguage.value,
    on_click: (task: { id: string }) => {
      const id = Number(task.id);
      if (!Number.isNaN(id)) {
        emit("view", id);
      }
    },
    on_date_change: (task: { id: string }, start: Date, end: Date) => {
      void persistDateChange(Number(task.id), start, end);
    },
  };
}

function afterRender(scrollToday = false) {
  const gantt = ganttInstance.value;
  if (!gantt) return;

  applyTodayMarker(asChart(gantt));
  if (scrollToday) {
    scrollToToday(asChart(gantt));
  }
}

function destroyGantt() {
  ganttInstance.value?.clear();
  if (containerRef.value) {
    containerRef.value.innerHTML = "";
  }
  ganttInstance.value = null;
}

function mountGantt(scrollToday = true) {
  if (!containerRef.value || !props.todos.length) return;

  destroyGantt();

  try {
    ganttInstance.value = new Gantt(containerRef.value, ganttTasks.value, buildOptions());
    afterRender(scrollToday);
  } catch (err) {
    console.error("[GanttView] failed to mount gantt chart", err);
  }
}

function scheduleMount(scrollToday = true) {
  void nextTick(() => {
    requestAnimationFrame(() => {
      mountGantt(scrollToday);
    });
  });
}

function onViewModeChange(mode: GanttViewMode) {
  if (!ganttInstance.value || !props.todos.length) return;

  ganttInstance.value.change_view_mode(mode);
  void nextTick(() => afterRender(true));
}

function goToday() {
  const gantt = ganttInstance.value;
  if (!gantt) return;
  scrollToToday(asChart(gantt), true);
}

async function persistDateChange(id: number, start: Date, end: Date) {
  if (Number.isNaN(id) || savingDates.value) return;

  const todo = props.todos.find((item) => item.id === id);
  if (!todo) return;

  const { startDate, dueDate } = ganttDatesToApi(start, end);
  savingDates.value = true;

  try {
    const detail = await todoApi.get(id);
    await todoApi.update({
      id,
      title: todo.title,
      contentHtml: detail.contentHtml,
      completed: todo.completed,
      priority: todo.priority,
      startDate,
      dueDate,
      categoryId: todo.categoryId ?? null,
      tagIds: todo.tagIds,
      sortOrder: todo.sortOrder,
      pinned: todo.pinned,
      assignee: todo.assignee,
      kanbanColumnId: todo.kanbanColumnId,
      quiet: true,
    });
    emit("refresh");
  } finally {
    savingDates.value = false;
  }
}

let resizeObserver: ResizeObserver | null = null;

function setupResizeObserver() {
  if (!containerRef.value || typeof ResizeObserver === "undefined") return;

  resizeObserver = new ResizeObserver(() => {
    if (!props.todos.length) return;
    if (!ganttInstance.value) {
      scheduleMount(true);
    }
  });

  resizeObserver.observe(containerRef.value);
}

watch(
  () => props.todos.length,
  (length) => {
    if (length === 0) {
      destroyGantt();
      return;
    }
    scheduleMount(true);
  },
);

watch(
  ganttTasks,
  (tasks) => {
    if (savingDates.value || !ganttInstance.value || !props.todos.length) return;
    ganttInstance.value.refresh(tasks);
    void nextTick(() => afterRender(false));
  },
  { deep: true },
);

watch(ganttLanguage, () => {
  if (!props.todos.length) return;
  scheduleMount(true);
});

onMounted(() => {
  setupResizeObserver();
  scheduleMount(true);
});

onBeforeUnmount(() => {
  resizeObserver?.disconnect();
  resizeObserver = null;
  destroyGantt();
});
</script>

<template>
  <div v-loading="loading" class="gantt-view-root">
    <div v-if="todos.length" class="gantt-toolbar">
      <el-radio-group
        v-model="viewMode"
        size="small"
        @change="onViewModeChange"
      >
        <el-radio-button value="Day">{{ t("task.ganttDay") }}</el-radio-button>
        <el-radio-button value="Week">{{ t("task.ganttWeek") }}</el-radio-button>
        <el-radio-button value="Month">{{ t("task.ganttMonth") }}</el-radio-button>
      </el-radio-group>
      <el-button size="small" @click="goToday">{{ t("task.ganttToday") }}</el-button>
    </div>

    <div ref="containerRef" class="gantt-host" />
    <div v-if="!todos.length" class="gantt-empty">
      <el-empty :description="t('task.noTasks')" />
    </div>
  </div>
</template>

<style scoped>
.gantt-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 10px 16px;
  border-bottom: 1px solid var(--border-light);
  background: var(--panel-bg);
  flex-shrink: 0;
}
</style>
