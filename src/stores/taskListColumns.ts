import { defineStore } from "pinia";
import { settingsApi } from "../api";
import {
  DEFAULT_TASK_LIST_COLUMNS,
  getVisibleColumns,
  parseTaskListColumns,
  serializeTaskListColumns,
  type TaskListColumnId,
  type TaskListColumnItem,
} from "../utils/taskListColumns";

const SETTING_KEY = "task_list_columns";

export const useTaskListColumnStore = defineStore("taskListColumns", {
  state: () => ({
    columns: DEFAULT_TASK_LIST_COLUMNS.map((column) => ({ ...column })),
    ready: false,
  }),
  actions: {
    async init() {
      if (this.ready) return;
      try {
        const saved = await settingsApi.get(SETTING_KEY);
        this.columns = parseTaskListColumns(saved);
      } catch (error) {
        console.error("load task list columns failed", error);
      } finally {
        this.ready = true;
      }
    },
    getDisplayColumns(options: { minimal?: boolean; isTrashMode?: boolean }) {
      return getVisibleColumns(this.columns, options);
    },
    async setVisible(id: TaskListColumnId, visible: boolean) {
      const column = this.columns.find((item) => item.id === id);
      if (!column || column.locked) return;
      column.visible = visible;
      await this.save();
    },
    async setColumns(columns: TaskListColumnItem[]) {
      this.columns = columns.map((column) => ({ ...column }));
      await this.save();
    },
    async reset() {
      this.columns = DEFAULT_TASK_LIST_COLUMNS.map((column) => ({ ...column }));
      await this.save();
    },
    async save() {
      try {
        await settingsApi.set(SETTING_KEY, serializeTaskListColumns(this.columns));
      } catch (error) {
        console.error("save task list columns failed", error);
      }
    },
  },
});
