<script setup lang="ts">
import { computed, ref } from "vue";
import { useI18n } from "vue-i18n";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { ElMessage } from "element-plus";
import { todoApi } from "../api";
import { useCategoryStore } from "../stores/category";
import { useShortcutStore } from "../stores/shortcut";

const { t } = useI18n();
const categoryStore = useCategoryStore();
const shortcutStore = useShortcutStore();

const invokeHint = computed(() => shortcutStore.quickCaptureLabel);

const title = ref("");
const categoryId = ref<number | undefined>();

async function submit() {
  const value = title.value.trim();
  if (!value) {
    ElMessage.warning(t("quickCapture.titleRequired"));
    return;
  }
  await todoApi.quickCreate({
    title: value,
    categoryId: categoryId.value,
  });
  title.value = "";
  ElMessage.success(t("quickCapture.added"));
  await getCurrentWindow().hide();
}

function cancel() {
  title.value = "";
  getCurrentWindow().hide();
}

categoryStore.fetchAll();
</script>

<template>
  <div class="quick-capture">
    <div class="capture-row">
      <el-input
        v-model="title"
        class="title-input"
        size="small"
        :placeholder="t('quickCapture.placeholder')"
        autofocus
        @keyup.enter="submit"
        @keyup.esc="cancel"
      />
      <el-select
        v-model="categoryId"
        class="category-select"
        :placeholder="t('quickCapture.category')"
        clearable
        size="small"
        fit-input-width
      >
        <el-option
          v-for="cat in categoryStore.categories"
          :key="cat.id"
          :label="cat.name"
          :value="cat.id"
        />
      </el-select>
    </div>
    <div class="hint">{{ t("quickCapture.hint", { shortcut: invokeHint }) }}</div>
  </div>
</template>

<style scoped>
.quick-capture {
  box-sizing: border-box;
  padding: 12px 16px;
  background: var(--panel-bg);
  height: 100vh;
  overflow: hidden;
}

.capture-row {
  display: flex;
  gap: 8px;
}

.title-input {
  flex: 1;
}

.category-select {
  width: 120px;
}

.hint {
  margin-top: 8px;
  font-size: 12px;
  color: var(--text-secondary);
}
</style>
