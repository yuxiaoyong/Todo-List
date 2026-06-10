import { defineStore } from "pinia";

export type ViewMode = "all" | "category" | "tag" | "trash";
export type TimeFilter = "all" | "year" | "month" | "week" | "today";
export type CategoryFilter = "all" | number;
export type TaskViewMode = "list" | "kanban" | "gantt";

export const useUiStore = defineStore("ui", {
  state: () => ({
    viewMode: "all" as ViewMode,
    categoryFilter: "all" as CategoryFilter,
    selectedTagIds: [] as number[],
    selectedTodoId: null as number | null,
    searchQuery: "",
    completedFilter: null as boolean | null,
    priorityFilter: null as string | null,
    timeFilter: "all" as TimeFilter,
    todoSort: "default:desc",
    taskViewMode: "list" as TaskViewMode,
    detailDrawerOpen: false,
    newTaskDialogOpen: false,
  }),
  actions: {
    leaveTrashIfNeeded() {
      if (this.viewMode === "trash") {
        this.viewMode = "all";
      }
    },
    selectAll() {
      this.viewMode = "all";
      this.categoryFilter = "all";
      this.selectedTagIds = [];
    },
    selectAllCategories() {
      this.leaveTrashIfNeeded();
      this.categoryFilter = "all";
    },
    selectCategory(id: number) {
      this.leaveTrashIfNeeded();
      this.categoryFilter = id;
    },
    toggleTag(id: number) {
      this.leaveTrashIfNeeded();
      if (this.selectedTagIds.includes(id)) {
        this.selectedTagIds = this.selectedTagIds.filter((t) => t !== id);
      } else {
        this.selectedTagIds = [...this.selectedTagIds, id];
      }
    },
    selectTrash() {
      this.viewMode = "trash";
      this.categoryFilter = "all";
      this.selectedTagIds = [];
      this.taskViewMode = "list";
    },
    selectTodo(id: number | null) {
      this.selectedTodoId = id;
    },
    openDetail(id: number) {
      this.selectedTodoId = id;
      this.detailDrawerOpen = true;
    },
    closeDetail() {
      this.detailDrawerOpen = false;
    },
    openNewTaskDialog() {
      this.newTaskDialogOpen = true;
    },
    closeNewTaskDialog() {
      this.newTaskDialogOpen = false;
    },
    setSearchQuery(query: string) {
      this.searchQuery = query;
    },
    setCompletedFilter(value: boolean | null) {
      this.completedFilter = value;
    },
    setPriorityFilter(value: string | null) {
      this.priorityFilter = value;
    },
    setTimeFilter(value: TimeFilter) {
      this.timeFilter = value;
    },
    setTodoSort(value: string) {
      this.todoSort = value;
    },
    setTaskViewMode(value: TaskViewMode) {
      if (this.viewMode === "trash") return;
      this.taskViewMode = value;
    },
    buildFilter() {
      const inTrash = this.viewMode === "trash";
      return {
        categoryId:
          !inTrash && this.categoryFilter !== "all" ? this.categoryFilter : undefined,
        tagIds: !inTrash && this.selectedTagIds.length ? this.selectedTagIds : undefined,
        completed: this.completedFilter ?? undefined,
        priority: this.priorityFilter ?? undefined,
        includeDeleted: this.viewMode === "trash" ? true : false,
        searchQuery: this.searchQuery.trim() || undefined,
      };
    },
  },
});
