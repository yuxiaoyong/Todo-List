import { ref, watch, type Ref } from "vue";
import { todoApi } from "../api";
import type { UpdateTodoInput } from "../types";

const DEFAULT_IDLE_DELAY = 3000;

function buildSnapshot(value: UpdateTodoInput): string {
  return JSON.stringify({
    id: value.id,
    title: value.title,
    contentHtml: value.contentHtml,
    completed: value.completed,
    priority: value.priority,
    startDate: value.startDate ?? null,
    dueDate: value.dueDate ?? null,
    categoryId: value.categoryId ?? null,
    tagIds: [...value.tagIds].sort((a, b) => a - b),
    assignee: value.assignee ?? null,
    sortOrder: value.sortOrder ?? null,
    pinned: value.pinned ?? null,
    kanbanColumnId: value.kanbanColumnId ?? null,
    recurrenceJson: value.recurrenceJson ?? null,
  });
}

export function useAutoSave(source: Ref<UpdateTodoInput | null>, delay = DEFAULT_IDLE_DELAY) {
  const saving = ref(false);
  let idleTimer: ReturnType<typeof setTimeout> | null = null;
  let lastSavedSnapshot = "";
  let pendingValue: UpdateTodoInput | null = null;

  function clearIdleTimer() {
    if (idleTimer) {
      clearTimeout(idleTimer);
      idleTimer = null;
    }
  }

  async function performSave(value: UpdateTodoInput, quiet = true) {
    const snapshot = buildSnapshot(value);
    if (snapshot === lastSavedSnapshot) return;

    saving.value = true;
    try {
      await todoApi.update({ ...value, quiet });
      lastSavedSnapshot = snapshot;
    } catch (error) {
      console.error("auto save failed", error);
    } finally {
      saving.value = false;
    }
  }

  function scheduleIdleSave(value: UpdateTodoInput) {
    pendingValue = value;
    clearIdleTimer();
    idleTimer = setTimeout(() => {
      if (pendingValue) void performSave(pendingValue);
    }, delay);
  }

  watch(
    source,
    (value) => {
      if (!value) {
        clearIdleTimer();
        pendingValue = null;
        return;
      }
      scheduleIdleSave(value);
    },
    { deep: true },
  );

  async function flush(quiet = true) {
    clearIdleTimer();
    if (!source.value) return;
    await performSave(source.value, quiet);
  }

  function resetSnapshot(value?: UpdateTodoInput | null) {
    lastSavedSnapshot = value ? buildSnapshot(value) : "";
  }

  return { saving, flush, resetSnapshot };
}
