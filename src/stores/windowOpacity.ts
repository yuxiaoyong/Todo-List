import { emit, listen } from "@tauri-apps/api/event";
import { defineStore } from "pinia";
import { settingsApi, windowApi } from "../api";

export const WINDOW_OPACITY_SETTING_KEY = "windowOpacity";
export const MIN_WINDOW_OPACITY = 0.5;
export const MAX_WINDOW_OPACITY = 1;
export const DEFAULT_WINDOW_OPACITY = 1;

let opacitySyncBound = false;

function clampOpacity(value: number): number {
  return Math.min(MAX_WINDOW_OPACITY, Math.max(MIN_WINDOW_OPACITY, value));
}

function applyCssOpacity(opacity: number) {
  document.documentElement.style.setProperty("--window-opacity", String(opacity));
  if (opacity < MAX_WINDOW_OPACITY) {
    document.documentElement.dataset.windowOpacity = "on";
  } else {
    delete document.documentElement.dataset.windowOpacity;
  }
}

async function bindOpacitySyncListener(store: ReturnType<typeof useWindowOpacityStore>) {
  if (opacitySyncBound) return;
  opacitySyncBound = true;
  await listen<{ opacity: number }>("window-opacity-changed", (event) => {
    const opacity = event.payload?.opacity;
    if (typeof opacity !== "number" || store.opacity === opacity) return;
    store.opacity = opacity;
    applyCssOpacity(opacity);
  });
}

export const useWindowOpacityStore = defineStore("windowOpacity", {
  state: () => ({
    opacity: DEFAULT_WINDOW_OPACITY,
    ready: false,
  }),
  getters: {
    opacityPercent(state): number {
      return Math.round(state.opacity * 100);
    },
  },
  actions: {
    async init() {
      try {
        const saved = await settingsApi.get(WINDOW_OPACITY_SETTING_KEY);
        if (saved) {
          const value = Number.parseFloat(saved);
          if (Number.isFinite(value)) {
            this.opacity = clampOpacity(value);
          }
        }
      } catch (error) {
        console.error("load window opacity setting failed", error);
      }

      applyCssOpacity(this.opacity);
      try {
        await windowApi.setOpacity(this.opacity);
      } catch (error) {
        console.error("apply window opacity failed", error);
      }

      await bindOpacitySyncListener(this);
      this.ready = true;
    },
    applyCssOpacity() {
      applyCssOpacity(this.opacity);
    },
    async applyOpacity() {
      this.applyCssOpacity();
      await windowApi.setOpacity(this.opacity);
    },
    async setOpacityLive(opacity: number) {
      this.opacity = clampOpacity(opacity);
      await this.applyOpacity();
    },
    async setOpacity(opacity: number) {
      this.opacity = clampOpacity(opacity);
      await this.applyOpacity();
      try {
        await settingsApi.set(WINDOW_OPACITY_SETTING_KEY, String(this.opacity));
      } catch (error) {
        console.error("save window opacity setting failed", error);
      }
      try {
        await emit("window-opacity-changed", { opacity: this.opacity });
      } catch (error) {
        console.error("broadcast window opacity change failed", error);
      }
    },
    async setOpacityPercent(percent: number) {
      await this.setOpacity(percent / 100);
    },
  },
});
