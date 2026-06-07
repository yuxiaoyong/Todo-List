<script setup lang="ts">
import { computed, inject, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { formatDateCn } from "../../utils/formatDate";
import {
  Calendar,
  CircleCheck,
  Clock,
  Document,
  Star,
  StarFilled,
  Timer,
  User,
} from "@element-plus/icons-vue";
import AttachmentPanel from "../attachment/AttachmentPanel.vue";
import WysiwygEditor from "../editor/WysiwygEditor.vue";
import TaskDetailReminderPanel from "./TaskDetailReminderPanel.vue";
import TaskDetailSubtasks from "./TaskDetailSubtasks.vue";
import { taskDetailPanelKey } from "./taskDetailPanelContext";
import {
  formatRecurrenceSummary,
  recurrenceAnchorDate,
  recurrenceLunarReady,
  syncLunarFromSolar,
} from "../../utils/recurrence";

const { locale } = useI18n();
const ctx = inject(taskDetailPanelKey);
if (!ctx) {
  throw new Error("TaskDetailPanelBody must be used inside TaskDetailPanel");
}

const {
  form,
  detail,
  editable,
  activeTab,
  tagPickerValue,
  saving,
  savingNow,
  categoryColor,
  selectedTags,
  priorityClass,
  statusText,
  assigneeText,
  panelAttachments,
  subtasks,
  onSubtasksChange,
  categoryStore,
  kanbanColumnStore,
  tagStore,
  t,
  togglePin,
  toggleCompleted,
  onFieldBlur,
  closeDrawer,
  saveNow,
  onRestore,
  onDelete,
  removeTag,
  onTagPickerChange,
  onAttachmentsUploaded,
  refreshAttachments,
  formatDateTimeCn,
} = ctx;

const startDateText = computed(() => formatDateCn(form.value?.startDate));
const dueDateText = computed(() => formatDateCn(form.value?.dueDate));
const startPopoverVisible = ref(false);
const duePopoverVisible = ref(false);

watch(
  () => form.value?.startDate,
  () => {
    if (startPopoverVisible.value) startPopoverVisible.value = false;
  },
);

watch(
  () => form.value?.dueDate,
  () => {
    if (duePopoverVisible.value) duePopoverVisible.value = false;
  },
);

watch(
  () => form.value,
  (value) => {
    if (!value) {
      startPopoverVisible.value = false;
      duePopoverVisible.value = false;
    }
  },
);

const recurrenceEnabled = computed(() => !!form.value?.recurrenceJson?.enabled);

const recurrenceSummary = computed(() => {
  if (!form.value?.recurrenceJson) return "";
  return formatRecurrenceSummary(
    form.value.recurrenceJson,
    { startDate: form.value.startDate, dueDate: form.value.dueDate },
    t,
    locale.value,
  );
});

watch(
  () => form.value?.recurrenceJson?.enabled,
  (enabled, prev) => {
    if (enabled && !prev) {
      const recurrence = form.value?.recurrenceJson;
      if (recurrence && form.value) {
        const anchor = recurrenceAnchorDate(recurrence.anchor, {
          startDate: form.value.startDate,
          dueDate: form.value.dueDate,
        });
        if (recurrence.calendar === "lunar") {
          if (!recurrenceLunarReady(recurrence) && anchor) {
            syncLunarFromSolar(recurrence, anchor);
          }
        } else if (!recurrence.firstReminderDate && anchor) {
          recurrence.firstReminderDate = anchor;
        }
      }
      activeTab.value = "reminder";
      return;
    }
    if (!enabled && activeTab.value === "reminder") {
      activeTab.value = "info";
    }
  },
);

function openReminderTab() {
  if (!recurrenceEnabled.value) return;
  activeTab.value = "reminder";
}
</script>

<template>
  <div v-if="form && detail" class="detail-body">
    <header class="detail-header">
      <div class="title-row">
        <button
          v-if="editable"
          type="button"
          class="pin-btn"
          :class="{ active: form.pinned }"
          :title="form.pinned ? t('task.unpin') : t('task.pin')"
          @click="togglePin"
        >
          <el-icon>
            <StarFilled v-if="form.pinned" />
            <Star v-else />
          </el-icon>
        </button>
        <span class="title-icon" :style="{ background: categoryColor }" />
        <el-input
          v-model="form.title"
          class="title-input"
          :placeholder="t('taskDetail.titlePlaceholder')"
          :disabled="!editable"
          @blur="onFieldBlur"
        />
      </div>

      <div class="info-cards">
        <button
          type="button"
          class="info-card info-card--status"
          :class="{ done: form.completed }"
          :disabled="!editable"
          @click="toggleCompleted"
        >
          <div class="info-icon status-icon">
            <el-icon><CircleCheck v-if="form.completed" /><Clock v-else /></el-icon>
          </div>
          <div class="info-main">{{ statusText }}</div>
          <div class="info-label">{{ t("taskDetail.currentStatus") }}</div>
        </button>

        <div class="info-card">
          <div class="info-icon"><el-icon><User /></el-icon></div>
          <input
            v-if="editable"
            v-model="form.assignee"
            class="info-main assignee-input"
            type="text"
            :placeholder="t('taskDetail.assigneeDefault')"
            @blur="onFieldBlur"
          />
          <div v-else class="info-main">{{ assigneeText }}</div>
          <div class="info-label">{{ t("taskDetail.assignee") }}</div>
        </div>

        <div class="info-card start-card">
          <div class="info-icon"><el-icon><Timer /></el-icon></div>
          <el-popover
            v-if="editable"
            v-model:visible="startPopoverVisible"
            placement="bottom-start"
            :width="380"
            trigger="click"
            teleported
            popper-class="due-date-popover"
          >
            <template #reference>
              <div class="info-main due-main due-main--clickable">
                <span class="due-text">{{ startDateText }}</span>
              </div>
            </template>
            <el-date-picker-panel
              v-model="form.startDate"
              type="date"
              value-format="YYYY-MM-DD"
              :border="false"
            />
          </el-popover>
          <div v-else class="info-main due-main">
            <span class="due-text">{{ startDateText }}</span>
          </div>
          <div class="info-label">{{ t("taskDetail.startTime") }}</div>
        </div>

        <div class="info-card due-card">
          <div class="info-icon"><el-icon><Calendar /></el-icon></div>
          <el-popover
            v-if="editable"
            v-model:visible="duePopoverVisible"
            placement="bottom-start"
            :width="380"
            trigger="click"
            teleported
            popper-class="due-date-popover"
          >
            <template #reference>
              <div class="info-main due-main due-main--clickable">
                <span class="due-text">{{ dueDateText }}</span>
              </div>
            </template>
            <el-date-picker-panel
              v-model="form.dueDate"
              type="date"
              value-format="YYYY-MM-DD"
              :border="false"
            />
          </el-popover>
          <div v-else class="info-main due-main">
            <span class="due-text">{{ dueDateText }}</span>
          </div>
          <div class="info-label">{{ t("taskDetail.dueTime") }}</div>
        </div>
      </div>
    </header>

    <nav class="detail-tabs">
      <button
        type="button"
        class="tab-item"
        :class="{ active: activeTab === 'info' }"
        @click="activeTab = 'info'"
      >
        {{ t("taskDetail.infoTab") }}
      </button>
      <button
        v-if="recurrenceEnabled"
        type="button"
        class="tab-item"
        :class="{ active: activeTab === 'reminder' }"
        @click="activeTab = 'reminder'"
      >
        {{ t("taskDetail.reminderTab") }}
      </button>
      <button
        type="button"
        class="tab-item"
        :class="{ active: activeTab === 'subtasks' }"
        @click="activeTab = 'subtasks'"
      >
        {{ t("taskDetail.subtasksTab") }}
        <span v-if="subtasks.length" class="tab-badge">{{ subtasks.length }}</span>
      </button>
      <button
        type="button"
        class="tab-item"
        :class="{ active: activeTab === 'attachments' }"
        @click="activeTab = 'attachments'"
      >
        {{ t("taskDetail.attachmentsTab") }}
        <span v-if="panelAttachments.length" class="tab-badge">{{ panelAttachments.length }}</span>
      </button>
    </nav>

    <div v-show="activeTab === 'info'" class="detail-content">
      <div class="meta-grid">
        <div class="meta-field">
          <div class="meta-label">{{ t("taskDetail.category") }}</div>
          <el-select
            v-model="form.categoryId"
            :placeholder="t('common.none')"
            clearable
            :disabled="!editable"
            class="meta-value-select"
          >
            <el-option
              v-for="cat in categoryStore.categories"
              :key="cat.id"
              :label="cat.name"
              :value="cat.id"
            />
          </el-select>
        </div>

        <div class="meta-field">
          <div class="meta-label">{{ t("task.priority") }}</div>
          <el-select
            v-model="form.priority"
            :disabled="!editable"
            class="meta-value-select priority-select"
            :class="priorityClass"
          >
            <el-option :label="t('priority.highOption')" value="high" />
            <el-option :label="t('priority.mediumOption')" value="medium" />
            <el-option :label="t('priority.low')" value="low" />
          </el-select>
        </div>

        <div class="meta-field">
          <div class="meta-label">{{ t("taskDetail.kanbanGroup") }}</div>
          <el-select
            v-model="form.kanbanColumnId"
            :placeholder="t('taskDetail.unassigned')"
            clearable
            :disabled="!editable"
            class="meta-value-select"
          >
            <el-option
              v-for="column in kanbanColumnStore.columns"
              :key="column.id"
              :label="column.name"
              :value="column.id"
            />
          </el-select>
        </div>

        <div class="meta-field meta-field-wide">
          <div class="meta-label">{{ t("task.tags") }}</div>
          <div class="tag-area">
            <span
              v-for="tag in selectedTags"
              :key="tag!.id"
              class="tag-chip"
              :style="{ borderColor: tag!.color, color: tag!.color }"
            >
              {{ tag!.name }}
              <button
                v-if="editable"
                type="button"
                class="tag-remove"
                @click="removeTag(tag!.id)"
              >
                ×
              </button>
            </span>
            <el-select
              v-if="editable"
              v-model="tagPickerValue"
              filterable
              clearable
              :placeholder="t('taskDetail.addTag')"
              class="tag-add-select"
              @change="onTagPickerChange"
            >
              <el-option
                v-for="tag in tagStore.tags"
                :key="tag.id"
                :label="tag.name"
                :value="tag.id"
                :disabled="form.tagIds.includes(tag.id)"
              />
            </el-select>
            <span v-else-if="!selectedTags.length" class="meta-empty">{{ t("common.none") }}</span>
          </div>
        </div>

        <div class="meta-field meta-field-wide recurrence-field">
          <div class="meta-label">{{ t("taskDetail.recurrenceEnabled") }}</div>
          <el-switch
            v-if="form.recurrenceJson"
            v-model="form.recurrenceJson.enabled"
            :disabled="!editable"
          />
        </div>

        <button
          v-if="recurrenceEnabled"
          type="button"
          class="recurrence-summary"
          :disabled="!editable"
          @click="openReminderTab"
        >
          <span class="recurrence-summary-text">{{ recurrenceSummary }}</span>
          <span class="recurrence-summary-action">{{ t("taskDetail.recurrenceConfigure") }}</span>
        </button>
      </div>

      <section class="editor-section">
        <div class="editor-section-head">
          <span class="editor-label">{{ t("taskDetail.description") }}</span>
          <span class="editor-mode">
            <el-icon><Document /></el-icon>
            {{ t("taskDetail.wysiwyg") }}
          </span>
        </div>
        <WysiwygEditor
          v-if="activeTab === 'info'"
          v-model="form.contentHtml"
          :todo-id="form.id"
          :editable="editable"
          variant="detail"
          @blur="onFieldBlur"
        />
      </section>
    </div>

    <div v-show="activeTab === 'reminder' && recurrenceEnabled" class="detail-content reminder-tab">
      <TaskDetailReminderPanel
        v-if="form.recurrenceJson"
        :recurrence="form.recurrenceJson"
        :start-date="form.startDate"
        :due-date="form.dueDate"
        :editable="editable"
      />
    </div>

    <div v-show="activeTab === 'subtasks'" class="detail-content subtask-tab">
      <TaskDetailSubtasks
        :todo-id="form.id"
        :subtasks="subtasks"
        :editable="editable"
        @change="onSubtasksChange"
      />
    </div>

    <div v-show="activeTab === 'attachments'" class="detail-content attachment-tab">
      <AttachmentPanel
        :todo-id="form.id"
        :attachments="panelAttachments"
        :editable="editable"
        @uploaded="onAttachmentsUploaded"
        @refresh="refreshAttachments"
      />
    </div>

    <footer class="detail-footer">
      <div class="audit-trail">
        <span>{{ t("taskDetail.createdAt", { date: formatDateTimeCn(detail.createdAt) }) }}</span>
        <span class="audit-sep">|</span>
        <span>{{ t("taskDetail.updatedAt", { date: formatDateTimeCn(detail.updatedAt) }) }}</span>
        <span v-if="saving || savingNow" class="save-hint">{{ t("common.saving") }}</span>
      </div>
      <div class="footer-actions">
        <template v-if="editable">
          <el-button link type="danger" @click="onDelete">{{ t("common.delete") }}</el-button>
          <el-button @click="closeDrawer">{{ t("common.cancel") }}</el-button>
          <el-button type="primary" :loading="savingNow" @click="saveNow">
            {{ t("common.save") }}
          </el-button>
        </template>
        <template v-else>
          <el-button type="primary" @click="onRestore">{{ t("common.restore") }}</el-button>
          <el-button type="danger" @click="onDelete">{{ t("common.delete") }}</el-button>
          <el-button @click="closeDrawer">{{ t("common.close") }}</el-button>
        </template>
      </div>
    </footer>
  </div>
</template>

<style scoped>
.detail-body {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-height: 0;
}

.detail-header {
  padding: 24px 28px 0;
  flex-shrink: 0;
}

.title-row {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 20px;
}

.pin-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border: none;
  background: transparent;
  color: var(--text-tertiary);
  cursor: pointer;
  border-radius: 8px;
  padding: 0;
  flex-shrink: 0;
  transition: color 0.15s, background 0.15s;
}

.pin-btn:hover,
.pin-btn.active {
  color: var(--pin-color);
  background: var(--pin-bg);
}

.title-icon {
  width: 12px;
  height: 12px;
  border-radius: 2px;
  transform: rotate(45deg);
  flex-shrink: 0;
}

.title-input :deep(.el-input__wrapper) {
  box-shadow: none;
  padding: 0;
  background: transparent;
  --el-input-bg-color: transparent;
}

.title-input :deep(.el-input__inner) {
  font-size: 22px;
  font-weight: 600;
  color: var(--text-primary);
  height: 36px;
  line-height: 36px;
}

.info-cards {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 12px;
  padding-bottom: 18px;
  border-bottom: 1px solid var(--border-light);
  align-items: start;
}

.info-card {
  display: grid;
  grid-template-rows: 28px 20px 16px;
  row-gap: 6px;
  align-items: center;
  align-content: start;
  min-width: 0;
  padding: 0;
  border: none;
  background: transparent;
  text-align: left;
  cursor: default;
}

.info-card--status {
  cursor: pointer;
}

.info-card--status:not(:disabled):hover .info-main {
  color: var(--primary);
}

.start-card :deep(.el-popover__reference) {
  display: block;
  min-height: 20px;
}

.info-card.done .info-icon {
  color: #13a8a8;
}

.info-icon {
  width: 28px;
  height: 28px;
  border-radius: 50%;
  background: var(--surface-subtle);
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-secondary);
  font-size: 16px;
  flex-shrink: 0;
}

.info-main {
  font-size: 15px;
  font-weight: 500;
  color: var(--text-primary);
  line-height: 20px;
  min-height: 20px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.info-label {
  font-size: 12px;
  color: var(--text-secondary);
  line-height: 16px;
  min-height: 16px;
}

.due-card :deep(.el-popover__reference) {
  display: block;
  min-height: 20px;
}

.due-main {
  position: relative;
  display: flex;
  align-items: center;
  min-height: 20px;
}

.due-main--clickable {
  cursor: pointer;
}

.due-main--clickable:hover .due-text {
  color: var(--primary);
}

.due-text {
  font-size: 15px;
  font-weight: 500;
  color: var(--text-primary);
  line-height: 20px;
}

.assignee-input {
  width: 100%;
  border: none;
  outline: none;
  padding: 0;
  background: transparent;
  font: inherit;
  color: var(--text-primary);
}

.assignee-input::placeholder {
  color: var(--text-secondary);
}

.assignee-input:focus {
  color: var(--primary);
}

.detail-tabs {
  display: flex;
  gap: 28px;
  padding: 0 28px;
  border-bottom: 1px solid var(--border-light);
  flex-shrink: 0;
}

.tab-item {
  position: relative;
  border: none;
  background: transparent;
  padding: 14px 0;
  font-size: 14px;
  color: var(--text-secondary);
  cursor: pointer;
}

.tab-item.active {
  color: var(--primary);
  font-weight: 500;
}

.tab-item.active::after {
  content: "";
  position: absolute;
  left: 0;
  right: 0;
  bottom: 0;
  height: 2px;
  background: var(--primary);
  border-radius: 2px 2px 0 0;
}

.tab-badge {
  margin-left: 4px;
  font-size: 12px;
  color: var(--text-secondary);
}

.recurrence-field {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.recurrence-summary {
  grid-column: 1 / -1;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  width: 100%;
  padding: 10px 12px;
  border: 1px solid var(--border-light);
  border-radius: 8px;
  background: var(--surface-subtle);
  text-align: left;
  cursor: pointer;
}

.recurrence-summary:disabled {
  cursor: default;
  opacity: 0.7;
}

.recurrence-summary:not(:disabled):hover {
  border-color: var(--primary);
}

.recurrence-summary-text {
  font-size: 13px;
  color: var(--text-primary);
}

.recurrence-summary-action {
  flex-shrink: 0;
  font-size: 13px;
  color: var(--primary);
}

.detail-content {
  flex: 1;
  overflow: auto;
  padding: 20px 28px;
}

.subtask-tab {
  display: flex;
  flex-direction: column;
  min-height: 0;
}

.attachment-tab {
  display: flex;
  flex-direction: column;
  min-height: 0;
}

.meta-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 20px 32px;
  margin-bottom: 24px;
}

.meta-field-wide {
  grid-column: 1 / -1;
}

.meta-label {
  font-size: 13px;
  color: var(--text-secondary);
  margin-bottom: 8px;
}

.meta-value-select {
  width: 100%;
}

.meta-value-select :deep(.el-select__wrapper) {
  box-shadow: none;
  padding-left: 0;
  background: transparent;
  --el-fill-color-blank: transparent;
}

.meta-value-select :deep(.el-select__placeholder),
.meta-value-select :deep(.el-select__selected-item) {
  font-size: 14px;
  font-weight: 500;
}

.priority-select.priority-high :deep(.el-select__selected-item) {
  color: var(--priority-high-text);
}

.priority-select.priority-medium :deep(.el-select__selected-item) {
  color: var(--priority-medium-text);
}

.tag-area {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  align-items: center;
}

.tag-chip {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 2px 10px;
  border: 1px solid var(--tag-border);
  border-radius: 14px;
  font-size: 12px;
  background: var(--tag-bg);
}

.tag-remove {
  border: none;
  background: transparent;
  cursor: pointer;
  color: inherit;
  font-size: 14px;
  line-height: 1;
  padding: 0;
}

.tag-add-select {
  width: 120px;
  flex-shrink: 0;
}

.tag-add-select :deep(.el-select__wrapper) {
  box-shadow: none;
  background: var(--surface-muted);
  border: 1px dashed var(--border-color);
  min-height: 28px;
  height: 28px;
}

.meta-empty {
  color: var(--text-tertiary);
}

.editor-section-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 10px;
}

.editor-label {
  font-size: 13px;
  color: var(--text-secondary);
}

.editor-mode {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
  color: var(--text-tertiary);
}

.detail-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  padding: 14px 28px;
  border-top: 1px solid var(--border-light);
  flex-shrink: 0;
  background: var(--panel-bg);
}

.audit-trail {
  font-size: 12px;
  color: var(--text-secondary);
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 6px;
}

.audit-sep {
  color: var(--text-tertiary);
}

.save-hint {
  color: var(--primary);
}

.footer-actions {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
}
</style>
