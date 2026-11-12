#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod pixa_acceptance {
    use super::*;
    #[test]
    fn failed_rechecks_clear_an_old_update_offer() {
        for disconnected in [false, true] {
            let mut updates = Updates::default();
            updates.version = Some("0.2.0".into());
            let (tx, rx) = std::sync::mpsc::channel(); updates.checking = Some(rx);
            if !disconnected { tx.send(Err("Offline".into())).unwrap(); }
            drop(tx);
            updates.tick(&egui::Context::default());
            assert!(updates.version.is_none());
            assert_eq!(updates.message, if disconnected { "Update check stopped." } else { "Offline" });
        }
    }
}
#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod pixa_startup_acceptance {
    use super::*;
    #[test]
    fn a_background_release_offer_opens_the_updates_window() {
        let mut updates = Updates::default();
        assert!(!updates.checked_once);
        let (tx, rx) = std::sync::mpsc::channel(); updates.checking = Some(rx);
        tx.send(Ok(Some(pixa_runtime::release_feed::Release { version: "0.2.0".into() }))).unwrap();
        updates.tick(&egui::Context::default());
        assert!(updates.checked_once);
        assert!(updates.open);
        assert_eq!(updates.version.as_deref(), Some("0.2.0"));
    }
}
