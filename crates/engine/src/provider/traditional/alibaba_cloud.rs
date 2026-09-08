use crate::common::http_client::HttpClient;
use crate::common::ocr::{coordinate, load_image, rect_from_points};
use crate::common::signing::{hex, hmac_sha256, nonce, sha256_hex, UtcTime};
use async_trait::async_trait;
use beyondtranslate_core::{
    DetectLanguageRequest, DetectLanguageResponse, OcrError, OcrService, Provider,
    RecognizeTextRequest, RecognizeTextResponse, TextDetection, TextRecognition, TextTranslation,
    TranslateRequest, TranslateResponse, TranslationError, TranslationService,
};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Deserialize, PartialEq, Serialize)]
pub struct AlibabaCloudProviderConfig {
    #[serde(rename = "accessKeyId", alias = "access_key_id")]
    pub access_key_id: String,
    #[serde(rename = "accessKeySecret", alias = "access_key_secret")]
    pub access_key_secret: String,
    #[serde(rename = "baseUrl", alias = "base_url")]
    pub base_url: Option<String>,
    /// 文字识别 OCR answers on its own host; leave blank for the public one.
    #[serde(default, rename = "ocrBaseUrl", alias = "ocr_base_url")]
    pub ocr_base_url: Option<String>,
}
pub struct AlibabaCloudProvider {
    service: AlibabaCloudTranslationService,
    ocr_service: AlibabaCloudOcrService,
}
struct AlibabaCloudTranslationService {
    access_key_id: String,
    access_key_secret: String,
    http: HttpClient,
}
/// 通用文字识别 (`RecognizeGeneral`) on the `ocr-api` product, signed the
/// ACS3-HMAC-SHA256 way with the same access key pair as translation.
struct AlibabaCloudOcrService {
    access_key_id: String,
    access_key_secret: String,
    http: HttpClient,
}
impl AlibabaCloudProvider {
    pub fn new(c: AlibabaCloudProviderConfig) -> Result<Self, String> {
        if c.access_key_id.trim().is_empty() || c.access_key_secret.trim().is_empty() {
            return Err("access key id and secret must not be empty".into());
        }
        let client = reqwest::Client::default();
        Ok(Self {
            service: AlibabaCloudTranslationService {
                access_key_id: c.access_key_id.clone(),
                access_key_secret: c.access_key_secret.clone(),
                http: HttpClient::new(
                    c.base_url
                        .unwrap_or_else(|| "https://mt.cn-hangzhou.aliyuncs.com".into()),
                    client.clone(),
                ),
            },
            ocr_service: AlibabaCloudOcrService {
                access_key_id: c.access_key_id,
                access_key_secret: c.access_key_secret,
                http: HttpClient::new(
                    c.ocr_base_url
                        .unwrap_or_else(|| "https://ocr-api.cn-hangzhou.aliyuncs.com".into()),
                    client,
                ),
            },
        })
    }
}
#[async_trait(?Send)]
impl TranslationService for AlibabaCloudTranslationService {
    async fn detect_language(
        &self,
        r: DetectLanguageRequest,
    ) -> Result<DetectLanguageResponse, TranslationError> {
        let text = r
            .texts
            .into_iter()
            .next()
            .ok_or_else(|| TranslationError::InvalidRequest("texts is required".into()))?;
        let resp = self
            .call("GetDetectLanguage", vec![("SourceText", text.clone())])
            .await?;
        let lang = resp["DetectedLanguage"].as_str().unwrap_or("").to_owned();
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
        let source = r.source_language.unwrap_or_else(|| "auto".into());
        let resp = self
            .call(
                "TranslateGeneral",
                vec![
                    ("FormatType", "text".into()),
                    ("Scene", "general".into()),
                    ("SourceLanguage", source),
                    ("TargetLanguage", target),
                    ("SourceText", r.text),
                ],
            )
            .await?;
        let text = resp["Data"]["Translated"].as_str().ok_or_else(|| {
            TranslationError::SerializationError("missing Data.Translated".into())
        })?;
        Ok(TranslateResponse {
            translations: vec![TextTranslation {
                detected_source_language: resp["Data"]["DetectedLanguage"]
                    .as_str()
                    .map(str::to_owned),
                text: text.to_owned(),
                audio_url: None,
            }],
        })
    }
}
impl AlibabaCloudTranslationService {
    async fn call(
        &self,
        action: &str,
        params: Vec<(&str, String)>,
    ) -> Result<serde_json::Value, TranslationError> {
        let mut q = vec![
            ("Action", action.to_owned()),
            ("Version", "2018-10-12".into()),
            ("Format", "JSON".into()),
            ("AccessKeyId", self.access_key_id.clone()),
        ];
        q.extend(params);
        let req = self.http.post("/").query(&q);
        let resp = self
            .http
            .execute(req)
            .await
            .map_err(TranslationError::from_network_error)?;
        let resp = TranslationError::from_response("aliyun", resp).await?;
        resp.json()
            .await
            .map_err(|e| TranslationError::SerializationError(e.to_string()))
    }
}

#[async_trait(?Send)]
impl OcrService for AlibabaCloudOcrService {
    async fn recognize_text(
        &self,
        request: RecognizeTextRequest,
    ) -> Result<RecognizeTextResponse, OcrError> {
        let image = load_image(&request)?;
        let host = Url::parse(self.http.base_url())
            .ok()
            .and_then(|url| url.host_str().map(ToOwned::to_owned))
            .unwrap_or_else(|| "ocr-api.cn-hangzhou.aliyuncs.com".to_owned());
        let signed = Acs3Signer {
            access_key_id: &self.access_key_id,
            access_key_secret: &self.access_key_secret,
        }
        .sign(
            &host,
            "RecognizeGeneral",
            "2021-07-07",
            &image.bytes,
            UtcTime::now(),
            nonce(),
        );

        let mut req = self
            .http
            .post("/")
            .header("Content-Type", "application/octet-stream")
            .header("Host", host)
            .header("Authorization", signed.authorization)
            .body(image.bytes);
        for (name, value) in signed.headers {
            req = req.header(name, value);
        }
        let resp = self
            .http
            .execute(req)
            .await
            .map_err(OcrError::from_network_error)?;
        let resp = OcrError::from_response("aliyun", resp).await?;
        let data: Value = resp
            .json()
            .await
            .map_err(|e| OcrError::SerializationError(e.to_string()))?;
        parse_ocr_response(&data)
    }
}

/// `RecognizeGeneral` wraps its result in `Data` as a JSON *string*: the
/// full `content`, and one `prism_wordsInfo` entry per line with the four
/// corners it was read from.
fn parse_ocr_response(data: &Value) -> Result<RecognizeTextResponse, OcrError> {
    if let Some(code) = data["Code"].as_str() {
        let message = data["Message"].as_str().unwrap_or("unknown error");
        return Err(OcrError::NetworkError(format!("aliyun: {code}: {message}")));
    }
    let payload: Value = match &data["Data"] {
        Value::String(text) => serde_json::from_str(text)
            .map_err(|e| OcrError::SerializationError(format!("Data is not JSON: {e}")))?,
        other => other.clone(),
    };
    let recognitions = payload["prism_wordsInfo"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|line| {
            let text = line["word"].as_str()?.to_owned();
            if text.is_empty() {
                return None;
            }
            let points = line["pos"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|point| Some((coordinate(&point["x"])?, coordinate(&point["y"])?)));
            Some(TextRecognition {
                text,
                recognized_rect: rect_from_points(points),
            })
        })
        .collect::<Vec<_>>();
    let text = payload["content"]
        .as_str()
        .map(|content| content.trim_end().to_owned())
        .filter(|content| !content.is_empty())
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

/// Alibaba Cloud's V3 signature: `ACS3-HMAC-SHA256` over a canonical request
/// whose action and version travel as `x-acs-*` headers.
struct Acs3Signer<'a> {
    access_key_id: &'a str,
    access_key_secret: &'a str,
}

struct Acs3Signature {
    authorization: String,
    headers: Vec<(&'static str, String)>,
}

impl Acs3Signer<'_> {
    fn sign(
        &self,
        host: &str,
        action: &str,
        version: &str,
        body: &[u8],
        time: UtcTime,
        nonce: String,
    ) -> Acs3Signature {
        let content_sha256 = sha256_hex(body);
        let headers: Vec<(&'static str, String)> = vec![
            ("x-acs-action", action.to_owned()),
            ("x-acs-content-sha256", content_sha256.clone()),
            ("x-acs-date", time.iso8601()),
            ("x-acs-signature-nonce", nonce),
            ("x-acs-version", version.to_owned()),
        ];
        let mut canonical: Vec<(&str, &str)> = headers
            .iter()
            .map(|(name, value)| (*name, value.as_str()))
            .collect();
        canonical.push(("host", host));
        canonical.sort();
        let canonical_headers = canonical
            .iter()
            .map(|(name, value)| format!("{name}:{}\n", value.trim()))
            .collect::<String>();
        let signed_headers = canonical
            .iter()
            .map(|(name, _)| *name)
            .collect::<Vec<_>>()
            .join(";");
        let canonical_request =
            format!("POST\n/\n\n{canonical_headers}\n{signed_headers}\n{content_sha256}");
        let string_to_sign = format!(
            "ACS3-HMAC-SHA256\n{}",
            sha256_hex(canonical_request.as_bytes())
        );
        let signature = hex(&hmac_sha256(
            self.access_key_secret.as_bytes(),
            string_to_sign.as_bytes(),
        ));
        Acs3Signature {
            authorization: format!(
                "ACS3-HMAC-SHA256 Credential={},SignedHeaders={signed_headers},Signature={signature}",
                self.access_key_id
            ),
            headers,
        }
    }
}

impl Provider for AlibabaCloudProvider {
    fn name(&self) -> &'static str {
        "alibaba_cloud"
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
    use serde_json::json;

    #[test]
    fn parses_the_stringified_data_payload() {
        let inner = json!({
            "content": "你好 world",
            "height": 100, "width": 200,
            "prism_wordsInfo": [
                { "word": "你好", "prob": 99, "pos": [
                    { "x": 10, "y": 10 }, { "x": 50, "y": 10 }, { "x": 50, "y": 30 }, { "x": 10, "y": 30 }
                ] },
                { "word": "world", "pos": [
                    { "x": "60", "y": "10" }, { "x": "120", "y": "10" }, { "x": "120", "y": "30" }, { "x": "60", "y": "30" }
                ] }
            ]
        });
        let data = json!({ "RequestId": "r", "Data": inner.to_string() });
        let parsed = parse_ocr_response(&data).expect("valid response");
        assert_eq!(parsed.text, "你好 world");
        let recognitions = parsed.recognitions.expect("lines");
        assert_eq!(recognitions.len(), 2);
        let rect = recognitions[1].recognized_rect.as_ref().expect("box");
        assert_eq!(
            (rect.x, rect.y, rect.width, rect.height),
            (60.0, 10.0, 60.0, 20.0)
        );
    }

    #[test]
    fn surfaces_the_api_error() {
        let data = json!({ "Code": "invalidImage.content", "Message": "image is broken", "RequestId": "r" });
        assert!(matches!(
            parse_ocr_response(&data),
            Err(OcrError::NetworkError(message)) if message == "aliyun: invalidImage.content: image is broken"
        ));
    }

    #[test]
    fn signature_headers_are_sorted_and_scoped() {
        let signed = Acs3Signer {
            access_key_id: "AK",
            access_key_secret: "SK",
        }
        .sign(
            "ocr-api.cn-hangzhou.aliyuncs.com",
            "RecognizeGeneral",
            "2021-07-07",
            b"img",
            UtcTime::from_unix(1_700_000_000),
            "nonce".to_owned(),
        );
        assert!(signed.authorization.starts_with(
            "ACS3-HMAC-SHA256 Credential=AK,SignedHeaders=host;x-acs-action;x-acs-content-sha256;x-acs-date;x-acs-signature-nonce;x-acs-version,Signature="
        ));
        assert!(signed
            .headers
            .contains(&("x-acs-date", "2023-11-14T22:13:20Z".to_owned())));
        assert!(signed
            .headers
            .contains(&("x-acs-content-sha256", sha256_hex(b"img"))));
    }
}
