import type { ComputedRef, InjectionKey, Ref } from "vue";
import type { ComposerTranslation } from "vue-i18n";
import type { AttachmentInfo, Subtask, Tag, TodoDetail, UpdateTodoInput } from "../../types";
import type { useCategoryStore } from "../../stores/category";
import type { useKanbanColumnStore } from "../../stores/kanbanColumn";
import type { useTagStore } from "../../stores/tag";
import type { formatDateTimeCn } from "../../utils/formatDate";

export interface TaskDetailPanelContext {
  form: Ref<UpdateTodoInput | null>;
  detail: Ref<TodoDetail | null>;
  editable: ComputedRef<boolean>;
  activeTab: Ref<"info" | "reminder" | "subtasks" | "attachments">;
  tagPickerValue: Ref<number | undefined>;
  saving: Ref<boolean>;
  savingNow: Ref<boolean>;
  categoryColor: ComputedRef<string>;
  selectedTags: ComputedRef<(Tag | undefined)[]>;
  priorityClass: ComputedRef<string>;
  statusText: ComputedRef<string>;
  assigneeText: ComputedRef<string>;
  panelAttachments: ComputedRef<AttachmentInfo[]>;
  subtasks: Ref<Subtask[]>;
  onSubtasksChange: (subtasks: Subtask[]) => void;
  categoryStore: ReturnType<typeof useCategoryStore>;
  kanbanColumnStore: ReturnType<typeof useKanbanColumnStore>;
  tagStore: ReturnType<typeof useTagStore>;
  t: ComposerTranslation;
  togglePin: () => Promise<void>;
  toggleCompleted: () => Promise<void>;
  onFieldBlur: () => void;
  closeDrawer: () => Promise<void>;
  saveNow: () => Promise<void>;
  onRestore: () => Promise<void>;
  onDelete: () => Promise<void>;
  removeTag: (id: number) => void;
  onTagPickerChange: (id: number | undefined) => void;
  onAttachmentsUploaded: (items: AttachmentInfo[]) => void;
  refreshAttachments: () => Promise<void>;
  formatDateTimeCn: typeof formatDateTimeCn;
}

export const taskDetailPanelKey: InjectionKey<TaskDetailPanelContext> =
  Symbol.for("taskDetailPanel");
