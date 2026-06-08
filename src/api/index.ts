import { tauriInvoke } from "../utils/tauriInvoke";
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
  list: () => tauriInvoke<Category[]>("category_list"),
  create: (name: string, color?: string, icon?: string) =>
    tauriInvoke<Category>("category_create", { input: { name, color, icon } }),
  update: (id: number, name: string, color: string, icon?: string) =>
    tauriInvoke<Category>("category_update", { input: { id, name, color, icon } }),
  delete: (id: number) => tauriInvoke<void>("category_delete", { id }),
  reorder: (ids: number[]) => tauriInvoke<void>("category_reorder", { ids }),
};

export const kanbanColumnApi = {
  list: () => tauriInvoke<KanbanColumn[]>("kanban_column_list"),
  create: (name: string, color?: string, subtitle?: string) =>
    tauriInvoke<KanbanColumn>("kanban_column_create", { input: { name, color, subtitle } }),
  update: (id: number, name: string, color: string, subtitle?: string | null) =>
    tauriInvoke<KanbanColumn>("kanban_column_update", { input: { id, name, color, subtitle } }),
  delete: (id: number) => tauriInvoke<void>("kanban_column_delete", { id }),
  reorder: (ids: number[]) => tauriInvoke<void>("kanban_column_reorder", { ids }),
};

export const tagApi = {
  list: () => tauriInvoke<Tag[]>("tag_list"),
  create: (name: string, color?: string) =>
    tauriInvoke<Tag>("tag_create", { input: { name, color } }),
  update: (id: number, name: string, color: string) =>
    tauriInvoke<Tag>("tag_update", { input: { id, name, color } }),
  delete: (id: number) => tauriInvoke<void>("tag_delete", { id }),
  reorder: (ids: number[]) => tauriInvoke<void>("tag_reorder", { ids }),
};

export const todoApi = {
  list: (filter: TodoListFilter) => tauriInvoke<TodoSummary[]>("todo_list", { filter }),
  get: (id: number) => tauriInvoke<TodoDetail>("todo_get", { id }),
  create: (input: {
    title: string;
    categoryId?: number;
    tagIds?: number[];
    priority?: string;
    dueDate?: string;
    contentHtml?: string;
  }) => tauriInvoke<TodoDetail>("todo_create", { input }),
  update: (input: UpdateTodoInput) => tauriInvoke<TodoDetail>("todo_update", { input }),
  quickCreate: (input: QuickCreateInput) =>
    tauriInvoke<TodoDetail>("todo_quick_create", { input }),
  toggleComplete: (id: number) => tauriInvoke<TodoDetail>("todo_toggle_complete", { id }),
  togglePin: (id: number) => tauriInvoke<TodoDetail>("todo_toggle_pin", { id }),
  delete: (id: number) => tauriInvoke<void>("todo_delete", { id }),
  restore: (id: number) => tauriInvoke<TodoDetail>("todo_restore", { id }),
  permanentDelete: (id: number) => tauriInvoke<void>("todo_permanent_delete", { id }),
  emptyTrash: () => tauriInvoke<number>("todo_empty_trash"),
  reorder: (ids: number[]) => tauriInvoke<void>("todo_reorder", { ids }),
  reorderPositions: (items: { id: number; sortOrder: number }[]) =>
    tauriInvoke<void>("todo_reorder_positions", { items }),
  setKanbanColumn: (id: number, kanbanColumnId: number | null) =>
    tauriInvoke<TodoDetail>("todo_set_kanban_column", { id, kanbanColumnId }),
  incompleteCount: () => tauriInvoke<number>("todo_incomplete_count"),
  dueToday: () => tauriInvoke<TodoSummary[]>("todo_due_today"),
};

export const subtaskApi = {
  create: (todoId: number, title: string) =>
    tauriInvoke<Subtask>("subtask_create", { todoId, title }),
  update: (id: number, title: string) => tauriInvoke<Subtask>("subtask_update", { id, title }),
  toggle: (id: number) => tauriInvoke<Subtask>("subtask_toggle", { id }),
  delete: (id: number) => tauriInvoke<void>("subtask_delete", { id }),
};

export const attachmentApi = {
  save: (
    todoId: number,
    dataBase64: string,
    originalName?: string,
    mimeType = "image/png",
    kind: "inline" | "attachment" = "attachment",
  ) =>
    tauriInvoke<AttachmentInfo>("attachment_save", {
      todoId,
      dataBase64,
      originalName,
      mimeType,
      kind,
    }),
  delete: (id: number) => tauriInvoke<void>("attachment_delete", { id }),
  list: (todoId: number) => tauriInvoke<AttachmentInfo[]>("attachment_list", { todoId }),
  read: (todoId: number, filename: string) =>
    tauriInvoke<string>("attachment_read", { todoId, filename }),
  getPath: (todoId: number, filename: string) =>
    tauriInvoke<string>("attachment_get_path", { todoId, filename }),
  open: (todoId: number, filename: string) =>
    tauriInvoke<void>("attachment_open", { todoId, filename }),
};

export const settingsApi = {
  get: (key: string) =>
    tauriInvoke<string | null>("settings_get", { key }, { silent: true }),
  set: (key: string, value: string) => tauriInvoke<void>("settings_set", { key, value }),
  getAll: () => tauriInvoke<Record<string, string>>("settings_get_all", undefined, { silent: true }),
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
  getConfig: () =>
    tauriInvoke<EmailGatewayPublicConfig>("email_gateway_get_config", undefined, {
      silent: true,
    }),
  saveConfig: (config: EmailGatewaySaveInput) =>
    tauriInvoke<EmailGatewayPublicConfig>("email_gateway_save_config", { config }),
  sendTest: () => tauriInvoke<void>("email_gateway_send_test"),
};

export const dataApi = {
  getInfo: () => tauriInvoke<DataInfo>("data_get_info"),
  openAppDataDir: () => tauriInvoke<void>("data_open_app_data_dir"),
  createBackup: () => tauriInvoke<string | null>("data_create_backup"),
  restoreBackup: () => tauriInvoke<boolean>("data_restore_backup"),
  exportJson: () => tauriInvoke<string | null>("data_export_json"),
  importJson: () => tauriInvoke<DataImportResult | null>("data_import_json"),
};

export const windowApi = {
  showMain: () => tauriInvoke<void>("window_show_main"),
  openTaskDetail: (id: number) => tauriInvoke<void>("window_open_task_detail", { id }),
  setOpacity: (opacity: number) => tauriInvoke<void>("window_set_opacity", { opacity }),
  minimalDockOnBlur: () => tauriInvoke<void>("minimal_dock_on_blur"),
};

export const shortcutApi = {
  getQuickCapture: () =>
    tauriInvoke<ShortcutBinding>("shortcut_get_quick_capture", undefined, { silent: true }),
  setQuickCapture: (binding: ShortcutBinding) =>
    tauriInvoke<ShortcutBinding>("shortcut_set_quick_capture", { binding }),
  getToggleMain: () =>
    tauriInvoke<ShortcutBinding>("shortcut_get_toggle_main", undefined, { silent: true }),
  setToggleMain: (binding: ShortcutBinding) =>
    tauriInvoke<ShortcutBinding>("shortcut_set_toggle_main", { binding }),
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
