export type TaskListColumnId =
  | "pin"
  | "check"
  | "title"
  | "priority"
  | "status"
  | "startDate"
  | "dueDate"
  | "category"
  | "tags"
  | "assignee"
  | "kanbanColumn"
  | "recurrence"
  | "createdAt"
  | "updatedAt"
  | "actions";

export type TaskListColumnZone = "left" | "scroll" | "right";

export interface TaskListColumnItem {
  id: TaskListColumnId;
  visible: boolean;
  locked?: boolean;
  fixed?: TaskListColumnZone;
}

/** Fixed pixel widths — required for horizontal scroll + sticky columns */
export const TASK_LIST_COLUMN_WIDTH_PX: Record<TaskListColumnId, number> = {
  pin: 40,
  check: 48,
  title: 280,
  priority: 108,
  status: 88,
  startDate: 118,
  dueDate: 118,
  category: 128,
  tags: 180,
  assignee: 112,
  kanbanColumn: 128,
  recurrence: 168,
  createdAt: 148,
  updatedAt: 148,
  actions: 120,
};

const TRASH_ACTIONS_WIDTH_PX = 140;

export const TASK_LIST_COLUMN_GAP_PX = 8;
export const TASK_LIST_ZONE_PADDING_PX = 12;
export const TASK_LIST_SETTINGS_WIDTH_PX = 28;
export const TASK_LIST_SETTINGS_RIGHT_INSET_PX = 20;
export const TASK_LIST_SETTINGS_INSET_PX =
  TASK_LIST_SETTINGS_WIDTH_PX + TASK_LIST_SETTINGS_RIGHT_INSET_PX;

export const LEFT_FIXED_COLUMN_IDS: TaskListColumnId[] = ["pin", "check", "title"];
export const RIGHT_FIXED_COLUMN_IDS: TaskListColumnId[] = ["actions"];

export const DEFAULT_TASK_LIST_COLUMNS: TaskListColumnItem[] = [
  { id: "pin", visible: true, fixed: "left" },
  { id: "check", visible: true, locked: true, fixed: "left" },
  { id: "title", visible: true, locked: true, fixed: "left" },
  { id: "priority", visible: true },
  { id: "status", visible: false },
  { id: "startDate", visible: false },
  { id: "dueDate", visible: true },
  { id: "category", visible: true },
  { id: "tags", visible: true },
  { id: "assignee", visible: false },
  { id: "kanbanColumn", visible: false },
  { id: "recurrence", visible: false },
  { id: "createdAt", visible: false },
  { id: "updatedAt", visible: false },
  { id: "actions", visible: true, locked: true, fixed: "right" },
];

export const TASK_LIST_COLUMN_I18N_KEYS: Record<TaskListColumnId, string> = {
  pin: "task.pin",
  check: "task.columnCheck",
  title: "task.taskName",
  priority: "task.priority",
  status: "task.columnStatus",
  startDate: "task.startDate",
  dueDate: "task.dueDate",
  category: "task.category",
  tags: "task.tags",
  assignee: "task.columnAssignee",
  kanbanColumn: "task.columnKanban",
  recurrence: "task.columnRecurrence",
  createdAt: "task.columnCreatedAt",
  updatedAt: "task.columnUpdatedAt",
  actions: "task.actions",
};

const KNOWN_IDS = new Set(DEFAULT_TASK_LIST_COLUMNS.map((column) => column.id));

function cloneDefaults(): TaskListColumnItem[] {
  return DEFAULT_TASK_LIST_COLUMNS.map((column) => ({ ...column }));
}

function mergeColumnItem(saved: TaskListColumnItem): TaskListColumnItem {
  const def = DEFAULT_TASK_LIST_COLUMNS.find((column) => column.id === saved.id)!;
  return {
    id: saved.id,
    visible: def.locked ? true : saved.visible,
    locked: def.locked,
    fixed: def.fixed,
  };
}

function insertIndexForDefaultColumn(
  result: TaskListColumnItem[],
  defaultIndex: number,
): number {
  for (let i = defaultIndex - 1; i >= 0; i -= 1) {
    const prevId = DEFAULT_TASK_LIST_COLUMNS[i].id;
    const prevIdx = result.findIndex((column) => column.id === prevId);
    if (prevIdx >= 0) return prevIdx + 1;
  }
  for (let i = defaultIndex + 1; i < DEFAULT_TASK_LIST_COLUMNS.length; i += 1) {
    const nextId = DEFAULT_TASK_LIST_COLUMNS[i].id;
    const nextIdx = result.findIndex((column) => column.id === nextId);
    if (nextIdx >= 0) return nextIdx;
  }
  return result.length;
}

function mergeWithDefaults(saved: TaskListColumnItem[]): TaskListColumnItem[] {
  const ordered: TaskListColumnItem[] = [];

  for (const item of saved) {
    if (!KNOWN_IDS.has(item.id)) continue;
    ordered.push(mergeColumnItem(item));
  }

  for (const [defaultIndex, def] of DEFAULT_TASK_LIST_COLUMNS.entries()) {
    if (ordered.some((column) => column.id === def.id)) continue;
    const insertAt = insertIndexForDefaultColumn(ordered, defaultIndex);
    ordered.splice(insertAt, 0, { ...def });
  }

  return ordered;
}

export function parseTaskListColumns(raw: string | null | undefined): TaskListColumnItem[] {
  if (!raw) return cloneDefaults();
  try {
    const parsed = JSON.parse(raw) as TaskListColumnItem[];
    if (!Array.isArray(parsed)) return cloneDefaults();
    return mergeWithDefaults(parsed);
  } catch {
    return cloneDefaults();
  }
}

export function serializeTaskListColumns(columns: TaskListColumnItem[]): string {
  return JSON.stringify(columns);
}

export function getVisibleColumns(
  columns: TaskListColumnItem[],
  options: { minimal?: boolean; isTrashMode?: boolean },
): TaskListColumnItem[] {
  if (options.minimal) {
    return columns.filter(
      (column) =>
        (column.id === "pin" || column.id === "check" || column.id === "title") &&
        column.visible &&
        (!options.isTrashMode || column.id !== "pin"),
    );
  }

  return columns.filter((column) => {
    if (!column.visible) return false;
    if (options.isTrashMode && column.id === "pin") return false;
    return true;
  });
}

export function splitVisibleColumns(columns: TaskListColumnItem[]) {
  const leftSet = new Set(LEFT_FIXED_COLUMN_IDS);
  const rightSet = new Set(RIGHT_FIXED_COLUMN_IDS);

  const left = LEFT_FIXED_COLUMN_IDS.map((id) => columns.find((column) => column.id === id)).filter(
    (column): column is TaskListColumnItem => !!column,
  );
  const right = RIGHT_FIXED_COLUMN_IDS.map((id) => columns.find((column) => column.id === id)).filter(
    (column): column is TaskListColumnItem => !!column,
  );
  const scroll = columns.filter((column) => !leftSet.has(column.id) && !rightSet.has(column.id));

  return { left, scroll, right };
}

export function columnWidthPx(columnId: TaskListColumnId, isTrashMode: boolean): number {
  if (isTrashMode && columnId === "actions") return TRASH_ACTIONS_WIDTH_PX;
  return TASK_LIST_COLUMN_WIDTH_PX[columnId];
}

export function zoneWidthPx(
  columns: TaskListColumnItem[],
  isTrashMode: boolean,
  zone: TaskListColumnZone,
): number {
  if (!columns.length) return 0;
  const gaps = Math.max(0, columns.length - 1) * TASK_LIST_COLUMN_GAP_PX;
  const cells = columns.reduce((sum, column) => sum + columnWidthPx(column.id, isTrashMode), 0);
  const edgePadding =
    (zone === "left" ? TASK_LIST_ZONE_PADDING_PX : 0) +
    (zone === "right" ? TASK_LIST_ZONE_PADDING_PX : 0);
  return cells + gaps + edgePadding;
}

export function buildZoneGridTemplate(
  columns: TaskListColumnItem[],
  isTrashMode: boolean,
  flexColumnId?: TaskListColumnId,
): string {
  if (!columns.length) return "";
  return columns
    .map((column) => {
      const width = columnWidthPx(column.id, isTrashMode);
      if (column.id === flexColumnId) {
        return `minmax(${width}px, 1fr)`;
      }
      return `${width}px`;
    })
    .join(" ");
}

/** @deprecated use buildZoneGridTemplate per zone */
export function buildGridTemplate(
  visibleColumns: TaskListColumnItem[],
  isTrashMode: boolean,
): string {
  return buildZoneGridTemplate(visibleColumns, isTrashMode);
}

export function tableMinWidthPx(
  left: TaskListColumnItem[],
  scroll: TaskListColumnItem[],
  right: TaskListColumnItem[],
  isTrashMode: boolean,
): number {
  return (
    zoneWidthPx(left, isTrashMode, "left") +
    zoneWidthPx(scroll, isTrashMode, "scroll") +
    zoneWidthPx(right, isTrashMode, "right")
  );
}
