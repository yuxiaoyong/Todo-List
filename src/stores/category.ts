import { defineStore } from "pinia";
import { categoryApi } from "../api";
import type { Category } from "../types";

export const useCategoryStore = defineStore("category", {
  state: () => ({
    categories: [] as Category[],
    loading: false,
  }),
  actions: {
    async fetchAll() {
      this.loading = true;
      try {
        this.categories = await categoryApi.list();
      } finally {
        this.loading = false;
      }
    },
    async create(name: string, color?: string) {
      const category = await categoryApi.create(name, color);
      await this.fetchAll();
      return category;
    },
    async update(id: number, name: string, color: string) {
      await categoryApi.update(id, name, color);
      await this.fetchAll();
    },
    async remove(id: number) {
      await categoryApi.delete(id);
      await this.fetchAll();
    },
    async reorder(ids: number[]) {
      await categoryApi.reorder(ids);
      const byId = new Map(this.categories.map((category) => [category.id, category]));
      this.categories = ids
        .map((id, index) => {
          const category = byId.get(id);
          return category ? { ...category, sortOrder: index } : null;
        })
        .filter((category): category is Category => category !== null);
    },
  },
});
