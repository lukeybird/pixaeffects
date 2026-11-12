//! Checks a Pixa release manifest before any bytes are downloaded or executed.
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

const MAX_BYTES: u64 = 8 * 1024 * 1024 * 1024;
const ALLOWED_PREFIX: &str = "https://github.com/lukeybird/pixaeffects/releases/download/";

#[derive(Debug, PartialEq, Eq)]
pub struct ReleaseEntry {
    pub product: String,
    pub version: String,
    pub os: String,
    pub arch: String,
    pub bundle_id: String,
    pub url: String,
    pub size: u64,
    pub sha256: String,
}

#[derive(Debug)]
pub enum ManifestError {
    Signature,
    Json,
    Schema,
    Product,
    Version,
    Url,
    Digest,
    Size,
    Extra,
}

impl std::fmt::Display for ManifestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(Deserialize)]
struct RawEntry {
    schema: u32,
    product: String,
    version: String,
    channel: String,
    os: String,
    arch: String,
    minimum_os: String,
    bundle_id: String,
    publisher: String,
    url: String,
    size: u64,
    sha256: String,
}

/// Accept a manifest only when `signature` is a valid Ed25519 signature over
/// the exact JSON object bytes, excluding any surrounding whitespace.
pub fn accept_manifest(
    body: &str,
    signature_hex: &str,
    verifying_key: &VerifyingKey,
) -> Result<ReleaseEntry, ManifestError> {
    let signature_bytes = hex::decode(signature_hex).map_err(|_| ManifestError::Signature)?;
    let signature = Signature::from_slice(&signature_bytes).map_err(|_| ManifestError::Signature)?;
    verifying_key
        .verify(body.as_bytes(), &signature)
        .map_err(|_| ManifestError::Signature)?;
    let value: Value = serde_json::from_str(body).map_err(|_| ManifestError::Json)?;
    let object = value.as_object().ok_or(ManifestError::Schema)?;
    const FIELDS: [&str; 12] = [
        "schema",
        "product",
        "version",
        "channel",
        "os",
        "arch",
        "minimum_os",
        "bundle_id",
        "publisher",
        "url",
        "size",
        "sha256",
    ];
    if object.len() != FIELDS.len() || FIELDS.iter().any(|field| !object.contains_key(*field)) {
        return Err(ManifestError::Extra);
    }
    let raw: RawEntry = serde_json::from_value(value).map_err(|_| ManifestError::Schema)?;
    if raw.schema != 1 || raw.channel != "stable" || raw.minimum_os.is_empty() || raw.publisher.is_empty()
    {
        return Err(ManifestError::Schema);
    }
    if !valid_product(&raw.product) || !raw.bundle_id.starts_with("com.aiwebdesignfirm.") {
        return Err(ManifestError::Product);
    }
    if !plain_version(&raw.version) {
        return Err(ManifestError::Version);
    }
    if raw.os != "macos" || !matches!(raw.arch.as_str(), "universal" | "arm64" | "x86_64") {
        return Err(ManifestError::Schema);
    }
    if raw.size == 0 || raw.size > MAX_BYTES {
        return Err(ManifestError::Size);
    }
    if raw.sha256.len() != 64 || !raw.sha256.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(ManifestError::Digest);
    }
    expected_url(&raw.product, &raw.version, &raw.url)?;
    Ok(ReleaseEntry {
        product: raw.product,
        version: raw.version,
        os: raw.os,
        arch: raw.arch,
        bundle_id: raw.bundle_id,
        url: raw.url,
        size: raw.size,
        sha256: raw.sha256,
    })
}

pub fn verifying_key_fingerprint(key: &VerifyingKey) -> String {
    hex::encode(Sha256::digest(key.as_bytes()))
}

fn valid_product(id: &str) -> bool {
    id.starts_with("pixa")
        && id.len() <= 32
        && id.bytes().all(|byte| byte.is_ascii_lowercase())
}

fn plain_version(version: &str) -> bool {
    let mut parts = version.split('.');
    let numbers = parts.by_ref().take(3).collect::<Vec<_>>();
    numbers.len() == 3
        && parts.next().is_none()
        && numbers.iter().all(|part| !part.is_empty() && part.bytes().all(|b| b.is_ascii_digit()) && (part == &"0" || !part.starts_with('0')))
}

fn expected_url(product: &str, version: &str, url: &str) -> Result<(), ManifestError> {
    let name = format!("{product}-{version}-macos-universal.dmg");
    let expected = format!("{ALLOWED_PREFIX}{product}-v{version}/{name}");
    if url != expected {
        return Err(ManifestError::Url);
    }
    let path = url.trim_start_matches("https://github.com/");
    if path.split('/').any(|segment| segment.is_empty() || segment == "." || segment == "..") {
        return Err(ManifestError::Url);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};

    fn key() -> SigningKey {
        SigningKey::from_bytes(&[7u8; 32])
    }

    fn signed(body: &str) -> (String, String) {
        let signature = key().sign(body.as_bytes());
        (body.to_string(), hex::encode(signature.to_bytes()))
    }

    fn good() -> String {
        r#"{"schema":1,"product":"pixavector","version":"0.1.1","channel":"stable","os":"macos","arch":"universal","minimum_os":"11.0","bundle_id":"com.aiwebdesignfirm.pixavector","publisher":"Developer ID Application: Luke Barger (945M5X4DY3)","url":"https://github.com/lukeybird/pixaeffects/releases/download/pixavector-v0.1.1/pixavector-0.1.1-macos-universal.dmg","size":100,"sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}"#.to_string()
    }

    #[test]
    fn accepts_exact_signed_manifest() {
        let body = good();
        let (body, signature) = signed(&body);
        let entry = accept_manifest(&body, &signature, &key().verifying_key()).unwrap();
        assert_eq!(entry.product, "pixavector");
        assert_eq!(entry.arch, "universal");
    }

    #[test]
    fn rejects_tampered_body_wrong_product_and_foreign_url() {
        let body = good();
        let (_, signature) = signed(&body);
        let tampered = body.replace("100", "101");
        assert!(accept_manifest(&tampered, &signature, &key().verifying_key()).is_err());
        let other = body.replacen("\"product\":\"pixavector\"", "\"product\":\"pixafilm\"", 1);
        let (other, other_sig) = signed(&other);
        assert!(matches!(
            accept_manifest(&other, &other_sig, &key().verifying_key()),
            Err(ManifestError::Url)
        ));
        let beta = body.replacen("\"version\":\"0.1.1\"", "\"version\":\"0.1.1-beta\"", 1);
        let (beta, beta_sig) = signed(&beta);
        assert!(matches!(
            accept_manifest(&beta, &beta_sig, &key().verifying_key()),
            Err(ManifestError::Version)
        ));
    }
}
