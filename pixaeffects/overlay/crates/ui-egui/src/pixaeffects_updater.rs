//! PixaEffects's macOS updater. Downloaded code is verified before the running app is touched.
#[cfg(target_os = "macos")]
mod mac {
    use std::{
        path::{Path, PathBuf},
        process::{Command, Output},
    };
    const TEAM: &str = "945M5X4DY3";
    const BUNDLE: &str = "com.aiwebdesignfirm.pixaeffects";
    pub struct Prepared {
        root: PathBuf,
        target: PathBuf,
    }
    impl Drop for Prepared {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.root);
        }
    }
    fn run(cmd: &mut Command) -> Result<Output, String> {
        let out = cmd.output().map_err(|e| format!("Could not start updater tool: {e}"))?;
        if !out.status.success() {
            return Err("Update verification failed. Your installed app has not been changed.".into());
        }
        Ok(out)
    }
    fn curl(url: &str, destination: &Path, limit: u64) -> Result<(), String> {
        run(Command::new("/usr/bin/curl")
            .args([
                "--fail",
                "--silent",
                "--show-error",
                "--location",
                "--proto",
                "=https",
                "--proto-redir",
                "=https",
                "--connect-timeout",
                "30",
                "--max-time",
                "900",
                "--max-filesize",
                &limit.to_string(),
                "--output",
            ])
            .arg(destination)
            .arg(url))?;
        Ok(())
    }
    fn installed() -> Result<PathBuf, String> {
        let exe = std::env::current_exe().map_err(|e| e.to_string())?;
        let app = exe.ancestors().find(|p| p.extension().is_some_and(|e| e == "app")).ok_or("Move PixaEffects into Applications before updating.")?;
        let parent = app.parent().ok_or("Cannot locate Applications folder.")?;
        let user_apps = std::env::var_os("HOME").map(PathBuf::from).map(|p| p.join("Applications"));
        if app.file_name().is_none_or(|n| n != "PixaEffects.app")
            || (parent != Path::new("/Applications") && user_apps.as_deref() != Some(parent))
            || std::fs::symlink_metadata(app).map_err(|e| e.to_string())?.file_type().is_symlink()
        {
            return Err("Move PixaEffects into Applications before updating.".into());
        }
        Ok(app.to_path_buf())
    }
    fn verify(app: &Path, version: Option<&str>) -> Result<(), String> {
        run(Command::new("/usr/bin/codesign").args(["--verify", "--deep", "--strict"]).arg(app))?;
        let info = run(Command::new("/usr/bin/codesign").args(["--display", "--verbose=4"]).arg(app))?;
        let text = String::from_utf8_lossy(&info.stderr);
        if !text.lines().any(|l| l == format!("TeamIdentifier={TEAM}"))
            || !text.lines().any(|l| l == format!("Identifier={BUNDLE}"))
            || !text.contains("runtime")
        {
            return Err("The update is not signed by the PixaEffects publisher.".into());
        }
        run(Command::new("/usr/sbin/spctl").args(["--assess", "--type", "execute"]).arg(app))?;
        if let Some(version) = version {
            let out = run(Command::new("/usr/libexec/PlistBuddy").args(["-c", "Print :CFBundleShortVersionString"]).arg(app.join("Contents/Info.plist")))?;
            if String::from_utf8_lossy(&out.stdout).trim() != version {
                return Err("The update version does not match the release.".into());
            }
        }
        Ok(())
    }
    pub fn prepare(version: &str) -> Result<Prepared, String> {
        if !super::valid_version(version) {
            return Err("Invalid update version.".into());
        }
        let target = installed()?;
        verify(&target, None)?;
        // A private sibling directory keeps the staged bundle on the destination filesystem.
        let template = target.parent().ok_or("Missing application folder")?.join(".pixaeffects-update.XXXXXXXX");
        let out = run(Command::new("/usr/bin/mktemp").arg("-d").arg(template))?;
        let root = PathBuf::from(String::from_utf8_lossy(&out.stdout).trim());
        let prepared = Prepared { root, target };
        let metadata = prepared.root.join("release.json");
        curl(&format!("https://api.github.com/repos/lukeybird/pixaeffects/releases/tags/v{version}"), &metadata, 4_000_000)?;
        let data: serde_json::Value = serde_json::from_slice(&std::fs::read(metadata).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
        let tag = format!("v{version}");
        if data["draft"].as_bool() != Some(false) || data["tag_name"].as_str() != Some(tag.as_str()) {
            return Err("This update has not been published.".into());
        }
        let name = format!("pixaeffects-{version}-macos-universal.dmg");
        let asset = data["assets"]
            .as_array()
            .and_then(|a| a.iter().find(|a| a["name"].as_str() == Some(name.as_str())))
            .ok_or("No verified Mac installer is published for this version yet.")?;
        let digest = asset["digest"]
            .as_str()
            .and_then(|d| d.strip_prefix("sha256:"))
            .filter(|d| d.len() == 64 && d.bytes().all(|b| b.is_ascii_hexdigit()))
            .ok_or("The release is missing its integrity checksum.")?;
        let url = format!("https://github.com/lukeybird/pixaeffects/releases/download/v{version}/{name}");
        if asset["browser_download_url"].as_str() != Some(url.as_str()) {
            return Err("Unexpected update source.".into());
        }
        let dmg = prepared.root.join("update.dmg");
        curl(&url, &dmg, 1_000_000_000)?;
        let sum = run(Command::new("/usr/bin/shasum").arg("-a").arg("256").arg(&dmg))?;
        if String::from_utf8_lossy(&sum.stdout).split_whitespace().next() != Some(digest) {
            return Err("The update checksum does not match.".into());
        }
        let mount = prepared.root.join("mounted");
        std::fs::create_dir(&mount).map_err(|e| e.to_string())?;
        run(Command::new("/usr/bin/hdiutil").args(["attach", "-readonly", "-nobrowse", "-mountpoint"]).arg(&mount).arg(&dmg))?;
        let result = (|| {
            let app = mount.join("PixaEffects.app");
            if std::fs::symlink_metadata(&app).map_err(|e| e.to_string())?.file_type().is_symlink() {
                return Err("Unexpected application link.".into());
            }
            verify(&app, Some(version))?;
            let staged = prepared.root.join("PixaEffects.app");
            run(Command::new("/usr/bin/ditto").arg(&app).arg(&staged))?;
            verify(&staged, Some(version))
        })();
        let detached = run(Command::new("/usr/bin/hdiutil").arg("detach").arg(&mount));
        result?;
        detached?;
        Ok(prepared)
    }
    pub fn restart(prepared: Prepared) -> Result<(), String> {
        let helper = prepared.root.join("restart.sh");
        std::fs::write(&helper, include_str!("pixaeffects_restart.sh")).map_err(|e| e.to_string())?;
        let log = std::fs::File::create(prepared.root.join("install.log")).map_err(|e| e.to_string())?;
        Command::new("/bin/sh")
            .arg(&helper)
            .arg(std::process::id().to_string())
            .arg(&prepared.root)
            .arg(&prepared.target)
            .stdin(std::process::Stdio::null())
            .stdout(log.try_clone().map_err(|e| e.to_string())?)
            .stderr(log)
            .spawn()
            .map_err(|e| format!("Could not schedule restart: {e}"))?;
        // The helper now owns cleanup, including rollback on a failed replacement.
        std::mem::forget(prepared);
        Ok(())
    }
}

pub(crate) fn valid_version(version: &str) -> bool {
    let parts: Vec<_> = version.split('.').collect();
    parts.len() == 3 && parts.iter().all(|s| !s.is_empty() && s.len() <= 9 && s.bytes().all(|b| b.is_ascii_digit()))
}
#[cfg(target_os = "macos")]
pub use mac::{Prepared, prepare, restart};
#[cfg(test)]
mod tests {
    #[test]
    fn release_tags_cannot_escape_the_publisher_url() {
        assert!(super::valid_version("0.20.2"));
        for bad in ["", "../x", "1.2.3/x", "1.2.3;open", "v1.2.3", "1.2", "1.2.3-beta", "1.2.3\n"] {
            assert!(!super::valid_version(bad));
        }
    }
}
