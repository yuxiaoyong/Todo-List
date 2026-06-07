import { defineStore } from "pinia";
import { todoApi } from "../api";
import type { TodoDetail, TodoListFilter, TodoSummary } from "../types";

export const useTodoStore = defineStore("todo", {
  state: () => ({
    todos: [] as TodoSummary[],
    allTodos: [] as TodoSummary[],
    selectedTodo: null as TodoDetail | null,
    loading: false,
    highlightId: null as number | null,
  }),
  actions: {
    async fetchList(filter: TodoListFilter) {
      this.loading = true;
      try {
        this.todos = await todoApi.list(filter);
      } finally {
        this.loading = false;
      }
    },
    async fetchAllTodos() {
      this.allTodos = await todoApi.list({});
    },
    async fetchDetail(id: number) {
      this.selectedTodo = await todoApi.get(id);
    },
    clearSelection() {
      this.selectedTodo = null;
    },
    setHighlight(id: number | null) {
      this.highlightId = id;
    },
    async quickCreate(input: {
      title: string;
      categoryId?: number;
      tagIds?: number[];
      priority?: string;
    }) {
      const detail = await todoApi.quickCreate(input);
      this.setHighlight(detail.id);
      return detail;
    },
  },
});
