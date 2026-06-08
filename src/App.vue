<script setup lang="ts">
import { computed } from "vue";
import { RouterView } from "vue-router";
import { ElConfigProvider } from "element-plus";
import zhCn from "element-plus/es/locale/lang/zh-cn";
import en from "element-plus/es/locale/lang/en";
import { useAppBootstrap } from "./composables/useAppBootstrap";
import { useDueNotifications } from "./composables/useDueNotifications";
import { useLocaleStore } from "./stores/locale";

const localeStore = useLocaleStore();
const elementLocale = computed(() => (localeStore.locale === "en" ? en : zhCn));

useAppBootstrap();
useDueNotifications();
</script>

<template>
  <ElConfigProvider :locale="elementLocale">
    <RouterView />
  </ElConfigProvider>
</template>
