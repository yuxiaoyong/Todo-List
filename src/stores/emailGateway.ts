import { defineStore } from "pinia";
import { emailGatewayApi, type EmailGatewayPublicConfig, type EmailGatewaySaveInput } from "../api";

export type EmailSecurity = "none" | "tls" | "ssl";
export type EmailAuthType = "none" | "password" | "authCode";

export interface EmailGatewayDraft {
  enabled: boolean;
  host: string;
  port: number;
  security: EmailSecurity;
  authType: EmailAuthType;
  username: string;
  password: string;
  fromAddress: string;
  fromName: string;
  defaultRecipient: string;
  hasPassword: boolean;
}

function toDraft(config: EmailGatewayPublicConfig): EmailGatewayDraft {
  return {
    enabled: config.enabled,
    host: config.host,
    port: config.port,
    security: (config.security as EmailSecurity) || "tls",
    authType: (config.authType as EmailAuthType) || "none",
    username: config.username,
    password: "",
    fromAddress: config.fromAddress,
    fromName: config.fromName,
    defaultRecipient: config.defaultRecipient,
    hasPassword: config.hasPassword,
  };
}

export const useEmailGatewayStore = defineStore("emailGateway", {
  state: (): {
    draft: EmailGatewayDraft;
    ready: boolean;
    loading: boolean;
    saving: boolean;
    testing: boolean;
  } => ({
    draft: {
      enabled: false,
      host: "",
      port: 587,
      security: "tls",
      authType: "none",
      username: "",
      password: "",
      fromAddress: "",
      fromName: "",
      defaultRecipient: "",
      hasPassword: false,
    },
    ready: false,
    loading: false,
    saving: false,
    testing: false,
  }),
  actions: {
    async load() {
      this.loading = true;
      try {
        const config = await emailGatewayApi.getConfig();
        this.draft = toDraft(config);
      } catch (error) {
        console.error("load email gateway config failed", error);
        throw error;
      } finally {
        this.loading = false;
        this.ready = true;
      }
    },
    async save() {
      this.saving = true;
      try {
        const input: EmailGatewaySaveInput = {
          enabled: this.draft.enabled,
          host: this.draft.host,
          port: this.draft.port,
          security: this.draft.security,
          authType: this.draft.authType,
          username: this.draft.username,
          fromAddress: this.draft.fromAddress,
          fromName: this.draft.fromName,
          defaultRecipient: this.draft.defaultRecipient,
        };
        const trimmedPassword = this.draft.password.trim();
        if (trimmedPassword) {
          input.password = trimmedPassword;
        }
        const saved = await emailGatewayApi.saveConfig(input);
        this.draft = toDraft(saved);
      } finally {
        this.saving = false;
      }
    },
    async sendTest() {
      this.testing = true;
      try {
        await emailGatewayApi.sendTest();
      } finally {
        this.testing = false;
      }
    },
  },
});
