import { marked } from "marked";

marked.setOptions({
  gfm: true,
  breaks: true,
});

export function isHtmlContent(content: string): boolean {
  const trimmed = content.trim();
  if (!trimmed) return false;
  return /^<(p|div|h[1-6]|ul|ol|li|blockquote|pre|table|img|br|span|strong|em)\b/i.test(trimmed);
}

/** 将数据库内容转为 TipTap 可编辑的 HTML（兼容旧 Markdown） */
export function contentToEditorHtml(content: string): string {
  const trimmed = content.trim();
  if (!trimmed) return "<p></p>";
  if (isHtmlContent(trimmed)) return trimmed;
  try {
    return marked.parse(trimmed, { async: false }) as string;
  } catch {
    return `<p>${trimmed.replace(/</g, "&lt;").replace(/>/g, "&gt;")}</p>`;
  }
}
