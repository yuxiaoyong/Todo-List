import type { TimeFilter } from "../stores/ui";
import type { TodoSummary } from "../types";

function parseDate(str: string): Date {
  return new Date(str.length <= 10 ? `${str}T00:00:00` : str);
}

function startOfDay(d: Date) {
  return new Date(d.getFullYear(), d.getMonth(), d.getDate());
}

function startOfWeek(d: Date) {
  const day = d.getDay() || 7;
  const monday = new Date(d);
  monday.setDate(d.getDate() - day + 1);
  return startOfDay(monday);
}

export function matchesTimeFilter(todo: TodoSummary, filter: TimeFilter): boolean {
  if (filter === "all") return true;

  const ref = todo.dueDate || todo.createdAt.slice(0, 10);
  const date = parseDate(ref);
  const now = new Date();
  const today = startOfDay(now);

  switch (filter) {
    case "today":
      return startOfDay(date).getTime() === today.getTime();
    case "week": {
      const weekStart = startOfWeek(now);
      const weekEnd = new Date(weekStart);
      weekEnd.setDate(weekEnd.getDate() + 7);
      return date >= weekStart && date < weekEnd;
    }
    case "month":
      return date.getFullYear() === now.getFullYear() && date.getMonth() === now.getMonth();
    case "year":
      return date.getFullYear() === now.getFullYear();
    default:
      return true;
  }
}
