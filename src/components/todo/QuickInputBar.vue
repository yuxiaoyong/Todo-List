<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useI18n } from "vue-i18n";
import { ElMessage } from "element-plus";
import { Loading, MagicStick } from "@element-plus/icons-vue";
import type { AiParsedTask } from "../../api";
import { aiGatewayApi } from "../../api";
import { useAiGatewayStore } from "../../stores/aiGateway";
import { useCategoryStore } from "../../stores/category";
import { useTagStore } from "../../stores/tag";
import { useUiStore } from "../../stores/ui";
import { useTodoStore } from "../../stores/todo";
import AiTaskPreviewDialog from "./AiTaskPreviewDialog.vue";

const { t } = useI18n();
const emit = defineEmits<{ created: [openDetail: boolean]; refresh: [] }>();

const uiStore = useUiStore();
const todoStore = useTodoStore();
const aiStore = useAiGatewayStore();
const categoryStore = useCategoryStore();
const tagStore = useTagStore();

const title = ref("");
const creating = ref(false);
const parsing = ref(false);
const previewOpen = ref(false);
const parsedTask = ref<AiParsedTask | null>(null);

onMounted(() => {
  if (!aiStore.ready) {
    void aiStore.load().catch(() => {
      // silent: AI button stays hidden until enabled
    });
  }
});

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

async function parseWithAi() {
  const value = title.value.trim();
  if (!value) {
    ElMessage.warning(t("aiTask.inputRequired"));
    return;
  }
  if (parsing.value) return;
  parsing.value = true;
  try {
    const categories = categoryStore.categories.map((c) => ({ id: c.id, name: c.name }));
    const tags = tagStore.tags.map((tag) => ({ id: tag.id, name: tag.name }));
    parsedTask.value = await aiGatewayApi.parseTask({ text: value, categories, tags });
    previewOpen.value = true;
  } catch (error) {
    console.error("ai parse task failed", error);
    const message =
      typeof error === "string"
        ? error
        : error instanceof Error
          ? error.message
          : t("aiTask.parseFailed");
    ElMessage.error(message);
  } finally {
    parsing.value = false;
  }
}

function onAiCreated(openDetail: boolean) {
  title.value = "";
  emit("refresh");
  emit("created", openDetail);
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
      class="quick-input__field"
      :class="{ 'has-ai': aiStore.isActive }"
      @keydown="onKeydown"
    >
      <template #prepend>+</template>
      <template v-if="aiStore.isActive" #suffix>
        <button
          type="button"
          class="quick-input__ai"
          :title="t('aiTask.parseHint')"
          :disabled="creating || parsing"
          @click="parseWithAi"
        >
          <el-icon v-if="parsing" class="is-loading">
            <Loading />
          </el-icon>
          <el-icon v-else>
            <MagicStick />
          </el-icon>
        </button>
      </template>
    </el-input>

    <AiTaskPreviewDialog
      v-model="previewOpen"
      :parsed="parsedTask"
      @created="onAiCreated"
      @refresh="emit('refresh')"
    />
  </div>
</template>

<style scoped>
.quick-input {
  padding: 12px 20px;
  border-bottom: 1px solid var(--border-light);
  background: var(--surface-muted);
  flex-shrink: 0;
}

.quick-input__field.has-ai :deep(.el-input__wrapper) {
  padding-right: 4px;
}

.quick-input__field :deep(.el-input__suffix) {
  display: flex;
  align-items: center;
}

.quick-input__ai {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  padding: 0;
  border: none;
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--primary);
  cursor: pointer;
  transition:
    background 0.15s,
    color 0.15s;
}

.quick-input__ai:hover:not(:disabled) {
  background: var(--primary-light);
  color: var(--primary-hover);
}

.quick-input__ai:disabled {
  cursor: not-allowed;
  opacity: 0.55;
}

.quick-input__ai .el-icon {
  font-size: 16px;
}
</style>
