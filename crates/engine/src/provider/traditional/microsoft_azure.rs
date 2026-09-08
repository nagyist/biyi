use crate::common::http_client::HttpClient;
use async_trait::async_trait;
use beyondtranslate_core::{
    DetectLanguageRequest, DetectLanguageResponse, Provider, TextDetection, TextTranslation,
    TranslateRequest, TranslateResponse, TranslationError, TranslationService,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, PartialEq, Serialize)]
pub struct MicrosoftAzureProviderConfig {
    #[serde(rename = "apiKey", alias = "api_key")]
    pub api_key: String,
    #[serde(rename = "region")]
    pub region: Option<String>,
    #[serde(rename = "baseUrl", alias = "base_url")]
    pub base_url: Option<String>,
}
pub struct MicrosoftAzureProvider {
    service: MicrosoftAzureTranslationService,
}
struct MicrosoftAzureTranslationService {
    api_key: String,
    region: Option<String>,
    http: HttpClient,
}
impl MicrosoftAzureProvider {
    pub fn new(c: MicrosoftAzureProviderConfig) -> Result<Self, String> {
        if c.api_key.trim().is_empty() {
            return Err("api_key must not be empty".into());
        }
        Ok(Self {
            service: MicrosoftAzureTranslationService {
                api_key: c.api_key,
                region: c.region,
                http: HttpClient::new(
                    c.base_url
                        .unwrap_or_else(|| "https://api.cognitive.microsofttranslator.com".into()),
                    Default::default(),
                ),
            },
        })
    }
}
#[derive(Deserialize)]
struct Translation {
    text: String,
    #[serde(rename = "to")]
    _to: String,
}
#[derive(Deserialize)]
struct Item {
    translations: Option<Vec<Translation>>,
    language: Option<String>,
}
#[async_trait(?Send)]
impl TranslationService for MicrosoftAzureTranslationService {
    async fn detect_language(
        &self,
        r: DetectLanguageRequest,
    ) -> Result<DetectLanguageResponse, TranslationError> {
        let text = r
            .texts
            .into_iter()
            .next()
            .ok_or_else(|| TranslationError::InvalidRequest("texts is required".into()))?;
        let req = self
            .http
            .post("/detect")
            .query(&[("api-version", "3.0")])
            .header("Ocp-Apim-Subscription-Key", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&serde_json::json!([{"Text":text}]));
        let resp = self
            .http
            .execute(req)
            .await
            .map_err(TranslationError::from_network_error)?;
        let resp = TranslationError::from_response("microsoft", resp).await?;
        let mut items: Vec<Item> = resp
            .json()
            .await
            .map_err(|e| TranslationError::SerializationError(e.to_string()))?;
        let lang = items.pop().and_then(|x| x.language);
        Ok(DetectLanguageResponse {
            detections: Some(vec![TextDetection {
                detected_language: lang,
                text,
                candidates: vec![],
            }]),
        })
    }
    async fn translate(&self, r: TranslateRequest) -> Result<TranslateResponse, TranslationError> {
        let target = r.target_language.ok_or_else(|| {
            TranslationError::InvalidRequest("target_language is required".into())
        })?;
        let mut url = self
            .http
            .post("/translate")
            .query(&[("api-version", "3.0"), ("to", target.as_str())]);
        if let Some(s) = r.source_language {
            url = url.query(&[("from", s)]);
        }
        url = url
            .header("Ocp-Apim-Subscription-Key", &self.api_key)
            .header("Content-Type", "application/json");
        if let Some(region) = &self.region {
            url = url.header("Ocp-Apim-Subscription-Region", region);
        }
        let resp = self
            .http
            .execute(url.json(&serde_json::json!([{"Text":r.text}])))
            .await
            .map_err(TranslationError::from_network_error)?;
        let resp = TranslationError::from_response("microsoft", resp).await?;
        let data: Vec<Item> = resp
            .json()
            .await
            .map_err(|e| TranslationError::SerializationError(e.to_string()))?;
        Ok(TranslateResponse {
            translations: data
                .into_iter()
                .flat_map(|x| x.translations.unwrap_or_default())
                .map(|x| TextTranslation {
                    detected_source_language: None,
                    text: x.text,
                    audio_url: None,
                })
                .collect(),
        })
    }
}
impl Provider for MicrosoftAzureProvider {
    fn name(&self) -> &'static str {
        "microsoft_azure"
    }
    fn translation(&self) -> Option<&dyn TranslationService> {
        Some(&self.service)
    }
}
