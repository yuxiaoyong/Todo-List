<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { Grid, Rank } from "@element-plus/icons-vue";
import { useDraggable } from "vue-draggable-plus";
import { useTaskListColumnStore } from "../../stores/taskListColumns";
import { TASK_LIST_COLUMN_I18N_KEYS } from "../../utils/taskListColumns";

const { t } = useI18n();
const columnStore = useTaskListColumnStore();
const sortableEl = ref<HTMLElement | null>(null);
const reordering = ref(false);
const localColumns = ref(columnStore.columns.map((column) => ({ ...column })));

watch(
  () => columnStore.columns,
  (columns) => {
    if (reordering.value) return;
    localColumns.value = columns.map((column) => ({ ...column }));
  },
  { deep: true },
);

const sortableColumns = computed({
  get: () => localColumns.value,
  set: (value) => {
    localColumns.value = value.map((column) => ({ ...column }));
  },
});

const draggable = useDraggable(sortableEl, sortableColumns, {
  animation: 150,
  handle: ".col-drag-handle",
  draggable: ".col-item--draggable",
  ghostClass: "col-item-ghost",
  onStart() {
    reordering.value = true;
    document.body.style.userSelect = "none";
    document.body.style.webkitUserSelect = "none";
  },
  onEnd() {
    reordering.value = false;
    document.body.style.userSelect = "";
    document.body.style.webkitUserSelect = "";
    document
      .querySelectorAll("body > .sortable-fallback, body > .col-item-ghost")
      .forEach((node) => node.remove());
    void columnStore.setColumns(localColumns.value);
  },
});

async function initSortable() {
  await nextTick();
  if (!sortableEl.value) return;
  draggable.start(sortableEl.value);
  draggable.resume?.();
}

onMounted(() => {
  void initSortable();
});

onUnmounted(() => {
  document.body.style.userSelect = "";
  document.body.style.webkitUserSelect = "";
  draggable.destroy?.();
});

function columnLabel(id: keyof typeof TASK_LIST_COLUMN_I18N_KEYS) {
  return t(TASK_LIST_COLUMN_I18N_KEYS[id]);
}
</script>

<template>
  <el-popover
    placement="bottom-end"
    :width="280"
    trigger="click"
    teleported
    popper-class="task-column-settings-popover"
    @show="initSortable"
  >
    <template #reference>
      <button
        type="button"
        class="column-settings-btn"
        :title="t('task.columnSettings')"
        @click.stop
      >
        <el-icon><Grid /></el-icon>
      </button>
    </template>

    <div class="column-settings">
      <div class="column-settings__title">{{ t("task.columnSettingsTitle") }}</div>
      <p class="column-settings__hint">{{ t("task.columnSettingsHint") }}</p>

      <ul
        ref="sortableEl"
        class="column-settings__list"
        :class="{ 'is-reordering': reordering }"
      >
        <li
          v-for="column in localColumns"
          :key="column.id"
          class="col-item"
          :class="{
            'col-item--locked': column.locked,
            'col-item--draggable': !column.locked,
          }"
        >
          <span
            class="col-drag-handle"
            :class="{ 'col-drag-handle--disabled': column.locked }"
            :title="column.locked ? undefined : t('sidebar.dragSort')"
          >
            <el-icon><Rank /></el-icon>
          </span>
          <el-checkbox
            :model-value="column.visible"
            :disabled="column.locked"
            @change="(value: boolean) => columnStore.setVisible(column.id, value)"
            @click.stop
          >
            {{ columnLabel(column.id) }}
          </el-checkbox>
        </li>
      </ul>

      <el-button text type="primary" class="column-settings__reset" @click="columnStore.reset()">
        {{ t("task.resetColumns") }}
      </el-button>
    </div>
  </el-popover>
</template>

<style scoped>
.column-settings-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border: none;
  border-radius: 6px;
  background: transparent;
  color: var(--text-secondary);
  cursor: pointer;
  padding: 0;
  transition: color 0.15s, background 0.15s;
}

.column-settings-btn:hover {
  color: var(--primary);
  background: var(--nav-hover);
}

.column-settings__title {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
}

.column-settings__hint {
  margin: 6px 0 10px;
  font-size: 12px;
  line-height: 1.5;
  color: var(--text-secondary);
}

.column-settings__list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.col-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 6px;
  border-radius: 6px;
  background: var(--surface-muted);
}

.col-item--draggable {
  cursor: default;
}

.col-item-ghost {
  opacity: 0.5;
}

.col-drag-handle {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 20px;
  height: 20px;
  color: var(--text-tertiary);
  cursor: grab;
  flex-shrink: 0;
}

.col-drag-handle--disabled {
  opacity: 0.35;
  cursor: default;
}

.col-item :deep(.el-checkbox) {
  flex: 1;
  min-width: 0;
  height: auto;
}

.col-item :deep(.el-checkbox__label) {
  font-size: 13px;
  color: var(--text-primary);
}

.column-settings__reset {
  margin-top: 8px;
  padding-left: 0;
}
</style>
