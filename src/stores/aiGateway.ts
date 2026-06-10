import { defineStore } from "pinia";
import { aiGatewayApi, type AiGatewayPublicConfig, type AiGatewaySaveInput } from "../api";

export type AiProvider = "cloud" | "ollama";

export interface AiGatewayDraft {
  enabled: boolean;
  provider: AiProvider;
  cloudBaseUrl: string;
  cloudApiKey: string;
  cloudModel: string;
  ollamaBaseUrl: string;
  ollamaModel: string;
  timeoutSecs: number;
  hasApiKey: boolean;
}

function toDraft(config: AiGatewayPublicConfig): AiGatewayDraft {
  return {
    enabled: config.enabled,
    provider: config.provider === "ollama" ? "ollama" : "cloud",
    cloudBaseUrl: config.cloudBaseUrl,
    cloudApiKey: "",
    cloudModel: config.cloudModel,
    ollamaBaseUrl: config.ollamaBaseUrl,
    ollamaModel: config.ollamaModel,
    timeoutSecs: config.timeoutSecs,
    hasApiKey: config.hasApiKey,
  };
}

export const useAiGatewayStore = defineStore("aiGateway", {
  state: (): {
    draft: AiGatewayDraft;
    ready: boolean;
    loading: boolean;
    saving: boolean;
    testing: boolean;
  } => ({
    draft: {
      enabled: false,
      provider: "cloud",
      cloudBaseUrl: "",
      cloudApiKey: "",
      cloudModel: "gpt-4o-mini",
      ollamaBaseUrl: "http://127.0.0.1:11434",
      ollamaModel: "llama3.2",
      timeoutSecs: 30,
      hasApiKey: false,
    },
    ready: false,
    loading: false,
    saving: false,
    testing: false,
  }),
  getters: {
    isActive: (state) => state.ready && state.draft.enabled,
  },
  actions: {
    async load() {
      this.loading = true;
      try {
        const config = await aiGatewayApi.getConfig();
        this.draft = toDraft(config);
      } catch (error) {
        console.error("load ai gateway config failed", error);
        throw error;
      } finally {
        this.loading = false;
        this.ready = true;
      }
    },
    async save() {
      this.saving = true;
      try {
        const input: AiGatewaySaveInput = {
          enabled: this.draft.enabled,
          provider: this.draft.provider,
          cloudBaseUrl: this.draft.cloudBaseUrl,
          cloudModel: this.draft.cloudModel,
          ollamaBaseUrl: this.draft.ollamaBaseUrl,
          ollamaModel: this.draft.ollamaModel,
          timeoutSecs: this.draft.timeoutSecs,
        };
        const trimmedKey = this.draft.cloudApiKey.trim();
        if (trimmedKey) {
          input.cloudApiKey = trimmedKey;
        }
        const saved = await aiGatewayApi.saveConfig(input);
        this.draft = toDraft(saved);
      } finally {
        this.saving = false;
      }
    },
    async testConnection() {
      this.testing = true;
      try {
        await aiGatewayApi.testConnection();
      } finally {
        this.testing = false;
      }
    },
  },
});
