import StarterKit from "@tiptap/starter-kit";
import Link from "@tiptap/extension-link";
import Placeholder from "@tiptap/extension-placeholder";
import Underline from "@tiptap/extension-underline";
import Table from "@tiptap/extension-table";
import TableRow from "@tiptap/extension-table-row";
import TableCell from "@tiptap/extension-table-cell";
import TableHeader from "@tiptap/extension-table-header";
import TaskList from "@tiptap/extension-task-list";
import TaskItem from "@tiptap/extension-task-item";
import CodeBlockLowlight from "@tiptap/extension-code-block-lowlight";
import { common, createLowlight } from "lowlight";
import { ResizableImage } from "./resizableImage";

const lowlight = createLowlight(common);

export function createEditorExtensions(options: {
  placeholder: string;
  onImagePreview?: (src: string) => void;
}) {
  return [
    StarterKit.configure({
      codeBlock: false,
      blockquote: false,
      code: false,
    }),
    Underline,
    Link.configure({
      openOnClick: false,
      HTMLAttributes: {
        rel: "noopener noreferrer",
        target: "_blank",
      },
    }),
    Table.configure({
      resizable: true,
    }),
    TableRow,
    TableHeader,
    TableCell,
    TaskList,
    TaskItem.configure({
      nested: true,
    }),
    CodeBlockLowlight.configure({
      lowlight,
      defaultLanguage: null,
      languageClassPrefix: "language-",
    }),
    ResizableImage.configure({
      allowBase64: true,
      onPreview: options.onImagePreview,
    }),
    Placeholder.configure({
      placeholder: options.placeholder,
    }),
  ];
}
