#![cfg_attr(not(feature = "tencent_cloud"), allow(dead_code))]

use crate::common::http_client::HttpClient;
use crate::common::ocr::{load_image, rect_from_bounds};
use async_trait::async_trait;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use beyondtranslate_core::{
    DetectLanguageRequest, DetectLanguageResponse, OcrError, OcrService, Provider,
    RecognizeTextRequest, RecognizeTextResponse, TextDetection, TextRecognition, TextTranslation,
    TranslateRequest, TranslateResponse, TranslationError, TranslationService,
};
use hmac::{Hmac, Mac};
use rand::random;
use reqwest::{Response, Url};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha1::Sha1;
use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};

type HmacSha1 = Hmac<Sha1>;

#[derive(Debug, Clone, Deserialize, PartialEq, Serialize)]
pub struct TencentCloudProviderConfig {
    #[serde(rename = "secretId", alias = "secret_id")]
    pub secret_id: String,
    #[serde(rename = "secretKey", alias = "secret_key")]
    pub secret_key: String,
    #[serde(rename = "baseUrl", alias = "base_url")]
    pub base_url: Option<String>,
    /// The OCR product answers on its own host; leave blank for the public one.
    #[serde(default, rename = "ocrBaseUrl", alias = "ocr_base_url")]
    pub ocr_base_url: Option<String>,
}

pub struct TencentCloudProvider {
    config: TencentCloudProviderConfig,
    translation_service: TencentCloudTranslationService,
    ocr_service: TencentCloudOcrService,
}

struct TencentCloudTranslationService {
    client: TencentCloudClient,
}

/// 通用印刷体识别 (`GeneralBasicOCR`) — same credentials and signature as
/// 机器翻译, a different product endpoint and API version.
struct TencentCloudOcrService {
    client: TencentCloudClient,
}

/// One Tencent Cloud API 3.0 product: its endpoint and version, signed the
/// v1 way (HMAC-SHA1 over the sorted form body).
struct TencentCloudClient {
    secret_id: String,
    secret_key: String,
    version: &'static str,
    fallback_host: &'static str,
    http: HttpClient,
}

impl TencentCloudProvider {
    pub fn new(config: TencentCloudProviderConfig) -> Result<Self, String> {
        if config.secret_id.trim().is_empty() {
            return Err("secret_id must not be empty".to_owned());
        }
        if config.secret_key.trim().is_empty() {
            return Err("secret_key must not be empty".to_owned());
        }
        let client = reqwest::Client::default();
        let product = |base_url: Option<String>, host: &'static str, version: &'static str| {
            TencentCloudClient {
                secret_id: config.secret_id.clone(),
                secret_key: config.secret_key.clone(),
                version,
                fallback_host: host,
                http: HttpClient::new(
                    base_url.unwrap_or_else(|| format!("https://{host}")),
                    client.clone(),
                ),
            }
        };
        Ok(Self {
            translation_service: TencentCloudTranslationService {
                client: product(
                    config.base_url.clone(),
                    "tmt.tencentcloudapi.com",
                    "2018-03-21",
                ),
            },
            ocr_service: TencentCloudOcrService {
                client: product(
                    config.ocr_base_url.clone(),
                    "ocr.tencentcloudapi.com",
                    "2018-11-19",
                ),
            },
            config,
        })
    }
}

#[async_trait(?Send)]
impl TranslationService for TencentCloudTranslationService {
    async fn detect_language(
        &self,
        request: DetectLanguageRequest,
    ) -> Result<DetectLanguageResponse, TranslationError> {
        let text = request
            .texts
            .into_iter()
            .next()
            .ok_or_else(|| TranslationError::InvalidRequest("texts is required".to_owned()))?;

        let mut body = self.client.base_params("LanguageDetect");
        body.insert("Text".to_owned(), text.clone());
        let response = self.execute_signed(body).await?;
        let lang = response["Response"]["Lang"].as_str().ok_or_else(|| {
            TranslationError::SerializationError("missing Response.Lang".to_owned())
        })?;

        Ok(DetectLanguageResponse {
            detections: Some(vec![TextDetection {
                detected_language: Some(lang.to_owned()),
                // TencentCloud answers with one language and no ranking.
                candidates: Vec::new(),
                text,
            }]),
        })
    }

    async fn translate(
        &self,
        request: TranslateRequest,
    ) -> Result<TranslateResponse, TranslationError> {
        let mut body = self.client.base_params("TextTranslate");
        body.insert(
            "Source".to_owned(),
            request.source_language.unwrap_or_else(|| "auto".to_owned()),
        );
        body.insert("SourceText".to_owned(), request.text);
        body.insert(
            "Target".to_owned(),
            request.target_language.ok_or_else(|| {
                TranslationError::InvalidRequest("target_language is required".to_owned())
            })?,
        );

        let response = self.execute_signed(body).await?;
        let text = response["Response"]["TargetText"].as_str().ok_or_else(|| {
            TranslationError::SerializationError("missing Response.TargetText".to_owned())
        })?;

        Ok(TranslateResponse {
            translations: vec![TextTranslation {
                detected_source_language: None,
                text: text.to_owned(),
                audio_url: None,
            }],
        })
    }
}

impl TencentCloudTranslationService {
    async fn execute_signed(
        &self,
        body: BTreeMap<String, String>,
    ) -> Result<Value, TranslationError> {
        let response = self
            .client
            .send_signed(body)
            .await
            .map_err(|error| match error {
                SendError::Config(message) => TranslationError::ConfigError(message),
                SendError::Network(error) => TranslationError::from_network_error(error),
            })?;
        let response = TranslationError::from_response("tencent", response).await?;
        let data: Value = response
            .json()
            .await
            .map_err(|error| TranslationError::SerializationError(error.to_string()))?;
        if let Some(message) = api_error(&data) {
            return Err(TranslationError::NetworkError(format!(
                "tencent: {message}"
            )));
        }
        Ok(data)
    }
}

#[async_trait(?Send)]
impl OcrService for TencentCloudOcrService {
    async fn recognize_text(
        &self,
        request: RecognizeTextRequest,
    ) -> Result<RecognizeTextResponse, OcrError> {
        let image = load_image(&request)?;
        let mut body = self.client.base_params("GeneralBasicOCR");
        body.insert("ImageBase64".to_owned(), image.base64);

        let response = self
            .client
            .send_signed(body)
            .await
            .map_err(|error| match error {
                SendError::Config(message) => OcrError::ConfigError(message),
                SendError::Network(error) => OcrError::from_network_error(error),
            })?;
        let response = OcrError::from_response("tencent", response).await?;
        let data: Value = response
            .json()
            .await
            .map_err(|error| OcrError::SerializationError(error.to_string()))?;
        parse_ocr_response(&data)
    }
}

/// `GeneralBasicOCR` answers one `TextDetection` per line, each with the
/// axis-aligned box it sits in.
fn parse_ocr_response(data: &Value) -> Result<RecognizeTextResponse, OcrError> {
    if let Some(message) = api_error(data) {
        return Err(OcrError::NetworkError(format!("tencent: {message}")));
    }
    let recognitions = data["Response"]["TextDetections"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|line| {
            let text = line["DetectedText"].as_str()?.to_owned();
            if text.is_empty() {
                return None;
            }
            let polygon = &line["ItemPolygon"];
            let recognized_rect = match (
                polygon["X"].as_f64(),
                polygon["Y"].as_f64(),
                polygon["Width"].as_f64(),
                polygon["Height"].as_f64(),
            ) {
                (Some(x), Some(y), Some(width), Some(height)) => {
                    Some(rect_from_bounds(x, y, width, height))
                }
                _ => None,
            };
            Some(TextRecognition {
                text,
                recognized_rect,
            })
        })
        .collect::<Vec<_>>();
    let text = recognitions
        .iter()
        .map(|recognition| recognition.text.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    Ok(RecognizeTextResponse {
        text,
        recognitions: Some(recognitions),
    })
}

/// `Response.Error`, as `code: message`, when the API declined the call.
fn api_error(data: &Value) -> Option<String> {
    let error = data["Response"]["Error"].as_object()?;
    let code = error
        .get("Code")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let message = error
        .get("Message")
        .and_then(Value::as_str)
        .unwrap_or("unknown error");
    Some(format!("{code}: {message}"))
}

impl Provider for TencentCloudProvider {
    fn name(&self) -> &'static str {
        "tencent_cloud"
    }

    fn translation(&self) -> Option<&dyn TranslationService> {
        Some(&self.translation_service)
    }

    fn ocr(&self) -> Option<&dyn OcrService> {
        Some(&self.ocr_service)
    }
}

enum SendError {
    Config(String),
    Network(reqwest::Error),
}

impl TencentCloudClient {
    fn base_params(&self, action: &str) -> BTreeMap<String, String> {
        let mut body = BTreeMap::new();
        body.insert("Action".to_owned(), action.to_owned());
        body.insert("Language".to_owned(), "zh-CN".to_owned());
        body.insert("Nonce".to_owned(), (random::<u32>() % 9999).to_string());
        body.insert("ProjectId".to_owned(), "0".to_owned());
        body.insert("Region".to_owned(), "ap-guangzhou".to_owned());
        body.insert("SecretId".to_owned(), self.secret_id.clone());
        body.insert("Timestamp".to_owned(), current_timestamp().to_string());
        body.insert("Version".to_owned(), self.version.to_owned());
        body
    }

    async fn send_signed(&self, mut body: BTreeMap<String, String>) -> Result<Response, SendError> {
        let endpoint = Url::parse(self.http.base_url())
            .ok()
            .and_then(|url| url.host_str().map(ToOwned::to_owned))
            .unwrap_or_else(|| self.fallback_host.to_owned());
        let query = body
            .iter()
            .map(|(key, value)| format!("{key}={value}"))
            .collect::<Vec<_>>()
            .join("&");
        let src = format!("POST{endpoint}/?{query}");

        let mut mac = HmacSha1::new_from_slice(self.secret_key.as_bytes())
            .map_err(|error| SendError::Config(error.to_string()))?;
        mac.update(src.as_bytes());
        let signature = STANDARD.encode(mac.finalize().into_bytes());
        body.insert("Signature".to_owned(), signature);

        let response = self
            .http
            .client()
            .post(self.http.base_url())
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Accept", "application/json")
            .form(&body);
        self.http
            .execute(response)
            .await
            .map_err(SendError::Network)
    }
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parses_lines_with_their_boxes() {
        let data = json!({ "Response": { "TextDetections": [
            { "DetectedText": "第一行", "ItemPolygon": { "X": 5, "Y": 10, "Width": 100, "Height": 20 } },
            { "DetectedText": "", "ItemPolygon": { "X": 0, "Y": 0, "Width": 0, "Height": 0 } },
            { "DetectedText": "second", "ItemPolygon": {} }
        ], "RequestId": "x" } });
        let parsed = parse_ocr_response(&data).expect("valid response");
        assert_eq!(parsed.text, "第一行\nsecond");
        let recognitions = parsed.recognitions.expect("lines");
        assert_eq!(recognitions.len(), 2);
        let rect = recognitions[0].recognized_rect.as_ref().expect("box");
        assert_eq!(
            (rect.x, rect.y, rect.width, rect.height),
            (5.0, 10.0, 100.0, 20.0)
        );
        assert_eq!(rect.bottom, Some(30.0));
        assert!(recognitions[1].recognized_rect.is_none());
    }

    #[test]
    fn surfaces_the_api_error() {
        let data = json!({ "Response": { "Error": {
            "Code": "FailedOperation.ImageDecodeFailed", "Message": "图片解码失败" } } });
        assert!(matches!(
            parse_ocr_response(&data),
            Err(OcrError::NetworkError(message))
                if message == "tencent: FailedOperation.ImageDecodeFailed: 图片解码失败"
        ));
    }
}
