declare module "frappe-gantt" {
  export interface GanttTask {
    id: string;
    name: string;
    start: string;
    end: string;
    progress?: number;
    dependencies?: string;
    custom_class?: string;
  }

  export interface GanttOptions {
    view_mode?: string;
    bar_height?: number;
    padding?: number;
    column_width?: number;
    header_height?: number;
    date_format?: string;
    language?: string;
    on_click?: (task: GanttTask) => void;
    on_date_change?: (task: GanttTask, start: Date, end: Date) => void;
    on_progress_change?: (task: GanttTask, progress: number) => void;
  }

  export default class Gantt {
    constructor(element: HTMLElement | string, tasks: GanttTask[], options?: GanttOptions);
    refresh(tasks: GanttTask[]): void;
    change_view_mode(mode?: string): void;
    clear(): void;
  }
}
