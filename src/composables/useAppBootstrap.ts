import { onMounted } from "vue";
import { useI18n } from "vue-i18n";
import { ElMessage } from "element-plus";
import { systemApi } from "../utils/tauriInvoke";

let bootstrapDone = false;

export function useAppBootstrap() {
  const { t } = useI18n();

  onMounted(() => {
    if (bootstrapDone) return;
    bootstrapDone = true;
    void runHealthCheck(t);
  });
}

async function runHealthCheck(t: (key: string, params?: Record<string, unknown>) => string) {
  try {
    const result = await systemApi.healthCheck();
    if (result.ok) return;

    const detail = result.message ?? t("errors.healthCheckUnknown");
    ElMessage.error({
      message: `${t("errors.healthCheckFailed")} ${detail}`,
      duration: 0,
      showClose: true,
    });
    if (result.appDataDir) {
      ElMessage.warning({
        message: t("errors.healthCheckDataDir", { dir: result.appDataDir }),
        duration: 0,
        showClose: true,
      });
    }
    await systemApi.log("error", `health_check failed: ${detail}`);
  } catch (error) {
    console.error("health check failed", error);
    ElMessage.error({
      message: t("errors.healthCheckUnavailable"),
      duration: 0,
      showClose: true,
    });
  }
}
