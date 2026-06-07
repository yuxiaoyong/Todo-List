<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { attachmentApi } from "../../api";
import {
  decodeBase64Utf8,
  getAttachmentName,
  getFileTypeKind,
} from "../../utils/attachmentTypes";
import type { AttachmentInfo } from "../../types";

const { t } = useI18n();

const props = defineProps<{
  visible: boolean;
  item: AttachmentInfo | null;
}>();

const emit = defineEmits<{
  "update:visible": [value: boolean];
  "open-local": [item: AttachmentInfo];
}>();

const loading = ref(false);
const textContent = ref("");
const loadError = ref("");

const dialogVisible = computed({
  get: () => props.visible,
  set: (value: boolean) => emit("update:visible", value),
});

const title = computed(() =>
  props.item ? getAttachmentName(props.item) : t("attachment.previewTitle"),
);

async function loadPreview(item: AttachmentInfo) {
  loading.value = true;
  loadError.value = "";
  textContent.value = "";

  try {
    if (getFileTypeKind(item) === "text") {
      const base64 = await attachmentApi.read(item.todoId, item.filename);
      const maxBytes = 512 * 1024;
      if (item.fileSize > maxBytes) {
        textContent.value = `${decodeBase64Utf8(base64).slice(0, maxBytes)}\n\n${t("attachment.previewTruncated")}`;
      } else {
        textContent.value = decodeBase64Utf8(base64);
      }
    }
  } catch (err) {
    console.error("Failed to load attachment preview:", err);
    loadError.value = t("attachment.previewFailed");
  } finally {
    loading.value = false;
  }
}

watch(
  () => [props.visible, props.item?.id] as const,
  ([visible, id]) => {
    if (!visible || !id || !props.item) return;
    void loadPreview(props.item);
  },
);

function openLocal() {
  if (!props.item) return;
  emit("open-local", props.item);
}
</script>

<template>
  <el-dialog
    v-model="dialogVisible"
    :title="title"
    width="860px"
    class="attachment-preview-dialog"
    destroy-on-close
    align-center
  >
    <div v-loading="loading" class="attachment-preview-dialog__body">
      <div v-if="loadError" class="attachment-preview-dialog__error">
        {{ loadError }}
      </div>

      <pre
        v-else-if="textContent"
        class="attachment-preview-dialog__text"
      >{{ textContent }}</pre>
    </div>

    <template #footer>
      <el-button @click="dialogVisible = false">{{ t("common.close") }}</el-button>
      <el-button type="primary" @click="openLocal">{{ t("attachment.openLocal") }}</el-button>
    </template>
  </el-dialog>
</template>

<style scoped>
.attachment-preview-dialog__body {
  min-height: 200px;
  max-height: 70vh;
  overflow: auto;
}

.attachment-preview-dialog__text {
  margin: 0;
  white-space: pre-wrap;
  word-break: break-word;
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
  font-size: 13px;
  line-height: 1.5;
}

.attachment-preview-dialog__error {
  color: var(--el-color-danger);
  font-size: 14px;
}
</style>
