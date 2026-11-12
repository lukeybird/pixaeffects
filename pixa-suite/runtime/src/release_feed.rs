//! Product-scoped publisher releases. Mac installation separately requires Apple publisher validation.
use semver::Version;
#[derive(Clone, Debug)]
pub struct Release {
    pub version: String,
}
pub fn valid_product(id: &str) -> bool {
    id.starts_with("pixa") && id.len() <= 32 && id.bytes().all(|b| b.is_ascii_lowercase())
}
pub fn latest(id: &str, current: &str, platform: &str) -> Result<Option<Release>, String> {
    if !valid_product(id) {
        return Err("Invalid Pixa product.".into());
    }
    let current = Version::parse(current).map_err(|_| "Invalid installed version.")?;
    let suffix = match platform {
        "macos" => "macos-universal.dmg",
        "windows" => "windows-x64-portable.zip",
        "linux" => "linux-x86_64.tar.gz",
        _ => return Err("Updates are not supported on this platform.".into()),
    };
    let agent = ureq::Agent::config_builder().timeout_global(Some(std::time::Duration::from_secs(20))).max_redirects(0).build().new_agent();
    let mut best: Option<Version> = None;
    for page in 1..=3 {
        let url = format!("https://api.github.com/repos/lukeybird/pixaeffects/releases?per_page=100&page={page}");
        let mut response = agent.get(&url).header("User-Agent", "Pixa desktop updater").call().map_err(|_| "Could not reach the Pixa release service.")?;
        let records: Vec<serde_json::Value> = response.body_mut().with_config().limit(4_000_000).read_json().map_err(|_| "Invalid Pixa release response.")?;
        select_release(&records, id, &current, suffix, &mut best);
        if records.len() < 100 {
            break;
        }
    }
    Ok(best.map(|v| Release { version: v.to_string() }))
}
fn select_release(records: &[serde_json::Value], id: &str, current: &Version, suffix: &str, best: &mut Option<Version>) {
    let prefix = format!("{id}-v");
    for record in records {
        if record["draft"].as_bool() != Some(false) || record["prerelease"].as_bool() != Some(false) {
            continue;
        }
        let Some(version) = record["tag_name"].as_str().and_then(|s| s.strip_prefix(&prefix)).and_then(|s| Version::parse(s).ok()) else {
            continue;
        };
        if !version.pre.is_empty() || !version.build.is_empty() || &version <= current || best.as_ref().is_some_and(|v| version <= *v) {
            continue;
        }
        let name = format!("{id}-{version}-{suffix}");
        let expected = format!("https://github.com/lukeybird/pixaeffects/releases/download/{id}-v{version}/{name}");
        let eligible = record["assets"].as_array().is_some_and(|assets| {
            assets.iter().any(|a| {
                a["name"].as_str() == Some(name.as_str())
                    && a["browser_download_url"].as_str() == Some(expected.as_str())
                    && a["size"].as_u64().is_some_and(|n| n > 0 && n <= 8 * 1024 * 1024 * 1024)
                    && a["digest"].as_str().and_then(|s| s.strip_prefix("sha256:")).is_some_and(|s| s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit()))
            })
        });
        if eligible {
            *best = Some(version);
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn record(id: &str, version: &str) -> serde_json::Value {
        let name = format!("{id}-{version}-macos-universal.dmg");
        serde_json::json!({"tag_name":format!("{id}-v{version}"),"draft":false,"prerelease":false,"assets":[{"name":name,"browser_download_url":format!("https://github.com/lukeybird/pixaeffects/releases/download/{id}-v{version}/{name}"),"size":100,"digest":format!("sha256:{}","a".repeat(64))}]})
    }
    #[test]
    fn ignores_other_products_and_nonstable_or_untrusted_assets() {
        let good = record("pixavector", "0.2.0");
        for bad in [
            record("pixafilm", "9.0.0"),
            record("pixavector", "0.1.0"),
            record("pixavector", "0.3.0-beta.1"),
            {
                let mut r = record("pixavector", "0.3.0");
                r["prerelease"] = true.into();
                r
            },
            {
                let mut r = record("pixavector", "0.3.0");
                r["assets"][0]["browser_download_url"] = "https://elsewhere.invalid/app.dmg".into();
                r
            },
            {
                let mut r = record("pixavector", "0.3.0");
                r["assets"][0]["digest"] = "".into();
                r
            },
            {
                let mut r = record("pixavector", "0.3.0");
                r["draft"] = true.into();
                r
            },
        ] {
            let mut best = None;
            select_release(&[good.clone(), bad], "pixavector", &Version::parse("0.1.0").unwrap(), "macos-universal.dmg", &mut best);
            assert_eq!(best.unwrap(), Version::parse("0.2.0").unwrap());
        }
    }
    #[test]
    fn latest_is_independent_of_api_order() {
        let mut best = None;
        select_release(
            &[record("pixavector", "0.5.0"), record("pixavector", "0.2.0")],
            "pixavector",
            &Version::parse("0.1.0").unwrap(),
            "macos-universal.dmg",
            &mut best,
        );
        assert_eq!(best.unwrap().to_string(), "0.5.0");
    }

    #[test]
    fn product_names_cannot_escape_the_feed() {
        for bad in ["../x", "PixaVector", "pixa/x", "pixavector?x", "pixa-vector"] {
            assert!(!super::valid_product(bad));
        }
        assert!(super::valid_product("pixavector"));
    }
}
