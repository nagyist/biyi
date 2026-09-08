//! What every remote OCR adapter needs before it can talk to its API: the
//! image the request carries, in bytes and in base64, and a way to turn the
//! corners an API reports back into the rect the app draws.

use base64::engine::general_purpose::STANDARD;
use base64::Engine as _;
use beyondtranslate_core::{OcrError, RecognizeTextRequest, RecognizedRect};

/// An image ready to send: the raw bytes for APIs that take a binary body,
/// the base64 form for those that take a string.
pub struct LoadedImage {
    pub bytes: Vec<u8>,
    pub base64: String,
}

impl LoadedImage {
    pub fn mime_type(&self) -> ImageFormat {
        ImageFormat::sniff(&self.bytes)
    }
}

/// Reads the image out of a request, preferring inline base64 over a path,
/// as the system and Youdao adapters do.
pub fn load_image(request: &RecognizeTextRequest) -> Result<LoadedImage, OcrError> {
    match (&request.base64_image, &request.image_path) {
        (Some(base64), _) => {
            // A data URI sneaks in from web callers now and then; the API
            // only wants what follows the comma.
            let payload = base64
                .rsplit_once(',')
                .filter(|(head, _)| head.starts_with("data:"))
                .map(|(_, tail)| tail)
                .unwrap_or(base64)
                .trim();
            let bytes = STANDARD.decode(payload).map_err(|error| {
                OcrError::InvalidRequest(format!("base64_image is not valid base64: {error}"))
            })?;
            Ok(LoadedImage {
                bytes,
                base64: payload.to_owned(),
            })
        }
        (None, Some(path)) => {
            let bytes = std::fs::read(path).map_err(|error| {
                OcrError::InvalidRequest(format!("failed to read image file '{path}': {error}"))
            })?;
            let base64 = STANDARD.encode(&bytes);
            Ok(LoadedImage { bytes, base64 })
        }
        (None, None) => Err(OcrError::InvalidRequest(
            "either base64_image or image_path must be provided".to_owned(),
        )),
    }
}

/// The container formats the remote OCR APIs accept, told apart by magic
/// bytes — the request never says which one it is sending.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    Png,
    Jpeg,
    Webp,
    Gif,
    Bmp,
    Unknown,
}

impl ImageFormat {
    pub fn sniff(bytes: &[u8]) -> Self {
        if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
            Self::Png
        } else if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
            Self::Jpeg
        } else if bytes.len() >= 12 && &bytes[..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
            Self::Webp
        } else if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
            Self::Gif
        } else if bytes.starts_with(b"BM") {
            Self::Bmp
        } else {
            Self::Unknown
        }
    }
}

/// The axis-aligned box around a polygon — APIs report four corners, the
/// app wants x/y/width/height plus the four edges.
pub fn rect_from_points(points: impl IntoIterator<Item = (f64, f64)>) -> Option<RecognizedRect> {
    let mut bounds: Option<(f64, f64, f64, f64)> = None;
    for (x, y) in points {
        bounds = Some(match bounds {
            None => (x, y, x, y),
            Some((left, top, right, bottom)) => {
                (left.min(x), top.min(y), right.max(x), bottom.max(y))
            }
        });
    }
    let (left, top, right, bottom) = bounds?;
    Some(RecognizedRect {
        x: left,
        y: top,
        width: right - left,
        height: bottom - top,
        top: Some(top),
        right: Some(right),
        bottom: Some(bottom),
        left: Some(left),
    })
}

/// A rect already given as origin and size.
pub fn rect_from_bounds(x: f64, y: f64, width: f64, height: f64) -> RecognizedRect {
    RecognizedRect {
        x,
        y,
        width,
        height,
        top: Some(y),
        right: Some(x + width),
        bottom: Some(y + height),
        left: Some(x),
    }
}

/// Reads a coordinate that an API may serialize as a number or, as protobuf
/// JSON does for int64, as a string.
pub fn coordinate(value: &serde_json::Value) -> Option<f64> {
    value
        .as_f64()
        .or_else(|| value.as_str().and_then(|text| text.parse().ok()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_image_strips_a_data_uri_prefix() {
        let request = RecognizeTextRequest {
            image_path: None,
            base64_image: Some("data:image/png;base64,aGVsbG8=".to_owned()),
        };
        let image = load_image(&request).expect("valid image");
        assert_eq!(image.bytes, b"hello");
        assert_eq!(image.base64, "aGVsbG8=");
    }

    #[test]
    fn load_image_rejects_an_empty_request() {
        let request = RecognizeTextRequest {
            image_path: None,
            base64_image: None,
        };
        assert!(matches!(
            load_image(&request),
            Err(OcrError::InvalidRequest(_))
        ));
    }

    #[test]
    fn sniffs_common_formats() {
        assert_eq!(
            ImageFormat::sniff(b"\x89PNG\r\n\x1a\n\0\0"),
            ImageFormat::Png
        );
        assert_eq!(
            ImageFormat::sniff(&[0xFF, 0xD8, 0xFF, 0xE0]),
            ImageFormat::Jpeg
        );
        assert_eq!(
            ImageFormat::sniff(b"RIFF\0\0\0\0WEBPVP8 "),
            ImageFormat::Webp
        );
        assert_eq!(ImageFormat::sniff(b"nope"), ImageFormat::Unknown);
    }

    #[test]
    fn rect_from_points_takes_the_bounding_box() {
        let rect = rect_from_points([(10.0, 5.0), (30.0, 5.0), (30.0, 15.0), (10.0, 15.0)])
            .expect("four points");
        assert_eq!(
            (rect.x, rect.y, rect.width, rect.height),
            (10.0, 5.0, 20.0, 10.0)
        );
        assert_eq!(rect.right, Some(30.0));
        assert_eq!(rect.bottom, Some(15.0));
        assert!(rect_from_points(std::iter::empty()).is_none());
    }

    #[test]
    fn coordinate_reads_numbers_and_numeric_strings() {
        assert_eq!(coordinate(&serde_json::json!(12)), Some(12.0));
        assert_eq!(coordinate(&serde_json::json!("12")), Some(12.0));
        assert_eq!(coordinate(&serde_json::json!(null)), None);
    }
}
