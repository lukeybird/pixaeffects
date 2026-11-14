//! Opt-in network acceptance; never mounts, installs or launches an editor.
use pixa_launcher::{catalog, installer};
use std::sync::atomic::AtomicBool;
#[test]
#[ignore = "downloads a public release; run explicitly for network acceptance"]
fn published_catalog_and_streaming_download() {
    let products = catalog::refresh(&catalog::embedded_catalog().unwrap()).unwrap();
    assert_eq!(products.len(), 16);
    let p = products.iter().min_by_key(|p| p.size).unwrap();
    let response = catalog::client().unwrap().get(&p.url).send().unwrap().error_for_status().unwrap();
    let mut destination = tempfile::tempfile().unwrap();
    installer::copy_verified(response, &mut destination, p, &AtomicBool::new(false), |_| {}).unwrap();
    assert_eq!(destination.metadata().unwrap().len(), p.size);
    println!("Verified {} {}: {} bytes, SHA-256 {}", p.name, p.version, p.size, p.sha256);
}
