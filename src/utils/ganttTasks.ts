import type { TodoSummary } from "../types";

export interface FrappeGanttTask {
  id: string;
  name: string;
  start: string;
  end: string;
  progress: number;
  custom_class?: string;
}

const MAX_YEARS_PAST = 5;
const MAX_YEARS_FUTURE = 3;

function parseTodoDate(value: string): Date {
  return new Date(value.length <= 10 ? `${value}T00:00:00` : value);
}

function formatDateOnly(date: Date): string {
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, "0");
  const day = String(date.getDate()).padStart(2, "0");
  return `${year}-${month}-${day}`;
}

function addDays(date: Date, days: number): Date {
  const next = new Date(date);
  next.setDate(next.getDate() + days);
  return next;
}

function ensureRange(start: Date, end: Date): { start: Date; end: Date } {
  if (start.getTime() <= end.getTime()) {
    return { start, end };
  }
  return { start: end, end: start };
}

function isReasonableScheduleDate(date: Date, todo: TodoSummary): boolean {
  if (Number.isNaN(date.getTime())) return false;

  const now = new Date();
  const oldest = addDays(now, -365 * MAX_YEARS_PAST);
  const newest = addDays(now, 365 * MAX_YEARS_FUTURE);
  if (date.getTime() < oldest.getTime() || date.getTime() > newest.getTime()) {
    return false;
  }

  const created = parseTodoDate(todo.createdAt);
  return date.getTime() >= addDays(created, -30).getTime();
}

export function resolveTodoDateRange(todo: TodoSummary): { start: string; end: string } {
  const created = parseTodoDate(todo.createdAt);
  const startRaw =
    todo.startDate && isReasonableScheduleDate(parseTodoDate(todo.startDate), todo)
      ? parseTodoDate(todo.startDate)
      : null;
  const dueRaw =
    todo.dueDate && isReasonableScheduleDate(parseTodoDate(todo.dueDate), todo)
      ? parseTodoDate(todo.dueDate)
      : null;

  let start: Date;
  let end: Date;

  if (startRaw && dueRaw) {
    ({ start, end } = ensureRange(startRaw, dueRaw));
  } else if (startRaw) {
    start = startRaw;
    end = addDays(startRaw, 1);
  } else if (dueRaw) {
    end = dueRaw;
    start = addDays(dueRaw, -3);
    if (start.getTime() < created.getTime()) {
      start = created;
    }
  } else {
    start = created;
    end = addDays(created, 7);
  }

  return {
    start: formatDateOnly(start),
    end: formatDateOnly(end),
  };
}

export function todoToGanttTask(todo: TodoSummary): FrappeGanttTask {
  const { start, end } = resolveTodoDateRange(todo);

  return {
    id: String(todo.id),
    name: todo.title,
    start,
    end,
    progress: todo.completed ? 100 : 0,
    custom_class: `gantt-priority-${todo.priority}${todo.completed ? " gantt-completed" : ""}`,
  };
}

export function todosToGanttTasks(todos: TodoSummary[]): FrappeGanttTask[] {
  return todos.map(todoToGanttTask);
}

export function ganttDatesToApi(start: Date, end: Date): { startDate: string; dueDate: string } {
  const range = ensureRange(start, end);
  return {
    startDate: formatDateOnly(range.start),
    dueDate: formatDateOnly(range.end),
  };
}

export function ganttLocaleFromAppLocale(locale: string): string {
  return locale === "en" ? "en" : "zh";
}
