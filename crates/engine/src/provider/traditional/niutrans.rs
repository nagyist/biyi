//! 小牛翻译 (NiuTrans) text translation.
//!
//! Docs: https://niutrans.com/documents/contents/trans_text
//!
//! One form-encoded POST with an `apikey`; the server answers 200 for
//! everything and signals failure with `error_code` / `error_msg` in the
//! body, so the status line alone never tells us whether it worked.

use crate::common::http_client::HttpClient;
use async_trait::async_trait;
use beyondtranslate_core::{
    Provider, TextTranslation, TranslateRequest, TranslateResponse, TranslationError,
    TranslationService,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Deserialize, PartialEq, Serialize)]
pub struct NiutransProviderConfig {
    #[serde(rename = "apiKey", alias = "api_key", alias = "apikey")]
    pub api_key: String,
    #[serde(rename = "baseUrl", alias = "base_url")]
    pub base_url: Option<String>,
}

pub struct NiutransProvider {
    service: NiutransTranslationService,
}

struct NiutransTranslationService {
    api_key: String,
    http: HttpClient,
}

impl NiutransProvider {
    pub fn new(config: NiutransProviderConfig) -> Result<Self, String> {
        if config.api_key.trim().is_empty() {
            return Err("api_key must not be empty".to_owned());
        }
        Ok(Self {
            service: NiutransTranslationService {
                api_key: config.api_key,
                http: HttpClient::new(
                    config
                        .base_url
                        .unwrap_or_else(|| "https://api.niutrans.com".to_owned()),
                    Default::default(),
                ),
            },
        })
    }
}

#[async_trait(?Send)]
impl TranslationService for NiutransTranslationService {
    async fn translate(
        &self,
        request: TranslateRequest,
    ) -> Result<TranslateResponse, TranslationError> {
        let target = request.target_language.ok_or_else(|| {
            TranslationError::InvalidRequest("target_language is required".to_owned())
        })?;
        let source = request
            .source_language
            .as_deref()
            .map(niutrans_language_code)
            .unwrap_or_else(|| "auto".to_owned());

        let form = [
            ("from", source),
            ("to", niutrans_language_code(&target)),
            ("apikey", self.api_key.clone()),
            ("src_text", request.text),
        ];
        let req = self.http.post("/NiuTransServer/translation").form(&form);
        let resp = self
            .http
            .execute(req)
            .await
            .map_err(TranslationError::from_network_error)?;
        let resp = TranslationError::from_response("niutrans", resp).await?;
        let data: Value = resp
            .json()
            .await
            .map_err(|error| TranslationError::SerializationError(error.to_string()))?;
        ensure_niutrans_success(&data)?;

        let text = data["tgt_text"]
            .as_str()
            .ok_or_else(|| {
                TranslationError::SerializationError("niutrans: missing tgt_text".to_owned())
            })?
            .to_owned();
        let detected_source_language = data["from"]
            .as_str()
            .filter(|code| *code != "auto")
            .map(app_language_code);

        Ok(TranslateResponse {
            translations: vec![TextTranslation {
                detected_source_language,
                text,
                audio_url: None,
            }],
        })
    }
}

impl Provider for NiutransProvider {
    fn name(&self) -> &'static str {
        "niutrans"
    }

    fn translation(&self) -> Option<&dyn TranslationService> {
        Some(&self.service)
    }
}

/// The app speaks BCP 47 (`zh-Hans`, `pt-BR`); NiuTrans wants its own short
/// codes, with Traditional Chinese as `cht` and no region subtags.
fn niutrans_language_code(language: &str) -> String {
    match language {
        "zh" | "zh-Hans" | "zh-CN" | "zh-SG" => "zh".to_owned(),
        "zh-Hant" | "zh-TW" | "zh-HK" | "zh-MO" => "cht".to_owned(),
        other => other.split(['-', '_']).next().unwrap_or(other).to_owned(),
    }
}

/// The reverse of [`niutrans_language_code`] for the detected source.
fn app_language_code(code: &str) -> String {
    match code {
        "zh" => "zh-Hans".to_owned(),
        "cht" => "zh-Hant".to_owned(),
        other => other.to_owned(),
    }
}

fn ensure_niutrans_success(data: &Value) -> Result<(), TranslationError> {
    let code = match &data["error_code"] {
        Value::String(code) if !code.is_empty() => code.clone(),
        Value::Number(code) => code.to_string(),
        _ => return Ok(()),
    };
    let message = data["error_msg"].as_str().unwrap_or("unknown error");
    let message = format!("niutrans: {code}: {message}");
    // https://niutrans.com/documents/contents/trans_text#error
    Err(match code.as_str() {
        // 13001 / 13002 / 13003: apikey missing, invalid, or account frozen.
        "13001" | "13002" | "13003" => TranslationError::AuthError(message),
        // 13007: characters exhausted, 13008: too many requests.
        "13007" | "13008" => TranslationError::RateLimitError(message),
        // 10000: missing parameter, 10001: text too long, 10002: unsupported
        // language, 10003: source equals target.
        "10000" | "10001" | "10002" | "10003" => TranslationError::InvalidRequest(message),
        _ => TranslationError::NetworkError(message),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_chinese_variants_to_niutrans_codes() {
        assert_eq!(niutrans_language_code("zh-Hans"), "zh");
        assert_eq!(niutrans_language_code("zh-Hant"), "cht");
        assert_eq!(niutrans_language_code("pt-BR"), "pt");
        assert_eq!(niutrans_language_code("en"), "en");
        assert_eq!(app_language_code("cht"), "zh-Hant");
        assert_eq!(app_language_code("zh"), "zh-Hans");
    }

    #[test]
    fn rejects_error_bodies() {
        let auth = serde_json::json!({"error_code": "13002", "error_msg": "apikey invalid"});
        assert!(matches!(
            ensure_niutrans_success(&auth),
            Err(TranslationError::AuthError(_))
        ));
        let ok = serde_json::json!({"from": "en", "to": "zh", "tgt_text": "你好"});
        assert!(ensure_niutrans_success(&ok).is_ok());
    }
}
