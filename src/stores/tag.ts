import { defineStore } from "pinia";
import { tagApi } from "../api";
import type { Tag } from "../types";

export const useTagStore = defineStore("tag", {
  state: () => ({
    tags: [] as Tag[],
    loading: false,
  }),
  actions: {
    async fetchAll() {
      this.loading = true;
      try {
        this.tags = await tagApi.list();
      } finally {
        this.loading = false;
      }
    },
    async create(name: string, color?: string) {
      const tag = await tagApi.create(name, color);
      await this.fetchAll();
      return tag;
    },
    async update(id: number, name: string, color: string) {
      await tagApi.update(id, name, color);
      await this.fetchAll();
    },
    async remove(id: number) {
      await tagApi.delete(id);
      await this.fetchAll();
    },
    async reorder(ids: number[]) {
      await tagApi.reorder(ids);
      const byId = new Map(this.tags.map((tag) => [tag.id, tag]));
      this.tags = ids
        .map((id, index) => {
          const tag = byId.get(id);
          return tag ? { ...tag, sortOrder: index } : null;
        })
        .filter((tag): tag is Tag => tag !== null);
    },
  },
});
