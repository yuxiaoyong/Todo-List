<script setup lang="ts">
import { ref } from "vue";
import { useI18n } from "vue-i18n";
import { ElMessage } from "element-plus";
import { useUiStore } from "../../stores/ui";
import { useTodoStore } from "../../stores/todo";

const { t } = useI18n();
const emit = defineEmits<{ created: [openDetail: boolean]; refresh: [] }>();

const uiStore = useUiStore();
const todoStore = useTodoStore();

const title = ref("");
const creating = ref(false);

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

async function create(openDetail: boolean) {
  const value = title.value.trim();
  if (!value) {
    ElMessage.warning(t("task.titleRequired"));
    return;
  }
  if (creating.value) return;
  creating.value = true;
  try {
    const detail = await todoStore.quickCreate(buildCreateInput(value));
    title.value = "";
    ElMessage.success(t("task.created"));
    emit("refresh");
    emit("created", openDetail);
    if (openDetail) {
      uiStore.openDetail(detail.id);
    }
  } finally {
    creating.value = false;
  }
}

function onKeydown(event: KeyboardEvent) {
  if (event.key === "Enter" && event.shiftKey) {
    event.preventDefault();
    create(true);
  } else if (event.key === "Enter") {
    event.preventDefault();
    create(false);
  }
}
</script>

<template>
  <div class="quick-input">
    <el-input
      v-model="title"
      :placeholder="t('task.quickInputPlaceholder')"
      :disabled="creating"
      @keydown="onKeydown"
    >
      <template #prepend>+</template>
    </el-input>
  </div>
</template>

<style scoped>
.quick-input {
  padding: 12px 20px;
  border-bottom: 1px solid var(--border-light);
  background: var(--surface-muted);
  flex-shrink: 0;
}
</style>
