<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Check, Fold, MoreFilled, Refresh } from "@element-plus/icons-vue";
import { ElMessage } from "element-plus";
import type { SortableEvent } from "sortablejs";
import MinimalCommandInput from "../components/todo/MinimalCommandInput.vue";
import MinimalTaskList from "../components/layout/MinimalTaskList.vue";
import { todoApi, windowApi } from "../api";
import { buildBucketSortPositions } from "../utils/kanban";
import { matchesTimeFilter } from "../utils/timeFilter";
import { parseTodoSort, sortTodos } from "../utils/sortTodos";
import { useTodoStore } from "../stores/todo";
import { useUiStore } from "../stores/ui";
import type { TodoSummary } from "../types";

const { t } = useI18n();
const todoStore = useTodoStore();
const uiStore = useUiStore();
const sortableTodos = ref<TodoSummary[]>([]);
const listReordering = ref(false);
const dockAnimating = ref(false);

const filteredTodos = computed(() =>
  todoStore.todos.filter((item) => matchesTimeFilter(item, uiStore.timeFilter)),
);

const parsedSort = computed(() => parseTodoSort(uiStore.todoSort));

const displayTodos = computed(() => {
  const { field, direction } = parsedSort.value;
  return sortTodos(filteredTodos.value, field, direction);
});

const canDrag = computed(() => parsedSort.value.field === "default");

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

async function onToggle(todo: TodoSummary) {
  await todoApi.toggleComplete(todo.id);
  await refresh();
}

async function onView(id: number) {
  await windowApi.openTaskDetail(id);
}

async function onQuickCreated(openDetail: boolean) {
  const id = todoStore.highlightId;
  if (openDetail && id) {
    await windowApi.openTaskDetail(id);
    return;
  }
  if (!openDetail && id) {
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
    const pinned = sortableTodos.value.filter((todo) => todo.pinned);
    const unpinned = sortableTodos.value.filter((todo) => !todo.pinned);
    const items = buildBucketSortPositions([...pinned, ...unpinned]);
    if (items.length) {
      await todoApi.reorderPositions(items);
    }
    await refresh();
  } catch (error) {
    console.error("minimal list reorder failed", error);
    sortableTodos.value = [...displayTodos.value];
    ElMessage.error(t("kanban.reorderFailed"));
  } finally {
    listReordering.value = false;
  }
}

function setCompletedFilter(value: boolean | null) {
  uiStore.setCompletedFilter(value);
  void refresh();
}

function onMenuCommand(cmd: string) {
  if (cmd === "refresh") {
    void refresh();
    return;
  }
  if (cmd === "filter-all") {
    setCompletedFilter(null);
  } else if (cmd === "filter-incomplete") {
    setCompletedFilter(false);
  } else if (cmd === "filter-completed") {
    setCompletedFilter(true);
  }
}

let unlisten: (() => void) | undefined;
let unlistenBlur: (() => void) | undefined;
let unlistenDockAnim: (() => void) | undefined;

onMounted(async () => {
  await refresh();
  unlisten = await listen("todo-changed", () => {
    void refresh();
  });

  const currentWindow = getCurrentWindow();
  unlistenBlur = await currentWindow.onFocusChanged(({ payload: focused }) => {
    if (!focused) {
      void windowApi.minimalDockOnBlur();
    }
  });

  unlistenDockAnim = await listen<{ active: boolean }>("minimal-dock-animating", (event) => {
    dockAnimating.value = event.payload.active;
  });
});

onUnmounted(() => {
  unlisten?.();
  unlistenBlur?.();
  unlistenDockAnim?.();
});
</script>

<template>
  <div class="minimal-shell" :class="{ 'is-dock-animating': dockAnimating }">
    <header class="minimal-toolbar" data-tauri-drag-region>
      <button
        type="button"
        class="toolbar-btn"
        data-tauri-drag-region="false"
        :title="t('minimal.openMain')"
        @click="windowApi.showMain()"
      >
        <el-icon><Fold /></el-icon>
      </button>

      <el-dropdown trigger="click" @command="onMenuCommand">
        <button
          type="button"
          class="toolbar-btn"
          data-tauri-drag-region="false"
          :title="t('minimal.more')"
        >
          <el-icon><MoreFilled /></el-icon>
        </button>
        <template #dropdown>
          <el-dropdown-menu>
            <el-dropdown-item command="filter-all" class="filter-menu-item">
              <el-icon v-if="uiStore.completedFilter === null" class="filter-check"><Check /></el-icon>
              <span v-else class="filter-check-placeholder" />
              {{ t("common.all") }}
            </el-dropdown-item>
            <el-dropdown-item command="filter-incomplete" class="filter-menu-item">
              <el-icon v-if="uiStore.completedFilter === false" class="filter-check"><Check /></el-icon>
              <span v-else class="filter-check-placeholder" />
              {{ t("status.incomplete") }}
            </el-dropdown-item>
            <el-dropdown-item command="filter-completed" class="filter-menu-item">
              <el-icon v-if="uiStore.completedFilter === true" class="filter-check"><Check /></el-icon>
              <span v-else class="filter-check-placeholder" />
              {{ t("status.completed") }}
            </el-dropdown-item>
            <el-dropdown-item command="refresh" divided>
              <el-icon><Refresh /></el-icon>
              {{ t("common.refresh") }}
            </el-dropdown-item>
          </el-dropdown-menu>
        </template>
      </el-dropdown>
    </header>

    <MinimalCommandInput @refresh="refresh" @created="onQuickCreated" />

    <div class="minimal-list-wrap">
      <MinimalTaskList
        v-model="sortableTodos"
        :loading="todoStore.loading"
        :can-drag="canDrag"
        :highlight-id="todoStore.highlightId"
        @sort-start="onListSortStart"
        @sort-end="onListSortEnd"
        @toggle="onToggle"
        @view="onView"
      />
    </div>
  </div>
</template>

<style scoped>
.minimal-shell {
  display: flex;
  flex-direction: column;
  height: 100vh;
  background: var(--app-bg);
  overflow: hidden;
  opacity: 1;
  transition: opacity 0.22s ease;
}

.minimal-shell.is-dock-animating {
  opacity: 0.92;
}

.minimal-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 12px 6px;
  padding-top: calc(10px + env(safe-area-inset-top, 0px));
  flex-shrink: 0;
  background: var(--app-bg);
  -webkit-app-region: drag;
  app-region: drag;
}

.minimal-toolbar :deep(button),
.minimal-toolbar :deep(.el-dropdown) {
  -webkit-app-region: no-drag;
  app-region: no-drag;
}

.toolbar-btn {
  width: 34px;
  height: 34px;
  border: none;
  border-radius: 8px;
  background: transparent;
  color: var(--text-primary);
  display: inline-flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: background 0.15s;
}

.toolbar-btn:hover {
  background: color-mix(in srgb, var(--text-primary) 6%, transparent);
}

.toolbar-btn :deep(.el-icon) {
  font-size: 18px;
}

.minimal-list-wrap {
  flex: 1;
  min-height: 0;
  overflow: auto;
  scrollbar-width: none;
  -ms-overflow-style: none;
}

.minimal-list-wrap::-webkit-scrollbar {
  display: none;
}

:deep(.filter-menu-item) {
  min-width: 148px;
}

:deep(.filter-menu-item .el-dropdown-menu__item) {
  display: flex;
  align-items: center;
}

:deep(.filter-check) {
  width: 16px;
  margin-right: 8px;
  color: var(--primary);
  font-size: 14px;
  flex-shrink: 0;
}

:deep(.filter-check-placeholder) {
  display: inline-block;
  width: 16px;
  margin-right: 8px;
  flex-shrink: 0;
}
</style>
