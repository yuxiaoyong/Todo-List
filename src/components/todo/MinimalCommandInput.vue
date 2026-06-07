<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from "vue";
import type { ElInput } from "element-plus";
import { useI18n } from "vue-i18n";
import { ElMessage } from "element-plus";
import { useUiStore } from "../../stores/ui";
import { useTodoStore } from "../../stores/todo";

const { t } = useI18n();
const emit = defineEmits<{ created: [openDetail: boolean]; refresh: [] }>();

const uiStore = useUiStore();
const todoStore = useTodoStore();

const inputRef = ref<InstanceType<typeof ElInput> | null>(null);
const mode = ref<"search" | "add">("search");
const inputValue = ref(uiStore.searchQuery);
const creating = ref(false);

let searchTimer: ReturnType<typeof setTimeout> | null = null;

const placeholder = computed(() =>
  mode.value === "add" ? t("minimal.addPlaceholder") : t("minimal.searchPlaceholder"),
);

function buildCreateInput(taskTitle: string) {
  const categoryId =
    uiStore.categoryFilter === "all" ? undefined : uiStore.categoryFilter;
  const tagIds = uiStore.selectedTagIds.length ? [...uiStore.selectedTagIds] : undefined;
  return {
    title: taskTitle,
    categoryId,
    tagIds,
    priority: "medium",
  };
}

function scheduleSearch() {
  if (searchTimer) clearTimeout(searchTimer);
  searchTimer = setTimeout(() => {
    emit("refresh");
  }, 300);
}

watch(inputValue, (value) => {
  if (mode.value !== "search") return;
  uiStore.setSearchQuery(value);
  scheduleSearch();
});

function focusInput() {
  nextTick(() => {
    const el = inputRef.value?.$el?.querySelector("input") as HTMLInputElement | null;
    el?.focus();
  });
}

function enterAddMode() {
  mode.value = "add";
  inputValue.value = "";
  focusInput();
}

function exitAddMode() {
  mode.value = "search";
  inputValue.value = uiStore.searchQuery;
}

async function create(openDetail: boolean) {
  const value = inputValue.value.trim();
  if (!value) {
    ElMessage.warning(t("task.titleRequired"));
    return;
  }
  if (creating.value) return;
  creating.value = true;
  try {
    const detail = await todoStore.quickCreate(buildCreateInput(value));
    ElMessage.success(t("task.created"));
    emit("refresh");
    emit("created", openDetail);
    exitAddMode();
    if (openDetail) {
      uiStore.openDetail(detail.id);
    }
  } finally {
    creating.value = false;
  }
}

function onKeydown(event: KeyboardEvent) {
  if (event.key === "/" && mode.value === "search" && !inputValue.value) {
    event.preventDefault();
    enterAddMode();
    return;
  }

  if (event.key === "Escape" && mode.value === "add") {
    event.preventDefault();
    exitAddMode();
    return;
  }

  if (mode.value !== "add") return;

  if (event.key === "Enter" && event.shiftKey) {
    event.preventDefault();
    void create(true);
  } else if (event.key === "Enter") {
    event.preventDefault();
    void create(false);
  }
}

function isTypingTarget(target: EventTarget | null) {
  if (!(target instanceof HTMLElement)) return false;
  const tag = target.tagName;
  return tag === "INPUT" || tag === "TEXTAREA" || target.isContentEditable;
}

function onWindowKeydown(event: KeyboardEvent) {
  if (event.key !== "/" || event.ctrlKey || event.metaKey || event.altKey) return;
  if (isTypingTarget(event.target)) return;
  event.preventDefault();
  enterAddMode();
}

onMounted(() => {
  window.addEventListener("keydown", onWindowKeydown);
});

onUnmounted(() => {
  if (searchTimer) clearTimeout(searchTimer);
  window.removeEventListener("keydown", onWindowKeydown);
});
</script>

<template>
  <div class="minimal-command-input" :class="{ 'is-add-mode': mode === 'add' }">
    <el-input
      ref="inputRef"
      v-model="inputValue"
      :placeholder="placeholder"
      :disabled="creating"
      clearable
      @keydown="onKeydown"
    />
  </div>
</template>

<style scoped>
.minimal-command-input {
  padding: 8px 14px 10px;
  flex-shrink: 0;
}

.minimal-command-input :deep(.el-input__wrapper) {
  border-radius: 10px;
  box-shadow: 0 0 0 1px var(--border-color) inset;
  background: var(--panel-bg);
  transition: box-shadow 0.15s;
}

.minimal-command-input.is-add-mode :deep(.el-input__wrapper) {
  box-shadow: 0 0 0 1px color-mix(in srgb, var(--primary) 45%, var(--border-color)) inset;
}
</style>
