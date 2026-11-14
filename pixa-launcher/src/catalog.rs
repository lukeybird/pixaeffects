//! Compiled catalog plus bounded, publisher-scoped GitHub release discovery.
use serde::Deserialize;
use std::cmp::Ordering;
use std::io::Read;
use std::time::Duration;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IconRect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub color: String,
    pub radius: f32,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Product {
    pub id: String,
    pub name: String,
    pub category: String,
    pub summary: String,
    pub version: String,
    pub url: String,
    pub size: u64,
    pub sha256: String,
    pub icon: Vec<IconRect>,
}
impl Product {
    pub fn bundle_id(&self) -> String {
        format!("com.aiwebdesignfirm.{}", self.id)
    }
    pub fn validate(&self) -> Result<(), String> {
        if !super::valid_product(&self.id) || self.name != display_name(&self.id)? {
            return Err("Unknown product identity".into());
        }
        if !super::plain_version(&self.version) {
            return Err("Invalid release version".into());
        }
        super::expected_url(&self.id, &self.version, &self.url).map_err(|e| e.to_string())?;
        if self.size == 0 || self.size > super::MAX_BYTES {
            return Err("Invalid download size".into());
        }
        if self.sha256.len() != 64 || !self.sha256.bytes().all(|v| v.is_ascii_hexdigit()) {
            return Err("Invalid release checksum".into());
        }
        Ok(())
    }
}
pub fn display_name(id: &str) -> Result<String, String> {
    let suffix = match id {
        "pixashop" => "Shop",
        "pixaeffects" => "Effects",
        "pixavector" => "Vector",
        "pixafilm" => "Film",
        "pixalight" => "Light",
        "pixapdf" => "PDF",
        "pixadesign" => "Design",
        "pixainterface" => "Interface",
        "pixatexture" => "Texture",
        "pixamaterial" => "Material",
        "pixaword" => "Word",
        "pixasound" => "Sound",
        "pixadeck" => "Deck",
        "pixacad" => "CAD",
        "pixagrid" => "Grid",
        "pixaanything" => "Anything",
        _ => return Err("Unknown product".into()),
    };
    Ok(format!("Pixa{suffix}"))
}
pub fn embedded_catalog() -> Result<Vec<Product>, String> {
    let products: Vec<Product> = serde_json::from_str(include_str!("../catalog.json")).map_err(|e| e.to_string())?;
    let mut ids = std::collections::HashSet::new();
    for p in &products {
        p.validate()?;
        if !ids.insert(&p.id) {
            return Err("Duplicate catalog product".into());
        }
    }
    if products.len() != 16 {
        return Err("Expected sixteen products".into());
    }
    Ok(products)
}
pub fn compare_versions(a: &str, b: &str) -> Result<Ordering, String> {
    fn parts(v: &str) -> Result<Vec<u64>, String> {
        if !super::plain_version(v) {
            return Err("Invalid version".into());
        }
        v.split('.').map(|s| s.parse().map_err(|_| "Version number too large".into())).collect()
    }
    Ok(parts(a)?.cmp(&parts(b)?))
}
pub fn client() -> Result<reqwest::blocking::Client, String> {
    reqwest::blocking::Client::builder()
        .user_agent("PixaLauncher/0.2.0")
        .connect_timeout(Duration::from_secs(20))
        .timeout(Duration::from_secs(1800))
        .redirect(reqwest::redirect::Policy::custom(|attempt| {
            let host = attempt.url().host_str().unwrap_or("");
            if attempt.previous().len() > 5
                || attempt.url().scheme() != "https"
                || !(host == "github.com"
                    || host == "api.github.com"
                    || host == "release-assets.githubusercontent.com"
                    || host == "objects.githubusercontent.com")
            {
                attempt.error("Unapproved download redirect")
            } else {
                attempt.follow()
            }
        }))
        .build()
        .map_err(|e| e.to_string())
}
#[derive(Deserialize)]
pub struct Release {
    pub tag_name: String,
    pub draft: bool,
    pub prerelease: bool,
    pub assets: Vec<Asset>,
}
#[derive(Deserialize)]
pub struct Asset {
    pub name: String,
    pub browser_download_url: String,
    pub size: u64,
    pub digest: Option<String>,
}
/// Metadata discovery never approves an executable. The installer independently
/// enforces SHA, Apple signature, publisher, bundle identity and Gatekeeper.
pub fn refresh(current: &[Product]) -> Result<Vec<Product>, String> {
    let client = client()?;
    let mut output = current.to_vec();
    for repo in ["pixashop", "pixaeffects"] {
        let response = client
            .get(format!("https://api.github.com/repos/lukeybird/{repo}/releases?per_page=100"))
            .timeout(Duration::from_secs(45))
            .send()
            .and_then(|r| r.error_for_status())
            .map_err(|e| format!("Release check failed: {e}"))?;
        let mut bytes = Vec::new();
        response.take(2_000_001).read_to_end(&mut bytes).map_err(|e| e.to_string())?;
        if bytes.len() > 2_000_000 {
            return Err("Release response exceeds limit".into());
        }
        let releases: Vec<Release> = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
        for p in &mut output {
            if (p.id == "pixashop") != (repo == "pixashop") {
                continue;
            }
            for release in &releases {
                if let Some(candidate) = candidate(p, release)? {
                    *p = candidate;
                }
            }
        }
    }
    Ok(output)
}
pub fn candidate(current: &Product, r: &Release) -> Result<Option<Product>, String> {
    if r.draft || r.prerelease {
        return Ok(None);
    }
    let prefix = if current.id == "pixashop" { "v".to_string() } else { format!("{}-v", current.id) };
    let Some(version) = r.tag_name.strip_prefix(&prefix) else { return Ok(None) };
    if !super::plain_version(version) || compare_versions(version, &current.version)? != Ordering::Greater {
        return Ok(None);
    }
    let expected = format!("{}-{version}-macos-universal.dmg", current.id);
    let Some(asset) = r.assets.iter().find(|a| a.name == expected) else { return Ok(None) };
    let Some(hash) = asset.digest.as_deref().and_then(|d| d.strip_prefix("sha256:")) else { return Ok(None) };
    let mut p = current.clone();
    p.version = version.into();
    p.url = asset.browser_download_url.clone();
    p.size = asset.size;
    p.sha256 = hash.into();
    p.validate()?;
    Ok(Some(p))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn catalog_has_sixteen_real_scoped_releases() {
        let p = embedded_catalog().unwrap();
        assert_eq!(p.len(), 16);
        assert!(p.iter().find(|p| p.id == "pixaeffects").unwrap().url.contains("pixaeffects-v0.1.3"));
    }
    #[test]
    fn ordering_is_numeric_and_bounded() {
        assert_eq!(compare_versions("0.20.10", "0.20.5").unwrap(), Ordering::Greater);
        assert!(compare_versions("1.0.0-beta", "1.0.0").is_err());
        assert!(compare_versions("9999999999999999999999999.0.0", "1.0.0").is_err());
    }
    #[test]
    fn rejects_foreign_assets_and_identity() {
        let mut p = embedded_catalog().unwrap().remove(0);
        p.url = p.url.replace("lukeybird", "other");
        assert!(p.validate().is_err());
        p.id = "../../outside".into();
        assert!(p.validate().is_err());
    }
    #[test]
    fn discovery_refuses_downgrade_and_draft() {
        let p = embedded_catalog().unwrap().remove(0);
        let r = Release { tag_name: "v0.1.0".into(), draft: false, prerelease: false, assets: vec![] };
        assert!(candidate(&p, &r).unwrap().is_none());
    }
    #[test]
    fn validates_new_release_exact_url_and_digest() {
        let p = embedded_catalog().unwrap().remove(0);
        let mut r = Release {
            tag_name: "v0.20.6".into(),
            draft: false,
            prerelease: false,
            assets: vec![Asset {
                name: "pixashop-0.20.6-macos-universal.dmg".into(),
                browser_download_url: "https://github.com/lukeybird/pixashop/releases/download/v0.20.6/pixashop-0.20.6-macos-universal.dmg".into(),
                size: 100,
                digest: Some(format!("sha256:{}", "a".repeat(64))),
            }],
        };
        assert_eq!(candidate(&p, &r).unwrap().unwrap().version, "0.20.6");
        r.assets[0].browser_download_url = "https://example.com/test".into();
        assert!(candidate(&p, &r).is_err());
    }
}
