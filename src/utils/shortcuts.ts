export interface ShortcutBinding {
  ctrl: boolean;
  shift: boolean;
  alt: boolean;
  meta: boolean;
  code: string;
}

export const DEFAULT_QUICK_CAPTURE: ShortcutBinding = {
  ctrl: true,
  shift: true,
  alt: false,
  meta: false,
  code: "KeyN",
};

export const DEFAULT_TOGGLE_MAIN: ShortcutBinding = {
  ctrl: true,
  shift: true,
  alt: false,
  meta: false,
  code: "KeyH",
};

export type CustomizableShortcutId = "quickCapture" | "toggleMain";

export interface CustomizableShortcutDef {
  id: CustomizableShortcutId;
  default: ShortcutBinding;
}

export const CUSTOMIZABLE_SHORTCUT_DEFS: CustomizableShortcutDef[] = [
  { id: "quickCapture", default: DEFAULT_QUICK_CAPTURE },
  { id: "toggleMain", default: DEFAULT_TOGGLE_MAIN },
];

/** @deprecated use CUSTOMIZABLE_SHORTCUT_DEFS */
export const CUSTOMIZABLE_SHORTCUTS = CUSTOMIZABLE_SHORTCUT_DEFS;

export interface StaticShortcutDef {
  id: string;
  keys: string;
}

export const STATIC_SHORTCUT_DEFS: StaticShortcutDef[] = [
  { id: "quick_create", keys: "Enter" },
  { id: "quick_create_open", keys: "Shift+Enter" },
  { id: "quick_capture_save", keys: "Enter" },
  { id: "quick_capture_cancel", keys: "Esc" },
];

/** @deprecated use STATIC_SHORTCUT_DEFS */
export const STATIC_SHORTCUTS = STATIC_SHORTCUT_DEFS;

const STATIC_SHORTCUT_I18N_KEYS: Record<string, string> = {
  quick_create: "quickCreate",
  quick_create_open: "quickCreateOpen",
  quick_capture_save: "quickCaptureSave",
  quick_capture_cancel: "quickCaptureCancel",
};

export function staticShortcutI18nKey(id: string): string {
  return STATIC_SHORTCUT_I18N_KEYS[id] ?? id;
}

const KEY_CODE_LABELS: Record<string, string> = {
  Space: "Space",
  Enter: "Enter",
  Escape: "Esc",
  Tab: "Tab",
  Backspace: "Backspace",
  Delete: "Delete",
  ArrowUp: "↑",
  ArrowDown: "↓",
  ArrowLeft: "←",
  ArrowRight: "→",
};

function formatKeyCode(code: string): string {
  if (KEY_CODE_LABELS[code]) return KEY_CODE_LABELS[code];
  if (code.startsWith("Key") && code.length === 4) return code.slice(3);
  if (code.startsWith("Digit")) return code.slice(5);
  if (code.startsWith("F") && /^F\d+$/.test(code)) return code;
  return code;
}

export function formatShortcut(binding: ShortcutBinding): string {
  const parts: string[] = [];
  if (binding.ctrl) parts.push("Ctrl");
  if (binding.shift) parts.push("Shift");
  if (binding.alt) parts.push("Alt");
  if (binding.meta) parts.push("Win");
  parts.push(formatKeyCode(binding.code));
  return parts.join("+");
}

export function isModifierOnlyKey(key: string): boolean {
  return key === "Control" || key === "Shift" || key === "Alt" || key === "Meta";
}

export function captureShortcutFromEvent(event: KeyboardEvent): ShortcutBinding | null {
  if (isModifierOnlyKey(event.key)) return null;
  if (!event.ctrlKey && !event.shiftKey && !event.altKey && !event.metaKey) return null;
  if (!event.code || event.code === "Unidentified") return null;
  return {
    ctrl: event.ctrlKey,
    shift: event.shiftKey,
    alt: event.altKey,
    meta: event.metaKey,
    code: event.code,
  };
}

export function normalizeShortcutBinding(binding: ShortcutBinding): ShortcutBinding {
  return {
    ctrl: Boolean(binding.ctrl),
    shift: Boolean(binding.shift),
    alt: Boolean(binding.alt),
    meta: Boolean(binding.meta),
    code: binding.code,
  };
}

export function isSameShortcut(a: ShortcutBinding, b: ShortcutBinding): boolean {
  const left = normalizeShortcutBinding(a);
  const right = normalizeShortcutBinding(b);
  return (
    left.ctrl === right.ctrl &&
    left.shift === right.shift &&
    left.alt === right.alt &&
    left.meta === right.meta &&
    left.code === right.code
  );
}
