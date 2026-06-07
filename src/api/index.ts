import { invoke } from "@tauri-apps/api/core";
import type {
  AttachmentInfo,
  Category,
  KanbanColumn,
  QuickCreateInput,
  Tag,
  Subtask,
  TodoDetail,
  TodoListFilter,
  TodoSummary,
  UpdateTodoInput,
} from "../types";

export const categoryApi = {
  list: () => invoke<Category[]>("category_list"),
  create: (name: string, color?: string, icon?: string) =>
    invoke<Category>("category_create", { input: { name, color, icon } }),
  update: (id: number, name: string, color: string, icon?: string) =>
    invoke<Category>("category_update", { input: { id, name, color, icon } }),
  delete: (id: number) => invoke<void>("category_delete", { id }),
  reorder: (ids: number[]) => invoke<void>("category_reorder", { ids }),
};

export const kanbanColumnApi = {
  list: () => invoke<KanbanColumn[]>("kanban_column_list"),
  create: (name: string, color?: string, subtitle?: string) =>
    invoke<KanbanColumn>("kanban_column_create", { input: { name, color, subtitle } }),
  update: (id: number, name: string, color: string, subtitle?: string | null) =>
    invoke<KanbanColumn>("kanban_column_update", { input: { id, name, color, subtitle } }),
  delete: (id: number) => invoke<void>("kanban_column_delete", { id }),
  reorder: (ids: number[]) => invoke<void>("kanban_column_reorder", { ids }),
};

export const tagApi = {
  list: () => invoke<Tag[]>("tag_list"),
  create: (name: string, color?: string) =>
    invoke<Tag>("tag_create", { input: { name, color } }),
  update: (id: number, name: string, color: string) =>
    invoke<Tag>("tag_update", { input: { id, name, color } }),
  delete: (id: number) => invoke<void>("tag_delete", { id }),
};

export const todoApi = {
  list: (filter: TodoListFilter) => invoke<TodoSummary[]>("todo_list", { filter }),
  get: (id: number) => invoke<TodoDetail>("todo_get", { id }),
  create: (input: {
    title: string;
    categoryId?: number;
    tagIds?: number[];
    priority?: string;
    dueDate?: string;
    contentHtml?: string;
  }) => invoke<TodoDetail>("todo_create", { input }),
  update: (input: UpdateTodoInput) => invoke<TodoDetail>("todo_update", { input }),
  quickCreate: (input: QuickCreateInput) =>
    invoke<TodoDetail>("todo_quick_create", { input }),
  toggleComplete: (id: number) => invoke<TodoDetail>("todo_toggle_complete", { id }),
  togglePin: (id: number) => invoke<TodoDetail>("todo_toggle_pin", { id }),
  delete: (id: number) => invoke<void>("todo_delete", { id }),
  restore: (id: number) => invoke<TodoDetail>("todo_restore", { id }),
  permanentDelete: (id: number) => invoke<void>("todo_permanent_delete", { id }),
  emptyTrash: () => invoke<number>("todo_empty_trash"),
  reorder: (ids: number[]) => invoke<void>("todo_reorder", { ids }),
  reorderPositions: (items: { id: number; sortOrder: number }[]) =>
    invoke<void>("todo_reorder_positions", { items }),
  setKanbanColumn: (id: number, kanbanColumnId: number | null) =>
    invoke<TodoDetail>("todo_set_kanban_column", { id, kanbanColumnId }),
  incompleteCount: () => invoke<number>("todo_incomplete_count"),
  dueToday: () => invoke<TodoSummary[]>("todo_due_today"),
};

export const subtaskApi = {
  create: (todoId: number, title: string) =>
    invoke<Subtask>("subtask_create", { todoId, title }),
  update: (id: number, title: string) => invoke<Subtask>("subtask_update", { id, title }),
  toggle: (id: number) => invoke<Subtask>("subtask_toggle", { id }),
  delete: (id: number) => invoke<void>("subtask_delete", { id }),
};

export const attachmentApi = {
  save: (
    todoId: number,
    dataBase64: string,
    originalName?: string,
    mimeType = "image/png",
    kind: "inline" | "attachment" = "attachment",
  ) =>
    invoke<AttachmentInfo>("attachment_save", {
      todoId,
      dataBase64,
      originalName,
      mimeType,
      kind,
    }),
  delete: (id: number) => invoke<void>("attachment_delete", { id }),
  list: (todoId: number) => invoke<AttachmentInfo[]>("attachment_list", { todoId }),
  read: (todoId: number, filename: string) =>
    invoke<string>("attachment_read", { todoId, filename }),
  getPath: (todoId: number, filename: string) =>
    invoke<string>("attachment_get_path", { todoId, filename }),
  open: (todoId: number, filename: string) =>
    invoke<void>("attachment_open", { todoId, filename }),
};

export const settingsApi = {
  get: (key: string) => invoke<string | null>("settings_get", { key }),
  set: (key: string, value: string) => invoke<void>("settings_set", { key, value }),
  getAll: () => invoke<Record<string, string>>("settings_get_all"),
};

export interface ShortcutBinding {
  ctrl: boolean;
  shift: boolean;
  alt: boolean;
  meta: boolean;
  code: string;
}

export interface DataInfo {
  appDataDir: string;
  dbSizeBytes: number;
  attachmentCount: number;
  todoCount: number;
}

export interface DataImportResult {
  categoriesCreated: number;
  categoriesReused: number;
  tagsCreated: number;
  tagsReused: number;
  kanbanColumnsCreated: number;
  kanbanColumnsReused: number;
  todosImported: number;
  todosSkipped: number;
}

export interface EmailGatewayPublicConfig {
  enabled: boolean;
  host: string;
  port: number;
  security: string;
  authType: string;
  username: string;
  fromAddress: string;
  fromName: string;
  defaultRecipient: string;
  hasPassword: boolean;
}

export interface EmailGatewaySaveInput {
  enabled: boolean;
  host: string;
  port: number;
  security: string;
  authType: string;
  username: string;
  password?: string;
  fromAddress: string;
  fromName: string;
  defaultRecipient: string;
}

export const emailGatewayApi = {
  getConfig: () => invoke<EmailGatewayPublicConfig>("email_gateway_get_config"),
  saveConfig: (config: EmailGatewaySaveInput) =>
    invoke<EmailGatewayPublicConfig>("email_gateway_save_config", { config }),
  sendTest: () => invoke<void>("email_gateway_send_test"),
};

export const dataApi = {
  getInfo: () => invoke<DataInfo>("data_get_info"),
  openAppDataDir: () => invoke<void>("data_open_app_data_dir"),
  createBackup: () => invoke<string | null>("data_create_backup"),
  restoreBackup: () => invoke<boolean>("data_restore_backup"),
  exportJson: () => invoke<string | null>("data_export_json"),
  importJson: () => invoke<DataImportResult | null>("data_import_json"),
};

export const windowApi = {
  showMain: () => invoke<void>("window_show_main"),
  openTaskDetail: (id: number) => invoke<void>("window_open_task_detail", { id }),
  setOpacity: (opacity: number) => invoke<void>("window_set_opacity", { opacity }),
  minimalDockOnBlur: () => invoke<void>("minimal_dock_on_blur"),
};

export const shortcutApi = {
  getQuickCapture: () => invoke<ShortcutBinding>("shortcut_get_quick_capture"),
  setQuickCapture: (binding: ShortcutBinding) =>
    invoke<ShortcutBinding>("shortcut_set_quick_capture", { binding }),
  getToggleMain: () => invoke<ShortcutBinding>("shortcut_get_toggle_main"),
  setToggleMain: (binding: ShortcutBinding) =>
    invoke<ShortcutBinding>("shortcut_set_toggle_main", { binding }),
};

export function parseAttachmentUrl(url: string): { todoId: number; filename: string } | null {
  const match = url.match(/^local:\/\/attachment\/(\d+)\/(.+)$/);
  if (!match) return null;
  return { todoId: Number(match[1]), filename: match[2] };
}

export async function resolveAttachmentUrl(url: string): Promise<string> {
  const parsed = parseAttachmentUrl(url);
  if (!parsed) return url;
  const base64 = await attachmentApi.read(parsed.todoId, parsed.filename);
  const info = await attachmentApi.list(parsed.todoId);
  const file = info.find((item) => item.filename === parsed.filename);
  const mime = file?.mimeType || "image/png";
  return `data:${mime};base64,${base64}`;
}

export async function resolveHtmlImages(html: string): Promise<string> {
  const doc = new DOMParser().parseFromString(html, "text/html");
  const images = doc.querySelectorAll("img");
  for (const img of images) {
    const src = img.getAttribute("src");
    if (src?.startsWith("local://attachment/")) {
      img.setAttribute("src", await resolveAttachmentUrl(src));
    }
  }
  return doc.body.innerHTML;
}
