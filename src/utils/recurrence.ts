import type { ComposerTranslation } from "vue-i18n";
import {
  formatLunarLabel,
  lunarToSolar,
  resolveLunarYear,
  solarToLunar,
} from "./lunar";

export type RecurrenceFreq = "daily" | "weekly" | "monthly" | "quarterly" | "yearly";
export type RecurrenceAnchor = "startDate" | "dueDate";
export type RecurrenceOnComplete = "reschedule" | "stay";
export type RecurrenceCalendar = "solar" | "lunar";

export interface RecurrenceConfig {
  enabled: boolean;
  freq: RecurrenceFreq;
  interval: number;
  anchor: RecurrenceAnchor;
  calendar: RecurrenceCalendar;
  lunarMonth?: number | null;
  lunarDay?: number | null;
  isLeapMonth?: boolean;
  firstReminderDate?: string | null;
  time: string;
  advanceMinutes: number;
  onComplete: RecurrenceOnComplete;
}

export const RECURRENCE_FREQ_OPTIONS: RecurrenceFreq[] = [
  "daily",
  "weekly",
  "monthly",
  "quarterly",
  "yearly",
];

export const RECURRENCE_ADVANCE_OPTIONS = [0, 15, 30, 60, 1440] as const;

export function defaultRecurrenceConfig(): RecurrenceConfig {
  return {
    enabled: false,
    freq: "yearly",
    interval: 1,
    anchor: "dueDate",
    calendar: "solar",
    lunarMonth: null,
    lunarDay: null,
    isLeapMonth: false,
    firstReminderDate: null,
    time: "09:00",
    advanceMinutes: 0,
    onComplete: "reschedule",
  };
}

export function parseRecurrenceConfig(
  raw?: RecurrenceConfig | null,
): RecurrenceConfig {
  if (!raw) return defaultRecurrenceConfig();
  return {
    ...defaultRecurrenceConfig(),
    ...raw,
    calendar: raw.calendar === "lunar" ? "lunar" : "solar",
    interval: Math.max(1, Math.min(99, raw.interval || 1)),
    lunarMonth: raw.lunarMonth ?? null,
    lunarDay: raw.lunarDay ?? null,
    isLeapMonth: !!raw.isLeapMonth,
  };
}

export function recurrenceAnchorDate(
  anchor: RecurrenceAnchor,
  dates: { startDate?: string | null; dueDate?: string | null },
): string | undefined {
  const value = anchor === "startDate" ? dates.startDate : dates.dueDate;
  if (!value) return undefined;
  return value.length > 10 ? value.slice(0, 10) : value;
}

export function recurrenceLunarReady(config: RecurrenceConfig): boolean {
  return !!(config.lunarMonth && config.lunarDay);
}

export function recurrenceStartDate(
  config: RecurrenceConfig,
  dates: { startDate?: string | null; dueDate?: string | null },
): string | undefined {
  if (config.calendar === "lunar") {
    if (!recurrenceLunarReady(config)) return undefined;
    const lunarYear = resolveLunarYear(
      config.firstReminderDate,
      recurrenceAnchorDate(config.anchor, dates),
    );
    return lunarToSolar(
      lunarYear,
      config.lunarMonth!,
      config.lunarDay!,
      config.isLeapMonth,
    ) ?? undefined;
  }
  const first = config.firstReminderDate?.trim();
  if (first) {
    return first.length > 10 ? first.slice(0, 10) : first;
  }
  return recurrenceAnchorDate(config.anchor, dates);
}

export function formatRecurrenceSummary(
  config: RecurrenceConfig,
  dates: { startDate?: string | null; dueDate?: string | null },
  t: ComposerTranslation,
  locale: string,
): string {
  if (!config.enabled) return t("taskDetail.recurrenceDisabled");
  const freqLabel = t(`taskDetail.recurrenceFreq.${config.freq}`);
  const intervalLabel =
    config.interval > 1
      ? t("taskDetail.recurrenceEveryN", { n: config.interval, freq: freqLabel })
      : freqLabel;

  if (config.calendar === "lunar" && recurrenceLunarReady(config)) {
    const lunarYear = resolveLunarYear(
      config.firstReminderDate,
      recurrenceAnchorDate(config.anchor, dates),
    );
    const lunarLabel = formatLunarLabel(
      lunarYear,
      config.lunarMonth!,
      config.lunarDay!,
      !!config.isLeapMonth,
      locale,
    );
    return `${intervalLabel} · ${t("taskDetail.recurrenceCalendar.lunar")}${lunarLabel} · ${config.time}`;
  }

  const startDate = recurrenceStartDate(config, dates);
  const datePart = startDate ? ` · ${startDate}` : "";
  return `${intervalLabel}${datePart} · ${config.time}`;
}

export function syncLunarFromSolar(
  config: RecurrenceConfig,
  solarYmd?: string | null,
): void {
  if (!solarYmd) return;
  const parts = solarToLunar(solarYmd);
  if (!parts) return;
  config.lunarMonth = parts.lunarMonth;
  config.lunarDay = parts.lunarDay;
  config.isLeapMonth = parts.isLeapMonth;
}

function parseDateParts(value: string): { year: number; month: number; day: number } | null {
  const match = /^(\d{4})-(\d{2})-(\d{2})/.exec(value);
  if (!match) return null;
  return {
    year: Number(match[1]),
    month: Number(match[2]),
    day: Number(match[3]),
  };
}

function addMonths(year: number, month: number, day: number, months: number) {
  let nextMonth = month - 1 + months;
  let nextYear = year + Math.floor(nextMonth / 12);
  nextMonth = ((nextMonth % 12) + 12) % 12;
  const lastDay = new Date(nextYear, nextMonth + 1, 0).getDate();
  const nextDay = Math.min(day, lastDay);
  return new Date(nextYear, nextMonth, nextDay);
}

function computeNextSolarRecurrenceDate(
  config: RecurrenceConfig,
  dates: { startDate?: string | null; dueDate?: string | null },
  from: Date,
): string | null {
  const start = recurrenceStartDate(config, dates);
  if (!start) return null;

  const parts = parseDateParts(start);
  if (!parts) return null;

  const [hour, minute] = config.time.split(":").map((part) => Number(part) || 0);
  let candidate = new Date(parts.year, parts.month - 1, parts.day, hour, minute, 0, 0);

  const advanceMs = config.advanceMinutes * 60_000;
  const triggerAt = (date: Date) => date.getTime() - advanceMs;

  while (triggerAt(candidate) <= from.getTime()) {
    switch (config.freq) {
      case "daily":
        candidate = new Date(candidate.getTime() + config.interval * 86_400_000);
        break;
      case "weekly":
        candidate = new Date(candidate.getTime() + config.interval * 7 * 86_400_000);
        break;
      case "monthly":
        candidate = addMonths(
          candidate.getFullYear(),
          candidate.getMonth() + 1,
          candidate.getDate(),
          config.interval,
        );
        candidate.setHours(hour, minute, 0, 0);
        break;
      case "quarterly":
        candidate = addMonths(
          candidate.getFullYear(),
          candidate.getMonth() + 1,
          candidate.getDate(),
          config.interval * 3,
        );
        candidate.setHours(hour, minute, 0, 0);
        break;
      case "yearly":
        candidate = new Date(
          candidate.getFullYear() + config.interval,
          candidate.getMonth(),
          candidate.getDate(),
          hour,
          minute,
          0,
          0,
        );
        break;
      default:
        return null;
    }
  }

  const y = candidate.getFullYear();
  const m = String(candidate.getMonth() + 1).padStart(2, "0");
  const d = String(candidate.getDate()).padStart(2, "0");
  return `${y}-${m}-${d}`;
}

function computeNextLunarRecurrenceDate(
  config: RecurrenceConfig,
  dates: { startDate?: string | null; dueDate?: string | null },
  from: Date,
): string | null {
  if (!recurrenceLunarReady(config)) return null;

  const month = config.lunarMonth!;
  const day = config.lunarDay!;
  const isLeap = !!config.isLeapMonth;
  const interval = Math.max(1, config.interval);
  const startYear = resolveLunarYear(
    config.firstReminderDate,
    recurrenceAnchorDate(config.anchor, dates),
    from,
  );

  const [hour, minute] = config.time.split(":").map((part) => Number(part) || 0);
  const advanceMs = config.advanceMinutes * 60_000;
  const fromMs = from.getTime();

  for (let year = startYear; year < startYear + 200; year += interval) {
    const solarYmd = lunarToSolar(year, month, day, isLeap);
    if (!solarYmd) continue;
    const parts = parseDateParts(solarYmd);
    if (!parts) continue;
    const candidate = new Date(parts.year, parts.month - 1, parts.day, hour, minute, 0, 0);
    if (candidate.getTime() - advanceMs > fromMs) {
      return solarYmd;
    }
  }
  return null;
}

export function computeNextRecurrenceDate(
  config: RecurrenceConfig,
  dates: { startDate?: string | null; dueDate?: string | null },
  from = new Date(),
): string | null {
  if (!config.enabled) return null;
  if (config.calendar === "lunar") {
    return computeNextLunarRecurrenceDate(config, dates, from);
  }
  return computeNextSolarRecurrenceDate(config, dates, from);
}
