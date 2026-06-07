import type { TodoSummary } from "../types";

export type TodoSortField =
  | "default"
  | "updatedAt"
  | "createdAt"
  | "dueDate"
  | "priority"
  | "title";

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
  dueDate: "dueDate",
  updatedAt: "updatedAt",
  createdAt: "createdAt",
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

export function todoSortToTableDefault(value: string): { prop: string; order: "ascending" | "descending" } | undefined {
  const { field, direction } = parseTodoSort(value);
  const prop = Object.entries(TODO_SORT_PROP_MAP).find(([, f]) => f === field)?.[0];
  if (!prop || field === "default") return undefined;
  return {
    prop,
    order: direction === "asc" ? "ascending" : "descending",
  };
}

export function sortTodos(
  todos: TodoSummary[],
  field: TodoSortField,
  direction: SortDirection,
): TodoSummary[] {
  const list = [...todos];
  list.sort((a, b) => {
    if (field === "default") {
      if (a.pinned !== b.pinned) return a.pinned ? -1 : 1;
      if (a.sortOrder !== b.sortOrder) return a.sortOrder - b.sortOrder;
      return compareDates(a.updatedAt, b.updatedAt, "desc");
    }
    switch (field) {
      case "updatedAt":
        return compareDates(a.updatedAt, b.updatedAt, direction);
      case "createdAt":
        return compareDates(a.createdAt, b.createdAt, direction);
      case "dueDate":
        return compareDates(a.dueDate, b.dueDate, direction);
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
  });
  return list;
}
