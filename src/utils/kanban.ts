import { i18n } from "../i18n";
import type { KanbanColumn, TodoSummary } from "../types";

export const KANBAN_COLOR_PRESETS = [
  "#1677ff",
  "#cf1322",
  "#d48806",
  "#52c41a",
  "#722ed1",
  "#8c8c8c",
  "#13c2c2",
  "#eb2f96",
];

export function getKanbanColumnLabel(
  columnId?: number | null,
  columns: KanbanColumn[] = [],
): string {
  if (!columnId) return i18n.global.t("kanban.unassigned");
  const column = columns.find((item) => item.id === columnId);
  return column?.name ?? i18n.global.t("kanban.unassigned");
}

function sortBucketTodos(todos: TodoSummary[]) {
  return [...todos].sort((a, b) => {
    if (a.sortOrder !== b.sortOrder) return a.sortOrder - b.sortOrder;
    return b.updatedAt.localeCompare(a.updatedAt);
  });
}

export function groupTodosByKanban(todos: TodoSummary[], columns: KanbanColumn[]) {
  const buckets: Record<number, TodoSummary[]> = {};
  for (const column of columns) {
    buckets[column.id] = [];
  }
  const unassigned: TodoSummary[] = [];

  for (const todo of todos) {
    const columnId = todo.kanbanColumnId;
    if (columnId && buckets[columnId]) {
      buckets[columnId].push(todo);
    } else {
      unassigned.push(todo);
    }
  }

  for (const column of columns) {
    buckets[column.id] = sortBucketTodos(buckets[column.id]);
  }

  return { buckets, unassigned: sortBucketTodos(unassigned) };
}

export function buildBucketSortPositions(bucket: TodoSummary[]) {
  if (!bucket.length) return [];
  const base = Math.min(...bucket.map((todo) => todo.sortOrder));
  return bucket.map((todo, index) => ({
    id: todo.id,
    sortOrder: base + index,
  }));
}

export function nextKanbanColor(columns: KanbanColumn[]): string {
  return KANBAN_COLOR_PRESETS[columns.length % KANBAN_COLOR_PRESETS.length];
}
