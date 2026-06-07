<script setup lang="ts">
import { nextTick, onMounted, onUnmounted, ref } from "vue";
import { listen } from "@tauri-apps/api/event";
import Sidebar from "../components/layout/Sidebar.vue";
import TaskListPanel from "../components/layout/TaskListPanel.vue";
import TaskDetailPanel from "../components/layout/TaskDetailPanel.vue";
import { useCategoryStore } from "../stores/category";
import { useKanbanColumnStore } from "../stores/kanbanColumn";
import { useTagStore } from "../stores/tag";
import { useTodoStore } from "../stores/todo";
const listPanel = ref<InstanceType<typeof TaskListPanel> | null>(null);
const categoryStore = useCategoryStore();
const kanbanColumnStore = useKanbanColumnStore();
const tagStore = useTagStore();
const todoStore = useTodoStore();

async function refreshAll() {
  try {
    await Promise.all([
      categoryStore.fetchAll(),
      kanbanColumnStore.fetchAll(),
      tagStore.fetchAll(),
      todoStore.fetchAllTodos(),
    ]);
    await nextTick();
    await listPanel.value?.refresh();
  } catch (err) {
    console.error("Failed to refresh app data:", err);
  }
}

let unlisten: (() => void) | undefined;

onMounted(async () => {
  await refreshAll();
  unlisten = await listen("todo-changed", () => {
    void refreshAll();
  });
});

onUnmounted(() => {
  unlisten?.();
});
</script>

<template>
  <div class="app-shell">
    <Sidebar @refresh="refreshAll" />
    <TaskListPanel ref="listPanel" @refresh="refreshAll" />
    <TaskDetailPanel @refresh="refreshAll" />
  </div>
</template>
