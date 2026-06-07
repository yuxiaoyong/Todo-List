import { defineStore } from "pinia";
import { settingsApi } from "../api";
import { setI18nLocale, type AppLocale } from "../i18n";

const LOCALE_SETTING_KEY = "locale";

export const useLocaleStore = defineStore("locale", {
  state: () => ({
    locale: "zh-CN" as AppLocale,
    ready: false,
  }),
  actions: {
    async init() {
      try {
        const saved = await settingsApi.get(LOCALE_SETTING_KEY);
        if (saved === "zh-CN" || saved === "en") {
          this.locale = saved;
        }
      } catch (error) {
        console.error("load locale setting failed", error);
      }
      setI18nLocale(this.locale);
      this.ready = true;
    },
    async setLocale(locale: AppLocale) {
      this.locale = locale;
      setI18nLocale(locale);
      try {
        await settingsApi.set(LOCALE_SETTING_KEY, locale);
      } catch (error) {
        console.error("save locale setting failed", error);
      }
    },
  },
});
