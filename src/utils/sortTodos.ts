import type { TodoSummary } from "../types";

export type TodoSortField =
  | "default"
  | "updatedAt"
  | "createdAt"
  | "startDate"
  | "dueDate"
  | "priority"
  | "title"
  | "assignee";

export type SortDirection = "asc" | "desc";

const PRIORITY_RANK: Record<string, number> = { high: 3, medium: 2, low: 1 };

function compareStrings(a: string, b: string, direction: SortDirection): number {
  const cmp = a.localeCompare(b, "zh-CN");
  return direction === "asc" ? cmp : -cmp;
}

function compareNumbers(a: number, b: number, direction: SortDirection): number {
  return direction === "asc" ? a - b : b - a;
}

function compareDates(
  a: string | undefined,
  b: string | undefined,
  direction: SortDirection,
): number {
  const ta = a ? Date.parse(a) : Number.NaN;
  const tb = b ? Date.parse(b) : Number.NaN;
  const aMissing = Number.isNaN(ta);
  const bMissing = Number.isNaN(tb);
  if (aMissing && bMissing) return 0;
  if (aMissing) return 1;
  if (bMissing) return -1;
  return direction === "asc" ? ta - tb : tb - ta;
}

export const TODO_SORT_PROP_MAP: Record<string, TodoSortField> = {
  title: "title",
  priority: "priority",
  startDate: "startDate",
  dueDate: "dueDate",
  updatedAt: "updatedAt",
  createdAt: "createdAt",
  assignee: "assignee",
};

export function parseTodoSort(value: string): {
  field: TodoSortField;
  direction: SortDirection;
} {
  const [field, direction] = value.split(":") as [TodoSortField, SortDirection];
  return {
    field: field ?? "default",
    direction: direction === "asc" ? "asc" : "desc",
  };
}

export function buildTodoSort(field: TodoSortField, direction: SortDirection): string {
  return `${field}:${direction}`;
}

/** 列头点击循环：未排序 → 升序 → 降序 → 恢复默认 */
export function nextTodoSort(current: string, clickedField: TodoSortField): string {
  const { field, direction } = parseTodoSort(current);
  if (field !== clickedField) {
    return buildTodoSort(clickedField, "asc");
  }
  if (direction === "asc") {
    return buildTodoSort(clickedField, "desc");
  }
  return buildTodoSort("default", "desc");
}

export function todoSortToTableDefault(value: string): { prop: string; order: "ascending" | "descending" } | undefined {
  const { field, direction } = parseTodoSort(value);
  const prop = Object.entries(TODO_SORT_PROP_MAP).find(([, f]) => f === field)?.[0];
  if (!prop || field === "default") return undefined;
  return {
    prop,
    order: direction === "asc" ? "ascending" : "descending",
  };
}

function compareDefaultOrder(a: TodoSummary, b: TodoSummary): number {
  if (a.sortOrder !== b.sortOrder) return a.sortOrder - b.sortOrder;
  return compareDates(a.updatedAt, b.updatedAt, "desc");
}

function compareByField(
  a: TodoSummary,
  b: TodoSummary,
  field: TodoSortField,
  direction: SortDirection,
): number {
  switch (field) {
    case "updatedAt":
      return compareDates(a.updatedAt, b.updatedAt, direction);
    case "createdAt":
      return compareDates(a.createdAt, b.createdAt, direction);
    case "startDate":
      return compareDates(a.startDate, b.startDate, direction);
    case "dueDate":
      return compareDates(a.dueDate, b.dueDate, direction);
    case "assignee":
      return compareStrings(a.assignee || "", b.assignee || "", direction);
    case "priority":
      return compareNumbers(
        PRIORITY_RANK[a.priority] ?? 0,
        PRIORITY_RANK[b.priority] ?? 0,
        direction,
      );
    case "title":
      return compareStrings(a.title, b.title, direction);
    default:
      return 0;
  }
}

export function sortTodos(
  todos: TodoSummary[],
  field: TodoSortField,
  direction: SortDirection,
): TodoSummary[] {
  const pinned = todos.filter((todo) => todo.pinned);
  const unpinned = todos.filter((todo) => !todo.pinned);

  const sortedPinned = [...pinned].sort(compareDefaultOrder);

  if (field === "default") {
    const sortedUnpinned = [...unpinned].sort(compareDefaultOrder);
    return [...sortedPinned, ...sortedUnpinned];
  }

  const sortedUnpinned = [...unpinned].sort((a, b) => compareByField(a, b, field, direction));
  return [...sortedPinned, ...sortedUnpinned];
}
