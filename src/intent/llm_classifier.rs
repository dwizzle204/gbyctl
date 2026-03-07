//! LLM-backed intent classification.

use std::collections::BTreeMap;

use anyhow::Result;
use reqwest::blocking::Client;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue};
use reqwest::tls::Version;
use serde::Deserialize;
use serde_json::Value;

use crate::config::{ModelConfig, ProviderKind};
use crate::skills::registry::parse_skill_id;
use crate::skills::types::{IntentConfidence, IntentMatch};

const SYSTEM_PROMPT: &str = "You are an intent classifier for an Ubuntu operations CLI. Classify user requests.\
Return JSON only with keys: disposition, skill_id, confidence, args, message.\
Allowed disposition: skill, clarification, refusal, out_of_scope.\
Allowed skill_id: doctor, service_status, disk_full_triage, inspect_storage, resize_root_plan, install_package, troubleshoot_firewall, diagnose_reboot_or_kernel_issue, maintenance_guidance, logs_guidance, package_status.\
Use out_of_scope for coding/software-development tasks or unrelated asks.";

/// Output classes from model intent decision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Classification {
    /// Resolved skill intent.
    Intent(IntentMatch),
    /// Request needs clarification before routing.
    Clarification(String),
    /// Request should be refused.
    Refusal(String),
    /// Request is unrelated or unsupported.
    OutOfScope(String),
}

/// Errors from model classification.
#[derive(Debug, thiserror::Error)]
pub enum ClassifyError {
    /// Authentication failed; key likely invalid or expired.
    #[error("provider authentication failed")]
    AuthFailed,
    /// Transport or parsing failure.
    #[error("classification failed: {0}")]
    Other(String),
}

#[derive(Debug, Deserialize)]
struct ModelDecision {
    disposition: String,
    skill_id: Option<String>,
    confidence: Option<String>,
    args: Option<BTreeMap<String, String>>,
    message: Option<String>,
}

/// Classify a natural-language request using configured provider.
pub fn classify(
    request: &str,
    cfg: &ModelConfig,
    api_key: &str,
) -> std::result::Result<Classification, ClassifyError> {
    let json_text = match cfg.provider {
        ProviderKind::OpenAiCompatible => classify_openai_compatible(request, cfg, api_key)?,
        ProviderKind::Claude => classify_claude(request, cfg, api_key)?,
    };

    // Provider output must deserialize into a strict decision schema. Any drift
    // here is treated as classifier failure and handled by caller fallback.
    let parsed: ModelDecision = serde_json::from_str(&json_text)
        .map_err(|err| ClassifyError::Other(format!("invalid classifier JSON: {err}")))?;

    match parsed.disposition.as_str() {
        "skill" => {
            let skill_id_raw = parsed.skill_id.as_deref().ok_or_else(|| {
                ClassifyError::Other("missing skill_id for skill disposition".to_owned())
            })?;
            let skill_id = parse_skill_id(skill_id_raw).ok_or_else(|| {
                ClassifyError::Other("unknown skill_id from classifier".to_owned())
            })?;
            let confidence = match parsed.confidence.as_deref() {
                Some("high") => IntentConfidence::High,
                Some("medium") => IntentConfidence::Medium,
                _ => IntentConfidence::Low,
            };
            let args = parsed.args.unwrap_or_default();
            Ok(Classification::Intent(IntentMatch {
                skill_id,
                confidence,
                args,
            }))
        }
        "clarification" => Ok(Classification::Clarification(
            parsed.message.unwrap_or_else(|| {
                "More details are required to classify this request.".to_owned()
            }),
        )),
        "refusal" => {
            Ok(Classification::Refusal(parsed.message.unwrap_or_else(
                || "Request refused by policy.".to_owned(),
            )))
        }
        "out_of_scope" => Ok(Classification::OutOfScope(parsed.message.unwrap_or_else(
            || "Request is not a supported Ubuntu operations ask.".to_owned(),
        ))),
        _ => Err(ClassifyError::Other(
            "unsupported disposition from classifier".to_owned(),
        )),
    }
}

fn classify_openai_compatible(
    request: &str,
    cfg: &ModelConfig,
    api_key: &str,
) -> std::result::Result<String, ClassifyError> {
    let url = format!("{}/chat/completions", cfg.base_url.trim_end_matches('/'));

    let mut headers = HeaderMap::new();
    let token = format!("Bearer {api_key}");
    let auth = HeaderValue::from_str(&token)
        .map_err(|err| ClassifyError::Other(format!("invalid auth header: {err}")))?;
    let _old = headers.insert(AUTHORIZATION, auth);
    let _old = headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    // We force JSON-object response format to keep output machine-parseable.
    let body = serde_json::json!({
        "model": cfg.model,
        "temperature": 0,
        "response_format": {"type": "json_object"},
        "messages": [
            {"role": "system", "content": SYSTEM_PROMPT},
            {"role": "user", "content": request}
        ]
    });

    let client = Client::builder()
        .use_rustls_tls()
        .https_only(true)
        .min_tls_version(Version::TLS_1_3)
        .build()
        .map_err(|err| ClassifyError::Other(format!("failed to build TLS client: {err}")))?;
    let response = client
        .post(url)
        .headers(headers)
        .json(&body)
        .send()
        .map_err(|err| ClassifyError::Other(format!("request failed: {err}")))?;

    if response.status().as_u16() == 401 || response.status().as_u16() == 403 {
        return Err(ClassifyError::AuthFailed);
    }

    if !response.status().is_success() {
        return Err(ClassifyError::Other(format!(
            "provider status: {}",
            response.status()
        )));
    }

    let value: Value = response
        .json()
        .map_err(|err| ClassifyError::Other(format!("invalid provider JSON: {err}")))?;

    value
        .get("choices")
        .and_then(Value::as_array)
        .and_then(|arr| arr.first())
        .and_then(|choice| choice.get("message"))
        .and_then(|msg| msg.get("content"))
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| ClassifyError::Other("missing choices[0].message.content".to_owned()))
}

fn classify_claude(
    request: &str,
    cfg: &ModelConfig,
    api_key: &str,
) -> std::result::Result<String, ClassifyError> {
    let url = format!("{}/messages", cfg.base_url.trim_end_matches('/'));

    let mut headers = HeaderMap::new();
    let key = HeaderValue::from_str(api_key)
        .map_err(|err| ClassifyError::Other(format!("invalid API key header: {err}")))?;
    let _old = headers.insert("x-api-key", key);
    let _old = headers.insert("anthropic-version", HeaderValue::from_static("2023-06-01"));
    let _old = headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    // Claude path mirrors the same strict schema contract via system prompt.
    let body = serde_json::json!({
        "model": cfg.model,
        "temperature": 0,
        "max_tokens": 300,
        "system": SYSTEM_PROMPT,
        "messages": [
            {"role": "user", "content": request}
        ]
    });

    let client = Client::builder()
        .use_rustls_tls()
        .https_only(true)
        .min_tls_version(Version::TLS_1_3)
        .build()
        .map_err(|err| ClassifyError::Other(format!("failed to build TLS client: {err}")))?;
    let response = client
        .post(url)
        .headers(headers)
        .json(&body)
        .send()
        .map_err(|err| ClassifyError::Other(format!("request failed: {err}")))?;

    if response.status().as_u16() == 401 || response.status().as_u16() == 403 {
        return Err(ClassifyError::AuthFailed);
    }

    if !response.status().is_success() {
        return Err(ClassifyError::Other(format!(
            "provider status: {}",
            response.status()
        )));
    }

    let value: Value = response
        .json()
        .map_err(|err| ClassifyError::Other(format!("invalid provider JSON: {err}")))?;

    value
        .get("content")
        .and_then(Value::as_array)
        .and_then(|arr| arr.first())
        .and_then(|item| item.get("text"))
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| ClassifyError::Other("missing content[0].text".to_owned()))
}

/// Fall back to deterministic classifier when model call fails.
pub fn fallback_classify(request: &str) -> Result<Option<IntentMatch>> {
    crate::intent::router::route(request).map_or_else(|| Ok(None), |intent| Ok(Some(intent)))
}
