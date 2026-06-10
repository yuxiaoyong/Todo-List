export type GanttViewMode = "Day" | "Week" | "Month";

export interface GanttChartInstance {
  $svg: SVGSVGElement;
  $container: HTMLElement;
  layers: { grid: SVGGElement; details: SVGGElement };
  gantt_start: Date;
  gantt_end: Date;
  tasks: unknown[];
  options: {
    step: number;
    column_width: number;
    header_height: number;
    bar_height: number;
    padding: number;
    view_mode: string;
  };
}

const TODAY_MARKER_CLASS = "gantt-today-marker";

function startOfDay(date: Date): Date {
  return new Date(date.getFullYear(), date.getMonth(), date.getDate());
}

function diffHours(from: Date, to: Date): number {
  return (to.getTime() - from.getTime()) / (1000 * 60 * 60);
}

export function todayColumnLeft(gantt: GanttChartInstance): number | null {
  const today = startOfDay(new Date());
  const rangeStart = startOfDay(gantt.gantt_start);
  const rangeEnd = startOfDay(gantt.gantt_end);

  if (today.getTime() < rangeStart.getTime() || today.getTime() > rangeEnd.getTime()) {
    return null;
  }

  const hours = diffHours(gantt.gantt_start, today);
  return (hours / gantt.options.step) * gantt.options.column_width;
}

function gridHeight(gantt: GanttChartInstance): number {
  const svgHeight = gantt.$svg.getAttribute("height");
  if (svgHeight) {
    return parseFloat(svgHeight);
  }

  return (
    gantt.options.header_height +
    gantt.options.padding +
    (gantt.options.bar_height + gantt.options.padding) * gantt.tasks.length
  );
}

function removeTodayMarkers(gantt: GanttChartInstance) {
  gantt.$svg.querySelectorAll(`.${TODAY_MARKER_CLASS}`).forEach((node) => node.remove());
}

export function applyTodayMarker(gantt: GanttChartInstance) {
  removeTodayMarkers(gantt);

  const left = todayColumnLeft(gantt);
  if (left == null) return;

  const height = gridHeight(gantt);
  const centerX = left + gantt.options.column_width / 2;
  const ns = "http://www.w3.org/2000/svg";

  const line = document.createElementNS(ns, "line");
  line.setAttribute("x1", String(centerX));
  line.setAttribute("x2", String(centerX));
  line.setAttribute("y1", "0");
  line.setAttribute("y2", String(height));
  line.setAttribute("class", `${TODAY_MARKER_CLASS} gantt-today-line`);

  gantt.layers.details.appendChild(line);
}

export function scrollToToday(gantt: GanttChartInstance, smooth = false) {
  const left = todayColumnLeft(gantt);
  if (left == null) return;

  const centerX = left + gantt.options.column_width / 2;
  const target = centerX - gantt.$container.clientWidth / 2;

  gantt.$container.scrollTo({
    left: Math.max(0, target),
    behavior: smooth ? "smooth" : "auto",
  });
}
