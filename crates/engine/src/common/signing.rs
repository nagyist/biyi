//! The pieces the cloud vendors' request-signing schemes share: SHA-256 and
//! HMAC-SHA256 as hex, a UTC clock formatted the way each scheme wants it,
//! percent-encoding for canonical strings and a random nonce.

use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

pub fn sha256_hex(data: &[u8]) -> String {
    hex(&Sha256::digest(data))
}

pub fn hmac_sha256(key: &[u8], data: &[u8]) -> Vec<u8> {
    let mut mac = Hmac::<Sha256>::new_from_slice(key).expect("HMAC accepts any key length");
    mac.update(data);
    mac.finalize().into_bytes().to_vec()
}

/// A random 32-hex-character nonce, unique enough for the replay guards
/// the signing schemes attach it to.
pub fn nonce() -> String {
    hex(&rand::random::<[u8; 16]>())
}

/// Percent-encodes everything but RFC 3986 unreserved characters, which is
/// what both canonical query strings and form bodies get signed as.
pub fn percent_encode(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for byte in input.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char)
            }
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

/// A moment in UTC, broken into fields so it can be printed in whichever
/// layout a signing scheme asks for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UtcTime {
    pub year: i64,
    pub month: u32,
    pub day: u32,
    pub hour: u32,
    pub minute: u32,
    pub second: u32,
}

impl UtcTime {
    pub fn now() -> Self {
        Self::from_unix(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64,
        )
    }

    pub fn from_unix(seconds: i64) -> Self {
        let days = seconds.div_euclid(86_400);
        let remainder = seconds.rem_euclid(86_400) as u32;
        let (year, month, day) = civil_from_days(days);
        Self {
            year,
            month,
            day,
            hour: remainder / 3600,
            minute: remainder % 3600 / 60,
            second: remainder % 60,
        }
    }

    /// `20240131`
    pub fn date(&self) -> String {
        format!("{:04}{:02}{:02}", self.year, self.month, self.day)
    }

    /// `20240131T235959Z` — the AWS-style stamp Volcengine uses.
    pub fn compact(&self) -> String {
        format!(
            "{}T{:02}{:02}{:02}Z",
            self.date(),
            self.hour,
            self.minute,
            self.second
        )
    }

    /// `2024-01-31T23:59:59Z` — what Alibaba Cloud's ACS3 scheme wants.
    pub fn iso8601(&self) -> String {
        format!(
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
            self.year, self.month, self.day, self.hour, self.minute, self.second
        )
    }
}

/// Days since 1970-01-01 to a proleptic Gregorian date (Howard Hinnant's
/// algorithm), so no calendar crate is needed for a timestamp header.
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let year = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let month = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if month <= 2 { year + 1 } else { year }, month, day)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_a_known_instant() {
        let time = UtcTime::from_unix(1_700_000_000);
        assert_eq!(time.iso8601(), "2023-11-14T22:13:20Z");
        assert_eq!(time.compact(), "20231114T221320Z");
        assert_eq!(time.date(), "20231114");
    }

    #[test]
    fn handles_the_epoch_and_leap_days() {
        assert_eq!(UtcTime::from_unix(0).iso8601(), "1970-01-01T00:00:00Z");
        // 2024-02-29T12:00:00Z
        assert_eq!(
            UtcTime::from_unix(1_709_208_000).iso8601(),
            "2024-02-29T12:00:00Z"
        );
    }

    #[test]
    fn hashes_match_known_vectors() {
        assert_eq!(
            sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        // RFC 4231 test case 2.
        assert_eq!(
            hex(&hmac_sha256(b"Jefe", b"what do ya want for nothing?")),
            "5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843"
        );
    }

    #[test]
    fn percent_encodes_form_values() {
        assert_eq!(percent_encode("a+b/c=d ~"), "a%2Bb%2Fc%3Dd%20~");
        assert_eq!(percent_encode("AZaz09-_.~"), "AZaz09-_.~");
    }

    #[test]
    fn nonce_is_32_hex_chars_and_varies() {
        let first = nonce();
        assert_eq!(first.len(), 32);
        assert!(first.bytes().all(|byte| byte.is_ascii_hexdigit()));
        assert_ne!(first, nonce());
    }
}
