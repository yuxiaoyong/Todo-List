<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { ElMessage } from "element-plus";
import {
  ADVANCE_HOUR_OPTIONS,
  REPEAT_MINUTE_OPTIONS,
  useNotificationStore,
} from "../../stores/notification";

const { t } = useI18n();
const notificationStore = useNotificationStore();

const enabled = computed({
  get: () => notificationStore.enabled,
  set: (value: boolean) => {
    void save(() => notificationStore.setEnabled(value));
  },
});

const system = computed({
  get: () => notificationStore.system,
  set: (value: boolean) => {
    void save(() => notificationStore.setSystem(value));
  },
});

const email = computed({
  get: () => notificationStore.email,
  set: (value: boolean) => {
    void save(() => notificationStore.setEmail(value));
  },
});

const advanceHours = computed({
  get: () => notificationStore.advanceHours,
  set: (value: number) => {
    void save(() => notificationStore.setAdvanceHours(value));
  },
});

const repeatMinutes = computed({
  get: () => notificationStore.repeatMinutes,
  set: (value: number) => {
    void save(() => notificationStore.setRepeatMinutes(value));
  },
});

const advanceHourOptions = computed(() =>
  ADVANCE_HOUR_OPTIONS.map((hours) => ({
    value: hours,
    label: t("notifications.advanceHoursOption", { hours }),
  })),
);

const repeatMinuteOptions = computed(() =>
  REPEAT_MINUTE_OPTIONS.map((minutes) => ({
    value: minutes,
    label: t(`notifications.repeatMinutes${minutes}`),
  })),
);

async function save(action: () => Promise<void>) {
  try {
    await action();
  } catch {
    ElMessage.error(t("notifications.saveFailed"));
  }
}
</script>

<template>
  <section class="settings-section">
    <h3 class="settings-section-title">{{ t("notifications.settingsTitle") }}</h3>
    <p class="settings-section-desc">{{ t("notifications.settingsDesc") }}</p>

    <div class="notification-options">
      <div class="notification-row">
        <div class="notification-info">
          <span class="notification-label">{{ t("notifications.enabled") }}</span>
          <span class="notification-hint">{{ t("notifications.enabledHint") }}</span>
        </div>
        <el-switch v-model="enabled" />
      </div>

      <div class="notification-row" :class="{ 'is-disabled': !enabled }">
        <div class="notification-info">
          <span class="notification-label">{{ t("notifications.system") }}</span>
          <span class="notification-hint">{{ t("notifications.systemHint") }}</span>
        </div>
        <el-switch v-model="system" :disabled="!enabled" />
      </div>

      <div class="notification-row" :class="{ 'is-disabled': !enabled }">
        <div class="notification-info">
          <span class="notification-label">{{ t("notifications.email") }}</span>
          <span class="notification-hint">{{ t("notifications.emailHint") }}</span>
        </div>
        <el-switch v-model="email" :disabled="!enabled" />
      </div>

      <div class="notification-row" :class="{ 'is-disabled': !enabled }">
        <div class="notification-info">
          <span class="notification-label">{{ t("notifications.advanceHours") }}</span>
          <span class="notification-hint">{{ t("notifications.advanceHoursHint") }}</span>
        </div>
        <el-select
          v-model="advanceHours"
          class="notification-select"
          :disabled="!enabled"
        >
          <el-option
            v-for="option in advanceHourOptions"
            :key="option.value"
            :label="option.label"
            :value="option.value"
          />
        </el-select>
      </div>

      <div class="notification-row" :class="{ 'is-disabled': !enabled }">
        <div class="notification-info">
          <span class="notification-label">{{ t("notifications.repeatInterval") }}</span>
          <span class="notification-hint">{{ t("notifications.repeatIntervalHint") }}</span>
        </div>
        <el-select
          v-model="repeatMinutes"
          class="notification-select"
          :disabled="!enabled"
        >
          <el-option
            v-for="option in repeatMinuteOptions"
            :key="option.value"
            :label="option.label"
            :value="option.value"
          />
        </el-select>
      </div>
    </div>

    <ul class="notification-hints">
      <li>{{ t("notifications.hintRepeat") }}</li>
      <li>{{ t("notifications.hintAdvance") }}</li>
      <li>{{ t("notifications.hintDateOnly") }}</li>
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

.notification-options {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.notification-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  padding: 12px 14px;
  border: 1px solid var(--border-color);
  border-radius: var(--radius);
  background: var(--surface-muted);
}

.notification-row.is-disabled {
  opacity: 0.72;
}

.notification-info {
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.notification-label {
  font-size: 14px;
  font-weight: 500;
  color: var(--text-primary);
}

.notification-hint {
  font-size: 12px;
  line-height: 1.5;
  color: var(--text-secondary);
}

.notification-select {
  width: 160px;
  flex-shrink: 0;
}

.notification-hints {
  margin: 14px 0 0;
  padding-left: 18px;
  font-size: 12px;
  color: var(--text-secondary);
  line-height: 1.6;
}
</style>
