//! Serial, non-elevated Mac installs. No shell strings or administrator passwords.
use crate::catalog::Product;
use fs2::FileExt;
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicBool, Ordering},
};

pub const TEAM: &str = "945M5X4DY3";
#[derive(Clone, Debug)]
pub struct Installed {
    pub path: PathBuf,
    pub version: String,
}
pub fn applications() -> Result<PathBuf, String> {
    let home = std::env::var_os("HOME").ok_or("Home directory is unavailable")?;
    let path = PathBuf::from(home);
    if !path.is_absolute() {
        return Err("Home directory must be absolute".into());
    }
    Ok(path.join("Applications"))
}
fn bundle_info(path: &Path) -> Result<plist::Dictionary, String> {
    plist::Value::from_file(path.join("Contents/Info.plist"))
        .map_err(|e| format!("Cannot read app metadata: {e}"))?
        .into_dictionary()
        .ok_or_else(|| "Invalid app metadata".into())
}
fn field(info: &plist::Dictionary, key: &str) -> Result<String, String> {
    info.get(key).and_then(|v| v.as_string()).map(str::to_owned).ok_or_else(|| format!("Missing {key}"))
}
pub fn installed(p: &Product) -> Option<Installed> {
    let user = applications().ok()?;
    [user.join(format!("{}.app", p.name)), Path::new("/Applications").join(format!("{}.app", p.name))].into_iter().find_map(|path| {
        let i = bundle_info(&path).ok()?;
        if field(&i, "CFBundleIdentifier").ok()? != p.bundle_id() {
            return None;
        }
        Some(Installed { path, version: field(&i, "CFBundleShortVersionString").ok()? })
    })
}
fn run(program: &str, args: &[&std::ffi::OsStr]) -> Result<String, String> {
    let output = Command::new(program).args(args).output().map_err(|e| format!("Cannot run {program}: {e}"))?;
    if !output.status.success() {
        return Err(format!("{program} refused the operation: {}", String::from_utf8_lossy(&output.stderr).chars().take(1600).collect::<String>()));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().into())
}
/// Validate the installed executable before every Open as well as every install.
pub fn verify_app(path: &Path, p: &Product, exact_version: bool) -> Result<(), String> {
    if !cfg!(target_os = "macos") {
        return Err("Native installation and opening are available on macOS only".into());
    }
    let info = bundle_info(path)?;
    if field(&info, "CFBundleIdentifier")? != p.bundle_id() {
        return Err("App bundle identity does not match this product".into());
    }
    if exact_version && field(&info, "CFBundleShortVersionString")? != p.version {
        return Err("App version differs from the selected release".into());
    }
    let executable = field(&info, "CFBundleExecutable")?;
    if executable.contains('/') || executable == "." || executable == ".." {
        return Err("Invalid app executable name".into());
    }
    let bin = path.join("Contents/MacOS").join(executable);
    if exact_version {
        verify_universal(&bin)?;
    }
    let requirement = format!("anchor apple generic and identifier \"{}\" and certificate leaf[subject.OU] = \"{TEAM}\"", p.bundle_id());
    run("/usr/bin/codesign", &["--verify".as_ref(), "--deep".as_ref(), "--strict".as_ref(), "-R".as_ref(), requirement.as_ref(), path.as_os_str()])?;
    run("/usr/sbin/spctl", &["--assess".as_ref(), "--type".as_ref(), "execute".as_ref(), path.as_os_str()])?;
    if let Some(min) = info.get("LSMinimumSystemVersion").and_then(|v| v.as_string()) {
        let current = run("/usr/bin/sw_vers", &["-productVersion".as_ref()])?;
        fn os_parts(v: &str) -> Result<Vec<u64>, String> {
            let mut p: Vec<u64> = v.split('.').map(|n| n.parse().map_err(|_| "Invalid Mac version".to_string())).collect::<Result<_, _>>()?;
            p.resize(3, 0);
            Ok(p)
        }
        if os_parts(&current)? < os_parts(min)? {
            return Err(format!("This app requires macOS {min} or later"));
        }
    }
    Ok(())
}
/// Read the bounded fat Mach-O architecture table without requiring Xcode/lipo.
pub fn verify_universal(path: &Path) -> Result<(), String> {
    let mut file = File::open(path).map_err(|e| e.to_string())?;
    let mut header = [0u8; 8];
    file.read_exact(&mut header).map_err(|e| e.to_string())?;
    let magic = u32::from_be_bytes([header[0], header[1], header[2], header[3]]);
    let little = match magic {
        0xcafebabe | 0xcafebabf => false,
        0xbebafeca | 0xbfbafeca => true,
        _ => return Err("Release is not a universal Mach-O executable".into()),
    };
    let read = |b: [u8; 4]| if little { u32::from_le_bytes(b) } else { u32::from_be_bytes(b) };
    let count = read([header[4], header[5], header[6], header[7]]);
    if !(2..=20).contains(&count) {
        return Err("Invalid universal architecture count".into());
    }
    let width = if magic == 0xcafebabf || magic == 0xbfbafeca { 32 } else { 20 };
    let mut arm = false;
    let mut intel = false;
    for _ in 0..count {
        let mut record = vec![0u8; width];
        file.read_exact(&mut record).map_err(|e| e.to_string())?;
        let cpu = read([record[0], record[1], record[2], record[3]]);
        arm |= cpu == 0x0100000c;
        intel |= cpu == 0x01000007;
    }
    if arm && intel {
        Ok(())
    } else {
        Err("Release must contain both Apple Silicon and Intel architectures".into())
    }
}
pub fn open_app(p: &Product) -> Result<(), String> {
    let app = installed(p).ok_or("App not installed")?;
    verify_app(&app.path, p, false)?;
    run("/usr/bin/open", &[app.path.as_os_str()])?;
    Ok(())
}
pub fn open_url(url: &str) -> Result<(), String> {
    if !url.starts_with("https://aiwebdesignfirm.com/") {
        return Err("Unapproved website address".into());
    }
    if cfg!(target_os = "macos") {
        run("/usr/bin/open", &[url.as_ref()])?;
    } else if cfg!(target_os = "windows") {
        Command::new("rundll32").args(["url.dll,FileProtocolHandler", url]).spawn().map_err(|e| e.to_string())?;
    } else {
        Command::new("xdg-open").arg(url).spawn().map_err(|e| e.to_string())?;
    }
    Ok(())
}
pub fn ensure_not_running(path: &Path) -> Result<(), String> {
    let processes = run("/bin/ps", &["-axo".as_ref(), "comm=".as_ref()])?;
    let prefix = format!("{}/Contents/", path.display());
    if processes.lines().any(|line| line.trim().starts_with(&prefix)) {
        return Err("Quit this app before installing an update".into());
    }
    Ok(())
}
/// Streaming checksum and exact size; corrupt/oversized/canceled downloads never install.
pub fn copy_verified(mut input: impl Read, mut output: impl Write, p: &Product, cancel: &AtomicBool, mut progress: impl FnMut(u64)) -> Result<(), String> {
    let mut hash = Sha256::new();
    let mut total = 0u64;
    let mut buf = [0u8; 65536];
    loop {
        if cancel.load(Ordering::Relaxed) {
            return Err("Download canceled".into());
        }
        let count = input.read(&mut buf).map_err(|e| e.to_string())?;
        if count == 0 {
            break;
        }
        total = total.checked_add(count as u64).ok_or("Download size overflow")?;
        if total > p.size {
            return Err("Download exceeds the published size".into());
        }
        output.write_all(&buf[..count]).map_err(|e| format!("Could not save download: {e}"))?;
        hash.update(&buf[..count]);
        progress(total);
    }
    if total != p.size {
        return Err("Download is incomplete".into());
    }
    if hex::encode(hash.finalize()) != p.sha256.to_lowercase() {
        return Err("Download checksum does not match the published release".into());
    }
    output.flush().map_err(|e| e.to_string())?;
    Ok(())
}
struct Mounted(PathBuf);
impl Drop for Mounted {
    fn drop(&mut self) {
        let _ = Command::new("/usr/bin/hdiutil").args(["detach", "-quiet"]).arg(&self.0).status();
    }
}
/// Transaction core: staging and previous app are on the destination filesystem.
/// On a failed promotion the prior app is restored, never silently discarded.
pub fn promote(staged: &Path, destination: &Path, backup: &Path) -> Result<(), String> {
    if destination.is_symlink() || backup.exists() {
        return Err("Refusing an unsafe install destination or existing backup".into());
    }
    let previous = destination.exists();
    if previous {
        fs::rename(destination, backup).map_err(|e| format!("Cannot preserve previous app: {e}"))?;
    }
    if let Err(e) = fs::rename(staged, destination) {
        if previous {
            fs::rename(backup, destination)
                .map_err(|rollback| format!("Install failed ({e}). Restore {} to {} manually: {rollback}", backup.display(), destination.display()))?;
        }
        return Err(format!("Install failed; previous app preserved: {e}"));
    }
    Ok(())
}
pub fn install(p: &Product, cancel: &AtomicBool, mut status: impl FnMut(String, u64)) -> Result<String, String> {
    p.validate()?;
    if !cfg!(target_os = "macos") {
        return Err("Mac installation requires macOS. Use the product website on this platform.".into());
    }
    let apps = applications()?;
    fs::create_dir_all(&apps).map_err(|e| e.to_string())?;
    if apps.is_symlink() {
        return Err("Applications folder must not be a symbolic link".into());
    }
    let lock =
        OpenOptions::new().read(true).write(true).create(true).truncate(false).open(apps.join(".pixa-launcher-install.lock")).map_err(|e| e.to_string())?;
    lock.try_lock_exclusive().map_err(|_| "Another Pixa launcher installation is in progress".to_string())?;
    let destination = apps.join(format!("{}.app", p.name));
    if destination.exists() {
        verify_app(&destination, p, false)?;
        ensure_not_running(&destination)?;
        let version = field(&bundle_info(&destination)?, "CFBundleShortVersionString")?;
        if crate::catalog::compare_versions(&version, &p.version)? == std::cmp::Ordering::Greater {
            return Err("A newer app is already installed; downgrades are refused".into());
        }
    }
    if fs2::available_space(&apps).map_err(|e| e.to_string())? < p.size.saturating_mul(6) {
        return Err("Not enough disk space. Free space yourself, then retry; no files were removed.".into());
    }
    let work = tempfile::Builder::new().prefix(".pixa-install-").tempdir_in(&apps).map_err(|e| e.to_string())?;
    let dmg = work.path().join("download.dmg");
    let mut file = File::create(&dmg).map_err(|e| e.to_string())?;
    status("Downloading".into(), 0);
    let response = crate::catalog::client()?.get(&p.url).send().and_then(|r| r.error_for_status()).map_err(|e| format!("Download failed: {e}"))?;
    if response.content_length().is_some_and(|s| s != p.size) {
        return Err("Server download size differs from the selected release".into());
    }
    copy_verified(response, &mut file, p, cancel, |n| status("Downloading".into(), n))?;
    file.sync_all().map_err(|e| e.to_string())?;
    drop(file);
    if cancel.load(Ordering::Relaxed) {
        return Err("Download canceled".into());
    }
    status("Checking Apple verification".into(), p.size);
    let mount = work.path().join("mounted");
    fs::create_dir(&mount).map_err(|e| e.to_string())?;
    run(
        "/usr/bin/hdiutil",
        &["attach".as_ref(), "-readonly".as_ref(), "-nobrowse".as_ref(), "-quiet".as_ref(), "-mountpoint".as_ref(), mount.as_os_str(), dmg.as_os_str()],
    )?;
    let mounted = Mounted(mount.clone());
    let source = mount.join(format!("{}.app", p.name));
    verify_app(&source, p, true)?;
    let staging = work.path().join(format!("{}.app", p.name));
    status("Installing".into(), p.size);
    run("/usr/bin/ditto", &[source.as_os_str(), staging.as_os_str()])?;
    verify_app(&staging, p, true)?;
    drop(mounted);
    if cancel.load(Ordering::Relaxed) {
        return Err("Installation canceled before replacing the app".into());
    }
    if destination.exists() {
        ensure_not_running(&destination)?;
        verify_app(&destination, p, false)?;
    }
    let backup_dir = tempfile::Builder::new().prefix(".pixa-previous-").tempdir_in(&apps).map_err(|e| e.to_string())?;
    let backup = backup_dir.path().join(format!("{}.app", p.name));
    promote(&staging, &destination, &backup)?;
    if backup.exists() {
        let kept = backup_dir.keep();
        Ok(format!("Installed {} {}. Previous app preserved at {}. Your documents are unchanged.", p.name, p.version, kept.display()))
    } else {
        Ok(format!("Installed {} {} in {}", p.name, p.version, destination.display()))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn product(bytes: &[u8]) -> Product {
        let mut p = crate::catalog::embedded_catalog().unwrap().remove(0);
        p.size = bytes.len() as u64;
        p.sha256 = hex::encode(Sha256::digest(bytes));
        p
    }
    #[test]
    fn exact_stream_passes_corrupt_short_large_and_cancel_refused() {
        let p = product(b"hello");
        let cancel = AtomicBool::new(false);
        assert!(copy_verified(&b"hello"[..], Vec::new(), &p, &cancel, |_| {}).is_ok());
        for b in [&b"wrong"[..], &b"hell"[..], &b"helloo"[..]] {
            assert!(copy_verified(b, Vec::new(), &p, &cancel, |_| {}).is_err());
        }
        cancel.store(true, Ordering::Relaxed);
        assert!(copy_verified(&b"hello"[..], Vec::new(), &p, &cancel, |_| {}).is_err());
    }
    #[test]
    fn replacement_keeps_previous_and_failed_promotion_restores_it() {
        let t = tempfile::tempdir().unwrap();
        let dest = t.path().join("App");
        let backup = t.path().join("Previous");
        fs::write(&dest, "original").unwrap();
        assert!(promote(&t.path().join("missing"), &dest, &backup).is_err());
        assert_eq!(fs::read_to_string(&dest).unwrap(), "original");
        let stage = t.path().join("new");
        fs::write(&stage, "updated").unwrap();
        promote(&stage, &dest, &backup).unwrap();
        assert_eq!(fs::read_to_string(&dest).unwrap(), "updated");
        assert_eq!(fs::read_to_string(&backup).unwrap(), "original");
    }
    #[test]
    fn universal_header_requires_both_cpus_and_refuses_truncation() {
        let t = tempfile::tempdir().unwrap();
        let p = t.path().join("binary");
        let mut bytes = Vec::new();
        bytes.extend(0xcafebabeu32.to_be_bytes());
        bytes.extend(2u32.to_be_bytes());
        for cpu in [0x01000007u32, 0x0100000cu32] {
            bytes.extend(cpu.to_be_bytes());
            bytes.extend([0u8; 16]);
        }
        fs::write(&p, &bytes).unwrap();
        assert!(verify_universal(&p).is_ok());
        fs::write(&p, &bytes[..12]).unwrap();
        assert!(verify_universal(&p).is_err());
        bytes[28..32].copy_from_slice(&0x01000007u32.to_be_bytes());
        fs::write(&p, &bytes).unwrap();
        assert!(verify_universal(&p).is_err());
    }
    #[test]
    fn existing_backup_is_never_overwritten() {
        let t = tempfile::tempdir().unwrap();
        let backup = t.path().join("previous");
        fs::write(&backup, "keep").unwrap();
        assert!(promote(&t.path().join("new"), &t.path().join("app"), &backup).is_err());
        assert_eq!(fs::read_to_string(backup).unwrap(), "keep");
    }
}
