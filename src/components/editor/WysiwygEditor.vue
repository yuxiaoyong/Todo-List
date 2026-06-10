<script setup lang="ts">
import { onBeforeUnmount, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { useEditor, EditorContent } from "@tiptap/vue-3";
import { ElMessageBox } from "element-plus";
import { attachmentApi, resolveAttachmentUrl } from "../../api";
import { fileToBase64 } from "../../utils/attachmentTypes";
import { contentToEditorHtml } from "../../utils/contentFormat";
import { showImagePreview } from "../../utils/imagePreview";
import { createEditorExtensions } from "./editorExtensions";
import { sanitizeEditorHtml } from "./sanitizeHtml";

const CODE_LANGUAGES = [
  { id: "auto", labelKey: "editor.codeLangAuto" },
  { id: "javascript", labelKey: "editor.codeLangJavaScript" },
  { id: "typescript", labelKey: "editor.codeLangTypeScript" },
  { id: "json", labelKey: "editor.codeLangJson" },
  { id: "python", labelKey: "editor.codeLangPython" },
  { id: "rust", labelKey: "editor.codeLangRust" },
  { id: "bash", labelKey: "editor.codeLangBash" },
  { id: "css", labelKey: "editor.codeLangCss" },
  { id: "html", labelKey: "editor.codeLangHtml" },
  { id: "markdown", labelKey: "editor.codeLangMarkdown" },
] as const;

const { t } = useI18n();

const props = defineProps<{
  modelValue: string;
  todoId: number;
  editable?: boolean;
  variant?: "default" | "detail";
}>();

const emit = defineEmits<{
  "update:modelValue": [value: string];
  blur: [];
}>();

const dataUrlToLocal = ref(new Map<string, string>());
const loading = ref(false);
const applyingExternal = ref(false);
const lastEmitted = ref("");

function toStorageHtml(html: string) {
  let result = sanitizeEditorHtml(html);
  dataUrlToLocal.value.forEach((localUrl, dataUrl) => {
    result = result.split(dataUrl).join(localUrl);
  });
  return result;
}

function normalizeStorage(html: string) {
  return sanitizeEditorHtml(toStorageHtml(html || "<p></p>"));
}

function registerDisplayUrl(dataUrl: string, localUrl: string) {
  const map = new Map(dataUrlToLocal.value);
  map.set(dataUrl, localUrl);
  dataUrlToLocal.value = map;
}

async function resolveForEditor(html: string) {
  const map = new Map<string, string>();
  const doc = new DOMParser().parseFromString(html || "<p></p>", "text/html");
  for (const img of doc.querySelectorAll("img")) {
    const src = img.getAttribute("src");
    if (src?.startsWith("local://attachment/")) {
      try {
        const dataUrl = await resolveAttachmentUrl(src);
        map.set(dataUrl, src);
        img.setAttribute("src", dataUrl);
      } catch (err) {
        console.error("Failed to load attachment image:", src, err);
      }
    }
  }
  dataUrlToLocal.value = map;
  return doc.body.innerHTML;
}

async function uploadImage(file: File) {
  const info = await attachmentApi.save(
    props.todoId,
    await fileToBase64(file),
    file.name,
    file.type || "image/png",
    "inline",
  );
  const localUrl = info.url;
  const dataUrl = await resolveAttachmentUrl(localUrl);
  registerDisplayUrl(dataUrl, localUrl);
  return dataUrl;
}

const editor = useEditor({
  extensions: createEditorExtensions({
    placeholder: t("editor.placeholder"),
    onImagePreview: (src) => void showImagePreview(src),
  }),
  editable: props.editable !== false,
  onUpdate: ({ editor: ed }) => {
    if (applyingExternal.value) return;
    const html = toStorageHtml(ed.getHTML());
    lastEmitted.value = html;
    emit("update:modelValue", html);
  },
  editorProps: {
    handleDOMEvents: {
      blur: () => {
        emit("blur");
        return false;
      },
      dblclick: (_view, event) => {
        const target = event.target as HTMLElement | null;
        if (target?.closest(".resizable-image")) {
          return true;
        }
        return false;
      },
    },
    handlePaste: (_view, event) => {
      const items = event.clipboardData?.items;
      if (!items) return false;
      for (const item of items) {
        if (item.type.startsWith("image/")) {
          event.preventDefault();
          const file = item.getAsFile();
          if (!file) return true;
          uploadImage(file).then((url) => {
            editor.value?.chain().focus().setImage({ src: url }).run();
          });
          return true;
        }
      }
      return false;
    },
    handleDrop: (_view, event) => {
      const files = event.dataTransfer?.files;
      if (!files?.length) return false;
      const file = files[0];
      if (!file.type.startsWith("image/")) return false;
      event.preventDefault();
      uploadImage(file).then((url) => {
        editor.value?.chain().focus().setImage({ src: url }).run();
      });
      return true;
    },
  },
});

watch(
  () => props.editable,
  (value) => {
    editor.value?.setEditable(value !== false);
  },
);

watch(
  () => editor.value,
  (ed) => {
    if (ed) void applyContent(props.modelValue || "");
  },
);

async function applyContent(raw: string) {
  if (!editor.value) return;
  const storage = normalizeStorage(raw || "");
  const current = normalizeStorage(editor.value.getHTML());
  if (storage === current) return;

  loading.value = true;
  applyingExternal.value = true;
  try {
    const html = contentToEditorHtml(raw || "");
    const resolved = await resolveForEditor(html);
    editor.value.commands.setContent(resolved, false);
    lastEmitted.value = normalizeStorage(raw || "");
  } catch (err) {
    console.error("Failed to apply editor content:", err);
    editor.value.commands.setContent("<p></p>", false);
  } finally {
    applyingExternal.value = false;
    loading.value = false;
  }
}

watch(
  () => props.modelValue,
  (value) => {
    const next = value || "";
    if (next === lastEmitted.value) return;
    void applyContent(next);
  },
  { immediate: true },
);

watch(
  () => props.todoId,
  () => {
    void applyContent(props.modelValue || "");
  },
);

onBeforeUnmount(() => {
  editor.value?.destroy();
});

async function pickImage() {
  const input = document.createElement("input");
  input.type = "file";
  input.accept = "image/*";
  input.onchange = async () => {
    const file = input.files?.[0];
    if (!file || !editor.value) return;
    const url = await uploadImage(file);
    editor.value.chain().focus().setImage({ src: url }).run();
  };
  input.click();
}

function runCommand(action: (ed: NonNullable<typeof editor.value>) => void) {
  if (!editor.value) return;
  action(editor.value);
}

async function editLink() {
  const ed = editor.value;
  if (!ed) return;
  const previous = ed.getAttributes("link").href as string | undefined;
  try {
    const { value } = await ElMessageBox.prompt(t("editor.linkUrl"), t("editor.link"), {
      confirmButtonText: t("common.confirm"),
      cancelButtonText: t("common.cancel"),
      inputValue: previous || "https://",
    });
    if (value === "") {
      ed.chain().focus().extendMarkRange("link").unsetLink().run();
      return;
    }
    ed.chain().focus().extendMarkRange("link").setLink({ href: value }).run();
  } catch {
    // cancelled
  }
}

type TableCommand =
  | "insert"
  | "addRowAfter"
  | "addColumnAfter"
  | "deleteRow"
  | "deleteColumn"
  | "deleteTable";

function runTableCommand(command: TableCommand) {
  runCommand((ed) => {
    const chain = ed.chain().focus();
    switch (command) {
      case "insert":
        chain.insertTable({ rows: 3, cols: 3, withHeaderRow: true }).run();
        break;
      case "addRowAfter":
        chain.addRowAfter().run();
        break;
      case "addColumnAfter":
        chain.addColumnAfter().run();
        break;
      case "deleteRow":
        chain.deleteRow().run();
        break;
      case "deleteColumn":
        chain.deleteColumn().run();
        break;
      case "deleteTable":
        chain.deleteTable().run();
        break;
    }
  });
}

function toggleCodeBlock() {
  runCommand((ed) => {
    ed.chain().focus().toggleCodeBlock().run();
  });
}

function setCodeLanguage(lang: string) {
  runCommand((ed) => {
    if (ed.isActive("codeBlock")) {
      if (lang === "auto") {
        ed.chain().focus().updateAttributes("codeBlock", { language: null }).run();
      } else {
        ed.chain().focus().updateAttributes("codeBlock", { language: lang }).run();
      }
      return;
    }
    if (lang === "auto") {
      ed.chain().focus().toggleCodeBlock().run();
    } else {
      ed.chain().focus().toggleCodeBlock({ language: lang }).run();
    }
  });
}
</script>

<template>
  <div
    v-loading="loading"
    class="wysiwyg-editor"
    :class="{ 'wysiwyg-editor--detail': variant === 'detail' }"
  >
    <div v-if="editable !== false" class="editor-toolbar">
      <el-button-group>
        <el-button size="small" :title="t('editor.bold')" @click="runCommand((ed) => ed.chain().focus().toggleBold().run())">
          <strong>B</strong>
        </el-button>
        <el-button size="small" :title="t('editor.italic')" @click="runCommand((ed) => ed.chain().focus().toggleItalic().run())">
          <em>I</em>
        </el-button>
        <el-button size="small" :title="t('editor.underline')" @click="runCommand((ed) => ed.chain().focus().toggleUnderline().run())">
          U
        </el-button>
        <el-button size="small" :title="t('editor.strike')" @click="runCommand((ed) => ed.chain().focus().toggleStrike().run())">
          S
        </el-button>
      </el-button-group>
      <el-button size="small" @click="runCommand((ed) => ed.chain().focus().toggleHeading({ level: 2 }).run())">
        {{ t("editor.heading") }}
      </el-button>
      <el-button size="small" @click="runCommand((ed) => ed.chain().focus().toggleBulletList().run())">
        {{ t("editor.bulletList") }}
      </el-button>
      <el-button size="small" @click="runCommand((ed) => ed.chain().focus().toggleOrderedList().run())">
        {{ t("editor.orderedList") }}
      </el-button>
      <el-button size="small" @click="runCommand((ed) => ed.chain().focus().toggleTaskList().run())">
        {{ t("editor.taskList") }}
      </el-button>
      <el-dropdown split-button size="small" @click="toggleCodeBlock" @command="setCodeLanguage">
        {{ t("editor.code") }}
        <template #dropdown>
          <el-dropdown-menu>
            <el-dropdown-item
              v-for="lang in CODE_LANGUAGES"
              :key="lang.id"
              :command="lang.id"
            >
              {{ t(lang.labelKey) }}
            </el-dropdown-item>
          </el-dropdown-menu>
        </template>
      </el-dropdown>
      <el-button size="small" @click="editLink">{{ t("editor.link") }}</el-button>
      <el-dropdown trigger="click" @command="runTableCommand">
        <el-button size="small">{{ t("editor.table") }}</el-button>
        <template #dropdown>
          <el-dropdown-menu>
            <el-dropdown-item command="insert">{{ t("editor.tableInsert") }}</el-dropdown-item>
            <el-dropdown-item command="addRowAfter">{{ t("editor.tableAddRow") }}</el-dropdown-item>
            <el-dropdown-item command="addColumnAfter">{{ t("editor.tableAddColumn") }}</el-dropdown-item>
            <el-dropdown-item divided command="deleteRow">{{ t("editor.tableDeleteRow") }}</el-dropdown-item>
            <el-dropdown-item command="deleteColumn">{{ t("editor.tableDeleteColumn") }}</el-dropdown-item>
            <el-dropdown-item command="deleteTable">{{ t("editor.tableDelete") }}</el-dropdown-item>
          </el-dropdown-menu>
        </template>
      </el-dropdown>
      <el-button size="small" @click="pickImage">{{ t("editor.image") }}</el-button>
    </div>
    <EditorContent :editor="editor" class="tiptap-editor" />
  </div>
</template>

<style scoped>
.wysiwyg-editor {
  width: 100%;
}

.wysiwyg-editor--detail .editor-toolbar {
  border: 1px solid var(--border-color);
  border-bottom: none;
  border-radius: var(--radius) var(--radius) 0 0;
  background: var(--surface-muted);
  padding: 8px 10px;
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  align-items: center;
}

.wysiwyg-editor--detail :deep(.tiptap-editor) {
  border-radius: 0 0 var(--radius) var(--radius);
  min-height: 360px;
}

.wysiwyg-editor--detail :deep(.tiptap-editor .ProseMirror) {
  min-height: 320px;
}

.wysiwyg-editor:not(.wysiwyg-editor--detail) .editor-toolbar {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
  margin-bottom: 8px;
}

.wysiwyg-editor :deep(.ProseMirror p.is-editor-empty:first-child::before) {
  color: var(--text-tertiary);
  content: attr(data-placeholder);
  float: left;
  height: 0;
  pointer-events: none;
}

.wysiwyg-editor :deep(.ProseMirror .resizable-image) {
  max-width: 100%;
}
</style>
