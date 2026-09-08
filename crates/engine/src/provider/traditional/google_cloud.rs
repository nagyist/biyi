#![cfg_attr(not(feature = "google_cloud"), allow(dead_code))]

use async_trait::async_trait;
use beyondtranslate_core::{
    DetectLanguageRequest, DetectLanguageResponse, OcrError, OcrService, Provider,
    RecognizeTextRequest, RecognizeTextResponse, TextDetection, TextRecognition, TextTranslation,
    TranslateRequest, TranslateResponse, TranslationError, TranslationService,
};

use crate::common::http_client::HttpClient;
use crate::common::ocr::{load_image, rect_from_points};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Clone, Deserialize, PartialEq, Serialize)]
pub struct GoogleCloudProviderConfig {
    #[serde(rename = "appKey", alias = "apiKey", alias = "api_key")]
    pub api_key: String,
    #[serde(rename = "baseUrl", alias = "base_url")]
    pub base_url: Option<String>,
    /// Cloud Vision lives on its own host; leave blank for the public one.
    #[serde(default, rename = "ocrBaseUrl", alias = "ocr_base_url")]
    pub ocr_base_url: Option<String>,
}

pub struct GoogleCloudProvider {
    #[allow(dead_code)]
    config: GoogleCloudProviderConfig,
    translation_service: GoogleCloudTranslationService,
    ocr_service: GoogleCloudOcrService,
}

struct GoogleCloudTranslationService {
    api_key: String,
    http: HttpClient,
}

/// Cloud Vision's `images:annotate` with `TEXT_DETECTION`, keyed by the same
/// API key as translation — the project just needs the Vision API enabled.
struct GoogleCloudOcrService {
    api_key: String,
    http: HttpClient,
}

impl GoogleCloudProvider {
    pub fn new(config: GoogleCloudProviderConfig) -> Result<Self, String> {
        if config.api_key.trim().is_empty() {
            return Err("api_key must not be empty".to_owned());
        }
        let client = reqwest::Client::default();
        Ok(Self {
            config: config.clone(),
            translation_service: GoogleCloudTranslationService {
                api_key: config.api_key.clone(),
                http: HttpClient::new(
                    config
                        .base_url
                        .unwrap_or_else(|| "https://translation.googleapis.com".to_owned()),
                    client.clone(),
                ),
            },
            ocr_service: GoogleCloudOcrService {
                api_key: config.api_key,
                http: HttpClient::new(
                    config
                        .ocr_base_url
                        .unwrap_or_else(|| "https://vision.googleapis.com".to_owned()),
                    client,
                ),
            },
        })
    }
}

#[async_trait(?Send)]
impl TranslationService for GoogleCloudTranslationService {
    async fn detect_language(
        &self,
        request: DetectLanguageRequest,
    ) -> Result<DetectLanguageResponse, TranslationError> {
        let text = request
            .texts
            .into_iter()
            .next()
            .ok_or_else(|| TranslationError::InvalidRequest("texts is required".to_owned()))?;

        let response = self
            .http
            .post("/language/translate/v2/detect")
            .query(&[("key", self.api_key.as_str())])
            .json(&json!({ "q": text }));
        let response = self
            .http
            .execute(response)
            .await
            .map_err(TranslationError::from_network_error)?;
        let response = TranslationError::from_response("google", response).await?;
        let data: Value = response
            .json()
            .await
            .map_err(|error| TranslationError::SerializationError(error.to_string()))?;

        let detections = data["data"]["detections"][0]
            .as_array()
            .ok_or_else(|| {
                TranslationError::SerializationError(
                    "missing detections in Google response".to_owned(),
                )
            })?
            .iter()
            .filter_map(|item| item["language"].as_str())
            .map(|language| TextDetection {
                detected_language: Some(language.to_owned()),
                // The detect endpoint reports a confidence, but only for the
                // one language it settled on — not a ranking to choose from.
                candidates: Vec::new(),
                text: text.clone(),
            })
            .collect();

        Ok(DetectLanguageResponse {
            detections: Some(detections),
        })
    }

    async fn translate(
        &self,
        request: TranslateRequest,
    ) -> Result<TranslateResponse, TranslationError> {
        let target_language = request.target_language.ok_or_else(|| {
            TranslationError::InvalidRequest("target_language is required".to_owned())
        })?;
        let response = self
            .http
            .post("/language/translate/v2")
            .query(&[("key", self.api_key.as_str())])
            .json(&json!({
                "q": request.text,
                "source": request.source_language,
                "target": target_language,
                "format": "text",
            }));
        let response = self
            .http
            .execute(response)
            .await
            .map_err(TranslationError::from_network_error)?;
        let response = TranslationError::from_response("google", response).await?;
        let data: Value = response
            .json()
            .await
            .map_err(|error| TranslationError::SerializationError(error.to_string()))?;

        let translations = data["data"]["translations"]
            .as_array()
            .ok_or_else(|| {
                TranslationError::SerializationError(
                    "missing translations in Google response".to_owned(),
                )
            })?
            .iter()
            .filter_map(|item| item["translatedText"].as_str())
            .map(|text| TextTranslation {
                detected_source_language: None,
                text: text.to_owned(),
                audio_url: None,
            })
            .collect();

        Ok(TranslateResponse { translations })
    }
}

impl Provider for GoogleCloudProvider {
    fn name(&self) -> &'static str {
        "google_cloud"
    }

    fn translation(&self) -> Option<&dyn TranslationService> {
        Some(&self.translation_service)
    }

    fn ocr(&self) -> Option<&dyn OcrService> {
        Some(&self.ocr_service)
    }
}

#[async_trait(?Send)]
impl OcrService for GoogleCloudOcrService {
    async fn recognize_text(
        &self,
        request: RecognizeTextRequest,
    ) -> Result<RecognizeTextResponse, OcrError> {
        let image = load_image(&request)?;
        let response = self
            .http
            .post("/v1/images:annotate")
            .query(&[("key", self.api_key.as_str())])
            .json(&json!({
                "requests": [{
                    "image": { "content": image.base64 },
                    "features": [{ "type": "TEXT_DETECTION" }],
                }],
            }));
        let response = self
            .http
            .execute(response)
            .await
            .map_err(OcrError::from_network_error)?;
        let response = OcrError::from_response("google", response).await?;
        let data: Value = response
            .json()
            .await
            .map_err(|error| OcrError::SerializationError(error.to_string()))?;
        parse_vision_response(&data)
    }
}

/// Reads one `AnnotateImageResponse`: the full text as Vision assembled it,
/// plus a recognition per paragraph so the app can place the words.
fn parse_vision_response(data: &Value) -> Result<RecognizeTextResponse, OcrError> {
    let response = &data["responses"][0];
    if let Some(error) = response["error"].as_object() {
        let message = error
            .get("message")
            .and_then(Value::as_str)
            .unwrap_or("unknown error");
        return Err(OcrError::NetworkError(format!("google: {message}")));
    }

    let annotation = &response["fullTextAnnotation"];
    let recognitions = annotation["pages"]
        .as_array()
        .into_iter()
        .flatten()
        .flat_map(|page| page["blocks"].as_array().into_iter().flatten())
        .flat_map(|block| block["paragraphs"].as_array().into_iter().flatten())
        .filter_map(|paragraph| {
            let text = paragraph_text(paragraph);
            if text.is_empty() {
                return None;
            }
            Some(TextRecognition {
                text,
                recognized_rect: rect_from_points(vertices(&paragraph["boundingBox"])),
            })
        })
        .collect::<Vec<_>>();

    let text = annotation["text"]
        .as_str()
        .or_else(|| response["textAnnotations"][0]["description"].as_str())
        .map(|text| text.trim_end().to_owned())
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

/// A paragraph's words are split into symbols, each telling what kind of
/// break — a space, a line break — follows it.
fn paragraph_text(paragraph: &Value) -> String {
    let mut text = String::new();
    for word in paragraph["words"].as_array().into_iter().flatten() {
        for symbol in word["symbols"].as_array().into_iter().flatten() {
            if let Some(glyph) = symbol["text"].as_str() {
                text.push_str(glyph);
            }
            let separator = match symbol["property"]["detectedBreak"]["type"].as_str() {
                Some("SPACE") | Some("SURE_SPACE") => " ",
                Some("EOL_SURE_SPACE") | Some("LINE_BREAK") => "\n",
                Some("HYPHEN") => "-\n",
                _ => "",
            };
            text.push_str(separator);
        }
    }
    text.trim_end().to_owned()
}

/// Vision omits a vertex coordinate that is zero.
fn vertices(bounding_box: &Value) -> Vec<(f64, f64)> {
    bounding_box["vertices"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|vertex| {
            (
                vertex["x"].as_f64().unwrap_or(0.0),
                vertex["y"].as_f64().unwrap_or(0.0),
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_paragraphs_with_breaks_and_boxes() {
        let data = json!({
            "responses": [{
                "textAnnotations": [{ "description": "Hello world\nBye" }],
                "fullTextAnnotation": {
                    "text": "Hello world\nBye\n",
                    "pages": [{ "blocks": [{ "paragraphs": [{
                        "boundingBox": { "vertices": [
                            { "x": 10, "y": 20 }, { "x": 110, "y": 20 },
                            { "x": 110, "y": 40 }, { "x": 10, "y": 40 }
                        ] },
                        "words": [
                            { "symbols": [
                                { "text": "H" }, { "text": "i",
                                  "property": { "detectedBreak": { "type": "SPACE" } } }
                            ] },
                            { "symbols": [
                                { "text": "!",
                                  "property": { "detectedBreak": { "type": "LINE_BREAK" } } }
                            ] }
                        ]
                    }] }] }]
                }
            }]
        });
        let parsed = parse_vision_response(&data).expect("valid response");
        assert_eq!(parsed.text, "Hello world\nBye");
        let recognitions = parsed.recognitions.expect("paragraphs");
        assert_eq!(recognitions.len(), 1);
        assert_eq!(recognitions[0].text, "Hi !");
        let rect = recognitions[0].recognized_rect.as_ref().expect("box");
        assert_eq!(
            (rect.x, rect.y, rect.width, rect.height),
            (10.0, 20.0, 100.0, 20.0)
        );
    }

    #[test]
    fn surfaces_a_per_image_error() {
        let data =
            json!({ "responses": [{ "error": { "code": 3, "message": "Bad image data." } }] });
        assert!(matches!(
            parse_vision_response(&data),
            Err(OcrError::NetworkError(message)) if message.contains("Bad image data.")
        ));
    }

    #[test]
    fn an_empty_image_yields_empty_text() {
        let data = json!({ "responses": [{}] });
        let parsed = parse_vision_response(&data).expect("valid response");
        assert_eq!(parsed.text, "");
        assert_eq!(parsed.recognitions.map(|r| r.len()), Some(0));
    }
}
