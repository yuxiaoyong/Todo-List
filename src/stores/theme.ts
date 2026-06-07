import { emit, listen } from "@tauri-apps/api/event";
import { defineStore } from "pinia";
import { settingsApi } from "../api";

export type ThemeMode = "light" | "dark" | "system";

const THEME_SETTING_KEY = "theme";

let systemThemeQuery: MediaQueryList | null = null;
let themeSyncBound = false;

function isThemeMode(value: unknown): value is ThemeMode {
  return value === "light" || value === "dark" || value === "system";
}

async function bindThemeSyncListener(store: ReturnType<typeof useThemeStore>) {
  if (themeSyncBound) return;
  themeSyncBound = true;
  await listen<{ mode: ThemeMode }>("theme-changed", (event) => {
    const mode = event.payload?.mode;
    if (!isThemeMode(mode) || store.mode === mode) return;
    store.mode = mode;
    store.applyTheme();
  });
}

function bindSystemThemeListener(store: ReturnType<typeof useThemeStore>) {
  if (systemThemeQuery) return;
  systemThemeQuery = window.matchMedia("(prefers-color-scheme: dark)");
  systemThemeQuery.addEventListener("change", (event) => {
    store.systemDark = event.matches;
    if (store.mode === "system") {
      store.applyTheme();
    }
  });
}

export const useThemeStore = defineStore("theme", {
  state: () => ({
    mode: "system" as ThemeMode,
    systemDark: false,
    ready: false,
  }),
  getters: {
    resolvedMode(state): "light" | "dark" {
      if (state.mode === "system") {
        return state.systemDark ? "dark" : "light";
      }
      return state.mode;
    },
  },
  actions: {
    async init() {
      this.systemDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
      bindSystemThemeListener(this);

      try {
        const saved = await settingsApi.get(THEME_SETTING_KEY);
        if (saved === "light" || saved === "dark" || saved === "system") {
          this.mode = saved;
        }
      } catch (error) {
        console.error("load theme setting failed", error);
      }

      this.applyTheme();
      await bindThemeSyncListener(this);
      this.ready = true;
    },
    async setMode(mode: ThemeMode) {
      this.mode = mode;
      this.applyTheme();
      try {
        await settingsApi.set(THEME_SETTING_KEY, mode);
      } catch (error) {
        console.error("save theme setting failed", error);
      }
      try {
        await emit("theme-changed", { mode });
      } catch (error) {
        console.error("broadcast theme change failed", error);
      }
    },
    applyTheme() {
      const isDark = this.resolvedMode === "dark";
      document.documentElement.classList.toggle("dark", isDark);
      document.documentElement.dataset.theme = this.resolvedMode;
      document.documentElement.dataset.themeMode = this.mode;
    },
  },
});
