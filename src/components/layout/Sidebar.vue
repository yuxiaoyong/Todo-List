<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, reactive, ref, watch } from "vue";
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
  Edit,
} from "@element-plus/icons-vue";
import { ElMessage, ElMessageBox } from "element-plus";
import { useDraggable } from "vue-draggable-plus";
import { useCategoryStore } from "../../stores/category";
import { useTagStore } from "../../stores/tag";
import { useTodoStore } from "../../stores/todo";
import SettingsDialog from "../settings/SettingsDialog.vue";
import { useUiStore, type TimeFilter } from "../../stores/ui";
import { computeStats } from "../../utils/stats";
import { nextKanbanColor } from "../../utils/kanban";
import type { Category, Tag } from "../../types";

const { t } = useI18n();
const emit = defineEmits<{ refresh: [] }>();
const categoryStore = useCategoryStore();
const tagStore = useTagStore();
const todoStore = useTodoStore();
const uiStore = useUiStore();

const settingsOpen = ref(false);

const categoryDialogOpen = ref(false);
const editingCategoryId = ref<number | null>(null);
const categoryForm = reactive({ name: "", color: "#409EFF" });

const tagDialogOpen = ref(false);
const editingTagId = ref<number | null>(null);
const tagForm = reactive({ name: "", color: "#909399" });

const categoryOrder = ref<Category[]>([]);
const tagOrder = ref<Tag[]>([]);
const categoryReordering = ref(false);
const tagReordering = ref(false);
const categorySortableEl = ref<HTMLElement | null>(null);
const tagSortableEl = ref<HTMLElement | null>(null);
let suppressItemClick = false;

const stats = computed(() => computeStats(todoStore.allTodos));

const timeTabs = computed(() => [
  { key: "all" as TimeFilter, label: t("timeFilter.all") },
  { key: "year" as TimeFilter, label: t("timeFilter.year") },
  { key: "month" as TimeFilter, label: t("timeFilter.month") },
  { key: "week" as TimeFilter, label: t("timeFilter.week") },
  { key: "today" as TimeFilter, label: t("timeFilter.today") },
]);

function syncCategoryOrder() {
  if (categoryReordering.value) return;
  categoryOrder.value = [...categoryStore.categories];
}

function syncTagOrder() {
  if (tagReordering.value) return;
  tagOrder.value = [...tagStore.tags];
}

watch(
  () => categoryStore.categories,
  () => syncCategoryOrder(),
  { immediate: true, deep: true },
);

watch(
  () => tagStore.tags,
  () => syncTagOrder(),
  { immediate: true, deep: true },
);

const sidebarDragOptions = {
  animation: 200,
  delay: 120,
  filter: ".no-drag",
  preventOnFilter: true,
  ghostClass: "sidebar-sort-ghost",
  chosenClass: "sidebar-sort-chosen",
  dragClass: "sidebar-sort-drag",
  forceFallback: true,
  fallbackOnBody: true,
  easing: "cubic-bezier(0.2, 0, 0, 1)",
};

const categoryDraggable = useDraggable(categorySortableEl, categoryOrder, {
  ...sidebarDragOptions,
  draggable: ".category-sort-item",
  onStart() {
    categoryReordering.value = true;
    document.body.style.userSelect = "none";
    document.body.style.webkitUserSelect = "none";
  },
  onEnd(evt) {
    suppressItemClick = evt.oldIndex !== evt.newIndex;
    categoryReordering.value = false;
    document.body.style.userSelect = "";
    document.body.style.webkitUserSelect = "";
    document
      .querySelectorAll("body > .sortable-fallback, body > .sidebar-sort-drag")
      .forEach((node) => node.remove());
    void persistCategoryOrder();
    window.setTimeout(() => {
      suppressItemClick = false;
    }, 200);
  },
});

const tagDraggable = useDraggable(tagSortableEl, tagOrder, {
  ...sidebarDragOptions,
  draggable: ".tag-sort-item",
  onStart() {
    tagReordering.value = true;
    document.body.style.userSelect = "none";
    document.body.style.webkitUserSelect = "none";
  },
  onEnd(evt) {
    suppressItemClick = evt.oldIndex !== evt.newIndex;
    tagReordering.value = false;
    document.body.style.userSelect = "";
    document.body.style.webkitUserSelect = "";
    document
      .querySelectorAll("body > .sortable-fallback, body > .sidebar-sort-drag")
      .forEach((node) => node.remove());
    void persistTagOrder();
    window.setTimeout(() => {
      suppressItemClick = false;
    }, 200);
  },
});

async function initSortables() {
  await nextTick();
  if (categorySortableEl.value) {
    categoryDraggable.start(categorySortableEl.value);
    categoryDraggable.resume?.();
  }
  if (tagSortableEl.value) {
    tagDraggable.start(tagSortableEl.value);
    tagDraggable.resume?.();
  }
}

onMounted(() => {
  void initSortables();
});

onUnmounted(() => {
  document.body.style.userSelect = "";
  document.body.style.webkitUserSelect = "";
  categoryDraggable.destroy?.();
  tagDraggable.destroy?.();
});

async function persistCategoryOrder() {
  const ids = categoryOrder.value.map((category) => category.id);
  const previous = categoryStore.categories.map((category) => category.id);
  if (ids.join(",") === previous.join(",")) return;

  try {
    await categoryStore.reorder(ids);
  } catch (error) {
    console.error("reorder categories failed", error);
    syncCategoryOrder();
    ElMessage.error(t("sidebar.reorderCategoryFailed"));
  }
}

async function persistTagOrder() {
  const ids = tagOrder.value.map((tag) => tag.id);
  const previous = tagStore.tags.map((tag) => tag.id);
  if (ids.join(",") === previous.join(",")) return;

  try {
    await tagStore.reorder(ids);
  } catch (error) {
    console.error("reorder tags failed", error);
    syncTagOrder();
    ElMessage.error(t("sidebar.reorderTagFailed"));
  }
}

const categoryIcons = [House, TrendCharts, Reading, VideoCamera, Folder];

function getCategoryIcon(index: number) {
  return categoryIcons[index % categoryIcons.length];
}

function openCreateCategory() {
  editingCategoryId.value = null;
  categoryForm.name = "";
  categoryForm.color = nextKanbanColor(categoryStore.categories);
  categoryDialogOpen.value = true;
}

function openEditCategory(cat: Category) {
  editingCategoryId.value = cat.id;
  categoryForm.name = cat.name;
  categoryForm.color = cat.color;
  categoryDialogOpen.value = true;
}

async function saveCategory() {
  const name = categoryForm.name.trim();
  if (!name) return;
  try {
    if (editingCategoryId.value) {
      await categoryStore.update(editingCategoryId.value, name, categoryForm.color);
    } else {
      await categoryStore.create(name, categoryForm.color);
    }
    categoryDialogOpen.value = false;
    emit("refresh");
  } catch {
    // 错误提示由 tauriInvoke 统一处理
  }
}

async function removeCategory(id: number, name: string) {
  await ElMessageBox.confirm(t("sidebar.deleteCategory", { name }), t("common.hint"), {
    type: "warning",
  });
  await categoryStore.remove(id);
  if (uiStore.categoryFilter === id) uiStore.selectAllCategories();
  emit("refresh");
}

function openCreateTag() {
  editingTagId.value = null;
  tagForm.name = "";
  tagForm.color = nextKanbanColor(tagStore.tags);
  tagDialogOpen.value = true;
}

function openEditTag(tag: Tag) {
  editingTagId.value = tag.id;
  tagForm.name = tag.name;
  tagForm.color = tag.color;
  tagDialogOpen.value = true;
}

async function saveTag() {
  const name = tagForm.name.trim();
  if (!name) return;
  try {
    if (editingTagId.value) {
      await tagStore.update(editingTagId.value, name, tagForm.color);
    } else {
      await tagStore.create(name, tagForm.color);
    }
    tagDialogOpen.value = false;
    emit("refresh");
  } catch {
    // 错误提示由 tauriInvoke 统一处理
  }
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
  if (suppressItemClick || tagReordering.value) return;
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
  if (suppressItemClick || categoryReordering.value) return;
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

      <div ref="categorySortableEl" class="category-sortable">
        <div
          v-for="(cat, index) in categoryOrder"
          :key="cat.id"
          class="category-item category-sort-item"
          :class="{ active: isCategoryActive(cat.id), 'is-sorting': categoryReordering }"
          @click="selectCategory(cat.id)"
        >
          <el-icon class="cat-icon" :style="{ color: cat.color }">
            <component :is="getCategoryIcon(index)" />
          </el-icon>
          <span class="cat-name">{{ cat.name }}</span>
          <span class="cat-count">{{ cat.todoCount }}</span>
          <div class="cat-actions no-drag">
            <el-button
              link
              class="cat-edit"
              :title="t('common.edit')"
              @click.stop="openEditCategory(cat)"
            >
              <el-icon><Edit /></el-icon>
            </el-button>
            <el-button
              link
              type="danger"
              class="cat-delete"
              @click.stop="removeCategory(cat.id, cat.name)"
            >
              ×
            </el-button>
          </div>
        </div>
      </div>

      <button type="button" class="add-category-btn" @click="openCreateCategory">
        <el-icon><Plus /></el-icon>
        {{ t("sidebar.addCategory") }}
      </button>
      </div>

      <div class="section-divider" />

      <div class="section-header">
        <span>{{ t("sidebar.tags") }}</span>
      </div>

      <div class="category-list">
      <div ref="tagSortableEl" class="tag-sortable">
        <div
          v-for="tag in tagOrder"
          :key="tag.id"
          class="category-item tag-sort-item"
          :class="{ active: isTagActive(tag.id), 'is-sorting': tagReordering }"
          @click="toggleTagFilter(tag.id)"
        >
          <span class="tag-dot" :style="{ background: tag.color }" />
          <span class="cat-name">#{{ tag.name }}</span>
          <span class="cat-count">{{ tag.todoCount }}</span>
          <div class="cat-actions no-drag">
            <el-button
              link
              class="cat-edit"
              :title="t('common.edit')"
              @click.stop="openEditTag(tag)"
            >
              <el-icon><Edit /></el-icon>
            </el-button>
            <el-button
              link
              type="danger"
              class="cat-delete"
              @click.stop="removeTag(tag.id, tag.name)"
            >
              ×
            </el-button>
          </div>
        </div>
      </div>

      <button type="button" class="add-category-btn" @click="openCreateTag">
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

    <el-dialog
      v-model="categoryDialogOpen"
      :title="editingCategoryId ? t('sidebar.editCategory') : t('sidebar.addCategory')"
      width="420px"
      destroy-on-close
      append-to-body
      align-center
      :z-index="4000"
      class="app-dialog"
    >
      <el-form label-width="132px" class="sidebar-meta-form">
        <el-form-item :label="t('sidebar.categoryName')" required>
          <el-input
            v-model="categoryForm.name"
            :placeholder="t('sidebar.categoryName')"
            maxlength="30"
            @keyup.enter="saveCategory"
          />
        </el-form-item>
        <el-form-item :label="t('sidebar.color')">
          <el-color-picker v-model="categoryForm.color" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="categoryDialogOpen = false">{{ t("common.cancel") }}</el-button>
        <el-button type="primary" @click="saveCategory">{{ t("common.save") }}</el-button>
      </template>
    </el-dialog>

    <el-dialog
      v-model="tagDialogOpen"
      :title="editingTagId ? t('sidebar.editTag') : t('sidebar.addTag')"
      width="420px"
      destroy-on-close
      append-to-body
      align-center
      :z-index="4000"
      class="app-dialog"
    >
      <el-form label-width="120px" class="sidebar-meta-form">
        <el-form-item :label="t('sidebar.tagName')" required>
          <el-input
            v-model="tagForm.name"
            :placeholder="t('sidebar.tagName')"
            maxlength="30"
            @keyup.enter="saveTag"
          />
        </el-form-item>
        <el-form-item :label="t('sidebar.color')">
          <el-color-picker v-model="tagForm.color" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="tagDialogOpen = false">{{ t("common.cancel") }}</el-button>
        <el-button type="primary" @click="saveTag">{{ t("common.save") }}</el-button>
      </template>
    </el-dialog>
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

.category-sortable,
.tag-sortable {
  display: block;
}

.category-sort-item,
.tag-sort-item {
  cursor: grab;
}

.category-sort-item.is-sorting,
.tag-sort-item.is-sorting {
  cursor: grabbing;
}

:global(.sidebar-sort-ghost) {
  opacity: 0.45;
}

:global(.sidebar-sort-chosen) {
  background: var(--primary-light);
}

:global(.sidebar-sort-drag) {
  opacity: 0.92;
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
  width: 44px;
  min-width: 44px;
  flex-shrink: 0;
}

.cat-actions {
  display: flex;
  align-items: center;
  gap: 2px;
  width: 44px;
  min-width: 44px;
  flex-shrink: 0;
  justify-content: flex-end;
}

.cat-edit,
.cat-delete {
  opacity: 0;
  width: 20px;
  min-width: 20px;
  padding: 0;
  margin: 0;
  font-size: 16px;
  flex-shrink: 0;
}

.cat-edit {
  color: var(--text-secondary);
}

.category-item:hover .cat-edit,
.category-item:hover .cat-delete {
  opacity: 1;
}

.cat-edit:hover {
  color: var(--primary);
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

.sidebar-meta-form :deep(.el-form-item__label) {
  white-space: nowrap;
}
</style>
