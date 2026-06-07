import type { TodoSummary } from "../types";

export interface TodoStats {
  total: number;
  completed: number;
  completionRate: number;
  noDeadline: number;
}

export function computeStats(todos: TodoSummary[]): TodoStats {
  const total = todos.length;
  const completed = todos.filter((t) => t.completed).length;
  const noDeadline = todos.filter((t) => !t.dueDate).length;
  const completionRate = total > 0 ? Math.round((completed / total) * 1000) / 10 : 0;

  return { total, completed, completionRate, noDeadline };
}
