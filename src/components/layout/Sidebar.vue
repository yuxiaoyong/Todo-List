<script setup lang="ts">
import { computed, ref } from "vue";
import { useI18n } from "vue-i18n";
import {
  Plus,
  Setting,
  Notebook,
  House,
  TrendCharts,
  Reading,
  VideoCamera,
  Folder,
  Delete,
} from "@element-plus/icons-vue";
import { ElMessageBox } from "element-plus";
import { useCategoryStore } from "../../stores/category";
import { useTagStore } from "../../stores/tag";
import { useTodoStore } from "../../stores/todo";
import SettingsDialog from "../settings/SettingsDialog.vue";
import { useUiStore, type TimeFilter } from "../../stores/ui";
import { computeStats } from "../../utils/stats";

const { t } = useI18n();
const emit = defineEmits<{ refresh: [] }>();
const categoryStore = useCategoryStore();
const tagStore = useTagStore();
const todoStore = useTodoStore();
const uiStore = useUiStore();

const newCategoryName = ref("");
const showCategoryInput = ref(false);
const newTagName = ref("");
const showTagInput = ref(false);
const settingsOpen = ref(false);

const categories = computed(() => categoryStore.categories);
const tags = computed(() => tagStore.tags);
const stats = computed(() => computeStats(todoStore.allTodos));

const timeTabs = computed(() => [
  { key: "all" as TimeFilter, label: t("timeFilter.all") },
  { key: "year" as TimeFilter, label: t("timeFilter.year") },
  { key: "month" as TimeFilter, label: t("timeFilter.month") },
  { key: "week" as TimeFilter, label: t("timeFilter.week") },
  { key: "today" as TimeFilter, label: t("timeFilter.today") },
]);

const categoryIcons = [House, TrendCharts, Reading, VideoCamera, Folder];

function getCategoryIcon(index: number) {
  return categoryIcons[index % categoryIcons.length];
}

async function addCategory() {
  const name = newCategoryName.value.trim();
  if (!name) return;
  await categoryStore.create(name);
  newCategoryName.value = "";
  showCategoryInput.value = false;
  emit("refresh");
}

async function removeCategory(id: number, name: string) {
  await ElMessageBox.confirm(t("sidebar.deleteCategory", { name }), t("common.hint"), {
    type: "warning",
  });
  await categoryStore.remove(id);
  if (uiStore.categoryFilter === id) uiStore.selectAllCategories();
  emit("refresh");
}

async function addTag() {
  const name = newTagName.value.trim();
  if (!name) return;
  await tagStore.create(name);
  newTagName.value = "";
  showTagInput.value = false;
  emit("refresh");
}

async function removeTag(id: number, name: string) {
  await ElMessageBox.confirm(t("sidebar.deleteTag", { name }), t("common.hint"), {
    type: "warning",
  });
  await tagStore.remove(id);
  if (uiStore.selectedTagIds.includes(id)) {
    uiStore.selectedTagIds = uiStore.selectedTagIds.filter((tagId) => tagId !== id);
  }
  emit("refresh");
}

function toggleTagFilter(id: number) {
  uiStore.toggleTag(id);
  emit("refresh");
}

function isTagActive(id: number) {
  return uiStore.selectedTagIds.includes(id);
}

function selectTimeFilter(key: TimeFilter) {
  uiStore.setTimeFilter(key);
  emit("refresh");
}

function isAllCategoryActive() {
  return uiStore.viewMode !== "trash" && uiStore.categoryFilter === "all";
}

function isCategoryActive(id: number) {
  return (
    uiStore.viewMode !== "trash" &&
    uiStore.categoryFilter !== "all" &&
    Number(uiStore.categoryFilter) === Number(id)
  );
}

function selectCategory(id: number) {
  uiStore.selectCategory(id);
  emit("refresh");
}

function selectAllCategories() {
  uiStore.selectAllCategories();
  emit("refresh");
}

function selectTrash() {
  uiStore.selectTrash();
  emit("refresh");
}
</script>

<template>
  <aside class="sidebar">
    <div class="sidebar-header">
      <div class="brand">
        <el-icon class="brand-icon" :size="22"><Notebook /></el-icon>
        <span class="brand-title">Todo List</span>
      </div>
      <el-button
        :icon="Setting"
        circle
        text
        class="settings-btn"
        :title="t('sidebar.settings')"
        @click="settingsOpen = true"
      />
    </div>

    <SettingsDialog v-model="settingsOpen" />

    <div class="time-tabs">
      <button
        v-for="tab in timeTabs"
        :key="tab.key"
        type="button"
        class="time-tab"
        :class="{ active: uiStore.timeFilter === tab.key }"
        @click="selectTimeFilter(tab.key)"
      >
        {{ tab.label }}
      </button>
    </div>

    <div class="stats-grid">
      <div class="stat-card">
        <div class="stat-value">{{ stats.total }}</div>
        <div class="stat-label">{{ t("sidebar.totalTasks") }}</div>
      </div>
      <div class="stat-card">
        <div class="stat-value">{{ stats.completed }}</div>
        <div class="stat-label">{{ t("sidebar.completed") }}</div>
      </div>
      <div class="stat-card">
        <div class="stat-value">{{ stats.completionRate }}%</div>
        <div class="stat-label">{{ t("sidebar.completionRate") }}</div>
      </div>
      <div class="stat-card">
        <div class="stat-value">{{ stats.noDeadline }}</div>
        <div class="stat-label">{{ t("sidebar.noDeadline") }}</div>
      </div>
    </div>

    <div class="sidebar-body">
      <div class="section-header">
        <span>{{ t("sidebar.categories") }}</span>
      </div>

      <div class="category-list">
      <div
        class="category-item"
        :class="{ active: isAllCategoryActive() }"
        @click="selectAllCategories"
      >
        <el-icon class="cat-icon all-icon"><Notebook /></el-icon>
        <span class="cat-name">{{ t("sidebar.allCategories") }}</span>
        <span class="cat-count">{{ stats.total }}</span>
        <span class="cat-action-slot" aria-hidden="true" />
      </div>

      <div
        v-for="(cat, index) in categories"
        :key="cat.id"
        class="category-item"
        :class="{ active: isCategoryActive(cat.id) }"
        @click="selectCategory(cat.id)"
      >
        <el-icon class="cat-icon" :style="{ color: cat.color }">
          <component :is="getCategoryIcon(index)" />
        </el-icon>
        <span class="cat-name">{{ cat.name }}</span>
        <span class="cat-count">{{ cat.todoCount }}</span>
        <el-button
          link
          type="danger"
          class="cat-delete"
          @click.stop="removeCategory(cat.id, cat.name)"
        >
          ×
        </el-button>
      </div>

      <div v-if="showCategoryInput" class="add-category-form">
        <el-input
          v-model="newCategoryName"
          size="small"
          :placeholder="t('sidebar.categoryName')"
          @keyup.enter="addCategory"
        />
        <div class="form-actions">
          <el-button size="small" type="primary" @click="addCategory">{{ t("common.confirm") }}</el-button>
          <el-button size="small" @click="showCategoryInput = false">{{ t("common.cancel") }}</el-button>
        </div>
      </div>
      <button v-else type="button" class="add-category-btn" @click="showCategoryInput = true">
        <el-icon><Plus /></el-icon>
        {{ t("sidebar.addCategory") }}
      </button>
      </div>

      <div class="section-divider" />

      <div class="section-header">
        <span>{{ t("sidebar.tags") }}</span>
      </div>

      <div class="category-list">
      <div
        v-for="tag in tags"
        :key="tag.id"
        class="category-item"
        :class="{ active: isTagActive(tag.id) }"
        @click="toggleTagFilter(tag.id)"
      >
        <span class="tag-dot" :style="{ background: tag.color }" />
        <span class="cat-name">#{{ tag.name }}</span>
        <span class="cat-count">{{ tag.todoCount }}</span>
        <el-button
          link
          type="danger"
          class="cat-delete"
          @click.stop="removeTag(tag.id, tag.name)"
        >
          ×
        </el-button>
      </div>

      <div v-if="showTagInput" class="add-category-form">
        <el-input
          v-model="newTagName"
          size="small"
          :placeholder="t('sidebar.tagName')"
          @keyup.enter="addTag"
        />
        <div class="form-actions">
          <el-button size="small" type="primary" @click="addTag">{{ t("common.confirm") }}</el-button>
          <el-button size="small" @click="showTagInput = false">{{ t("common.cancel") }}</el-button>
        </div>
      </div>
      <button v-else type="button" class="add-category-btn" @click="showTagInput = true">
        <el-icon><Plus /></el-icon>
        {{ t("sidebar.addTag") }}
      </button>

      <div
        class="category-item trash-item"
        :class="{ active: uiStore.viewMode === 'trash' }"
        @click="selectTrash"
      >
        <el-icon class="cat-icon"><Delete /></el-icon>
        <span class="cat-name">{{ t("sidebar.trash") }}</span>
      </div>
      </div>
    </div>
  </aside>
</template>

<style scoped>
.sidebar {
  width: var(--sidebar-width);
  flex-shrink: 0;
  background: var(--panel-bg);
  border-right: 1px solid var(--border-color);
  display: flex;
  flex-direction: column;
  height: 100vh;
  overflow: hidden;
}

.sidebar-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 20px 16px 12px;
}

.brand {
  display: flex;
  align-items: center;
  gap: 10px;
}

.brand-icon {
  color: var(--primary);
}

.brand-title {
  font-size: 18px;
  font-weight: 600;
  color: var(--text-primary);
}

.settings-btn {
  color: var(--text-secondary);
}

.time-tabs {
  display: flex;
  margin: 0 12px 16px;
  border: 1px solid var(--border-color);
  border-radius: 18px;
  overflow: hidden;
  background: var(--surface-muted);
}

.time-tab {
  flex: 1;
  min-width: 0;
  border: none;
  border-right: 1px solid var(--border-color);
  border-radius: 0;
  background: transparent;
  padding: 7px 4px;
  font-size: 12px;
  color: var(--text-secondary);
  cursor: pointer;
  transition: background 0.2s, color 0.2s;
}

.time-tab:last-child {
  border-right: none;
}

.time-tab:hover:not(.active) {
  color: var(--primary);
  background: var(--primary-light);
}

.time-tab.active {
  background: var(--primary);
  color: #fff;
}

.stats-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 10px;
  padding: 0 12px 20px;
}

.stat-card {
  background: var(--surface-muted);
  border: 1px solid var(--border-light);
  border-radius: var(--radius);
  padding: 12px 14px;
  text-align: center;
}

.stat-value {
  font-size: 22px;
  font-weight: 600;
  color: var(--text-primary);
  line-height: 1.2;
}

.stat-label {
  font-size: 12px;
  color: var(--text-secondary);
  margin-top: 4px;
}

.section-header {
  display: flex;
  align-items: center;
  margin: 0 8px 10px;
  padding: 0 0 0 9px;
  min-height: 34px;
  border-left: 3px solid var(--primary);
  font-size: 15px;
  font-weight: 600;
  color: var(--text-primary);
  line-height: 1;
  box-sizing: border-box;
}

.section-divider {
  margin: 16px 8px 10px;
  border-top: 1px solid var(--border-light);
}

.sidebar-body {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  padding-bottom: 8px;
}

.category-list {
  padding: 0 8px 12px;
}

.category-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 12px;
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: background 0.15s;
  margin-bottom: 2px;
}

.category-item:hover:not(.active) {
  background: var(--nav-hover);
}

.category-item.active {
  background: var(--nav-active);
  color: var(--primary);
  box-shadow: inset 3px 0 0 var(--primary);
}

.category-item.active:hover {
  background: var(--nav-active-hover);
}

.category-item.active .cat-name {
  font-weight: 600;
  color: var(--primary);
}

.category-item.active .cat-count {
  color: var(--primary);
}

.cat-icon {
  font-size: 18px;
  flex-shrink: 0;
}

.all-icon {
  color: var(--primary);
}

.tag-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  flex-shrink: 0;
}

.cat-name {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: 14px;
}

.cat-count {
  font-size: 12px;
  color: var(--text-secondary);
  width: 32px;
  min-width: 32px;
  text-align: right;
  flex-shrink: 0;
  font-variant-numeric: tabular-nums;
}

.cat-action-slot {
  width: 20px;
  min-width: 20px;
  flex-shrink: 0;
}

.cat-delete {
  opacity: 0;
  width: 20px;
  min-width: 20px;
  padding: 0;
  margin: 0;
  font-size: 16px;
  flex-shrink: 0;
}

.category-item:hover .cat-delete {
  opacity: 1;
}

.add-category-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  width: calc(100% - 8px);
  margin: 8px 4px;
  padding: 10px;
  border: 1px dashed var(--border-color);
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--text-secondary);
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s;
}

.add-category-btn:hover {
  border-color: var(--primary);
  color: var(--primary);
  background: var(--primary-light);
}

.add-category-form {
  padding: 8px 4px;
}

.form-actions {
  display: flex;
  gap: 8px;
  margin-top: 8px;
}

.trash-item {
  margin-top: 8px;
  border-top: 1px solid var(--border-light);
  padding-top: 14px;
  border-radius: 0;
}

.trash-item .cat-icon {
  color: var(--text-secondary);
}
</style>
