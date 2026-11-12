//! Checks a Pixa release manifest before any bytes are downloaded or executed.
use ed25519_dalek::{Signature, VerifyingKey};
use serde::Deserialize;
use sha2::{Digest, Sha256};

const MAX_BYTES: u64 = 8 * 1024 * 1024 * 1024;
const MAX_MANIFEST_BYTES: usize = 64 * 1024;
const PUBLISHER: &str = "Developer ID Application: Luke Barger (945M5X4DY3)";

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
    pub minimum_os: String,
    pub publisher: String,
    pub sequence: u64,
    pub expires_at: u64,
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
    Expired,
    Replay,
}

impl std::fmt::Display for ManifestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawEntry {
    schema: u32,
    sequence: u64,
    expires_at: u64,
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
/// the exact UTF-8 body bytes, including any whitespace. The caller supplies
/// trusted current Unix seconds and its persisted minimum manifest sequence.
/// Never derive either trust value from the received manifest itself.
pub fn accept_manifest(body: &str, signature_hex: &str, verifying_key: &VerifyingKey, now: u64, minimum_sequence: u64) -> Result<ReleaseEntry, ManifestError> {
    if body.len() > MAX_MANIFEST_BYTES {
        return Err(ManifestError::Size);
    }
    let signature_bytes = hex::decode(signature_hex).map_err(|_| ManifestError::Signature)?;
    let signature = Signature::from_slice(&signature_bytes).map_err(|_| ManifestError::Signature)?;
    verifying_key.verify_strict(body.as_bytes(), &signature).map_err(|_| ManifestError::Signature)?;
    // Direct deserialization rejects duplicate as well as unknown fields.
    let raw: RawEntry = serde_json::from_str(body).map_err(|_| ManifestError::Json)?;
    if raw.schema != 2 || raw.channel != "stable" || !numeric_os(&raw.minimum_os) || raw.publisher != PUBLISHER {
        return Err(ManifestError::Schema);
    }
    if raw.expires_at <= now || raw.expires_at.saturating_sub(now) > 31 * 86400 {
        return Err(ManifestError::Expired);
    }
    if raw.sequence == 0 || raw.sequence < minimum_sequence {
        return Err(ManifestError::Replay);
    }
    if !valid_product(&raw.product) || raw.bundle_id != format!("com.aiwebdesignfirm.{}", raw.product) {
        return Err(ManifestError::Product);
    }
    if !plain_version(&raw.version) {
        return Err(ManifestError::Version);
    }
    if raw.os != "macos" || raw.arch != "universal" {
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
        minimum_os: raw.minimum_os,
        publisher: raw.publisher,
        sequence: raw.sequence,
        expires_at: raw.expires_at,
    })
}

pub fn verifying_key_fingerprint(key: &VerifyingKey) -> String {
    hex::encode(Sha256::digest(key.as_bytes()))
}

fn valid_product(id: &str) -> bool {
    matches!(
        id,
        "pixashop"
            | "pixaeffects"
            | "pixavector"
            | "pixafilm"
            | "pixalight"
            | "pixapdf"
            | "pixadesign"
            | "pixainterface"
            | "pixatexture"
            | "pixamaterial"
            | "pixaword"
            | "pixasound"
            | "pixadeck"
            | "pixacad"
            | "pixagrid"
            | "pixaanything"
    )
}

fn numeric_os(version: &str) -> bool {
    let parts: Vec<_> = version.split('.').collect();
    (2..=3).contains(&parts.len()) && parts.iter().all(|part| !part.is_empty() && part.len() <= 5 && part.bytes().all(|b| b.is_ascii_digit()))
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
    let repository = if product == "pixashop" { "pixashop" } else { "pixaeffects" };
    let tag = if matches!(product, "pixashop" | "pixaeffects") { format!("v{version}") } else { format!("{product}-v{version}") };
    let expected = format!("https://github.com/lukeybird/{repository}/releases/download/{tag}/{name}");
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
        r#"{"schema":2,"sequence":1,"expires_at":2000,"product":"pixavector","version":"0.1.1","channel":"stable","os":"macos","arch":"universal","minimum_os":"11.0","bundle_id":"com.aiwebdesignfirm.pixavector","publisher":"Developer ID Application: Luke Barger (945M5X4DY3)","url":"https://github.com/lukeybird/pixaeffects/releases/download/pixavector-v0.1.1/pixavector-0.1.1-macos-universal.dmg","size":100,"sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}"#.to_string()
    }

    #[test]
    fn accepts_exact_signed_manifest() {
        let body = good();
        let (body, signature) = signed(&body);
        let entry = accept_manifest(&body, &signature, &key().verifying_key(), 1000, 1).unwrap();
        assert_eq!(entry.product, "pixavector");
        assert_eq!(entry.arch, "universal");
    }

    #[test]
    fn rejects_tampered_body_wrong_product_and_foreign_url() {
        let body = good();
        let (_, signature) = signed(&body);
        let tampered = body.replace("100", "101");
        assert!(accept_manifest(&tampered, &signature, &key().verifying_key(), 1000, 1).is_err());
        let other = body.replacen("\"product\":\"pixavector\"", "\"product\":\"pixafilm\"", 1);
        let (other, other_sig) = signed(&other);
        assert!(matches!(accept_manifest(&other, &other_sig, &key().verifying_key(), 1000, 1), Err(ManifestError::Product)));
        let beta = body.replacen("\"version\":\"0.1.1\"", "\"version\":\"0.1.1-beta\"", 1);
        let (beta, beta_sig) = signed(&beta);
        assert!(matches!(accept_manifest(&beta, &beta_sig, &key().verifying_key(), 1000, 1), Err(ManifestError::Version)));
    }
    #[test]
    fn rejects_signed_wrong_bundle_publisher_and_architecture() {
        for (old, new) in [
            ("com.aiwebdesignfirm.pixavector", "com.aiwebdesignfirm.pixafilm"),
            ("Developer ID Application: Luke Barger (945M5X4DY3)", "Foreign Publisher"),
            ("\"arch\":\"universal\"", "\"arch\":\"arm64\""),
            ("\"product\":\"pixavector\"", "\"product\":\"pixafiction\""),
        ] {
            let body = good().replace(old, new);
            let (body, signature) = signed(&body);
            assert!(accept_manifest(&body, &signature, &key().verifying_key(), 1000, 1).is_err(), "accepted {new}");
        }
    }

    #[test]
    fn refuses_expiry_replay_duplicates_and_unknown_fields() {
        for body in [
            good().replace("\"expires_at\":2000", "\"expires_at\":999"),
            good().replace("\"sequence\":1", "\"sequence\":0"),
            good().replace("\"schema\":2", "\"schema\":2,\"schema\":2"),
            good().replace("\"schema\":2", "\"schema\":2,\"extra\":true"),
        ] {
            let (body, signature) = signed(&body);
            assert!(accept_manifest(&body, &signature, &key().verifying_key(), 1000, 1).is_err());
        }
        let (body, signature) = signed(&good());
        assert!(matches!(accept_manifest(&body, &signature, &key().verifying_key(), 1000, 2), Err(ManifestError::Replay)));
    }

    #[test]
    fn exact_whitespace_is_signed_and_size_is_bounded() {
        let body = good();
        let (_, signature) = signed(&body);
        assert!(accept_manifest(&(body + " "), &signature, &key().verifying_key(), 1000, 1).is_err());
        assert!(matches!(accept_manifest(&" ".repeat(MAX_MANIFEST_BYTES + 1), "", &key().verifying_key(), 1000, 1), Err(ManifestError::Size)));
    }
}
