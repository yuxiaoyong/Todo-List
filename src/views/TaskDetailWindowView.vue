<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from "vue";
import { useRoute, useRouter } from "vue-router";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import TaskDetailPanel from "../components/layout/TaskDetailPanel.vue";
import { useCategoryStore } from "../stores/category";
import { useKanbanColumnStore } from "../stores/kanbanColumn";
import { useTagStore } from "../stores/tag";

const route = useRoute();
const router = useRouter();
const categoryStore = useCategoryStore();
const kanbanColumnStore = useKanbanColumnStore();
const tagStore = useTagStore();

const ready = ref(false);

function parseTaskId(value: unknown): number | null {
  const id = Number(value);
  return Number.isFinite(id) && id > 0 ? id : null;
}

function parseTaskIdFromHash(): number | null {
  const match = window.location.hash.match(/\/task-detail\/(\d+)/);
  return match ? parseTaskId(match[1]) : null;
}

const taskId = computed(() => {
  const fromRoute = parseTaskId(route.params.id);
  if (fromRoute) return fromRoute;
  return parseTaskIdFromHash();
});

let unlistenNavigate: (() => void) | undefined;

onMounted(async () => {
  await router.isReady();

  if (!taskId.value) {
    const fromHash = parseTaskIdFromHash();
    if (fromHash) {
      await router.replace(`/task-detail/${fromHash}`);
    }
  }

  ready.value = true;

  void Promise.all([
    categoryStore.fetchAll(),
    kanbanColumnStore.fetchAll(),
    tagStore.fetchAll(),
  ]);

  unlistenNavigate = await listen<number>("task-detail-navigate", (event) => {
    const id = parseTaskId(event.payload);
    if (!id) return;
    void router.replace(`/task-detail/${id}`);
  });
});

async function closeWindow() {
  await getCurrentWindow().close();
}

onUnmounted(() => {
  unlistenNavigate?.();
});
</script>

<template>
  <div class="task-detail-window">
    <TaskDetailPanel v-if="ready && taskId" standalone :task-id="taskId" />
    <div v-else-if="ready" class="task-detail-empty">
      <p>{{ $t("taskDetail.notFound") }}</p>
      <el-button type="primary" @click="closeWindow">{{ $t("common.close") }}</el-button>
    </div>
  </div>
</template>

<style scoped>
.task-detail-window {
  height: 100vh;
  overflow: hidden;
  background: var(--panel-bg);
}

.task-detail-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 16px;
  height: 100%;
  color: var(--text-secondary);
}
</style>
