use crate::uos::error::UosError;

/// Represents an account-introduction QR payload, used to share a Substrate
/// address (optionally pinned to a genesis hash and labelled with a human name)
/// as a scannable URI.
///
/// URI format:
/// ```text
/// substrate:<address>[:<genesis_hash>][?name=<url-encoded-name>]
/// ```
///
/// The `name` field is URL-encoded using Java `URLEncoder.encode(s, "UTF-8")`
/// rules (`application/x-www-form-urlencoded`):
/// - spaces → `+`
/// - unreserved chars (`A-Z a-z 0-9 . - _ *`) → unchanged
/// - everything else → `%XX` (hex, uppercase)
#[derive(Debug, Clone, PartialEq)]
pub struct AccountIntroduction {
    /// SS58 address string.
    pub address: String,
    /// Optional genesis hash (hex string without `0x` prefix).
    pub genesis_hash: Option<String>,
    /// Optional human-readable label.
    pub name: Option<String>,
}

impl AccountIntroduction {
    pub fn new(address: String, genesis_hash: Option<String>, name: Option<String>) -> Self {
        Self { address, genesis_hash, name }
    }

    /// Encodes this account introduction as a URI string.
    pub fn to_uri(&self) -> String {
        let mut uri = format!("substrate:{}", self.address);
        if let Some(ref gh) = self.genesis_hash {
            uri.push(':');
            uri.push_str(gh);
        }
        if let Some(ref name) = self.name {
            uri.push_str("?name=");
            uri.push_str(&url_encode(name));
        }
        uri
    }

    /// Parses an account introduction URI.
    pub fn from_uri(uri: &str) -> Result<Self, UosError> {
        let rest = uri.strip_prefix("substrate:").ok_or(UosError::InvalidUri)?;

        // Split off query string first.
        let (path, query) = match rest.split_once('?') {
            Some((p, q)) => (p, Some(q)),
            None => (rest, None),
        };

        // path is `<address>` or `<address>:<genesis_hash>`.
        let (address, genesis_hash) = match path.split_once(':') {
            Some((addr, gh)) => (addr.to_string(), Some(gh.to_string())),
            None => (path.to_string(), None),
        };

        if address.is_empty() {
            return Err(UosError::InvalidUri);
        }

        // Parse `name=<value>` from the query string.
        let name = query
            .and_then(|q| {
                q.split('&').find_map(|kv| {
                    let (k, v) = kv.split_once('=')?;
                    if k == "name" {
                        Some(url_decode(v))
                    } else {
                        None
                    }
                })
            })
            .transpose()
            .map_err(|_| UosError::InvalidUri)?;

        Ok(Self { address, genesis_hash, name })
    }
}

/// Encodes a string using `application/x-www-form-urlencoded` rules, matching
/// the output of Java's `URLEncoder.encode(s, "UTF-8")` exactly:
///
/// - Unreserved: `A-Z a-z 0-9 . - _ *` — passed through unchanged.
/// - Space (U+0020) → `+`.
/// - Everything else → `%XX` (percent + two uppercase hex digits).
fn url_encode(input: &str) -> String {
    let mut out = String::with_capacity(input.len() * 2);
    for byte in input.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'.' | b'-' | b'_' | b'*' => {
                out.push(byte as char);
            }
            b' ' => out.push('+'),
            b => {
                out.push('%');
                out.push(hex_nibble(b >> 4));
                out.push(hex_nibble(b & 0xF));
            }
        }
    }
    out
}

/// Decodes a `application/x-www-form-urlencoded` string.
///
/// Returns an error string (not `UosError`) so the caller can wrap it.
fn url_decode(input: &str) -> Result<String, String> {
    let bytes = input.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            b'%' => {
                if i + 2 >= bytes.len() {
                    return Err(format!("truncated percent-escape at position {i}"));
                }
                let hi = from_hex(bytes[i + 1]).ok_or_else(|| {
                    format!("bad hex digit '{}' at {}", bytes[i + 1] as char, i + 1)
                })?;
                let lo = from_hex(bytes[i + 2]).ok_or_else(|| {
                    format!("bad hex digit '{}' at {}", bytes[i + 2] as char, i + 2)
                })?;
                out.push((hi << 4) | lo);
                i += 3;
            }
            b => {
                out.push(b);
                i += 1;
            }
        }
    }
    String::from_utf8(out).map_err(|e| format!("invalid UTF-8 after decoding: {e}"))
}

fn hex_nibble(n: u8) -> char {
    match n {
        0..=9 => (b'0' + n) as char,
        10..=15 => (b'A' + n - 10) as char,
        _ => unreachable!(),
    }
}

fn from_hex(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'A'..=b'F' => Some(b - b'A' + 10),
        b'a'..=b'f' => Some(b - b'a' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn url_encode_spaces_as_plus() {
        assert_eq!(url_encode("hello world"), "hello+world");
    }

    #[test]
    fn url_encode_special_chars() {
        // Java URLEncoder leaves `*`, `.`, `-`, `_` unencoded.
        assert_eq!(url_encode("a*b.c-d_e"), "a*b.c-d_e");
    }

    #[test]
    fn url_encode_unicode() {
        // "café" in UTF-8: c a f é(0xC3 0xA9)
        let encoded = url_encode("café");
        assert_eq!(encoded, "caf%C3%A9");
    }

    #[test]
    fn url_encode_reserved_chars() {
        // Characters like `@`, `#`, `/` must be percent-encoded.
        assert_eq!(url_encode("a@b#c/d"), "a%40b%23c%2Fd");
    }

    #[test]
    fn round_trip_simple() {
        let ai = AccountIntroduction::new(
            "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY".into(),
            None,
            None,
        );
        let uri = ai.to_uri();
        let back = AccountIntroduction::from_uri(&uri).unwrap();
        assert_eq!(ai, back);
    }

    #[test]
    fn round_trip_with_genesis_and_name() {
        let ai = AccountIntroduction::new(
            "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY".into(),
            Some("91b171bb158e2d3848fa23a9f1c25182fb8e20313b2c1eb49219da7a70ce90c3".into()),
            Some("Alice Wonderland".into()),
        );
        let uri = ai.to_uri();
        assert!(uri.contains("name=Alice+Wonderland"));
        let back = AccountIntroduction::from_uri(&uri).unwrap();
        assert_eq!(ai, back);
    }
}
