import { defineStore } from "pinia";
import { kanbanColumnApi } from "../api";
import type { KanbanColumn } from "../types";

export const useKanbanColumnStore = defineStore("kanbanColumn", {
  state: () => ({
    columns: [] as KanbanColumn[],
    loading: false,
  }),
  actions: {
    async fetchAll() {
      this.loading = true;
      try {
        this.columns = await kanbanColumnApi.list();
      } finally {
        this.loading = false;
      }
    },
    async create(name: string, color?: string, subtitle?: string) {
      const column = await kanbanColumnApi.create(name, color, subtitle);
      await this.fetchAll();
      return column;
    },
    async update(id: number, name: string, color: string, subtitle?: string | null) {
      await kanbanColumnApi.update(id, name, color, subtitle);
      await this.fetchAll();
    },
    async remove(id: number) {
      await kanbanColumnApi.delete(id);
      await this.fetchAll();
    },
    async reorder(ids: number[]) {
      await kanbanColumnApi.reorder(ids);
      const byId = new Map(this.columns.map((column) => [column.id, column]));
      this.columns = ids
        .map((id, index) => {
          const column = byId.get(id);
          return column ? { ...column, sortOrder: index } : null;
        })
        .filter((column): column is KanbanColumn => column !== null);
    },
  },
});
