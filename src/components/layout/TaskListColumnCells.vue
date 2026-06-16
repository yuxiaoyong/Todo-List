<script setup lang="ts">
import { inject } from "vue";
import { useI18n } from "vue-i18n";
import { TASK_LIST_CELL_CTX } from "./taskListCellContext";
import type { TaskListColumnItem } from "../../utils/taskListColumns";

defineProps<{
  columns: TaskListColumnItem[];
}>();

const { t } = useI18n();
const ctx = inject(TASK_LIST_CELL_CTX)!;
</script>

<template>
  <template v-for="col in columns" :key="col.id">
    <span v-if="col.id === 'pin'" class="col-pin" />
    <span v-else-if="col.id === 'check'" class="col-check" />
    <button
      v-else-if="col.id === 'title'"
      type="button"
      class="col-title sortable"
      @click="ctx.onHeaderClick('title')"
    >
      {{ t("task.taskName") }}
      <span class="sort-mark">{{ ctx.sortIndicator("title") }}</span>
    </button>
    <button
      v-else-if="col.id === 'priority'"
      type="button"
      class="col-priority sortable"
      @click="ctx.onHeaderClick('priority')"
    >
      {{ t("task.priority") }}
      <span class="sort-mark">{{ ctx.sortIndicator("priority") }}</span>
    </button>
    <span v-else-if="col.id === 'status'" class="col-status">{{ t("task.columnStatus") }}</span>
    <button
      v-else-if="col.id === 'startDate'"
      type="button"
      class="col-start sortable"
      @click="ctx.onHeaderClick('startDate')"
    >
      {{ t("task.startDate") }}
      <span class="sort-mark">{{ ctx.sortIndicator("startDate") }}</span>
    </button>
    <button
      v-else-if="col.id === 'dueDate'"
      type="button"
      class="col-due sortable"
      @click="ctx.onHeaderClick('dueDate')"
    >
      {{ t("task.dueDate") }}
      <span class="sort-mark">{{ ctx.sortIndicator("dueDate") }}</span>
    </button>
    <span v-else-if="col.id === 'category'" class="col-category">{{ t("task.category") }}</span>
    <span v-else-if="col.id === 'tags'" class="col-tags">{{ t("task.tags") }}</span>
    <button
      v-else-if="col.id === 'assignee'"
      type="button"
      class="col-assignee sortable"
      @click="ctx.onHeaderClick('assignee')"
    >
      {{ t("task.columnAssignee") }}
      <span class="sort-mark">{{ ctx.sortIndicator("assignee") }}</span>
    </button>
    <span v-else-if="col.id === 'kanbanColumn'" class="col-kanban">{{ t("task.columnKanban") }}</span>
    <span v-else-if="col.id === 'recurrence'" class="col-recurrence">{{ t("task.columnRecurrence") }}</span>
    <button
      v-else-if="col.id === 'createdAt'"
      type="button"
      class="col-created sortable"
      @click="ctx.onHeaderClick('createdAt')"
    >
      {{ t("task.columnCreatedAt") }}
      <span class="sort-mark">{{ ctx.sortIndicator("createdAt") }}</span>
    </button>
    <button
      v-else-if="col.id === 'updatedAt'"
      type="button"
      class="col-updated sortable"
      @click="ctx.onHeaderClick('updatedAt')"
    >
      {{ t("task.columnUpdatedAt") }}
      <span class="sort-mark">{{ ctx.sortIndicator("updatedAt") }}</span>
    </button>
    <span v-else-if="col.id === 'actions'" class="col-actions">{{ t("task.actions") }}</span>
  </template>
</template>

<style scoped>
.col-pin,
.col-check {
  display: block;
}

.col-title,
.col-priority,
.col-start,
.col-due,
.col-status,
.col-category,
.col-tags,
.col-assignee,
.col-kanban,
.col-recurrence,
.col-created,
.col-updated,
.col-actions {
  display: flex;
  align-items: center;
  justify-content: flex-start;
  width: 100%;
  min-width: 0;
  text-align: left;
}

.col-title.sortable,
.col-priority.sortable,
.col-start.sortable,
.col-due.sortable,
.col-assignee.sortable,
.col-created.sortable,
.col-updated.sortable {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.sortable {
  border: none;
  background: transparent;
  padding: 0;
  text-align: left;
  color: inherit;
  font: inherit;
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  justify-content: flex-start;
  gap: 4px;
  max-width: 100%;
}

.sortable:hover {
  color: var(--primary);
}

.sort-mark {
  color: var(--primary);
  font-size: 12px;
  flex-shrink: 0;
}
</style>
