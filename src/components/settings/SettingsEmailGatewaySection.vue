<script setup lang="ts">
import { onMounted } from "vue";
import { useI18n } from "vue-i18n";
import { ElMessage } from "element-plus";
import { computed } from "vue";
import { useEmailGatewayStore, type EmailAuthType, type EmailSecurity } from "../../stores/emailGateway";

const { t } = useI18n();
const emailGatewayStore = useEmailGatewayStore();

const securityOptions: { value: EmailSecurity; labelKey: string }[] = [
  { value: "tls", labelKey: "emailGateway.securityTls" },
  { value: "ssl", labelKey: "emailGateway.securitySsl" },
  { value: "none", labelKey: "emailGateway.securityNone" },
];

const authTypeOptions: { value: EmailAuthType; labelKey: string }[] = [
  { value: "none", labelKey: "emailGateway.authNone" },
  { value: "password", labelKey: "emailGateway.authPassword" },
  { value: "authCode", labelKey: "emailGateway.authCode" },
];

const needsAuth = computed(
  () =>
    emailGatewayStore.draft.authType === "password" ||
    emailGatewayStore.draft.authType === "authCode",
);

const secretLabel = computed(() =>
  emailGatewayStore.draft.authType === "authCode"
    ? t("emailGateway.authCode")
    : t("emailGateway.password"),
);

const secretPlaceholder = computed(() => {
  if (emailGatewayStore.draft.hasPassword) {
    return emailGatewayStore.draft.authType === "authCode"
      ? t("emailGateway.authCodeKeepPlaceholder")
      : t("emailGateway.passwordKeepPlaceholder");
  }
  return emailGatewayStore.draft.authType === "authCode"
    ? t("emailGateway.authCodePlaceholder")
    : t("emailGateway.passwordPlaceholder");
});

onMounted(() => {
  if (!emailGatewayStore.ready) {
    void emailGatewayStore.load().catch(() => {
      ElMessage.error(t("emailGateway.loadFailed"));
    });
  }
});

async function onSave() {
  try {
    await emailGatewayStore.save();
    ElMessage.success(t("emailGateway.saveOk"));
  } catch (error) {
    console.error("save email gateway failed", error);
    const message =
      typeof error === "string"
        ? error
        : error instanceof Error
          ? error.message
          : t("emailGateway.saveFailed");
    ElMessage.error(message);
  }
}

async function onSendTest() {
  try {
    await emailGatewayStore.sendTest();
    ElMessage.success(t("emailGateway.testOk"));
  } catch (error) {
    console.error("send test email failed", error);
    const message =
      typeof error === "string"
        ? error
        : error instanceof Error
          ? error.message
          : t("emailGateway.testFailed");
    ElMessage.error(message);
  }
}
</script>

<template>
  <section v-loading="emailGatewayStore.loading" class="settings-section">
    <h3 class="settings-section-title">{{ t("emailGateway.title") }}</h3>
    <p class="settings-section-desc">{{ t("emailGateway.desc") }}</p>

    <div class="email-form">
      <div class="email-row email-row--switch">
        <div class="email-field-info">
          <span class="email-label">{{ t("emailGateway.enabled") }}</span>
          <span class="email-hint">{{ t("emailGateway.enabledHint") }}</span>
        </div>
        <el-switch v-model="emailGatewayStore.draft.enabled" />
      </div>

      <div class="email-grid" :class="{ 'is-disabled': !emailGatewayStore.draft.enabled }">
        <label class="email-field">
          <span class="email-label">{{ t("emailGateway.host") }}</span>
          <el-input
            v-model="emailGatewayStore.draft.host"
            :placeholder="t('emailGateway.hostPlaceholder')"
            :disabled="!emailGatewayStore.draft.enabled"
          />
        </label>

        <label class="email-field">
          <span class="email-label">{{ t("emailGateway.port") }}</span>
          <el-input-number
            v-model="emailGatewayStore.draft.port"
            :min="1"
            :max="65535"
            :disabled="!emailGatewayStore.draft.enabled"
            controls-position="right"
            class="email-port-input"
          />
        </label>

        <label class="email-field email-field--full">
          <span class="email-label">{{ t("emailGateway.security") }}</span>
          <el-select
            v-model="emailGatewayStore.draft.security"
            :disabled="!emailGatewayStore.draft.enabled"
            class="email-select"
          >
            <el-option
              v-for="option in securityOptions"
              :key="option.value"
              :label="t(option.labelKey)"
              :value="option.value"
            />
          </el-select>
        </label>

        <label class="email-field email-field--full">
          <span class="email-label">{{ t("emailGateway.authType") }}</span>
          <el-select
            v-model="emailGatewayStore.draft.authType"
            :disabled="!emailGatewayStore.draft.enabled"
            class="email-select"
          >
            <el-option
              v-for="option in authTypeOptions"
              :key="option.value"
              :label="t(option.labelKey)"
              :value="option.value"
            />
          </el-select>
        </label>

        <template v-if="needsAuth">
          <label class="email-field">
            <span class="email-label">{{ t("emailGateway.username") }}</span>
            <el-input
              v-model="emailGatewayStore.draft.username"
              :placeholder="t('emailGateway.usernamePlaceholder')"
              :disabled="!emailGatewayStore.draft.enabled"
              autocomplete="off"
            />
          </label>

          <label class="email-field">
            <span class="email-label">{{ secretLabel }}</span>
            <el-input
              v-model="emailGatewayStore.draft.password"
              type="password"
              show-password
              :placeholder="secretPlaceholder"
              :disabled="!emailGatewayStore.draft.enabled"
              autocomplete="new-password"
            />
          </label>
        </template>

        <label class="email-field">
          <span class="email-label">{{ t("emailGateway.fromAddress") }}</span>
          <el-input
            v-model="emailGatewayStore.draft.fromAddress"
            :placeholder="t('emailGateway.fromAddressPlaceholder')"
            :disabled="!emailGatewayStore.draft.enabled"
          />
        </label>

        <label class="email-field">
          <span class="email-label">{{ t("emailGateway.fromName") }}</span>
          <el-input
            v-model="emailGatewayStore.draft.fromName"
            :placeholder="t('emailGateway.fromNamePlaceholder')"
            :disabled="!emailGatewayStore.draft.enabled"
          />
        </label>

        <label class="email-field email-field--full">
          <span class="email-label">{{ t("emailGateway.defaultRecipient") }}</span>
          <el-input
            v-model="emailGatewayStore.draft.defaultRecipient"
            :placeholder="t('emailGateway.defaultRecipientPlaceholder')"
            :disabled="!emailGatewayStore.draft.enabled"
          />
        </label>
      </div>
    </div>

    <div class="email-actions">
      <el-button
        type="primary"
        :loading="emailGatewayStore.saving"
        @click="onSave"
      >
        {{ t("common.save") }}
      </el-button>
      <el-button
        :loading="emailGatewayStore.testing"
        :disabled="!emailGatewayStore.draft.defaultRecipient"
        @click="onSendTest"
      >
        {{ t("emailGateway.sendTest") }}
      </el-button>
    </div>

    <ul class="email-hints">
      <li>{{ t("emailGateway.hintSmtp") }}</li>
      <li>{{ t("emailGateway.hintDueReminder") }}</li>
      <li>{{ t("emailGateway.hintPassword") }}</li>
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

.email-form {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.email-row--switch {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  padding: 12px 14px;
  border: 1px solid var(--border-color);
  border-radius: var(--radius);
  background: var(--surface-muted);
}

.email-field-info {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.email-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 12px;
}

.email-grid.is-disabled {
  opacity: 0.72;
}

.email-field {
  display: flex;
  flex-direction: column;
  gap: 6px;
  min-width: 0;
}

.email-field--full {
  grid-column: 1 / -1;
}

.email-label {
  font-size: 13px;
  font-weight: 500;
  color: var(--text-primary);
}

.email-hint {
  font-size: 12px;
  line-height: 1.5;
  color: var(--text-secondary);
}

.email-select,
.email-port-input {
  width: 100%;
}

.email-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-top: 16px;
}

.email-hints {
  margin: 14px 0 0;
  padding-left: 18px;
  font-size: 12px;
  color: var(--text-secondary);
  line-height: 1.6;
}
</style>
