<script setup lang="ts">
import { inject } from "vue";
import { useI18n } from "vue-i18n";
import {
  Calendar,
  Delete,
  Edit,
  RefreshLeft,
  Star,
  StarFilled,
  View,
} from "@element-plus/icons-vue";
import { TASK_LIST_CELL_CTX } from "./taskListCellContext";
import type { TaskListColumnItem } from "../../utils/taskListColumns";
import type { TodoSummary } from "../../types";

defineProps<{
  columns: TaskListColumnItem[];
  todo: TodoSummary;
}>();

const { t } = useI18n();
const ctx = inject(TASK_LIST_CELL_CTX)!;

const isTrashMode = ctx.isTrashMode;
const categories = ctx.categories;
const tags = ctx.tags;
const kanbanColumns = ctx.kanbanColumns;
const duePopoverId = ctx.duePopoverId;
const startPopoverId = ctx.startPopoverId;
const tagPopoverId = ctx.tagPopoverId;
const tagDrafts = ctx.tagDrafts;
const titleOverflowMap = ctx.titleOverflowMap;
const priorityLabel = ctx.priorityLabel;
</script>

<template>
  <template v-for="col in columns" :key="`${todo.id}-${col.id}`">
    <div v-if="col.id === 'pin'" class="col-pin">
      <button
        type="button"
        class="pin-btn no-drag"
        :class="{ active: todo.pinned }"
        :title="todo.pinned ? t('task.unpin') : t('task.pin')"
        @click.stop="ctx.onTogglePin(todo)"
      >
        <el-icon>
          <StarFilled v-if="todo.pinned" />
          <Star v-else />
        </el-icon>
      </button>
    </div>

    <div v-else-if="col.id === 'check'" class="col-check">
      <el-checkbox
        class="no-drag"
        :model-value="todo.completed"
        :disabled="isTrashMode"
        @click.stop
        @change="ctx.onToggle(todo)"
      />
    </div>

    <div v-else-if="col.id === 'title'" class="col-title">
      <input
        v-if="!isTrashMode && ctx.isTitleEditing(todo.id)"
        :data-title-input="todo.id"
        class="title-input no-drag"
        :class="{ done: todo.completed }"
        :value="ctx.displayTitle(todo)"
        :placeholder="t('taskDetail.titlePlaceholder')"
        @input="ctx.onTitleInput(todo, $event)"
        @keydown="ctx.onTitleKeydown(todo, $event)"
        @blur="ctx.onTitleBlur()"
        @click.stop
        @dblclick.stop
        @mousedown.stop
        @pointerdown.stop
      />
      <div v-else class="title-cell">
        <el-tooltip
          :content="todo.title"
          placement="top"
          :show-after="300"
          teleported
          :disabled="!titleOverflowMap[todo.id]"
        >
          <span
            class="task-title"
            :class="{ done: todo.completed }"
            @mouseenter="ctx.checkTitleOverflow($event, todo.id)"
          >
            {{ todo.title }}
          </span>
        </el-tooltip>
        <button
          v-if="!isTrashMode"
          type="button"
          class="title-edit-btn no-drag"
          :title="t('common.edit')"
          @click.stop="ctx.startTitleEdit(todo)"
        >
          <el-icon><Edit /></el-icon>
        </button>
      </div>
    </div>

    <div v-else-if="col.id === 'priority'" class="col-priority no-drag" @click.stop>
      <el-dropdown
        v-if="!isTrashMode"
        trigger="click"
        teleported
        @command="(value: string) => ctx.onPriorityChange(todo, value)"
      >
        <span
          class="priority-pill priority-cell-trigger"
          :class="ctx.priorityClass(todo.priority)"
          :title="t('task.changePriority')"
        >
          {{ priorityLabel[todo.priority] ?? t("priority.none") }}
        </span>
        <template #dropdown>
          <el-dropdown-menu>
            <el-dropdown-item command="high">
              {{ t("priority.highOption") }}
            </el-dropdown-item>
            <el-dropdown-item command="medium">
              {{ t("priority.mediumOption") }}
            </el-dropdown-item>
            <el-dropdown-item command="low">
              {{ t("priority.low") }}
            </el-dropdown-item>
          </el-dropdown-menu>
        </template>
      </el-dropdown>
      <span v-else class="priority-pill" :class="ctx.priorityClass(todo.priority)">
        {{ priorityLabel[todo.priority] ?? t("priority.none") }}
      </span>
    </div>

    <div v-else-if="col.id === 'status'" class="col-status">
      <span
        class="status-pill"
        :class="todo.completed ? 'status-pill--done' : 'status-pill--open'"
      >
        {{ ctx.statusLabel(todo) }}
      </span>
    </div>

    <div v-else-if="col.id === 'startDate'" class="col-start no-drag" @click.stop>
      <el-popover
        v-if="!isTrashMode"
        :visible="startPopoverId === todo.id"
        placement="bottom-start"
        :width="380"
        trigger="click"
        teleported
        popper-class="due-date-popover"
        @update:visible="(visible: boolean) => ctx.onStartPopoverVisible(todo.id, visible)"
      >
        <template #reference>
          <span class="due-cell-trigger" :title="t('task.changeStartDate')">
            <span v-if="todo.startDate" class="due-date">
              <el-icon><Calendar /></el-icon>
              {{ ctx.formatDueDate(todo.startDate) }}
            </span>
            <span v-else class="due-empty">{{ t("common.dash") }}</span>
          </span>
        </template>
        <el-date-picker-panel
          :model-value="ctx.startDateValue(todo)"
          type="date"
          value-format="YYYY-MM-DD"
          :border="false"
          @update:model-value="(value: string | undefined) => ctx.onStartDateChange(todo, value)"
        />
        <button
          v-if="todo.startDate"
          type="button"
          class="due-clear-btn"
          @click="ctx.onStartDateChange(todo, undefined)"
        >
          {{ t("task.clearStartDate") }}
        </button>
      </el-popover>
      <template v-else>
        <span v-if="todo.startDate" class="due-date">
          <el-icon><Calendar /></el-icon>
          {{ ctx.formatDueDate(todo.startDate) }}
        </span>
        <span v-else class="due-empty">{{ t("common.dash") }}</span>
      </template>
    </div>

    <div v-else-if="col.id === 'dueDate'" class="col-due no-drag" @click.stop>
      <el-popover
        v-if="!isTrashMode"
        :visible="duePopoverId === todo.id"
        placement="bottom-start"
        :width="380"
        trigger="click"
        teleported
        popper-class="due-date-popover"
        @update:visible="(visible: boolean) => ctx.onDuePopoverVisible(todo.id, visible)"
      >
        <template #reference>
          <span class="due-cell-trigger" :title="t('task.changeDueDate')">
            <span v-if="todo.dueDate" class="due-date">
              <el-icon><Calendar /></el-icon>
              {{ ctx.formatDueDate(todo.dueDate) }}
            </span>
            <span v-else class="due-empty">{{ t("common.dash") }}</span>
          </span>
        </template>
        <el-date-picker-panel
          :model-value="ctx.dueDateValue(todo)"
          type="date"
          value-format="YYYY-MM-DD"
          :border="false"
          @update:model-value="(value: string | undefined) => ctx.onDueDateChange(todo, value)"
        />
        <button
          v-if="todo.dueDate"
          type="button"
          class="due-clear-btn"
          @click="ctx.onDueDateChange(todo, undefined)"
        >
          {{ t("task.clearDueDate") }}
        </button>
      </el-popover>
      <template v-else>
        <span v-if="todo.dueDate" class="due-date">
          <el-icon><Calendar /></el-icon>
          {{ ctx.formatDueDate(todo.dueDate) }}
        </span>
        <span v-else class="due-empty">{{ t("common.dash") }}</span>
      </template>
    </div>

    <div v-else-if="col.id === 'category'" class="col-category no-drag" @click.stop>
      <el-dropdown
        v-if="!isTrashMode"
        trigger="click"
        teleported
        @command="(value: number | 'none') => ctx.onCategoryChange(todo, value)"
      >
        <span
          class="category-pill category-cell-trigger"
          :class="{ 'category-pill--empty': !todo.categoryName }"
          :style="ctx.categoryPillStyle(todo)"
          :title="t('task.changeCategory')"
        >
          {{ todo.categoryName ?? t("common.dash") }}
        </span>
        <template #dropdown>
          <el-dropdown-menu>
            <el-dropdown-item command="none">
              {{ t("common.none") }}
            </el-dropdown-item>
            <el-dropdown-item
              v-for="category in categories"
              :key="category.id"
              :command="category.id"
            >
              {{ category.name }}
            </el-dropdown-item>
          </el-dropdown-menu>
        </template>
      </el-dropdown>
      <span
        v-else
        class="category-pill"
        :class="{ 'category-pill--empty': !todo.categoryName }"
        :style="ctx.categoryPillStyle(todo)"
      >
        {{ todo.categoryName ?? t("common.dash") }}
      </span>
    </div>

    <div v-else-if="col.id === 'tags'" class="col-tags no-drag" @click.stop>
      <el-popover
        v-if="!isTrashMode"
        :visible="tagPopoverId === todo.id"
        placement="bottom-start"
        :width="300"
        trigger="click"
        teleported
        popper-class="tag-picker-popover"
        @update:visible="(visible: boolean) => ctx.onTagPopoverVisible(todo, visible)"
      >
        <template #reference>
          <span class="tag-cell-trigger" :title="t('task.changeTags')">
            <span v-if="todo.tagNames.length" class="tag-list">
              <span
                v-for="(name, index) in todo.tagNames"
                :key="`${todo.id}-${name}`"
                class="tag-pill"
                :style="ctx.tagPillStyle(todo, index)"
              >
                {{ name }}
              </span>
            </span>
            <span v-else class="due-empty">{{ t("common.dash") }}</span>
          </span>
        </template>
        <el-select
          v-model="tagDrafts[todo.id]"
          multiple
          filterable
          collapse-tags
          collapse-tags-tooltip
          :teleported="false"
          :placeholder="t('task.selectTags')"
          class="tag-inline-select"
          style="width: 100%"
        >
          <el-option
            v-for="tag in tags"
            :key="tag.id"
            :label="tag.name"
            :value="tag.id"
          />
        </el-select>
      </el-popover>
      <template v-else>
        <div v-if="todo.tagNames.length" class="tag-list">
          <span
            v-for="(name, index) in todo.tagNames"
            :key="`${todo.id}-${name}`"
            class="tag-pill"
            :style="ctx.tagPillStyle(todo, index)"
          >
            {{ name }}
          </span>
        </div>
        <span v-else class="due-empty">{{ t("common.dash") }}</span>
      </template>
    </div>

    <div v-else-if="col.id === 'assignee'" class="col-assignee">
      <span class="meta-text" :title="ctx.assigneeLabel(todo)">{{ ctx.assigneeLabel(todo) }}</span>
    </div>

    <div v-else-if="col.id === 'kanbanColumn'" class="col-kanban no-drag" @click.stop>
      <el-dropdown
        v-if="!isTrashMode"
        trigger="click"
        teleported
        @command="(value: number | 'none') => ctx.onKanbanColumnChange(todo, value)"
      >
        <span
          class="category-pill category-cell-trigger"
          :class="{ 'category-pill--empty': !todo.kanbanColumnName }"
          :style="ctx.kanbanPillStyle(todo)"
          :title="t('task.changeKanbanColumn')"
        >
          {{ todo.kanbanColumnName ?? t("common.dash") }}
        </span>
        <template #dropdown>
          <el-dropdown-menu>
            <el-dropdown-item command="none">
              {{ t("common.none") }}
            </el-dropdown-item>
            <el-dropdown-item
              v-for="column in kanbanColumns"
              :key="column.id"
              :command="column.id"
            >
              {{ column.name }}
            </el-dropdown-item>
          </el-dropdown-menu>
        </template>
      </el-dropdown>
      <span
        v-else
        class="category-pill"
        :class="{ 'category-pill--empty': !todo.kanbanColumnName }"
        :style="ctx.kanbanPillStyle(todo)"
      >
        {{ todo.kanbanColumnName ?? t("common.dash") }}
      </span>
    </div>

    <div v-else-if="col.id === 'recurrence'" class="col-recurrence">
      <span class="meta-text meta-text--ellipsis" :title="ctx.recurrenceLabel(todo)">
        {{ ctx.recurrenceLabel(todo) }}
      </span>
    </div>

    <div v-else-if="col.id === 'createdAt'" class="col-created">
      <span class="meta-text">{{ ctx.formatDateTimeCn(todo.createdAt) }}</span>
    </div>

    <div v-else-if="col.id === 'updatedAt'" class="col-updated">
      <span class="meta-text">{{ ctx.formatDateTimeCn(todo.updatedAt) }}</span>
    </div>

    <div v-else-if="col.id === 'actions'" class="col-actions no-drag" @click.stop>
      <div class="row-actions">
        <template v-if="isTrashMode">
          <el-button
            link
            type="primary"
            :icon="RefreshLeft"
            :title="t('common.restore')"
            @click="ctx.onRestore(todo)"
          />
          <el-button
            link
            type="danger"
            :icon="Delete"
            :title="t('common.delete')"
            @click="ctx.onDelete(todo)"
          />
        </template>
        <template v-else>
          <el-button
            link
            type="primary"
            :icon="View"
            :title="t('common.view')"
            @click="ctx.onView(todo.id)"
          />
          <el-button
            link
            type="primary"
            :icon="Edit"
            :title="t('common.edit')"
            @click="ctx.onEdit(todo.id)"
          />
          <el-button
            link
            type="danger"
            :icon="Delete"
            :title="t('common.delete')"
            @click="ctx.onDelete(todo)"
          />
        </template>
      </div>
    </div>
  </template>
</template>
