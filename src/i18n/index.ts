import { createI18n } from "vue-i18n";
import zhCN from "./locales/zh-CN";
import en from "./locales/en";

export type AppLocale = "zh-CN" | "en";

export const i18n = createI18n({
  legacy: false,
  locale: "zh-CN",
  fallbackLocale: "zh-CN",
  messages: {
    "zh-CN": zhCN,
    en,
  },
});

export function setI18nLocale(locale: AppLocale) {
  i18n.global.locale.value = locale;
  document.documentElement.lang = locale === "en" ? "en" : "zh-CN";
}
