<script setup lang="ts">
import { ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { useCategoryStore } from "../../stores/category";
import { useTagStore } from "../../stores/tag";
import { todoApi } from "../../api";
import { useUiStore } from "../../stores/ui";

const { t } = useI18n();
const props = defineProps<{ modelValue: boolean }>();
const emit = defineEmits<{ "update:modelValue": [value: boolean]; created: [] }>();

const categoryStore = useCategoryStore();
const tagStore = useTagStore();
const uiStore = useUiStore();

const title = ref("");
const categoryId = ref<number | undefined>();
const tagIds = ref<number[]>([]);
const priority = ref("medium");
const dueDate = ref<string>();
const submitting = ref(false);

watch(
  () => props.modelValue,
  (open) => {
    if (!open) return;
    title.value = "";
    categoryId.value =
      uiStore.categoryFilter === "all" ? undefined : uiStore.categoryFilter;
    tagIds.value = [];
    priority.value = "medium";
    dueDate.value = undefined;
  },
);

async function submit() {
  const trimmed = title.value.trim();
  if (!trimmed || submitting.value) return;
  submitting.value = true;
  try {
    const detail = await todoApi.create({
      title: trimmed,
      categoryId: categoryId.value,
      tagIds: tagIds.value.length ? tagIds.value : undefined,
      priority: priority.value,
      dueDate: dueDate.value,
    });
    uiStore.openDetail(detail.id);
    emit("update:modelValue", false);
    emit("created");
  } finally {
    submitting.value = false;
  }
}
</script>

<template>
  <el-dialog
    :model-value="modelValue"
    :title="t('task.newTaskTitle')"
    width="480px"
    destroy-on-close
    @update:model-value="emit('update:modelValue', $event)"
    @keyup.enter="submit"
  >
    <el-form label-width="72px" @submit.prevent="submit">
      <el-form-item :label="t('task.title')" required>
        <el-input v-model="title" :placeholder="t('task.titlePlaceholder')" autofocus />
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
      <el-form-item :label="t('task.dueDate')">
        <el-date-picker
          v-model="dueDate"
          type="date"
          :placeholder="t('task.selectDate')"
          value-format="YYYY-MM-DD"
          style="width: 100%"
        />
      </el-form-item>
    </el-form>
    <template #footer>
      <el-button @click="emit('update:modelValue', false)">{{ t("common.cancel") }}</el-button>
      <el-button type="primary" :loading="submitting" @click="submit">{{ t("common.create") }}</el-button>
    </template>
  </el-dialog>
</template>
