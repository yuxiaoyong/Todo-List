import { defineStore } from "pinia";
import { shortcutApi } from "../api";
import {
  CUSTOMIZABLE_SHORTCUT_DEFS,
  DEFAULT_QUICK_CAPTURE,
  DEFAULT_TOGGLE_MAIN,
  formatShortcut,
  type CustomizableShortcutId,
  type ShortcutBinding,
} from "../utils/shortcuts";

export const useShortcutStore = defineStore("shortcut", {
  state: () => ({
    quickCapture: { ...DEFAULT_QUICK_CAPTURE } as ShortcutBinding,
    toggleMain: { ...DEFAULT_TOGGLE_MAIN } as ShortcutBinding,
    ready: false,
  }),
  getters: {
    quickCaptureLabel(state): string {
      return formatShortcut(state.quickCapture);
    },
    toggleMainLabel(state): string {
      return formatShortcut(state.toggleMain);
    },
  },
  actions: {
    async init() {
      try {
        const [quickCapture, toggleMain] = await Promise.all([
          shortcutApi.getQuickCapture(),
          shortcutApi.getToggleMain(),
        ]);
        this.quickCapture = quickCapture;
        this.toggleMain = toggleMain;
      } catch (error) {
        console.error("load shortcut failed", error);
        this.quickCapture = { ...DEFAULT_QUICK_CAPTURE };
        this.toggleMain = { ...DEFAULT_TOGGLE_MAIN };
      }
      this.ready = true;
    },
    async setQuickCapture(binding: ShortcutBinding) {
      this.quickCapture = await shortcutApi.setQuickCapture(binding);
    },
    async setToggleMain(binding: ShortcutBinding) {
      this.toggleMain = await shortcutApi.setToggleMain(binding);
    },
    async resetQuickCapture() {
      await this.setQuickCapture({ ...DEFAULT_QUICK_CAPTURE });
    },
    async resetToggleMain() {
      await this.setToggleMain({ ...DEFAULT_TOGGLE_MAIN });
    },
    async setShortcut(id: CustomizableShortcutId, binding: ShortcutBinding) {
      if (id === "quickCapture") {
        await this.setQuickCapture(binding);
        return;
      }
      await this.setToggleMain(binding);
    },
    async resetShortcut(id: CustomizableShortcutId) {
      const item = CUSTOMIZABLE_SHORTCUT_DEFS.find((entry) => entry.id === id);
      if (!item) return;
      await this.setShortcut(id, { ...item.default });
    },
    getShortcut(id: CustomizableShortcutId): ShortcutBinding {
      return { ...this[id] };
    },
  },
});
