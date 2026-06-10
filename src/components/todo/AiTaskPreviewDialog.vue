<script setup lang="ts">
import { ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { ElMessage } from "element-plus";
import type { AiParsedTask } from "../../api";
import { todoApi } from "../../api";
import { useCategoryStore } from "../../stores/category";
import { useTagStore } from "../../stores/tag";
import { useUiStore } from "../../stores/ui";

const { t } = useI18n();
const props = defineProps<{ modelValue: boolean; parsed: AiParsedTask | null }>();
const emit = defineEmits<{
  "update:modelValue": [value: boolean];
  created: [openDetail: boolean];
  refresh: [];
}>();

const categoryStore = useCategoryStore();
const tagStore = useTagStore();
const uiStore = useUiStore();

const title = ref("");
const categoryId = ref<number | undefined>();
const tagIds = ref<number[]>([]);
const priority = ref("medium");
const startDate = ref<string>();
const dueDate = ref<string>();
const submitting = ref(false);

watch(
  () => props.modelValue,
  (open) => {
    if (!open || !props.parsed) return;
    title.value = props.parsed.title;
    categoryId.value = props.parsed.categoryId ?? undefined;
    tagIds.value = [...props.parsed.tagIds];
    priority.value = props.parsed.priority || "medium";
    startDate.value = props.parsed.startDate ?? undefined;
    dueDate.value = props.parsed.dueDate ?? undefined;
  },
);

async function submit(openDetail: boolean) {
  const trimmed = title.value.trim();
  if (!trimmed || submitting.value) return;
  submitting.value = true;
  try {
    const detail = await todoApi.create({
      title: trimmed,
      categoryId: categoryId.value,
      tagIds: tagIds.value.length ? tagIds.value : undefined,
      priority: priority.value,
      startDate: startDate.value,
      dueDate: dueDate.value,
    });
    emit("update:modelValue", false);
    emit("refresh");
    emit("created", openDetail);
    ElMessage.success(t("task.created"));
    if (openDetail) {
      uiStore.openDetail(detail.id);
    }
  } catch {
    // tauriInvoke handles errors
  } finally {
    submitting.value = false;
  }
}
</script>

<template>
  <el-dialog
    :model-value="modelValue"
    :title="t('aiTask.previewTitle')"
    width="480px"
    destroy-on-close
    append-to-body
    align-center
    :z-index="4000"
    class="app-dialog ai-preview-dialog"
    @update:model-value="emit('update:modelValue', $event)"
  >
    <p class="ai-preview-desc">{{ t("aiTask.previewDesc") }}</p>
    <el-form label-width="72px" @submit.prevent="submit(false)">
      <el-form-item :label="t('task.title')" required>
        <el-input v-model="title" :placeholder="t('task.titlePlaceholder')" autofocus />
      </el-form-item>
      <el-form-item :label="t('task.startDate')">
        <el-date-picker
          v-model="startDate"
          type="date"
          :placeholder="t('task.selectDate')"
          value-format="YYYY-MM-DD"
          clearable
          style="width: 100%"
        />
      </el-form-item>
      <el-form-item :label="t('task.dueDate')">
        <el-date-picker
          v-model="dueDate"
          type="date"
          :placeholder="t('task.selectDate')"
          value-format="YYYY-MM-DD"
          clearable
          style="width: 100%"
        />
      </el-form-item>
      <el-form-item :label="t('taskDetail.category')">
        <el-select
          v-model="categoryId"
          :placeholder="t('task.selectCategory')"
          clearable
          style="width: 100%"
        >
          <el-option
            v-for="cat in categoryStore.categories"
            :key="cat.id"
            :label="cat.name"
            :value="cat.id"
          />
        </el-select>
      </el-form-item>
      <el-form-item :label="t('task.tags')">
        <el-select
          v-model="tagIds"
          multiple
          collapse-tags
          filterable
          :placeholder="t('task.selectTags')"
          style="width: 100%"
        >
          <el-option v-for="tag in tagStore.tags" :key="tag.id" :label="tag.name" :value="tag.id" />
        </el-select>
      </el-form-item>
      <el-form-item :label="t('task.priority')">
        <el-select v-model="priority" style="width: 100%">
          <el-option :label="t('priority.high')" value="high" />
          <el-option :label="t('priority.medium')" value="medium" />
          <el-option :label="t('priority.low')" value="low" />
        </el-select>
      </el-form-item>
    </el-form>
    <template #footer>
      <el-button @click="emit('update:modelValue', false)">{{ t("common.cancel") }}</el-button>
      <el-button :loading="submitting" @click="submit(false)">{{ t("common.create") }}</el-button>
      <el-button type="primary" :loading="submitting" @click="submit(true)">
        {{ t("aiTask.createAndOpen") }}
      </el-button>
    </template>
  </el-dialog>
</template>

<style scoped>
.ai-preview-desc {
  margin: 0 0 16px;
  font-size: 13px;
  line-height: 1.5;
  color: var(--text-secondary);
}
</style>
