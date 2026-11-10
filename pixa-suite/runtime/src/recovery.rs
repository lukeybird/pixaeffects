//! A new, private recovery folder in a location explicitly chosen by the person. Never overwrite originals.
use std::{
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};
static NEXT: AtomicU64 = AtomicU64::new(0);
pub fn create_directory(parent: &Path, product: &str) -> Result<PathBuf, String> {
    if product.is_empty()
        || product.len() > 64
        || !product
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-')
    {
        return Err("Invalid recovery product identifier.".into());
    }
    for _ in 0..64 {
        let index = NEXT
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |n| n.checked_add(1))
            .map_err(|_| "Recovery name limit reached.")?;
        let path = parent.join(format!("{product}-recovery-{}-{index}", std::process::id()));
        let mut builder = std::fs::DirBuilder::new();
        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt;
            builder.mode(0o700);
        }
        match builder.create(&path) {
            Ok(()) => return Ok(path),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(_) => {
                return Err(
                    "Could not create a recovery folder. Choose a writable location.".into(),
                );
            }
        }
    }
    Err("Could not choose a new recovery folder name.".into())
}
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    #[test]
    fn recovery_uses_new_directories_and_rejects_path_injection() {
        let root = std::env::temp_dir();
        let first = create_directory(&root, "pixa-runtime-test").unwrap();
        let second = create_directory(&root, "pixa-runtime-test").unwrap();
        assert_ne!(first, second);
        assert!(create_directory(&root, "../outside").is_err());
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                std::fs::metadata(&first).unwrap().permissions().mode() & 0o777,
                0o700
            );
        }
        std::fs::remove_dir(first).unwrap();
        std::fs::remove_dir(second).unwrap();
    }
}
