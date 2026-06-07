<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useI18n } from "vue-i18n";
import { ElMessage, ElMessageBox } from "element-plus";
import { dataApi, type DataInfo } from "../../api";

const { t } = useI18n();
const loading = ref(false);
const info = ref<DataInfo | null>(null);

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

async function loadInfo() {
  try {
    info.value = await dataApi.getInfo();
  } catch (error) {
    console.error("load data info failed", error);
  }
}

async function openDataDir() {
  try {
    await dataApi.openAppDataDir();
  } catch (error) {
    console.error("open data dir failed", error);
    ElMessage.error(t("data.openDirFailed"));
  }
}

async function createBackup() {
  loading.value = true;
  try {
    const saved = await dataApi.createBackup();
    if (saved) {
      ElMessage.success(t("data.backupSaved", { path: saved }));
    }
  } catch (error) {
    console.error("create backup failed", error);
    ElMessage.error(t("data.backupFailed"));
  } finally {
    loading.value = false;
  }
}

async function restoreBackup() {
  try {
    await ElMessageBox.confirm(t("data.restoreConfirm"), t("data.restoreTitle"), {
      type: "warning",
      confirmButtonText: t("common.restore"),
      cancelButtonText: t("common.cancel"),
    });
  } catch {
    return;
  }

  loading.value = true;
  try {
    const restored = await dataApi.restoreBackup();
    if (restored) {
      ElMessage.success(t("data.restoreOk"));
      await loadInfo();
    }
  } catch (error) {
    console.error("restore backup failed", error);
    const message =
      typeof error === "string"
        ? error
        : error instanceof Error
          ? error.message
          : t("data.restoreFailed");
    ElMessage.error(message);
  } finally {
    loading.value = false;
  }
}

async function exportJson() {
  loading.value = true;
  try {
    const saved = await dataApi.exportJson();
    if (saved) {
      ElMessage.success(t("data.exportSaved", { path: saved }));
    }
  } catch (error) {
    console.error("export json failed", error);
    ElMessage.error(t("data.exportFailed"));
  } finally {
    loading.value = false;
  }
}

async function importJson() {
  try {
    await ElMessageBox.confirm(t("data.importConfirm"), t("data.importTitle"), {
      type: "warning",
      confirmButtonText: t("common.import"),
      cancelButtonText: t("common.cancel"),
    });
  } catch {
    return;
  }

  loading.value = true;
  try {
    const result = await dataApi.importJson();
    if (result) {
      ElMessage.success(
        t("data.importOk", {
          imported: result.todosImported,
          skipped: result.todosSkipped,
        }),
      );
      await loadInfo();
    }
  } catch (error) {
    console.error("import json failed", error);
    const message =
      typeof error === "string"
        ? error
        : error instanceof Error
          ? error.message
          : t("data.importFailed");
    ElMessage.error(message);
  } finally {
    loading.value = false;
  }
}

onMounted(() => {
  void loadInfo();
});

defineExpose({ reload: loadInfo });
</script>

<template>
  <section class="settings-section">
    <h3 class="settings-section-title">{{ t("data.title") }}</h3>
    <p class="settings-section-desc">{{ t("data.desc") }}</p>

    <div v-if="info" class="data-info-card">
      <div class="data-info-row">
        <span class="data-info-label">{{ t("data.dataDir") }}</span>
        <span class="data-info-value" :title="info.appDataDir">{{ info.appDataDir }}</span>
      </div>
      <div class="data-info-row">
        <span class="data-info-label">{{ t("data.dbSize") }}</span>
        <span class="data-info-value">{{ formatBytes(info.dbSizeBytes) }}</span>
      </div>
      <div class="data-info-row">
        <span class="data-info-label">{{ t("data.todoAttachment") }}</span>
        <span class="data-info-value">{{
          t("data.todoAttachmentValue", {
            todos: info.todoCount,
            attachments: info.attachmentCount,
          })
        }}</span>
      </div>
    </div>

    <div class="data-actions">
      <el-button :loading="loading" @click="openDataDir">{{ t("data.openDir") }}</el-button>
      <el-button type="primary" :loading="loading" @click="createBackup">{{
        t("data.createBackup")
      }}</el-button>
      <el-button :loading="loading" @click="restoreBackup">{{ t("data.restoreBackup") }}</el-button>
      <el-button :loading="loading" @click="exportJson">{{ t("data.exportJson") }}</el-button>
      <el-button :loading="loading" @click="importJson">{{ t("data.importJson") }}</el-button>
    </div>

    <ul class="data-hints">
      <li>{{ t("data.hintBackup") }}</li>
      <li>{{ t("data.hintExport") }}</li>
      <li>{{ t("data.hintImport") }}</li>
    </ul>
  </section>
</template>

<style scoped>
.settings-section {
  padding: 0;
}

.settings-section-title {
  margin: 0 0 6px;
  font-size: 15px;
  font-weight: 600;
  color: var(--text-primary);
}

.settings-section-desc {
  margin: 0 0 16px;
  font-size: 13px;
  color: var(--text-secondary);
}

.data-info-card {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-bottom: 16px;
  padding: 12px 14px;
  border: 1px solid var(--border-color);
  border-radius: var(--radius);
  background: var(--surface-muted);
}

.data-info-row {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
  font-size: 13px;
}

.data-info-label {
  flex-shrink: 0;
  color: var(--text-secondary);
}

.data-info-value {
  min-width: 0;
  text-align: right;
  color: var(--text-primary);
  word-break: break-all;
}

.data-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.data-hints {
  margin: 14px 0 0;
  padding-left: 18px;
  font-size: 12px;
  color: var(--text-secondary);
  line-height: 1.6;
}
</style>
