use crate::common::http_client::HttpClient;
use crate::common::ocr::{coordinate, load_image, rect_from_points, ImageFormat};
use async_trait::async_trait;
use beyondtranslate_core::{
    DetectLanguageRequest, DetectLanguageResponse, OcrError, OcrService, Provider,
    RecognizeTextRequest, RecognizeTextResponse, TextDetection, TextRecognition, TextTranslation,
    TranslateRequest, TranslateResponse, TranslationError, TranslationService,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Clone, Deserialize, PartialEq, Serialize)]
pub struct YandexCloudProviderConfig {
    #[serde(rename = "apiKey", alias = "api_key")]
    pub api_key: String,
    #[serde(rename = "folderId", alias = "folder_id")]
    pub folder_id: Option<String>,
    #[serde(rename = "baseUrl", alias = "base_url")]
    pub base_url: Option<String>,
    /// Vision OCR answers on its own host; leave blank for the public one.
    #[serde(default, rename = "ocrBaseUrl", alias = "ocr_base_url")]
    pub ocr_base_url: Option<String>,
}
pub struct YandexCloudProvider {
    service: YandexCloudTranslationService,
    ocr_service: YandexCloudOcrService,
}
struct YandexCloudTranslationService {
    api_key: String,
    folder_id: Option<String>,
    http: HttpClient,
}
/// Vision OCR's `recognizeText`, keyed by the same API key and folder as
/// translation.
struct YandexCloudOcrService {
    api_key: String,
    folder_id: Option<String>,
    http: HttpClient,
}
impl YandexCloudProvider {
    pub fn new(c: YandexCloudProviderConfig) -> Result<Self, String> {
        if c.api_key.trim().is_empty() {
            return Err("api_key must not be empty".into());
        }
        let client = reqwest::Client::default();
        Ok(Self {
            service: YandexCloudTranslationService {
                api_key: c.api_key.clone(),
                folder_id: c.folder_id.clone(),
                http: HttpClient::new(
                    c.base_url
                        .unwrap_or_else(|| "https://translate.api.cloud.yandex.net".into()),
                    client.clone(),
                ),
            },
            ocr_service: YandexCloudOcrService {
                api_key: c.api_key,
                folder_id: c.folder_id,
                http: HttpClient::new(
                    c.ocr_base_url
                        .unwrap_or_else(|| "https://ocr.api.cloud.yandex.net".into()),
                    client,
                ),
            },
        })
    }
}
#[derive(Deserialize)]
struct Response {
    translations: Option<Vec<YandexTranslation>>,
}
#[derive(Deserialize)]
struct YandexTranslation {
    text: String,
    #[serde(rename = "detectedLanguageCode")]
    detected_language_code: Option<String>,
}
#[async_trait(?Send)]
impl TranslationService for YandexCloudTranslationService {
    async fn detect_language(
        &self,
        r: DetectLanguageRequest,
    ) -> Result<DetectLanguageResponse, TranslationError> {
        let text = r
            .texts
            .into_iter()
            .next()
            .ok_or_else(|| TranslationError::InvalidRequest("texts is required".into()))?;
        let mut body = json!({"text":text});
        if let Some(f) = &self.folder_id {
            body["folderId"] = json!(f);
        }
        let resp = self
            .http
            .execute(
                self.http
                    .post("/translate/v2/detect")
                    .header("Authorization", format!("Api-Key {}", self.api_key))
                    .json(&body),
            )
            .await
            .map_err(TranslationError::from_network_error)?;
        let resp = TranslationError::from_response("yandex", resp).await?;
        let data: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| TranslationError::SerializationError(e.to_string()))?;
        let lang = data["languageCode"]
            .as_str()
            .or_else(|| data["language"].as_str())
            .unwrap_or("")
            .to_owned();
        Ok(DetectLanguageResponse {
            detections: Some(vec![TextDetection {
                detected_language: (!lang.is_empty()).then_some(lang),
                text,
                candidates: vec![],
            }]),
        })
    }
    async fn translate(&self, r: TranslateRequest) -> Result<TranslateResponse, TranslationError> {
        let target = r.target_language.ok_or_else(|| {
            TranslationError::InvalidRequest("target_language is required".into())
        })?;
        let mut body = json!({"targetLanguageCode":target,"texts":[r.text],"format":"PLAIN_TEXT"});
        if let Some(s) = r.source_language {
            body["sourceLanguageCode"] = json!(s);
        }
        if let Some(f) = &self.folder_id {
            body["folderId"] = json!(f);
        }
        let req = self
            .http
            .post("/translate/v2/translate")
            .header("Authorization", format!("Api-Key {}", self.api_key))
            .json(&body);
        let resp = self
            .http
            .execute(req)
            .await
            .map_err(TranslationError::from_network_error)?;
        let resp = TranslationError::from_response("yandex", resp).await?;
        let data: Response = resp
            .json()
            .await
            .map_err(|e| TranslationError::SerializationError(e.to_string()))?;
        Ok(TranslateResponse {
            translations: data
                .translations
                .unwrap_or_default()
                .into_iter()
                .map(|x| TextTranslation {
                    detected_source_language: x.detected_language_code,
                    text: x.text,
                    audio_url: None,
                })
                .collect(),
        })
    }
}

#[async_trait(?Send)]
impl OcrService for YandexCloudOcrService {
    async fn recognize_text(
        &self,
        request: RecognizeTextRequest,
    ) -> Result<RecognizeTextResponse, OcrError> {
        let image = load_image(&request)?;
        // Vision wants to be told the container; it only takes these three
        // for images, so anything else is sent as the most forgiving one.
        let mime_type = match image.mime_type() {
            ImageFormat::Png => "PNG",
            ImageFormat::Webp => "WEBP",
            _ => "JPEG",
        };
        let body = json!({
            "mimeType": mime_type,
            "languageCodes": ["*"],
            "model": "page",
            "content": image.base64,
        });
        let mut req = self
            .http
            .post("/ocr/v1/recognizeText")
            .header("Authorization", format!("Api-Key {}", self.api_key))
            .json(&body);
        if let Some(folder_id) = &self.folder_id {
            req = req.header("x-folder-id", folder_id);
        }
        let resp = self
            .http
            .execute(req)
            .await
            .map_err(OcrError::from_network_error)?;
        let resp = OcrError::from_response("yandex", resp).await?;
        let data: Value = resp
            .json()
            .await
            .map_err(|e| OcrError::SerializationError(e.to_string()))?;
        parse_ocr_response(&data)
    }
}

/// `recognizeText` answers a `textAnnotation`: blocks of lines, each line
/// with its text and four corners, plus the page's `fullText`.
fn parse_ocr_response(data: &Value) -> Result<RecognizeTextResponse, OcrError> {
    if let Some(message) = data["error"]["message"].as_str().or_else(|| {
        data["message"]
            .as_str()
            .filter(|_| data["code"].is_number())
    }) {
        return Err(OcrError::NetworkError(format!("yandex: {message}")));
    }
    let annotation = &data["result"]["textAnnotation"];
    let recognitions = annotation["blocks"]
        .as_array()
        .into_iter()
        .flatten()
        .flat_map(|block| block["lines"].as_array().into_iter().flatten())
        .filter_map(|line| {
            let text = line["text"].as_str()?.to_owned();
            if text.is_empty() {
                return None;
            }
            let points = line["boundingBox"]["vertices"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|vertex| Some((coordinate(&vertex["x"])?, coordinate(&vertex["y"])?)));
            Some(TextRecognition {
                text,
                recognized_rect: rect_from_points(points),
            })
        })
        .collect::<Vec<_>>();
    let text = annotation["fullText"]
        .as_str()
        .map(|text| text.trim_end().to_owned())
        .filter(|text| !text.is_empty())
        .unwrap_or_else(|| {
            recognitions
                .iter()
                .map(|recognition| recognition.text.as_str())
                .collect::<Vec<_>>()
                .join("\n")
        });
    Ok(RecognizeTextResponse {
        text,
        recognitions: Some(recognitions),
    })
}

impl Provider for YandexCloudProvider {
    fn name(&self) -> &'static str {
        "yandex_cloud"
    }
    fn translation(&self) -> Option<&dyn TranslationService> {
        Some(&self.service)
    }
    fn ocr(&self) -> Option<&dyn OcrService> {
        Some(&self.ocr_service)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_lines_with_string_coordinates() {
        let data = json!({ "result": { "textAnnotation": {
            "width": "300", "height": "100",
            "blocks": [{ "lines": [
                { "boundingBox": { "vertices": [
                    { "x": "10", "y": "5" }, { "x": "90", "y": "5" }, { "x": "90", "y": "25" }, { "x": "10", "y": "25" }
                ] }, "text": "Привет", "words": [] },
                { "boundingBox": { "vertices": [] }, "text": "" }
            ] }],
            "fullText": "Привет\n"
        }, "page": "0" } });
        let parsed = parse_ocr_response(&data).expect("valid response");
        assert_eq!(parsed.text, "Привет");
        let recognitions = parsed.recognitions.expect("lines");
        assert_eq!(recognitions.len(), 1);
        let rect = recognitions[0].recognized_rect.as_ref().expect("box");
        assert_eq!(
            (rect.x, rect.y, rect.width, rect.height),
            (10.0, 5.0, 80.0, 20.0)
        );
    }

    #[test]
    fn surfaces_grpc_style_errors() {
        let data = json!({ "code": 16, "message": "The token is invalid", "details": [] });
        assert!(matches!(
            parse_ocr_response(&data),
            Err(OcrError::NetworkError(message)) if message == "yandex: The token is invalid"
        ));
    }
}
