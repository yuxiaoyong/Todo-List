import { onMounted, onUnmounted } from "vue";
import { useI18n } from "vue-i18n";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { ElMessage, ElNotification } from "element-plus";
import { todoApi, windowApi } from "../api";
import { useNotificationStore } from "../stores/notification";

interface DueReminderPayload {
  todoId: number;
  title: string;
}

export function useDueNotifications() {
  const { t } = useI18n();
  const notificationStore = useNotificationStore();
  let unlistenReminder: (() => void) | undefined;

  async function openTaskFromNotification(todoId: number) {
    try {
      await todoApi.get(todoId);
    } catch (error) {
      console.error("open task from notification failed", error);
      ElMessage.warning(t("notifications.taskNotFound"));
      return;
    }

    try {
      await windowApi.openTaskDetail(todoId);
    } catch (error) {
      console.error("open task detail window failed", error);
      ElMessage.warning(t("taskDetail.loadFailed"));
    }
  }

  function showInAppReminder(payload: DueReminderPayload) {
    ElNotification({
      title: t("notifications.dueTitle"),
      message: payload.title,
      type: "warning",
      duration: 0,
      onClick: () => {
        void openTaskFromNotification(payload.todoId);
      },
    });
  }

  onMounted(async () => {
    unlistenReminder = await listen<DueReminderPayload>("todo-due-reminder", async (event) => {
      if (!notificationStore.enabled) return;

      const visible = await getCurrentWindow().isVisible();
      if (!visible) return;

      showInAppReminder(event.payload);
    });
  });

  onUnmounted(() => {
    unlistenReminder?.();
  });
}
