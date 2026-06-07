import { createApp } from "vue";
import { createPinia } from "pinia";
import ElementPlus from "element-plus";
import hevueImgPreview from "hevue-img-preview/v3";
import "element-plus/dist/index.css";
import "element-plus/theme-chalk/dark/css-vars.css";
import App from "./App.vue";
import router from "./router";
import { i18n } from "./i18n";
import { useThemeStore } from "./stores/theme";
import { useWindowOpacityStore } from "./stores/windowOpacity";
import { useShortcutStore } from "./stores/shortcut";
import { useLocaleStore } from "./stores/locale";
import { useNotificationStore } from "./stores/notification";
import "./styles/main.css";

const app = createApp(App);
const pinia = createPinia();
app.use(pinia);
app.use(i18n);
app.use(router);
app.use(ElementPlus);
app.use(hevueImgPreview, { clickMaskCLose: true });

const themeStore = useThemeStore();
const windowOpacityStore = useWindowOpacityStore();
const shortcutStore = useShortcutStore();
const localeStore = useLocaleStore();
const notificationStore = useNotificationStore();
void Promise.all([
  themeStore.init(),
  windowOpacityStore.init(),
  shortcutStore.init(),
  localeStore.init(),
  notificationStore.init(),
]).finally(() => {
  app.mount("#app");
});
