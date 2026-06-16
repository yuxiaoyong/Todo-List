use lettre::message::header::ContentType;
use lettre::message::Mailbox;
use lettre::transport::smtp::authentication::Credentials;
use lettre::transport::smtp::client::{Tls, TlsParameters};
use lettre::{Message, SmtpTransport, Transport};
use serde::{Deserialize, Serialize};

use crate::db::repositories::{get_setting, set_setting, TodoSummary};
use crate::db::with_conn;
use crate::infra::error::{AppError, AppResult};

const CONFIG_KEY: &str = "email.gateway.config";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StoredEmailGatewayConfig {
    #[serde(default)]
    enabled: bool,
    #[serde(default)]
    host: String,
    #[serde(default = "default_port")]
    port: u16,
    #[serde(default = "default_security")]
    security: String,
    #[serde(default = "default_auth_type")]
    auth_type: String,
    #[serde(default)]
    username: String,
    #[serde(default)]
    password: String,
    #[serde(default)]
    from_address: String,
    #[serde(default)]
    from_name: String,
    #[serde(default)]
    default_recipient: String,
}

fn default_port() -> u16 {
    587
}

fn default_security() -> String {
    "tls".into()
}

fn default_auth_type() -> String {
    "none".into()
}

impl Default for StoredEmailGatewayConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            host: String::new(),
            port: default_port(),
            security: default_security(),
            auth_type: default_auth_type(),
            username: String::new(),
            password: String::new(),
            from_address: String::new(),
            from_name: String::new(),
            default_recipient: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EmailGatewayPublicConfig {
    pub enabled: bool,
    pub host: String,
    pub port: u16,
    pub security: String,
    pub auth_type: String,
    pub username: String,
    pub from_address: String,
    pub from_name: String,
    pub default_recipient: String,
    pub has_password: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmailGatewaySaveInput {
    pub enabled: bool,
    pub host: String,
    pub port: u16,
    pub security: String,
    pub auth_type: String,
    pub username: String,
    pub password: Option<String>,
    pub from_address: String,
    pub from_name: String,
    pub default_recipient: String,
}

pub fn get_public_config() -> AppResult<EmailGatewayPublicConfig> {
    let stored = load_stored_config()?;
    Ok(to_public_config(&stored))
}

pub fn save_config(input: EmailGatewaySaveInput) -> AppResult<EmailGatewayPublicConfig> {
    validate_save_input(&input)?;

    let mut stored = load_stored_config()?;
    stored.enabled = input.enabled;
    stored.host = input.host.trim().to_string();
    stored.port = input.port;
    stored.security = normalize_security(&input.security);
    stored.auth_type = normalize_auth_type(&input.auth_type);
    stored.username = input.username.trim().to_string();
    stored.from_address = input.from_address.trim().to_string();
    stored.from_name = input.from_name.trim().to_string();
    stored.default_recipient = input.default_recipient.trim().to_string();

    if stored.auth_type == "none" {
        stored.username.clear();
        stored.password.clear();
    } else if let Some(password) = input.password {
        let trimmed = password.trim();
        if !trimmed.is_empty() {
            stored.password = trimmed.to_string();
        }
    }

    persist_config(&stored)?;
    Ok(to_public_config(&stored))
}

pub fn send_test_email() -> AppResult<()> {
    let config = load_stored_config()?;
    if config.host.is_empty() {
        return Err(AppError::msg("请先填写 SMTP 服务器地址"));
    }
    if config.default_recipient.is_empty() {
        return Err(AppError::msg("请先填写默认收件人"));
    }
    if config.from_address.is_empty() {
        return Err(AppError::msg("请先填写发件人邮箱"));
    }

    let subject = "Todo List 邮件网关测试";
    let body = "这是一封测试邮件。若您收到此邮件，说明 SMTP 网关配置正确。";
    send_email(&config, &config.default_recipient, subject, body)
}

pub fn try_send_due_reminder(todo: &TodoSummary) -> AppResult<()> {
    let config = load_stored_config()?;
    if !config.enabled || config.default_recipient.is_empty() {
        return Ok(());
    }
    let subject = format!("任务到期提醒：{}", todo.title);
    let due = todo
        .due_date
        .as_deref()
        .unwrap_or("未设置")
        .to_string();
    let body = format!(
        "任务「{}」已到期或即将到期。\n\n截止日期：{}\n\n请在 Todo List 中查看并处理。",
        todo.title, due
    );
    let _ = send_email(&config, &config.default_recipient, &subject, &body);
    Ok(())
}

fn load_stored_config() -> AppResult<StoredEmailGatewayConfig> {
    with_conn(|conn| {
        let raw = get_setting(conn, CONFIG_KEY)?;
        Ok(match raw {
            Some(value) if !value.trim().is_empty() => {
                let mut config: StoredEmailGatewayConfig =
                    serde_json::from_str(&value).unwrap_or_default();
                migrate_auth_type(&mut config);
                config
            }
            _ => StoredEmailGatewayConfig::default(),
        })
    })
}

fn persist_config(config: &StoredEmailGatewayConfig) -> AppResult<()> {
    let raw = serde_json::to_string(config)?;
    with_conn(|conn| set_setting(conn, CONFIG_KEY, &raw))
}

fn to_public_config(stored: &StoredEmailGatewayConfig) -> EmailGatewayPublicConfig {
    EmailGatewayPublicConfig {
        enabled: stored.enabled,
        host: stored.host.clone(),
        port: stored.port,
        security: stored.security.clone(),
        auth_type: stored.auth_type.clone(),
        username: stored.username.clone(),
        from_address: stored.from_address.clone(),
        from_name: stored.from_name.clone(),
        default_recipient: stored.default_recipient.clone(),
        has_password: !stored.password.is_empty(),
    }
}

fn validate_save_input(input: &EmailGatewaySaveInput) -> AppResult<()> {
    if input.enabled {
        if input.host.trim().is_empty() {
            return Err(AppError::msg("启用邮件网关时需填写 SMTP 服务器"));
        }
        if input.from_address.trim().is_empty() {
            return Err(AppError::msg("启用邮件网关时需填写发件人邮箱"));
        }
        if input.default_recipient.trim().is_empty() {
            return Err(AppError::msg("启用邮件网关时需填写默认收件人"));
        }
    }
    if input.port == 0 {
        return Err(AppError::msg("SMTP 端口无效"));
    }
    if input.enabled && requires_auth(&normalize_auth_type(&input.auth_type)) {
        if input.username.trim().is_empty() {
            return Err(AppError::msg("当前认证方式需要填写用户名"));
        }
        let has_new_secret = input
            .password
            .as_ref()
            .is_some_and(|value| !value.trim().is_empty());
        let stored = load_stored_config()?;
        if !has_new_secret && stored.password.is_empty() {
            return Err(AppError::msg("当前认证方式需要填写密码或授权码"));
        }
    }
    Ok(())
}

fn normalize_security(value: &str) -> String {
    match value {
        "none" | "tls" | "ssl" => value.to_string(),
        _ => "tls".to_string(),
    }
}

fn normalize_auth_type(value: &str) -> String {
    match value {
        "none" | "password" | "authCode" => value.to_string(),
        _ => "none".to_string(),
    }
}

fn requires_auth(auth_type: &str) -> bool {
    auth_type == "password" || auth_type == "authCode"
}

fn migrate_auth_type(config: &mut StoredEmailGatewayConfig) {
    if config.auth_type != "none" {
        return;
    }
    if !config.username.is_empty() || !config.password.is_empty() {
        config.auth_type = "password".into();
    }
}

fn send_email(
    config: &StoredEmailGatewayConfig,
    to: &str,
    subject: &str,
    body: &str,
) -> AppResult<()> {
    let from_mailbox = build_from_mailbox(config)?;
    let to_mailbox: Mailbox = to
        .parse()
        .map_err(|_| AppError::msg(format!("收件人邮箱格式无效: {to}")))?;

    let email = Message::builder()
        .from(from_mailbox)
        .to(to_mailbox)
        .subject(subject)
        .header(ContentType::TEXT_PLAIN)
        .body(body.to_string())
        .map_err(|e| AppError::msg(format!("构建邮件失败: {e}")))?;

    let mailer = build_mailer(config)?;
    mailer
        .send(&email)
        .map_err(|e| AppError::msg(format!("发送邮件失败: {e}")))?;
    Ok(())
}

fn build_from_mailbox(config: &StoredEmailGatewayConfig) -> AppResult<Mailbox> {
    let address = config
        .from_address
        .parse::<Mailbox>()
        .map_err(|_| AppError::msg("发件人邮箱格式无效"))?;
    if config.from_name.trim().is_empty() {
        return Ok(address);
    }
    Ok(Mailbox::new(
        Some(config.from_name.trim().to_string()),
        address.email,
    ))
}

fn build_mailer(config: &StoredEmailGatewayConfig) -> AppResult<SmtpTransport> {
    let host = config.host.trim();
    if host.is_empty() {
        return Err(AppError::msg("SMTP 服务器地址为空"));
    }

    let credentials = if requires_auth(&config.auth_type) && !config.username.is_empty() {
        Some(Credentials::new(
            config.username.clone(),
            config.password.clone(),
        ))
    } else {
        None
    };

    let mut mailer = match config.security.as_str() {
        "ssl" => {
            let tls = TlsParameters::new(host.to_string())
                .map_err(|e| AppError::msg(format!("TLS 参数错误: {e}")))?;
            SmtpTransport::relay(host)
                .map_err(|e| AppError::msg(format!("SMTP 连接失败: {e}")))?
                .port(config.port)
                .tls(Tls::Wrapper(tls))
        }
        "tls" => SmtpTransport::starttls_relay(host)
            .map_err(|e| AppError::msg(format!("SMTP 连接失败: {e}")))?
            .port(config.port),
        _ => SmtpTransport::builder_dangerous(host).port(config.port),
    };

    if let Some(credentials) = credentials {
        mailer = mailer.credentials(credentials);
    }

    Ok(mailer.build())
}
