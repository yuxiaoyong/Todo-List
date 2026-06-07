import { i18n } from "../i18n";

function parseDate(value: string): Date {
  return new Date(value.length <= 10 ? `${value}T00:00:00` : value);
}

function startOfDay(date: Date): Date {
  return new Date(date.getFullYear(), date.getMonth(), date.getDate());
}

export function isTodoOverdue(todo: { dueDate?: string; completed: boolean }): boolean {
  if (todo.completed || !todo.dueDate) return false;

  const due = parseDate(todo.dueDate);
  if (Number.isNaN(due.getTime())) return false;

  const now = new Date();
  if (todo.dueDate.length <= 10) {
    return startOfDay(due).getTime() < startOfDay(now).getTime();
  }
  return due.getTime() < now.getTime();
}

function currentDateLocale(): string {
  return i18n.global.locale.value === "en" ? "en-US" : "zh-CN";
}

export function formatDateCn(value?: string | null): string {
  if (!value) return i18n.global.t("common.none");
  const d = new Date(value.length <= 10 ? `${value}T00:00:00` : value);
  if (Number.isNaN(d.getTime())) return value;
  return d.toLocaleDateString(currentDateLocale(), {
    year: "numeric",
    month: "short",
    day: "numeric",
  });
}

export function formatDateTimeCn(value?: string | null): string {
  if (!value) return i18n.global.t("common.none");
  const d = new Date(value);
  if (Number.isNaN(d.getTime())) return value;
  return d.toLocaleString(currentDateLocale(), {
    year: "numeric",
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
}

export function formatFileSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}
