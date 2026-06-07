<script setup lang="ts">
import { onBeforeUnmount, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { Delete, FolderOpened, Plus, View } from "@element-plus/icons-vue";
import { ElMessage, ElMessageBox } from "element-plus";
import AttachmentPreviewDialog from "./AttachmentPreviewDialog.vue";
import { attachmentApi, resolveAttachmentUrl } from "../../api";
import { formatDateTimeCn, formatFileSize } from "../../utils/formatDate";
import {
  ATTACHMENT_ACCEPT,
  fileToBase64,
  getAttachmentName,
  getFileTypeKind,
  getFileTypeLabel,
  guessMimeType,
  isImageAttachment,
  shouldOpenWithLocalApp,
  validateAttachmentFile,
} from "../../utils/attachmentTypes";
import { showImagePreview, showImagePreviewList } from "../../utils/imagePreview";
import type { AttachmentInfo } from "../../types";

const { t } = useI18n();

const props = defineProps<{
  todoId: number;
  attachments: AttachmentInfo[];
  editable?: boolean;
  uploaderName?: string;
}>();

const emit = defineEmits<{
  refresh: [];
  uploaded: [items: AttachmentInfo[]];
}>();

const uploading = ref(false);
const previewUrls = ref(new Map<number, string>());
const fileInputRef = ref<HTMLInputElement | null>(null);
const previewVisible = ref(false);
const previewItem = ref<AttachmentInfo | null>(null);

async function loadPreviews(items: AttachmentInfo[]) {
  const next = new Map<number, string>();
  await Promise.all(
    items.map(async (item) => {
      if (!isImageAttachment(item)) return;
      try {
        next.set(item.id, await resolveAttachmentUrl(item.url));
      } catch (err) {
        console.error("Failed to load attachment preview:", item.url, err);
      }
    }),
  );
  previewUrls.value = next;
}

watch(
  () => props.attachments,
  (items) => {
    void loadPreviews(items);
  },
  { immediate: true, deep: true },
);

onBeforeUnmount(() => {
  previewUrls.value.clear();
});

function pickFiles() {
  fileInputRef.value?.click();
}

async function onFilesSelected(event: Event) {
  const input = event.target as HTMLInputElement;
  const files = input.files ? Array.from(input.files) : [];
  input.value = "";
  if (!files.length) return;

  uploading.value = true;
  try {
    const saved: AttachmentInfo[] = [];
    for (const file of files) {
      const error = validateAttachmentFile(file);
      if (error) {
        ElMessage.warning(`${file.name}：${error}`);
        continue;
      }
      saved.push(
        await attachmentApi.save(
          props.todoId,
          await fileToBase64(file),
          file.name,
          guessMimeType(file),
          "attachment",
        ),
      );
    }
    if (!saved.length) return;
    emit("uploaded", saved);
    emit("refresh");
    ElMessage.success(
      saved.length > 1
        ? t("attachment.addedMany", { count: saved.length })
        : t("attachment.added"),
    );
  } catch (err) {
    console.error("Failed to upload attachment:", err);
    ElMessage.error(t("attachment.uploadFailed"));
  } finally {
    uploading.value = false;
  }
}

async function openWithLocalApp(item: AttachmentInfo) {
  try {
    await attachmentApi.open(item.todoId, item.filename);
    ElMessage.success(t("attachment.openedLocal"));
  } catch (err) {
    console.error("Failed to open attachment:", err);
    ElMessage.error(t("attachment.openFailed"));
  }
}

async function previewAttachment(item: AttachmentInfo) {
  if (isImageAttachment(item)) {
    const imageItems = props.attachments.filter((entry) => isImageAttachment(entry));
    const index = imageItems.findIndex((entry) => entry.id === item.id);
    const urls = imageItems.map((entry) => previewUrls.value.get(entry.id) || entry.url);
    if (imageItems.length > 1) {
      await showImagePreviewList(urls, index);
    } else {
      await showImagePreview(urls[0] || item.url);
    }
    return;
  }

  if (getFileTypeKind(item) === "text") {
    previewItem.value = item;
    previewVisible.value = true;
    return;
  }

  if (shouldOpenWithLocalApp(item)) {
    await openWithLocalApp(item);
    return;
  }

  await openWithLocalApp(item);
}

function onPreviewOpenLocal(item: AttachmentInfo) {
  previewVisible.value = false;
  void openWithLocalApp(item);
}

async function removeAttachment(item: AttachmentInfo) {
  await ElMessageBox.confirm(
    t("attachment.deleteConfirm", { name: getAttachmentName(item) }),
    t("common.hint"),
    { type: "warning" },
  );
  await attachmentApi.delete(item.id);
  emit("refresh");
}
</script>

<template>
  <div class="attachment-panel">
    <div class="attachment-panel__head">
      <span class="attachment-panel__count">{{
        t("attachment.count", { count: attachments.length })
      }}</span>
      <button
        v-if="editable !== false"
        type="button"
        class="attachment-panel__add"
        :disabled="uploading"
        @click="pickFiles"
      >
        <el-icon><Plus /></el-icon>
        {{ t("attachment.add") }}
      </button>
    </div>

    <input
      ref="fileInputRef"
      type="file"
      class="attachment-panel__input"
      multiple
      :accept="ATTACHMENT_ACCEPT"
      @change="onFilesSelected"
    />

    <div v-if="!attachments.length" class="attachment-panel__empty">
      {{ t("attachment.empty") }}
    </div>

    <ul v-else class="attachment-list">
      <li
        v-for="item in attachments"
        :key="item.id"
        class="attachment-card"
        @click="previewAttachment(item)"
      >
        <div class="attachment-card__thumb">
          <img
            v-if="isImageAttachment(item) && previewUrls.get(item.id)"
            :src="previewUrls.get(item.id)"
            :alt="getAttachmentName(item)"
          />
          <div
            v-else
            class="attachment-card__badge"
            :class="`attachment-card__badge--${getFileTypeKind(item)}`"
          >
            {{ getFileTypeLabel(getFileTypeKind(item)) }}
          </div>
        </div>

        <div class="attachment-card__body">
          <div class="attachment-card__name" :title="getAttachmentName(item)">
            {{ getAttachmentName(item) }}
          </div>
          <div class="attachment-card__meta">
            <span>{{ formatFileSize(item.fileSize) }}</span>
            <span class="attachment-card__sep">{{
              t("attachment.from", { name: uploaderName || t("attachment.localMachine") })
            }}</span>
            <span class="attachment-card__sep">|</span>
            <span>{{ formatDateTimeCn(item.createdAt) }}</span>
          </div>
        </div>

        <div class="attachment-card__actions">
          <button
            type="button"
            class="attachment-card__action"
            :title="t('attachment.preview')"
            @click.stop="previewAttachment(item)"
          >
            <el-icon><View /></el-icon>
          </button>
          <button
            type="button"
            class="attachment-card__action"
            :title="t('attachment.openLocal')"
            @click.stop="openWithLocalApp(item)"
          >
            <el-icon><FolderOpened /></el-icon>
          </button>
          <button
            v-if="editable !== false"
            type="button"
            class="attachment-card__action attachment-card__action--danger"
            :title="t('attachment.delete')"
            @click.stop="removeAttachment(item)"
          >
            <el-icon><Delete /></el-icon>
          </button>
        </div>
      </li>
    </ul>

    <AttachmentPreviewDialog
      v-model:visible="previewVisible"
      :item="previewItem"
      @open-local="onPreviewOpenLocal"
    />
  </div>
</template>

<style scoped>
.attachment-panel {
  height: 100%;
  overflow: auto;
}

.attachment-panel__head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 16px;
}

.attachment-panel__count {
  font-size: 13px;
  color: var(--text-secondary);
}

.attachment-panel__add {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  border: none;
  background: transparent;
  color: var(--primary);
  font-size: 13px;
  cursor: pointer;
  padding: 0;
}

.attachment-panel__add:hover:not(:disabled) {
  opacity: 0.8;
}

.attachment-panel__add:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.attachment-panel__input {
  display: none;
}

.attachment-panel__empty {
  color: var(--text-secondary);
  text-align: center;
  padding: 48px 0;
  font-size: 13px;
  line-height: 1.6;
}

.attachment-list {
  list-style: none;
  margin: 0;
  padding: 0;
}

.attachment-card {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 14px 0;
  border-bottom: 1px solid var(--border-light);
  cursor: pointer;
  transition: background 0.15s ease;
}

.attachment-card:first-child {
  border-top: 1px solid var(--border-light);
}

.attachment-card:hover {
  background: var(--surface-muted);
}

.attachment-card__thumb {
  width: 88px;
  height: 62px;
  flex-shrink: 0;
  border: 1px solid var(--border-color);
  border-radius: 4px;
  overflow: hidden;
  background: var(--surface-subtle);
  display: flex;
  align-items: center;
  justify-content: center;
}

.attachment-card__thumb img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  display: block;
}

.attachment-card__badge {
  font-size: 12px;
  font-weight: 600;
  letter-spacing: 0.02em;
}

.attachment-card__badge--pdf {
  color: #cf1322;
}

.attachment-card__badge--word {
  color: #1677ff;
}

.attachment-card__badge--excel {
  color: #389e0d;
}

.attachment-card__badge--text {
  color: #595959;
}

.attachment-card__badge--file {
  color: #8c8c8c;
}

.attachment-card__body {
  flex: 1;
  min-width: 0;
}

.attachment-card__name {
  font-size: 14px;
  color: var(--text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.attachment-card__meta {
  margin-top: 6px;
  font-size: 12px;
  color: var(--text-secondary);
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 6px;
}

.attachment-card__sep {
  color: var(--text-tertiary);
}

.attachment-card__actions {
  display: flex;
  align-items: center;
  gap: 4px;
  flex-shrink: 0;
  opacity: 0;
  transition: opacity 0.15s ease;
}

.attachment-card:hover .attachment-card__actions {
  opacity: 1;
}

.attachment-card__action {
  border: none;
  background: transparent;
  color: var(--text-tertiary);
  cursor: pointer;
  padding: 6px;
  border-radius: 4px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  transition: color 0.15s ease, background 0.15s ease;
}

.attachment-card__action:hover {
  color: var(--primary);
  background: var(--primary-light);
}

.attachment-card__action--danger:hover {
  color: var(--priority-high-text);
  background: var(--priority-high-bg);
}
</style>
