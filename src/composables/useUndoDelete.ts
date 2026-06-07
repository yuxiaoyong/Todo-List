import { useI18n } from "vue-i18n";
import { ElNotification, ElButton } from "element-plus";
import { h } from "vue";
import { todoApi } from "../api";
import type { TodoSummary } from "../types";

export function useUndoDelete(onRefresh: () => Promise<void>) {
  const { t } = useI18n();

  async function deleteWithUndo(todo: TodoSummary) {
    await todoApi.delete(todo.id);
    await onRefresh();

    let restored = false;
    const notification = ElNotification({
      title: t("taskDetail.deleted"),
      message: h("div", { style: "display:flex;align-items:center;gap:12px" }, [
        h("span", `「${todo.title}」`),
        h(
          ElButton,
          {
            size: "small",
            type: "primary",
            onClick: async () => {
              if (restored) return;
              restored = true;
              await todoApi.restore(todo.id);
              await onRefresh();
              notification.close();
            },
          },
          () => t("common.undo"),
        ),
      ]),
      duration: 5000,
      type: "info",
    });
  }

  return { deleteWithUndo };
}
