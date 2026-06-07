import type { AttachmentInfo } from "../types";

export const ATTACHMENT_ACCEPT =
  ".jpg,.jpeg,.png,.gif,.webp,.bmp,.pdf,.doc,.docx,.xls,.xlsx,.txt";

const IMAGE_EXT = /\.(jpe?g|png|gif|webp|bmp)$/i;
const ALLOWED_EXT =
  /\.(jpe?g|png|gif|webp|bmp|pdf|docx?|xlsx?|txt)$/i;

const MIME_BY_EXT: Record<string, string> = {
  jpg: "image/jpeg",
  jpeg: "image/jpeg",
  png: "image/png",
  gif: "image/gif",
  webp: "image/webp",
  bmp: "image/bmp",
  pdf: "application/pdf",
  doc: "application/msword",
  docx: "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
  xls: "application/vnd.ms-excel",
  xlsx: "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
  txt: "text/plain",
};

export type FileTypeKind = "image" | "pdf" | "word" | "excel" | "text" | "file";

export function getAttachmentName(item: AttachmentInfo): string {
  return item.originalName || item.filename;
}

export function isImageAttachment(item: AttachmentInfo): boolean {
  const mime = item.mimeType?.toLowerCase() ?? "";
  if (mime.startsWith("image/")) return true;
  return IMAGE_EXT.test(getAttachmentName(item));
}

/** 附件面板展示的条目（排除已嵌入描述正文的 inline 图片） */
export function isPanelAttachment(item: AttachmentInfo, contentHtml = ""): boolean {
  if (item.kind === "attachment") return true;
  if (item.kind === "inline" && contentHtml.includes(item.url)) return false;
  return true;
}

export function getFileTypeKind(item: AttachmentInfo): FileTypeKind {
  const name = getAttachmentName(item).toLowerCase();
  const mime = item.mimeType?.toLowerCase() ?? "";
  if (isImageAttachment(item)) return "image";
  if (name.endsWith(".pdf") || mime === "application/pdf") return "pdf";
  if (name.endsWith(".doc") || name.endsWith(".docx") || mime.includes("word")) return "word";
  if (name.endsWith(".xls") || name.endsWith(".xlsx") || mime.includes("sheet") || mime.includes("excel")) {
    return "excel";
  }
  if (name.endsWith(".txt") || mime === "text/plain") return "text";
  return "file";
}

export function getFileTypeLabel(kind: FileTypeKind): string {
  switch (kind) {
    case "pdf":
      return "PDF";
    case "word":
      return "DOC";
    case "excel":
      return "XLS";
    case "text":
      return "TXT";
    default:
      return "FILE";
  }
}

export function guessMimeType(file: File): string {
  const ext = file.name.split(".").pop()?.toLowerCase() ?? "";
  if (ext && MIME_BY_EXT[ext]) return MIME_BY_EXT[ext];
  if (file.type && file.type !== "application/octet-stream") return file.type;
  return MIME_BY_EXT[ext] || file.type || "application/octet-stream";
}

export function validateAttachmentFile(file: File): string | null {
  if (!ALLOWED_EXT.test(file.name)) {
    return "仅支持图片、PDF、Word、Excel、TXT 文件";
  }
  const maxSize = 20 * 1024 * 1024;
  if (file.size > maxSize) {
    return "单个文件不能超过 20MB";
  }
  return null;
}

export async function fileToBase64(file: File): Promise<string> {
  const buffer = await file.arrayBuffer();
  const bytes = new Uint8Array(buffer);
  let binary = "";
  for (let i = 0; i < bytes.length; i += 1) {
    binary += String.fromCharCode(bytes[i]);
  }
  return btoa(binary);
}

export function canPreviewInApp(item: AttachmentInfo): boolean {
  const kind = getFileTypeKind(item);
  return kind === "image" || kind === "text";
}

export function shouldOpenWithLocalApp(item: AttachmentInfo): boolean {
  const kind = getFileTypeKind(item);
  return kind === "pdf" || kind === "word" || kind === "excel";
}

export function decodeBase64Utf8(base64: string): string {
  const binary = atob(base64);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i += 1) {
    bytes[i] = binary.charCodeAt(i);
  }
  return new TextDecoder("utf-8").decode(bytes);
}
