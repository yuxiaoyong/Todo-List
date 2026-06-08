import { invoke as rawInvoke } from "@tauri-apps/api/core";
import { ElMessage } from "element-plus";
import { i18n } from "../i18n";

export interface HealthCheckResult {
  ok: boolean;
  appDataDir: string;
  dbPath: string;
  message?: string | null;
}

export interface InvokeOptions {
  /** 失败时不弹出 ElMessage */
  silent?: boolean;
  /** 用于日志与提示的操作名 */
  operation?: string;
}

function formatInvokeError(error: unknown): string {
  if (typeof error === "string") return error;
  if (error && typeof error === "object") {
    if ("message" in error && typeof (error as { message: unknown }).message === "string") {
      return (error as { message: string }).message;
    }
  }
  return String(error);
}

async function writeAppLog(level: string, message: string) {
  try {
    await rawInvoke("app_log", { level, message });
  } catch (error) {
    console.warn("write app log failed", error);
  }
}

export async function tauriInvoke<T>(
  cmd: string,
  args?: Record<string, unknown>,
  options?: InvokeOptions,
): Promise<T> {
  const operation = options?.operation ?? cmd;
  try {
    return await rawInvoke<T>(cmd, args);
  } catch (error) {
    const detail = formatInvokeError(error);
    console.error(`[invoke:${operation}]`, error);
    void writeAppLog("error", `[${operation}] ${detail}`);
    if (!options?.silent) {
      ElMessage.error({
        message: i18n.global.t("errors.invokeFailed", { detail }),
        duration: 8000,
        showClose: true,
      });
    }
    throw error;
  }
}

export const systemApi = {
  healthCheck: () => rawInvoke<HealthCheckResult>("app_health_check"),
  log: (level: string, message: string) => rawInvoke<void>("app_log", { level, message }),
};
