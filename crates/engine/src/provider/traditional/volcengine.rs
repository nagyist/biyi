use crate::common::http_client::HttpClient;
use crate::common::ocr::{load_image, rect_from_bounds};
use crate::common::signing::{hex, hmac_sha256, percent_encode, sha256_hex, UtcTime};
use async_trait::async_trait;
use beyondtranslate_core::{
    OcrError, OcrService, Provider, RecognizeTextRequest, RecognizeTextResponse, TextRecognition,
    TextTranslation, TranslateRequest, TranslateResponse, TranslationError, TranslationService,
};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Deserialize, PartialEq, Serialize)]
pub struct VolcengineProviderConfig {
    #[serde(rename = "accessKey", alias = "accessKeyId", alias = "access_key")]
    pub access_key: String,
    #[serde(rename = "secretKey", alias = "secretAccessKey", alias = "secret_key")]
    pub secret_key: String,
    #[serde(rename = "baseUrl", alias = "base_url")]
    pub base_url: Option<String>,
    /// 视觉智能 answers on its own host; leave blank for the public one.
    #[serde(default, rename = "ocrBaseUrl", alias = "ocr_base_url")]
    pub ocr_base_url: Option<String>,
}
pub struct VolcengineProvider {
    service: VolcengineTranslationService,
    ocr_service: VolcengineOcrService,
}
struct VolcengineTranslationService {
    access_key: String,
    secret_key: String,
    http: HttpClient,
}
/// 通用文字识别 (`OCRNormal`) on the `cv` service, signed with the same
/// access key pair as translation.
struct VolcengineOcrService {
    access_key: String,
    secret_key: String,
    http: HttpClient,
}
impl VolcengineProvider {
    pub fn new(c: VolcengineProviderConfig) -> Result<Self, String> {
        if c.access_key.trim().is_empty() || c.secret_key.trim().is_empty() {
            return Err("access key and secret key must not be empty".into());
        }
        let client = reqwest::Client::default();
        Ok(Self {
            service: VolcengineTranslationService {
                access_key: c.access_key.clone(),
                secret_key: c.secret_key.clone(),
                http: HttpClient::new(
                    c.base_url
                        .unwrap_or_else(|| "https://translate.volcengineapi.com".into()),
                    client.clone(),
                ),
            },
            ocr_service: VolcengineOcrService {
                access_key: c.access_key,
                secret_key: c.secret_key,
                http: HttpClient::new(
                    c.ocr_base_url
                        .unwrap_or_else(|| "https://visual.volcengineapi.com".into()),
                    client,
                ),
            },
        })
    }
}
#[derive(Deserialize)]
struct Output {
    #[serde(rename = "TranslationList")]
    list: Option<Vec<Entry>>,
}
#[derive(Deserialize)]
struct Entry {
    #[serde(rename = "Translation")]
    text: Option<String>,
    #[serde(rename = "DetectedLanguage")]
    detected: Option<String>,
}
#[async_trait(?Send)]
impl TranslationService for VolcengineTranslationService {
    async fn translate(&self, r: TranslateRequest) -> Result<TranslateResponse, TranslationError> {
        let target = r.target_language.ok_or_else(|| {
            TranslationError::InvalidRequest("target_language is required".into())
        })?;
        let body = serde_json::json!({"SourceLanguage":r.source_language.unwrap_or_else(||"auto".into()),"TargetLanguage":target,"TextList":[r.text]});
        let req = self
            .http
            .post("/")
            .header("Content-Type", "application/json")
            .header("X-Access-Key", &self.access_key)
            .header("X-Secret-Key", &self.secret_key)
            .json(&body);
        let resp = self
            .http
            .execute(req)
            .await
            .map_err(TranslationError::from_network_error)?;
        let resp = TranslationError::from_response("volcengine", resp).await?;
        let data: Output = resp
            .json()
            .await
            .map_err(|e| TranslationError::SerializationError(e.to_string()))?;
        Ok(TranslateResponse {
            translations: data
                .list
                .unwrap_or_default()
                .into_iter()
                .filter_map(|x| {
                    x.text.map(|text| TextTranslation {
                        detected_source_language: x.detected,
                        audio_url: None,
                        text,
                    })
                })
                .collect(),
        })
    }
}

#[async_trait(?Send)]
impl OcrService for VolcengineOcrService {
    async fn recognize_text(
        &self,
        request: RecognizeTextRequest,
    ) -> Result<RecognizeTextResponse, OcrError> {
        let image = load_image(&request)?;
        let host = Url::parse(self.http.base_url())
            .ok()
            .and_then(|url| url.host_str().map(ToOwned::to_owned))
            .unwrap_or_else(|| "visual.volcengineapi.com".to_owned());
        let query = [("Action", "OCRNormal"), ("Version", "2020-08-26")];
        let body = format!("image_base64={}", percent_encode(&image.base64));
        let signed = SignedRequest::new(&self.access_key, &self.secret_key).sign(
            &host,
            "cv",
            "cn-north-1",
            &query,
            FORM_CONTENT_TYPE,
            body.as_bytes(),
            UtcTime::now(),
        );

        let req = self
            .http
            .post("/")
            .query(&query)
            .header("Content-Type", FORM_CONTENT_TYPE)
            .header("Host", host)
            .header("X-Date", signed.date)
            .header("X-Content-Sha256", signed.content_sha256)
            .header("Authorization", signed.authorization)
            .body(body);
        let resp = self
            .http
            .execute(req)
            .await
            .map_err(OcrError::from_network_error)?;
        let resp = OcrError::from_response("volcengine", resp).await?;
        let data: Value = resp
            .json()
            .await
            .map_err(|e| OcrError::SerializationError(e.to_string()))?;
        parse_ocr_response(&data)
    }
}

const FORM_CONTENT_TYPE: &str = "application/x-www-form-urlencoded";

/// `OCRNormal` answers parallel arrays: one line of text per entry, with
/// the box it sits in at the same index.
fn parse_ocr_response(data: &Value) -> Result<RecognizeTextResponse, OcrError> {
    if let Some(error) = data["ResponseMetadata"]["Error"].as_object() {
        let code = error
            .get("Code")
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        let message = error
            .get("Message")
            .and_then(Value::as_str)
            .unwrap_or("unknown error");
        return Err(OcrError::NetworkError(format!(
            "volcengine: {code}: {message}"
        )));
    }
    if let Some(code) = data["code"].as_i64().filter(|code| *code != 10000) {
        let message = data["message"].as_str().unwrap_or("unknown error");
        return Err(OcrError::NetworkError(format!(
            "volcengine: {code}: {message}"
        )));
    }

    let payload = &data["data"];
    let rects = payload["line_rects"].as_array();
    let recognitions = payload["line_texts"]
        .as_array()
        .into_iter()
        .flatten()
        .enumerate()
        .filter_map(|(index, line)| {
            let text = line.as_str()?.to_owned();
            if text.is_empty() {
                return None;
            }
            let recognized_rect =
                rects.and_then(|rects| rects.get(index)).and_then(|rect| {
                    match (
                        rect["x"].as_f64(),
                        rect["y"].as_f64(),
                        rect["width"].as_f64(),
                        rect["height"].as_f64(),
                    ) {
                        (Some(x), Some(y), Some(width), Some(height)) => {
                            Some(rect_from_bounds(x, y, width, height))
                        }
                        _ => None,
                    }
                });
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

/// Volcengine's HMAC-SHA256 request signature — the SigV4 shape: a canonical
/// request hashed into a string to sign, a key derived date → region →
/// service → `request`.
struct SignedRequest<'a> {
    access_key: &'a str,
    secret_key: &'a str,
}

struct Signature {
    date: String,
    content_sha256: String,
    authorization: String,
}

impl<'a> SignedRequest<'a> {
    fn new(access_key: &'a str, secret_key: &'a str) -> Self {
        Self {
            access_key,
            secret_key,
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn sign(
        &self,
        host: &str,
        service: &str,
        region: &str,
        query: &[(&str, &str)],
        content_type: &str,
        body: &[u8],
        time: UtcTime,
    ) -> Signature {
        let date = time.compact();
        let short_date = time.date();
        let content_sha256 = sha256_hex(body);

        let mut sorted_query = query.to_vec();
        sorted_query.sort();
        let canonical_query = sorted_query
            .iter()
            .map(|(key, value)| format!("{}={}", percent_encode(key), percent_encode(value)))
            .collect::<Vec<_>>()
            .join("&");
        let headers = [
            ("content-type", content_type),
            ("host", host),
            ("x-content-sha256", content_sha256.as_str()),
            ("x-date", date.as_str()),
        ];
        let canonical_headers = headers
            .iter()
            .map(|(name, value)| format!("{name}:{}\n", value.trim()))
            .collect::<String>();
        let signed_headers = headers
            .iter()
            .map(|(name, _)| *name)
            .collect::<Vec<_>>()
            .join(";");
        let canonical_request = format!(
            "POST\n/\n{canonical_query}\n{canonical_headers}\n{signed_headers}\n{content_sha256}"
        );

        let scope = format!("{short_date}/{region}/{service}/request");
        let string_to_sign = format!(
            "HMAC-SHA256\n{date}\n{scope}\n{}",
            sha256_hex(canonical_request.as_bytes())
        );
        let k_date = hmac_sha256(self.secret_key.as_bytes(), short_date.as_bytes());
        let k_region = hmac_sha256(&k_date, region.as_bytes());
        let k_service = hmac_sha256(&k_region, service.as_bytes());
        let k_signing = hmac_sha256(&k_service, b"request");
        let signature = hex(&hmac_sha256(&k_signing, string_to_sign.as_bytes()));

        Signature {
            authorization: format!(
                "HMAC-SHA256 Credential={}/{scope}, SignedHeaders={signed_headers}, Signature={signature}",
                self.access_key
            ),
            date,
            content_sha256,
        }
    }
}

impl Provider for VolcengineProvider {
    fn name(&self) -> &'static str {
        "volcengine"
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
    fn parses_parallel_line_arrays() {
        let data = json!({
            "ResponseMetadata": { "Action": "OCRNormal" },
            "code": 10000,
            "message": "Success",
            "data": {
                "line_texts": ["你好", "", "world"],
                "line_rects": [
                    { "x": 1, "y": 2, "width": 30, "height": 10 },
                    { "x": 0, "y": 0, "width": 0, "height": 0 }
                ]
            }
        });
        let parsed = parse_ocr_response(&data).expect("valid response");
        assert_eq!(parsed.text, "你好\nworld");
        let recognitions = parsed.recognitions.expect("lines");
        assert_eq!(recognitions.len(), 2);
        let rect = recognitions[0].recognized_rect.as_ref().expect("box");
        assert_eq!(
            (rect.x, rect.y, rect.width, rect.height),
            (1.0, 2.0, 30.0, 10.0)
        );
        // The third line has no rect at its index.
        assert!(recognitions[1].recognized_rect.is_none());
    }

    #[test]
    fn surfaces_metadata_and_business_errors() {
        let data = json!({ "ResponseMetadata": { "Error": {
            "Code": "InvalidAccessKey", "Message": "no such key" } } });
        assert!(matches!(
            parse_ocr_response(&data),
            Err(OcrError::NetworkError(message)) if message == "volcengine: InvalidAccessKey: no such key"
        ));
        let data = json!({ "code": 50400, "message": "image decode failed", "data": null });
        assert!(matches!(
            parse_ocr_response(&data),
            Err(OcrError::NetworkError(message)) if message == "volcengine: 50400: image decode failed"
        ));
    }

    #[test]
    fn signature_is_deterministic_and_well_formed() {
        let signer = SignedRequest::new("AK", "SK");
        let time = UtcTime::from_unix(1_700_000_000);
        let query = [("Version", "2020-08-26"), ("Action", "OCRNormal")];
        let first = signer.sign(
            "visual.volcengineapi.com",
            "cv",
            "cn-north-1",
            &query,
            FORM_CONTENT_TYPE,
            b"image_base64=abc",
            time,
        );
        let second = signer.sign(
            "visual.volcengineapi.com",
            "cv",
            "cn-north-1",
            &query,
            FORM_CONTENT_TYPE,
            b"image_base64=abc",
            time,
        );
        assert_eq!(first.authorization, second.authorization);
        assert_eq!(first.date, "20231114T221320Z");
        assert_eq!(first.content_sha256, sha256_hex(b"image_base64=abc"));
        assert!(first.authorization.starts_with(
            "HMAC-SHA256 Credential=AK/20231114/cn-north-1/cv/request, SignedHeaders=content-type;host;x-content-sha256;x-date, Signature="
        ));
        assert_eq!(
            first.authorization.len(),
            first.authorization.find("Signature=").unwrap() + "Signature=".len() + 64
        );
    }
}
