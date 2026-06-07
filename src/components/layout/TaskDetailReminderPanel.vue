<script setup lang="ts">
import { computed, watch } from "vue";
import { useI18n } from "vue-i18n";
import {
  LUNAR_FESTIVAL_PRESETS,
  formatLunarLabel,
  getLunarMonthDayCount,
  yearHasLeapMonth,
} from "../../utils/lunar";
import {
  RECURRENCE_ADVANCE_OPTIONS,
  RECURRENCE_FREQ_OPTIONS,
  computeNextRecurrenceDate,
  recurrenceAnchorDate,
  recurrenceLunarReady,
  recurrenceStartDate,
  syncLunarFromSolar,
  type RecurrenceConfig,
} from "../../utils/recurrence";

const props = defineProps<{
  recurrence: RecurrenceConfig;
  startDate?: string | null;
  dueDate?: string | null;
  editable: boolean;
}>();

const { t, locale } = useI18n();

const calendarOptions = computed(() => [
  { value: "solar", label: t("taskDetail.recurrenceCalendar.solar") },
  { value: "lunar", label: t("taskDetail.recurrenceCalendar.lunar") },
]);

const isLunar = computed(() => props.recurrence.calendar === "lunar");

const freqOptions = computed(() => {
  const options = isLunar.value ? (["yearly"] as const) : RECURRENCE_FREQ_OPTIONS;
  return options.map((freq) => ({
    value: freq,
    label: t(`taskDetail.recurrenceFreq.${freq}`),
  }));
});

const anchorOptions = computed(() => [
  { value: "startDate", label: t("taskDetail.recurrenceAnchor.startDate") },
  { value: "dueDate", label: t("taskDetail.recurrenceAnchor.dueDate") },
]);

const advanceOptions = computed(() =>
  RECURRENCE_ADVANCE_OPTIONS.map((minutes) => ({
    value: minutes,
    label: t(`taskDetail.recurrenceAdvance.${minutes}`),
  })),
);

const onCompleteOptions = computed(() => [
  { value: "reschedule", label: t("taskDetail.recurrenceOnComplete.reschedule") },
  { value: "stay", label: t("taskDetail.recurrenceOnComplete.stay") },
]);

const lunarMonthOptions = computed(() =>
  Array.from({ length: 12 }, (_, index) => {
    const month = index + 1;
    return {
      value: month,
      label: t(`taskDetail.lunarMonth.${month}`),
    };
  }),
);

const lunarYear = computed(() => {
  const anchor = recurrenceAnchorDate(props.recurrence.anchor, {
    startDate: props.startDate,
    dueDate: props.dueDate,
  });
  if (props.recurrence.firstReminderDate) {
    return props.recurrence.firstReminderDate.slice(0, 4);
  }
  return anchor?.slice(0, 4) ?? String(new Date().getFullYear());
});

const lunarDayOptions = computed(() => {
  const year = Number(lunarYear.value) || new Date().getFullYear();
  const month = props.recurrence.lunarMonth ?? 1;
  const isLeap = !!props.recurrence.isLeapMonth;
  const count = getLunarMonthDayCount(year, month, isLeap);
  return Array.from({ length: count }, (_, index) => {
    const day = index + 1;
    return {
      value: day,
      label: t(`taskDetail.lunarDay.${day}`),
    };
  });
});

const showLeapMonth = computed(() => {
  const year = Number(lunarYear.value) || new Date().getFullYear();
  const month = props.recurrence.lunarMonth ?? 1;
  return yearHasLeapMonth(year, month);
});

const festivalOptions = computed(() =>
  LUNAR_FESTIVAL_PRESETS.map((item) => ({
    ...item,
    label: t(`taskDetail.lunarFestival.${item.id}`),
  })),
);

const anchorDate = computed(() =>
  recurrenceAnchorDate(props.recurrence.anchor, {
    startDate: props.startDate,
    dueDate: props.dueDate,
  }),
);

const startDateMissing = computed(() => {
  if (!props.recurrence.enabled) return false;
  if (isLunar.value) return !recurrenceLunarReady(props.recurrence);
  return !recurrenceStartDate(props.recurrence, {
    startDate: props.startDate,
    dueDate: props.dueDate,
  });
});

const nextReminderDate = computed(() =>
  computeNextRecurrenceDate(props.recurrence, {
    startDate: props.startDate,
    dueDate: props.dueDate,
  }),
);

const lunarPreviewLabel = computed(() => {
  if (!recurrenceLunarReady(props.recurrence)) return "";
  const year = Number(lunarYear.value) || new Date().getFullYear();
  return formatLunarLabel(
    year,
    props.recurrence.lunarMonth!,
    props.recurrence.lunarDay!,
    !!props.recurrence.isLeapMonth,
    locale.value,
  );
});

watch(
  () => props.recurrence.calendar,
  (calendar) => {
    if (calendar === "lunar") {
      props.recurrence.freq = "yearly";
      if (!recurrenceLunarReady(props.recurrence)) {
        syncLunarFromSolar(
          props.recurrence,
          props.recurrence.firstReminderDate ?? anchorDate.value,
        );
      }
    }
  },
);

watch(
  () => props.recurrence.lunarMonth,
  () => {
    if (!props.recurrence.isLeapMonth) return;
    if (!showLeapMonth.value) {
      props.recurrence.isLeapMonth = false;
    }
  },
);

watch(lunarDayOptions, (options) => {
  const maxDay = options[options.length - 1]?.value;
  if (maxDay && props.recurrence.lunarDay && props.recurrence.lunarDay > maxDay) {
    props.recurrence.lunarDay = maxDay;
  }
});

function applyAnchorDate() {
  if (!anchorDate.value) return;
  if (isLunar.value) {
    syncLunarFromSolar(props.recurrence, anchorDate.value);
    return;
  }
  props.recurrence.firstReminderDate = anchorDate.value;
}

function applyFestival(presetId?: string) {
  if (!presetId) return;
  const preset = LUNAR_FESTIVAL_PRESETS.find((item) => item.id === presetId);
  if (!preset) return;
  props.recurrence.calendar = "lunar";
  props.recurrence.freq = "yearly";
  props.recurrence.lunarMonth = preset.lunarMonth;
  props.recurrence.lunarDay = preset.lunarDay;
  props.recurrence.isLeapMonth = false;
}
</script>

<template>
  <div class="reminder-panel">
    <p class="reminder-desc">{{ t("taskDetail.recurrenceDesc") }}</p>

    <div class="reminder-field">
      <div class="reminder-label">{{ t("taskDetail.recurrenceCalendarLabel") }}</div>
      <el-segmented
        v-model="recurrence.calendar"
        :options="calendarOptions"
        :disabled="!editable"
        class="calendar-segmented"
      />
    </div>

    <el-alert
      v-if="startDateMissing"
      type="warning"
      :closable="false"
      show-icon
      class="reminder-alert"
      :title="isLunar ? t('taskDetail.recurrenceLunarMissing') : t('taskDetail.recurrenceStartMissing')"
    />

    <div class="reminder-field reminder-field--first">
      <div class="reminder-label">
        {{ isLunar ? t("taskDetail.recurrenceLunarDateLabel") : t("taskDetail.recurrenceFirstReminderLabel") }}
      </div>

      <template v-if="isLunar">
        <div class="lunar-picker-row">
          <el-select
            v-model="recurrence.lunarMonth"
            :placeholder="t('taskDetail.lunarMonthPlaceholder')"
            :disabled="!editable"
            class="lunar-picker"
          >
            <el-option
              v-for="option in lunarMonthOptions"
              :key="option.value"
              :label="option.label"
              :value="option.value"
            />
          </el-select>
          <el-select
            v-model="recurrence.lunarDay"
            :placeholder="t('taskDetail.lunarDayPlaceholder')"
            :disabled="!editable"
            class="lunar-picker"
          >
            <el-option
              v-for="option in lunarDayOptions"
              :key="option.value"
              :label="option.label"
              :value="option.value"
            />
          </el-select>
          <el-time-select
            v-model="recurrence.time"
            start="00:00"
            step="00:15"
            end="23:45"
            :disabled="!editable"
            class="first-reminder-time"
          />
        </div>
        <div class="lunar-extra-row">
          <el-checkbox
            v-if="showLeapMonth"
            v-model="recurrence.isLeapMonth"
            :disabled="!editable"
          >
            {{ t("taskDetail.lunarLeapMonth") }}
          </el-checkbox>
          <el-select
            v-if="editable"
            clearable
            :placeholder="t('taskDetail.lunarFestivalPlaceholder')"
            class="festival-select"
            @change="applyFestival"
          >
            <el-option
              v-for="option in festivalOptions"
              :key="option.id"
              :label="option.label"
              :value="option.id"
            />
          </el-select>
          <el-button
            v-if="editable && anchorDate"
            text
            type="primary"
            @click="applyAnchorDate"
          >
            {{ t("taskDetail.recurrenceUseAnchorDate") }}
          </el-button>
        </div>
        <p v-if="lunarPreviewLabel" class="reminder-hint">
          {{ t("taskDetail.recurrenceLunarPreview", { label: lunarPreviewLabel }) }}
        </p>
      </template>

      <template v-else>
        <div class="first-reminder-row">
          <el-date-picker
            v-model="recurrence.firstReminderDate"
            type="date"
            value-format="YYYY-MM-DD"
            :placeholder="t('taskDetail.recurrenceFirstReminderPlaceholder')"
            :disabled="!editable"
            clearable
            class="first-reminder-date"
          />
          <el-time-select
            v-model="recurrence.time"
            start="00:00"
            step="00:15"
            end="23:45"
            :disabled="!editable"
            class="first-reminder-time"
          />
          <el-button
            v-if="editable && anchorDate"
            text
            type="primary"
            class="first-reminder-fill"
            @click="applyAnchorDate"
          >
            {{ t("taskDetail.recurrenceUseAnchorDate") }}
          </el-button>
        </div>
        <p class="reminder-hint">{{ t("taskDetail.recurrenceFirstReminderHint") }}</p>
      </template>
    </div>

    <div class="reminder-field" v-if="isLunar">
      <div class="reminder-label">{{ t("taskDetail.recurrenceSeriesStartLabel") }}</div>
      <el-date-picker
        v-model="recurrence.firstReminderDate"
        type="date"
        value-format="YYYY-MM-DD"
        :placeholder="t('taskDetail.recurrenceSeriesStartPlaceholder')"
        :disabled="!editable"
        clearable
        class="reminder-control"
      />
      <p class="reminder-hint">{{ t("taskDetail.recurrenceSeriesStartHint") }}</p>
    </div>

    <div class="reminder-grid">
      <div class="reminder-field">
        <div class="reminder-label">{{ t("taskDetail.recurrenceFreqLabel") }}</div>
        <el-select
          v-model="recurrence.freq"
          :disabled="!editable || isLunar"
          class="reminder-control"
        >
          <el-option
            v-for="option in freqOptions"
            :key="option.value"
            :label="option.label"
            :value="option.value"
          />
        </el-select>
      </div>

      <div class="reminder-field">
        <div class="reminder-label">{{ t("taskDetail.recurrenceIntervalLabel") }}</div>
        <el-input-number
          v-model="recurrence.interval"
          :min="1"
          :max="99"
          :disabled="!editable"
          controls-position="right"
          class="reminder-control"
        />
      </div>

      <div class="reminder-field">
        <div class="reminder-label">{{ t("taskDetail.recurrenceAnchorLabel") }}</div>
        <el-select
          v-model="recurrence.anchor"
          :disabled="!editable"
          class="reminder-control"
        >
          <el-option
            v-for="option in anchorOptions"
            :key="option.value"
            :label="option.label"
            :value="option.value"
          />
        </el-select>
      </div>

      <div class="reminder-field">
        <div class="reminder-label">{{ t("taskDetail.recurrenceAdvanceLabel") }}</div>
        <el-select
          v-model="recurrence.advanceMinutes"
          :disabled="!editable"
          class="reminder-control"
        >
          <el-option
            v-for="option in advanceOptions"
            :key="option.value"
            :label="option.label"
            :value="option.value"
          />
        </el-select>
      </div>

      <div class="reminder-field">
        <div class="reminder-label">{{ t("taskDetail.recurrenceOnCompleteLabel") }}</div>
        <el-select
          v-model="recurrence.onComplete"
          :disabled="!editable"
          class="reminder-control"
        >
          <el-option
            v-for="option in onCompleteOptions"
            :key="option.value"
            :label="option.label"
            :value="option.value"
          />
        </el-select>
      </div>
    </div>

    <div v-if="nextReminderDate && !startDateMissing" class="next-reminder">
      <span class="next-reminder-label">{{ t("taskDetail.recurrenceNextLabel") }}</span>
      <span class="next-reminder-value">
        <template v-if="isLunar">
          {{ t("taskDetail.recurrenceNextSolar", { date: nextReminderDate, time: recurrence.time }) }}
        </template>
        <template v-else>
          {{ nextReminderDate }} {{ recurrence.time }}
        </template>
      </span>
    </div>
  </div>
</template>

<style scoped>
.reminder-panel {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.reminder-desc {
  margin: 0;
  font-size: 13px;
  color: var(--text-secondary);
  line-height: 1.6;
}

.reminder-alert {
  margin: 0;
}

.calendar-segmented {
  width: fit-content;
  max-width: 100%;
}

.reminder-field--first {
  padding-bottom: 4px;
  border-bottom: 1px solid var(--border-light);
}

.first-reminder-row,
.lunar-picker-row {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 10px;
}

.lunar-extra-row {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 12px;
  margin-top: 10px;
}

.first-reminder-date,
.lunar-picker,
.festival-select {
  flex: 1 1 160px;
  min-width: 0;
}

.first-reminder-time {
  flex: 0 0 120px;
}

.first-reminder-fill {
  flex-shrink: 0;
}

.reminder-hint {
  margin: 8px 0 0;
  font-size: 12px;
  color: var(--text-secondary);
  line-height: 1.5;
}

.reminder-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 16px 20px;
}

.reminder-field {
  display: flex;
  flex-direction: column;
  gap: 8px;
  min-width: 0;
}

.reminder-label {
  font-size: 13px;
  color: var(--text-secondary);
}

.reminder-control {
  width: 100%;
}

.next-reminder {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px 14px;
  border-radius: 8px;
  background: var(--surface-subtle);
  font-size: 13px;
}

.next-reminder-label {
  color: var(--text-secondary);
  flex-shrink: 0;
}

.next-reminder-value {
  color: var(--text-primary);
  font-weight: 500;
}
</style>
