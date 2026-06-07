// lunar-javascript has no bundled types
// eslint-disable-next-line @typescript-eslint/ban-ts-comment
// @ts-expect-error
import { Lunar, LunarYear, Solar } from "lunar-javascript";

export interface LunarDateParts {
  lunarYear: number;
  lunarMonth: number;
  lunarDay: number;
  isLeapMonth: boolean;
}

export const LUNAR_FESTIVAL_PRESETS = [
  { id: "spring", lunarMonth: 1, lunarDay: 1 },
  { id: "lantern", lunarMonth: 1, lunarDay: 15 },
  { id: "dragonBoat", lunarMonth: 5, lunarDay: 5 },
  { id: "midAutumn", lunarMonth: 8, lunarDay: 15 },
  { id: "chongyang", lunarMonth: 9, lunarDay: 9 },
] as const;

export function lunarToSolar(
  lunarYear: number,
  month: number,
  day: number,
  isLeapMonth = false,
): string | null {
  try {
    const lunarMonth = isLeapMonth ? -month : month;
    const lunar = Lunar.fromYmd(lunarYear, lunarMonth, day);
    const solar = lunar.getSolar();
    const y = solar.getYear();
    const m = String(solar.getMonth()).padStart(2, "0");
    const d = String(solar.getDay()).padStart(2, "0");
    return `${y}-${m}-${d}`;
  } catch {
    return null;
  }
}

export function solarToLunar(solarYmd: string): LunarDateParts | null {
  const match = /^(\d{4})-(\d{2})-(\d{2})/.exec(solarYmd.trim());
  if (!match) return null;
  try {
    const solar = Solar.fromYmd(Number(match[1]), Number(match[2]), Number(match[3]));
    const lunar = solar.getLunar();
    const month = lunar.getMonth();
    return {
      lunarYear: lunar.getYear(),
      lunarMonth: Math.abs(month),
      lunarDay: lunar.getDay(),
      isLeapMonth: month < 0,
    };
  } catch {
    return null;
  }
}

export function getLunarMonthDayCount(
  lunarYear: number,
  month: number,
  isLeapMonth = false,
): number {
  try {
    const lunarMonth = LunarYear.fromYear(lunarYear).getMonth(isLeapMonth ? -month : month);
    return lunarMonth?.getDayCount() ?? 30;
  } catch {
    return 30;
  }
}

export function yearHasLeapMonth(lunarYear: number, month: number): boolean {
  try {
    return LunarYear.fromYear(lunarYear).getMonth(-month) != null;
  } catch {
    return false;
  }
}

export function formatLunarLabel(
  lunarYear: number,
  month: number,
  day: number,
  isLeapMonth: boolean,
  locale: string,
): string {
  try {
    const lunarMonth = isLeapMonth ? -month : month;
    const lunar = Lunar.fromYmd(lunarYear, lunarMonth, day);
    if (locale.startsWith("zh")) {
      return `${lunar.getMonthInChinese()}${lunar.getDayInChinese()}`;
    }
    const leap = isLeapMonth ? "Leap " : "";
    return `${leap}Lunar ${month}/${day}`;
  } catch {
    return `${month}/${day}`;
  }
}

export function currentLunarYear(from = new Date()): number {
  const solar = Solar.fromYmd(from.getFullYear(), from.getMonth() + 1, from.getDate());
  return solar.getLunar().getYear();
}

export function resolveLunarYear(
  firstReminderDate: string | null | undefined,
  fallbackSolar?: string,
  from = new Date(),
): number {
  if (firstReminderDate) {
    const parts = solarToLunar(firstReminderDate);
    if (parts) return parts.lunarYear;
  }
  if (fallbackSolar) {
    const parts = solarToLunar(fallbackSolar);
    if (parts) return parts.lunarYear;
  }
  return currentLunarYear(from);
}
