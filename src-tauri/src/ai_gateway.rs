use chrono::Local;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::time::Duration;

use crate::db::repositories::{get_setting, set_setting};
use crate::db::with_conn;
use crate::error::{AppError, AppResult};

const CONFIG_KEY: &str = "ai.gateway.config";
const DEFAULT_TIMEOUT_SECS: u64 = 30;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StoredAiConfig {
    #[serde(default)]
    enabled: bool,
    #[serde(default = "default_provider")]
    provider: String,
    #[serde(default)]
    cloud_base_url: String,
    #[serde(default)]
    cloud_api_key: String,
    #[serde(default = "default_cloud_model")]
    cloud_model: String,
    #[serde(default = "default_ollama_base_url")]
    ollama_base_url: String,
    #[serde(default = "default_ollama_model")]
    ollama_model: String,
    #[serde(default = "default_timeout_secs")]
    timeout_secs: u64,
}

fn default_provider() -> String {
    "cloud".into()
}

fn default_cloud_model() -> String {
    "gpt-4o-mini".into()
}

fn default_ollama_base_url() -> String {
    "http://127.0.0.1:11434".into()
}

fn default_ollama_model() -> String {
    "llama3.2".into()
}

fn default_timeout_secs() -> u64 {
    DEFAULT_TIMEOUT_SECS
}

impl Default for StoredAiConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            provider: default_provider(),
            cloud_base_url: String::new(),
            cloud_api_key: String::new(),
            cloud_model: default_cloud_model(),
            ollama_base_url: default_ollama_base_url(),
            ollama_model: default_ollama_model(),
            timeout_secs: default_timeout_secs(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AiGatewayPublicConfig {
    pub enabled: bool,
    pub provider: String,
    pub cloud_base_url: String,
    pub cloud_model: String,
    pub has_api_key: bool,
    pub ollama_base_url: String,
    pub ollama_model: String,
    pub timeout_secs: u64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiGatewaySaveInput {
    pub enabled: bool,
    pub provider: String,
    pub cloud_base_url: String,
    pub cloud_api_key: Option<String>,
    pub cloud_model: String,
    pub ollama_base_url: String,
    pub ollama_model: String,
    pub timeout_secs: u64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiNameRef {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiParseTaskInput {
    pub text: String,
    pub categories: Vec<AiNameRef>,
    pub tags: Vec<AiNameRef>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AiParsedTask {
    pub title: String,
    pub start_date: Option<String>,
    pub due_date: Option<String>,
    pub priority: String,
    pub category_id: Option<i64>,
    pub tag_ids: Vec<i64>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiDecomposeSubtasksInput {
    pub title: String,
    pub content: Option<String>,
    pub existing_steps: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AiDecomposedSubtasks {
    pub steps: Vec<String>,
}

pub fn get_public_config() -> AppResult<AiGatewayPublicConfig> {
    Ok(to_public_config(&load_stored_config()?))
}

pub fn save_config(input: AiGatewaySaveInput) -> AppResult<AiGatewayPublicConfig> {
    let mut stored = load_stored_config()?;
    stored.enabled = input.enabled;
    stored.provider = normalize_provider(&input.provider);
    stored.cloud_base_url = input.cloud_base_url.trim().trim_end_matches('/').to_string();
    stored.cloud_model = input.cloud_model.trim().to_string();
    stored.ollama_base_url = input.ollama_base_url.trim().trim_end_matches('/').to_string();
    stored.ollama_model = input.ollama_model.trim().to_string();
    stored.timeout_secs = input.timeout_secs.clamp(5, 120);

    if stored.provider == "cloud" {
        if stored.cloud_base_url.is_empty() {
            return Err(AppError::msg("cloud base URL is required"));
        }
        if stored.cloud_model.is_empty() {
            return Err(AppError::msg("cloud model is required"));
        }
        if let Some(key) = input.cloud_api_key {
            stored.cloud_api_key = key.trim().to_string();
        }
        if stored.enabled && stored.cloud_api_key.is_empty() {
            return Err(AppError::msg("API key is required"));
        }
    } else if stored.ollama_model.is_empty() {
        return Err(AppError::msg("ollama model is required"));
    }

    persist_stored_config(&stored)?;
    Ok(to_public_config(&stored))
}

pub fn test_connection() -> AppResult<()> {
    let config = load_stored_config()?;
    if !config.enabled {
        return Err(AppError::msg("AI is disabled"));
    }
    let prompt = "Reply with JSON only: {\"ok\":true}";
    let text = match config.provider.as_str() {
        "ollama" => call_ollama(&config, prompt)?,
        _ => call_openai_compatible(&config, prompt)?,
    };
    if text.trim().is_empty() {
        return Err(AppError::msg("empty response from AI provider"));
    }
    Ok(())
}

pub fn parse_task(input: AiParseTaskInput) -> AppResult<AiParsedTask> {
    let text = input.text.trim();
    if text.is_empty() {
        return Err(AppError::msg("input text is empty"));
    }

    let config = load_stored_config()?;
    if !config.enabled {
        return Err(AppError::msg("AI is disabled"));
    }

    let today = Local::now().format("%Y-%m-%d").to_string();
    let prompt = build_parse_prompt(text, &today, &input.categories, &input.tags);
    let raw = match config.provider.as_str() {
        "ollama" => call_ollama(&config, &prompt)?,
        _ => call_openai_compatible(&config, &prompt)?,
    };

    let value = extract_json_value(&raw)?;
    map_parsed_task(value, &input.categories, &input.tags)
}

pub fn decompose_subtasks(input: AiDecomposeSubtasksInput) -> AppResult<AiDecomposedSubtasks> {
    let title = input.title.trim();
    if title.is_empty() {
        return Err(AppError::msg("task title is empty"));
    }

    let config = load_stored_config()?;
    if !config.enabled {
        return Err(AppError::msg("AI is disabled"));
    }

    let content = input
        .content
        .as_deref()
        .map(str::trim)
        .filter(|text| !text.is_empty());
    let prompt = build_decompose_prompt(title, content, &input.existing_steps);
    let raw = match config.provider.as_str() {
        "ollama" => call_ollama(&config, &prompt)?,
        _ => call_openai_compatible(&config, &prompt)?,
    };

    let value = extract_json_value(&raw)?;
    map_decomposed_subtasks(value, &input.existing_steps)
}

fn normalize_provider(value: &str) -> String {
    if value == "ollama" {
        "ollama".into()
    } else {
        "cloud".into()
    }
}

fn to_public_config(stored: &StoredAiConfig) -> AiGatewayPublicConfig {
    AiGatewayPublicConfig {
        enabled: stored.enabled,
        provider: stored.provider.clone(),
        cloud_base_url: stored.cloud_base_url.clone(),
        cloud_model: stored.cloud_model.clone(),
        has_api_key: !stored.cloud_api_key.is_empty(),
        ollama_base_url: stored.ollama_base_url.clone(),
        ollama_model: stored.ollama_model.clone(),
        timeout_secs: stored.timeout_secs,
    }
}

fn load_stored_config() -> AppResult<StoredAiConfig> {
    with_conn(|conn| {
        let raw = get_setting(conn, CONFIG_KEY)?;
        match raw {
            Some(value) => Ok(serde_json::from_str(&value)?),
            None => Ok(StoredAiConfig::default()),
        }
    })
}

fn persist_stored_config(config: &StoredAiConfig) -> AppResult<()> {
    let json = serde_json::to_string(config)?;
    with_conn(|conn| set_setting(conn, CONFIG_KEY, &json))
}

fn build_parse_prompt(
    text: &str,
    today: &str,
    categories: &[AiNameRef],
    tags: &[AiNameRef],
) -> String {
    let category_names: Vec<&str> = categories.iter().map(|c| c.name.as_str()).collect();
    let tag_names: Vec<&str> = tags.iter().map(|t| t.name.as_str()).collect();

    format!(
        r#"You parse natural-language todo items into structured JSON.

Today is {today} (YYYY-MM-DD). Resolve relative dates like "tomorrow", "next Friday", "6/10" against today.

Available categories (pick at most one by exact name, or null): {categories}
Available tags (pick only from this list, 0-3 items): {tags}

User input:
{text}

Return ONLY valid JSON with this schema:
{{
  "title": "string, required, concise task title",
  "startDate": "YYYY-MM-DD or null",
  "dueDate": "YYYY-MM-DD or null",
  "priority": "high" | "medium" | "low",
  "categoryName": "string or null",
  "tagNames": ["string"]
}}

Rules:
- title must not be empty
- use null when date/priority/category/tags are unclear
- categoryName and tagNames must match available lists or be null/empty
- do not invent categories or tags not in the lists"#,
        categories = serde_json::to_string(&category_names).unwrap_or_else(|_| "[]".into()),
        tags = serde_json::to_string(&tag_names).unwrap_or_else(|_| "[]".into()),
    )
}

fn build_decompose_prompt(title: &str, content: Option<&str>, existing_steps: &[String]) -> String {
    let existing: Vec<&str> = existing_steps
        .iter()
        .map(|step| step.as_str())
        .filter(|step| !step.trim().is_empty())
        .collect();
    let description = content.unwrap_or("(none)");

    format!(
        r#"You break down a todo task into actionable sub-steps (checklist items).

Task title:
{title}

Task description (may be empty):
{description}

Existing sub-steps (do NOT repeat these):
{existing}

Return ONLY valid JSON:
{{
  "steps": ["string", "..."]
}}

Rules:
- 3 to 12 concise steps, each starting with a verb when possible
- one actionable item per step, no dates/priority/category
- steps must be in logical execution order
- do not duplicate existing sub-steps (case-insensitive)
- respond in the same language as the task title/description"#,
        existing = serde_json::to_string(&existing).unwrap_or_else(|_| "[]".into()),
    )
}

fn http_client(timeout_secs: u64) -> AppResult<Client> {
    Client::builder()
        .timeout(Duration::from_secs(timeout_secs.clamp(5, 120)))
        .build()
        .map_err(|err| AppError::msg(format!("HTTP client error: {err}")))
}

fn call_openai_compatible(config: &StoredAiConfig, prompt: &str) -> AppResult<String> {
    if config.cloud_api_key.is_empty() {
        return Err(AppError::msg("API key is not configured"));
    }

    let base = if config.cloud_base_url.is_empty() {
        "https://api.openai.com/v1".to_string()
    } else {
        config.cloud_base_url.clone()
    };
    let url = format!("{base}/chat/completions");
    let client = http_client(config.timeout_secs)?;

    let body = json!({
        "model": config.cloud_model,
        "temperature": 0.2,
        "response_format": { "type": "json_object" },
        "messages": [
            { "role": "system", "content": "You extract todo task fields and respond with JSON only." },
            { "role": "user", "content": prompt }
        ]
    });

    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", config.cloud_api_key))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .map_err(|err| AppError::msg(format!("AI request failed: {err}")))?;

    let status = response.status();
    let payload: Value = response
        .json()
        .map_err(|err| AppError::msg(format!("invalid AI response: {err}")))?;

    if !status.is_success() {
        let message = payload
            .pointer("/error/message")
            .and_then(Value::as_str)
            .unwrap_or("AI request failed");
        return Err(AppError::msg(format!("AI error ({status}): {message}")));
    }

    extract_message_content(&payload)
}

fn call_ollama(config: &StoredAiConfig, prompt: &str) -> AppResult<String> {
    let base = if config.ollama_base_url.is_empty() {
        default_ollama_base_url()
    } else {
        config.ollama_base_url.clone()
    };
    let url = format!("{base}/api/chat");
    let client = http_client(config.timeout_secs)?;

    let body = json!({
        "model": config.ollama_model,
        "stream": false,
        "format": "json",
        "messages": [
            { "role": "system", "content": "You extract todo task fields and respond with JSON only." },
            { "role": "user", "content": prompt }
        ]
    });

    let response = client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .map_err(|err| AppError::msg(format!("Ollama request failed: {err}")))?;

    let status = response.status();
    let payload: Value = response
        .json()
        .map_err(|err| AppError::msg(format!("invalid Ollama response: {err}")))?;

    if !status.is_success() {
        return Err(AppError::msg(format!(
            "Ollama error ({status}): {}",
            payload
                .pointer("/error")
                .and_then(Value::as_str)
                .unwrap_or("request failed")
        )));
    }

    payload
        .pointer("/message/content")
        .and_then(Value::as_str)
        .map(|value| value.to_string())
        .ok_or_else(|| AppError::msg("Ollama returned empty content"))
}

fn extract_message_content(payload: &Value) -> AppResult<String> {
    payload
        .pointer("/choices/0/message/content")
        .and_then(Value::as_str)
        .map(|value| value.to_string())
        .ok_or_else(|| AppError::msg("AI returned empty content"))
}

fn extract_json_value(raw: &str) -> AppResult<Value> {
    let trimmed = raw.trim();
    if let Ok(value) = serde_json::from_str(trimmed) {
        return Ok(value);
    }

    if let Some(start) = trimmed.find('{') {
        if let Some(end) = trimmed.rfind('}') {
            if end > start {
                let slice = &trimmed[start..=end];
                if let Ok(value) = serde_json::from_str(slice) {
                    return Ok(value);
                }
            }
        }
    }

    Err(AppError::msg("AI response is not valid JSON"))
}

fn normalize_date(value: Option<&Value>) -> Option<String> {
    let text = value?.as_str()?.trim();
    if text.is_empty() || text.eq_ignore_ascii_case("null") {
        return None;
    }
    if text.len() >= 10 && text.as_bytes().get(4) == Some(&b'-') {
        return Some(text[..10].to_string());
    }
    None
}

fn normalize_priority(value: Option<&Value>) -> String {
    match value.and_then(Value::as_str).unwrap_or("medium").to_lowercase().as_str() {
        "high" => "high".into(),
        "low" => "low".into(),
        _ => "medium".into(),
    }
}

fn match_name_id(name: &str, items: &[AiNameRef]) -> Option<i64> {
    let needle = name.trim().to_lowercase();
    if needle.is_empty() {
        return None;
    }
    items
        .iter()
        .find(|item| item.name.trim().to_lowercase() == needle)
        .map(|item| item.id)
}

fn map_parsed_task(
    value: Value,
    categories: &[AiNameRef],
    tags: &[AiNameRef],
) -> AppResult<AiParsedTask> {
    let title = value
        .get("title")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .ok_or_else(|| AppError::msg("AI did not return a task title"))?
        .to_string();

    let category_id = value
        .get("categoryName")
        .and_then(Value::as_str)
        .and_then(|name| match_name_id(name, categories));

    let mut tag_ids = Vec::new();
    if let Some(names) = value.get("tagNames").and_then(Value::as_array) {
        for name in names {
            if let Some(text) = name.as_str() {
                if let Some(id) = match_name_id(text, tags) {
                    if !tag_ids.contains(&id) {
                        tag_ids.push(id);
                    }
                }
            }
        }
    }

    Ok(AiParsedTask {
        title,
        start_date: normalize_date(value.get("startDate")),
        due_date: normalize_date(value.get("dueDate")),
        priority: normalize_priority(value.get("priority")),
        category_id,
        tag_ids,
    })
}

fn map_decomposed_subtasks(value: Value, existing_steps: &[String]) -> AppResult<AiDecomposedSubtasks> {
    let steps_value = value
        .get("steps")
        .and_then(Value::as_array)
        .ok_or_else(|| AppError::msg("AI did not return subtask steps"))?;

    let existing_lower: Vec<String> = existing_steps
        .iter()
        .map(|step| step.trim().to_lowercase())
        .filter(|step| !step.is_empty())
        .collect();

    let mut steps = Vec::new();
    let mut seen = existing_lower;

    for item in steps_value {
        let Some(text) = item.as_str() else {
            continue;
        };
        let trimmed = text.trim();
        if trimmed.is_empty() {
            continue;
        }
        let key = trimmed.to_lowercase();
        if seen.contains(&key) {
            continue;
        }
        seen.push(key);
        steps.push(trimmed.to_string());
    }

    if steps.is_empty() {
        return Err(AppError::msg("AI returned no new subtask steps"));
    }

    Ok(AiDecomposedSubtasks { steps })
}
