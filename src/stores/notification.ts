import { emit, listen } from "@tauri-apps/api/event";
import { defineStore } from "pinia";
import { settingsApi } from "../api";

export const NOTIFICATION_ENABLED_KEY = "notification.enabled";
export const NOTIFICATION_SYSTEM_KEY = "notification.system";
export const NOTIFICATION_EMAIL_KEY = "notification.email";
export const NOTIFICATION_ADVANCE_HOURS_KEY = "notification.advanceHours";
export const NOTIFICATION_REPEAT_MINUTES_KEY = "notification.repeatMinutes";

export const ADVANCE_HOUR_OPTIONS = [0, 1, 2, 3, 6, 12, 24, 48, 72, 168] as const;
export const REPEAT_MINUTE_OPTIONS = [15, 30, 60, 120, 240, 360, 720, 1440] as const;

export interface NotificationSettingsPayload {
  enabled: boolean;
  system: boolean;
  email: boolean;
  advanceHours: number;
  repeatMinutes: number;
}

let syncBound = false;

function parseBool(value: string | null | undefined, defaultValue: boolean): boolean {
  if (value === "false" || value === "0") return false;
  if (value === "true" || value === "1") return true;
  return defaultValue;
}

function parsePositiveInt(
  value: string | null | undefined,
  defaultValue: number,
  allowed: readonly number[],
): number {
  const parsed = Number.parseInt(value ?? "", 10);
  if (!Number.isFinite(parsed) || parsed < 0) return defaultValue;
  return allowed.includes(parsed) ? parsed : defaultValue;
}

async function bindNotificationSyncListener(
  store: ReturnType<typeof useNotificationStore>,
) {
  if (syncBound) return;
  syncBound = true;
  await listen<NotificationSettingsPayload>("notification-settings-changed", (event) => {
    const payload = event.payload;
    if (!payload) return;
    store.enabled = payload.enabled;
    store.system = payload.system;
    store.email = payload.email;
    store.advanceHours = payload.advanceHours;
    store.repeatMinutes = payload.repeatMinutes;
  });
}

export const useNotificationStore = defineStore("notification", {
  state: () => ({
    enabled: true,
    system: true,
    email: false,
    advanceHours: 0,
    repeatMinutes: 1440,
    ready: false,
  }),
  actions: {
    async init() {
      try {
        const [enabled, system, email, advanceHours, repeatMinutes] = await Promise.all([
          settingsApi.get(NOTIFICATION_ENABLED_KEY),
          settingsApi.get(NOTIFICATION_SYSTEM_KEY),
          settingsApi.get(NOTIFICATION_EMAIL_KEY),
          settingsApi.get(NOTIFICATION_ADVANCE_HOURS_KEY),
          settingsApi.get(NOTIFICATION_REPEAT_MINUTES_KEY),
        ]);
        this.enabled = parseBool(enabled, true);
        this.system = parseBool(system, true);
        this.email = parseBool(email, false);
        this.advanceHours = parsePositiveInt(advanceHours, 0, ADVANCE_HOUR_OPTIONS);
        this.repeatMinutes = parsePositiveInt(repeatMinutes, 1440, REPEAT_MINUTE_OPTIONS);
      } catch (error) {
        console.error("load notification settings failed", error);
      }
      await bindNotificationSyncListener(this);
      this.ready = true;
    },
    async persist() {
      try {
        await Promise.all([
          settingsApi.set(NOTIFICATION_ENABLED_KEY, String(this.enabled)),
          settingsApi.set(NOTIFICATION_SYSTEM_KEY, String(this.system)),
          settingsApi.set(NOTIFICATION_EMAIL_KEY, String(this.email)),
          settingsApi.set(NOTIFICATION_ADVANCE_HOURS_KEY, String(this.advanceHours)),
          settingsApi.set(NOTIFICATION_REPEAT_MINUTES_KEY, String(this.repeatMinutes)),
        ]);
      } catch (error) {
        console.error("save notification settings failed", error);
        throw error;
      }
      try {
        await emit("notification-settings-changed", {
          enabled: this.enabled,
          system: this.system,
          email: this.email,
          advanceHours: this.advanceHours,
          repeatMinutes: this.repeatMinutes,
        } satisfies NotificationSettingsPayload);
      } catch (error) {
        console.error("broadcast notification settings failed", error);
      }
    },
    async setEnabled(value: boolean) {
      this.enabled = value;
      await this.persist();
    },
    async setSystem(value: boolean) {
      this.system = value;
      await this.persist();
    },
    async setEmail(value: boolean) {
      this.email = value;
      await this.persist();
    },
    async setAdvanceHours(value: number) {
      this.advanceHours = value;
      await this.persist();
    },
    async setRepeatMinutes(value: number) {
      this.repeatMinutes = value;
      await this.persist();
    },
  },
});
