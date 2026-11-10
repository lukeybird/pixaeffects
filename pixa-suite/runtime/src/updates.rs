//! Verify the exact signed payload before parsing or trusting an update URL or digest.
use base64::{Engine as _, engine::general_purpose::STANDARD};
use chrono::{DateTime, Utc};
use ed25519_dalek::{Signature, VerifyingKey};
use semver::Version;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::io::Read;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Envelope {
    payload: String,
    signature: String,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub schema_version: u32,
    pub product: String,
    pub platform: String,
    pub architecture: String,
    pub version: String,
    pub bundle_id: String,
    pub url: String,
    pub size: u64,
    pub sha256: String,
    pub expires_at: DateTime<Utc>,
}
pub struct Expected<'a> {
    pub product: &'a str,
    pub platform: &'a str,
    pub architecture: &'a str,
    pub current_version: &'a str,
    pub bundle_id: &'a str,
    pub asset_prefix: &'a str,
}
pub fn verify(
    envelope: &[u8],
    key: &[u8; 32],
    expected: &Expected<'_>,
    now: DateTime<Utc>,
) -> Result<Manifest, String> {
    if envelope.len() > 131_072 {
        return Err("Update response is too large.".into());
    }
    let envelope: Envelope =
        serde_json::from_slice(envelope).map_err(|_| "Invalid update envelope.")?;
    if envelope.payload.len() > 90_000 {
        return Err("Update payload is too large.".into());
    }
    let payload = STANDARD
        .decode(envelope.payload)
        .map_err(|_| "Invalid update payload.")?;
    let signature = STANDARD
        .decode(envelope.signature)
        .map_err(|_| "Invalid update signature.")?;
    let signature = Signature::from_slice(&signature).map_err(|_| "Invalid update signature.")?;
    let key = VerifyingKey::from_bytes(key).map_err(|_| "Invalid publisher verification key.")?;
    key.verify_strict(&payload, &signature)
        .map_err(|_| "Update signature did not verify.")?;
    let manifest: Manifest =
        serde_json::from_slice(&payload).map_err(|_| "Invalid signed update manifest.")?;
    if manifest.schema_version != 1
        || manifest.product != expected.product
        || manifest.platform != expected.platform
        || manifest.architecture != expected.architecture
        || manifest.bundle_id != expected.bundle_id
    {
        return Err("The update is for another product or platform.".into());
    }
    if manifest.expires_at <= now {
        return Err("The update manifest has expired.".into());
    }
    let current =
        Version::parse(expected.current_version).map_err(|_| "Invalid installed version.")?;
    let next = Version::parse(&manifest.version).map_err(|_| "Invalid update version.")?;
    if !next.cmp_precedence(&current).is_gt() || !next.pre.is_empty() {
        return Err("The update is not a newer stable version.".into());
    }
    if manifest.size == 0 || manifest.size > 8 * 1024 * 1024 * 1024 {
        return Err("Invalid installer size.".into());
    }
    if manifest.sha256.len() != 64 || !manifest.sha256.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("Invalid installer checksum.".into());
    }
    let url = url::Url::parse(&manifest.url).map_err(|_| "Invalid installer URL.")?;
    let prefix =
        url::Url::parse(expected.asset_prefix).map_err(|_| "Invalid publisher asset prefix.")?;
    if url.scheme() != "https"
        || !url.username().is_empty()
        || url.password().is_some()
        || url.fragment().is_some()
        || url.query().is_some()
        || url.path().contains('%')
        || url.port().is_some()
        || url.host_str() != prefix.host_str()
        || !prefix.path().ends_with('/')
        || !url.path().starts_with(prefix.path())
    {
        return Err("Installer URL is outside the publisher's release location.".into());
    }
    Ok(manifest)
}
pub fn verify_download(mut input: impl Read, manifest: &Manifest) -> Result<(), String> {
    let mut hasher = Sha256::new();
    let mut total = 0_u64;
    let mut buffer = [0_u8; 65_536];
    loop {
        let n = input
            .read(&mut buffer)
            .map_err(|_| "Could not read the downloaded installer.")?;
        if n == 0 {
            break;
        }
        total = total
            .checked_add(n as u64)
            .ok_or("Installer size overflow.")?;
        if total > manifest.size {
            return Err("Downloaded installer is larger than expected.".into());
        }
        hasher.update(buffer.get(..n).ok_or("Invalid download read length.")?);
    }
    let checksum = format!("{:x}", hasher.finalize());
    if total != manifest.size || !checksum.eq_ignore_ascii_case(&manifest.sha256) {
        return Err("Installer size or checksum did not match.".into());
    }
    Ok(())
}
#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};
    fn expected() -> Expected<'static> {
        Expected {
            product: "pixavector",
            platform: "macos",
            architecture: "universal",
            current_version: "0.1.0",
            bundle_id: "com.aiwebdesignfirm.pixavector",
            asset_prefix: "https://github.com/lukeybird/pixaeffects/releases/download/",
        }
    }
    fn payload() -> serde_json::Value {
        serde_json::json!({"schema_version":1,"product":"pixavector","platform":"macos","architecture":"universal","version":"0.2.0","bundle_id":"com.aiwebdesignfirm.pixavector","url":"https://github.com/lukeybird/pixaeffects/releases/download/pixavector-v0.2.0/pixavector.dmg","size":3,"sha256":format!("{:x}",Sha256::digest(b"abc")),"expires_at":"2030-01-01T00:00:00Z"})
    }
    fn envelope(payload: &serde_json::Value) -> (Vec<u8>, [u8; 32]) {
        // Deterministic fixture key only. Never a publisher signing identity.
        let key = SigningKey::from_bytes(&[7_u8; 32]);
        let bytes = serde_json::to_vec(payload).unwrap();
        (serde_json::to_vec(&serde_json::json!({"payload":STANDARD.encode(&bytes),"signature":STANDARD.encode(key.sign(&bytes).to_bytes())})).unwrap(),key.verifying_key().to_bytes())
    }
    fn now() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2026-11-09T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc)
    }
    #[test]
    fn signed_update_and_exact_download_verify() {
        let (bytes, key) = envelope(&payload());
        let m = verify(&bytes, &key, &expected(), now()).unwrap();
        verify_download(&b"abc"[..], &m).unwrap();
        assert!(verify_download(&b"abcd"[..], &m).is_err());
        assert!(verify_download(&b"abd"[..], &m).is_err());
    }
    #[test]
    fn tampered_signed_payload_fails() {
        let (bytes, key) = envelope(&payload());
        let mut value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        let mut changed = payload();
        changed["url"] = "https://attacker.invalid/installer".into();
        value["payload"] = STANDARD
            .encode(serde_json::to_vec(&changed).unwrap())
            .into();
        assert!(
            verify(
                &serde_json::to_vec(&value).unwrap(),
                &key,
                &expected(),
                now()
            )
            .is_err()
        );
    }
    #[test]
    fn wrong_product_version_expiry_and_url_are_rejected() {
        for (field, value) in [
            ("product", "pixaword"),
            ("version", "0.0.9"),
            ("version", "0.1.0+repack"),
            ("version", "0.3.0-beta.1"),
            ("expires_at", "2020-01-01T00:00:00Z"),
            (
                "url",
                "https://github.com.attacker.invalid/lukeybird/pixaeffects/releases/download/a",
            ),
            (
                "url",
                "https://github.com/lukeybird/pixaeffects/releases/download-evil/a",
            ),
        ] {
            let mut p = payload();
            p[field] = value.into();
            let (bytes, key) = envelope(&p);
            assert!(
                verify(&bytes, &key, &expected(), now()).is_err(),
                "{field}={value}"
            );
        }
    }
}
