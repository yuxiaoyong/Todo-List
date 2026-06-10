<script setup lang="ts">
import { computed, onMounted } from "vue";
import { useI18n } from "vue-i18n";
import { ElMessage } from "element-plus";
import { useAiGatewayStore, type AiProvider } from "../../stores/aiGateway";

const { t } = useI18n();
const aiStore = useAiGatewayStore();

const providerOptions: { value: AiProvider; labelKey: string }[] = [
  { value: "cloud", labelKey: "aiGateway.providerCloud" },
  { value: "ollama", labelKey: "aiGateway.providerOllama" },
];

const isCloud = computed(() => aiStore.draft.provider === "cloud");

const apiKeyPlaceholder = computed(() =>
  aiStore.draft.hasApiKey
    ? t("aiGateway.apiKeyKeepPlaceholder")
    : t("aiGateway.apiKeyPlaceholder"),
);

onMounted(() => {
  if (!aiStore.ready) {
    void aiStore.load().catch(() => {
      ElMessage.error(t("aiGateway.loadFailed"));
    });
  }
});

async function onSave() {
  try {
    await aiStore.save();
    ElMessage.success(t("aiGateway.saveOk"));
  } catch (error) {
    console.error("save ai gateway failed", error);
    const message =
      typeof error === "string"
        ? error
        : error instanceof Error
          ? error.message
          : t("aiGateway.saveFailed");
    ElMessage.error(message);
  }
}

async function onTest() {
  try {
    await aiStore.testConnection();
    ElMessage.success(t("aiGateway.testOk"));
  } catch (error) {
    console.error("ai test connection failed", error);
    const message =
      typeof error === "string"
        ? error
        : error instanceof Error
          ? error.message
          : t("aiGateway.testFailed");
    ElMessage.error(message);
  }
}
</script>

<template>
  <section v-loading="aiStore.loading" class="settings-section">
    <h3 class="settings-section-title">{{ t("aiGateway.title") }}</h3>
    <p class="settings-section-desc">{{ t("aiGateway.desc") }}</p>

    <div class="ai-form">
      <div class="ai-row ai-row--switch">
        <div class="ai-field-info">
          <span class="ai-label">{{ t("aiGateway.enabled") }}</span>
          <span class="ai-hint">{{ t("aiGateway.enabledHint") }}</span>
        </div>
        <el-switch v-model="aiStore.draft.enabled" />
      </div>

      <div class="ai-grid" :class="{ 'is-disabled': !aiStore.draft.enabled }">
        <label class="ai-field ai-field--full">
          <span class="ai-label">{{ t("aiGateway.provider") }}</span>
          <el-segmented
            v-model="aiStore.draft.provider"
            :options="providerOptions.map((o) => ({ label: t(o.labelKey), value: o.value }))"
            :disabled="!aiStore.draft.enabled"
            class="ai-segmented"
          />
        </label>

        <template v-if="isCloud">
          <label class="ai-field ai-field--full">
            <span class="ai-label">{{ t("aiGateway.cloudBaseUrl") }}</span>
            <el-input
              v-model="aiStore.draft.cloudBaseUrl"
              :placeholder="t('aiGateway.cloudBaseUrlPlaceholder')"
              :disabled="!aiStore.draft.enabled"
            />
          </label>

          <label class="ai-field ai-field--full">
            <span class="ai-label">{{ t("aiGateway.apiKey") }}</span>
            <el-input
              v-model="aiStore.draft.cloudApiKey"
              type="password"
              show-password
              :placeholder="apiKeyPlaceholder"
              :disabled="!aiStore.draft.enabled"
              autocomplete="new-password"
            />
          </label>

          <label class="ai-field ai-field--full">
            <span class="ai-label">{{ t("aiGateway.cloudModel") }}</span>
            <el-input
              v-model="aiStore.draft.cloudModel"
              :placeholder="t('aiGateway.cloudModelPlaceholder')"
              :disabled="!aiStore.draft.enabled"
            />
          </label>
        </template>

        <template v-else>
          <label class="ai-field ai-field--full">
            <span class="ai-label">{{ t("aiGateway.ollamaBaseUrl") }}</span>
            <el-input
              v-model="aiStore.draft.ollamaBaseUrl"
              :placeholder="t('aiGateway.ollamaBaseUrlPlaceholder')"
              :disabled="!aiStore.draft.enabled"
            />
          </label>

          <label class="ai-field ai-field--full">
            <span class="ai-label">{{ t("aiGateway.ollamaModel") }}</span>
            <el-input
              v-model="aiStore.draft.ollamaModel"
              :placeholder="t('aiGateway.ollamaModelPlaceholder')"
              :disabled="!aiStore.draft.enabled"
            />
          </label>
        </template>

        <label class="ai-field">
          <span class="ai-label">{{ t("aiGateway.timeout") }}</span>
          <el-input-number
            v-model="aiStore.draft.timeoutSecs"
            :min="5"
            :max="120"
            :disabled="!aiStore.draft.enabled"
            controls-position="right"
            class="ai-timeout-input"
          />
        </label>
      </div>
    </div>

    <div class="ai-actions">
      <el-button type="primary" :loading="aiStore.saving" @click="onSave">
        {{ t("common.save") }}
      </el-button>
      <el-button :loading="aiStore.testing" :disabled="!aiStore.draft.enabled" @click="onTest">
        {{ t("aiGateway.testConnection") }}
      </el-button>
    </div>

    <ul class="ai-hints">
      <li>{{ t("aiGateway.hintPreview") }}</li>
      <li>{{ t("aiGateway.hintLocal") }}</li>
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

.ai-form {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.ai-row--switch {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  padding: 12px 14px;
  border: 1px solid var(--border-color);
  border-radius: var(--radius);
  background: var(--surface-muted);
}

.ai-field-info {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.ai-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 12px;
}

.ai-grid.is-disabled {
  opacity: 0.72;
}

.ai-field {
  display: flex;
  flex-direction: column;
  gap: 6px;
  min-width: 0;
}

.ai-field--full {
  grid-column: 1 / -1;
}

.ai-label {
  font-size: 13px;
  font-weight: 500;
  color: var(--text-primary);
}

.ai-hint {
  font-size: 12px;
  line-height: 1.5;
  color: var(--text-secondary);
}

.ai-segmented {
  width: fit-content;
  max-width: 100%;
}

.ai-timeout-input {
  width: 100%;
}

.ai-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-top: 16px;
}

.ai-hints {
  margin: 14px 0 0;
  padding-left: 18px;
  font-size: 12px;
  color: var(--text-secondary);
  line-height: 1.6;
}
</style>
