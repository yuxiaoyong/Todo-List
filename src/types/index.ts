export interface Category {
  id: number;
  name: string;
  color: string;
  icon?: string;
  sortOrder: number;
  createdAt: string;
  todoCount: number;
  incompleteCount: number;
}

export interface Tag {
  id: number;
  name: string;
  color: string;
  sortOrder: number;
  createdAt: string;
  todoCount: number;
}

export interface KanbanColumn {
  id: number;
  name: string;
  color: string;
  subtitle?: string | null;
  sortOrder: number;
  createdAt: string;
  todoCount: number;
}

import type { RecurrenceConfig } from "../utils/recurrence";

export interface TodoSummary {
  id: number;
  title: string;
  contentText: string;
  completed: boolean;
  priority: "low" | "medium" | "high";
  startDate?: string;
  dueDate?: string;
  categoryId?: number;
  categoryName?: string;
  categoryColor?: string;
  sortOrder: number;
  pinned: boolean;
  assignee: string;
  kanbanColumnId?: number | null;
  kanbanColumnName?: string | null;
  kanbanColumnColor?: string | null;
  createdAt: string;
  updatedAt: string;
  tagIds: number[];
  tagNames: string[];
  tagColors: string[];
  recurrenceJson?: RecurrenceConfig | null;
}

export interface Subtask {
  id: number;
  todoId: number;
  title: string;
  completed: boolean;
  sortOrder: number;
  createdAt: string;
  updatedAt: string;
}

export interface TodoDetail extends TodoSummary {
  contentHtml: string;
  subtasks?: Subtask[];
}

export interface TodoListFilter {
  categoryId?: number;
  tagIds?: number[];
  completed?: boolean;
  priority?: string;
  includeDeleted?: boolean;
  searchQuery?: string;
}

export interface AttachmentInfo {
  id: number;
  todoId: number;
  filename: string;
  originalName?: string;
  mimeType?: string;
  fileSize: number;
  kind: string;
  url: string;
  createdAt: string;
}

export interface QuickCreateInput {
  title: string;
  categoryId?: number;
  tagIds?: number[];
  priority?: string;
}

export interface UpdateTodoInput {
  id: number;
  title: string;
  contentHtml: string;
  completed: boolean;
  priority: string;
  startDate?: string | null;
  dueDate?: string | null;
  categoryId?: number | null;
  tagIds: number[];
  sortOrder?: number;
  pinned?: boolean;
  assignee?: string;
  kanbanColumnId?: number | null;
  quiet?: boolean;
  recurrenceJson?: RecurrenceConfig | null;
}
