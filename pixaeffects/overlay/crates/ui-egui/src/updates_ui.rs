//! Publisher-owned release checks and verified Mac installation.
use crate::EffectcraftApp;
#[derive(Default)]
pub(crate) struct Runtime {
    startup_check: bool,
    checked: bool,
    open: bool,
    message: String,
    version: Option<String>,
    #[cfg(not(target_arch = "wasm32"))]
    checking: Option<std::sync::mpsc::Receiver<Result<Option<String>, String>>>,
    #[cfg(target_os = "macos")]
    installing: Option<std::sync::mpsc::Receiver<Result<crate::pixaeffects_updater::Prepared, String>>>,
    #[cfg(target_os = "macos")]
    ready: Option<crate::pixaeffects_updater::Prepared>,
}
impl Runtime {
    fn finish_check(&mut self, result: Result<Option<String>, String>) {
        match result {
            Ok(Some(version)) => {
                self.message = format!("PixaEffects {version} is available.");
                self.version = Some(version);
                self.open = true;
            }
            Ok(None) => {
                self.version = None;
                self.message = "You have the latest published PixaEffects version.".into();
            }
            Err(error) => {
                self.version = None;
                self.message = error;
            }
        }
    }
}

impl EffectcraftApp {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn set_startup_update_check(&mut self, on: bool) {
        self.updates_rt.startup_check = on;
    }
}
pub(crate) fn check(app: &mut EffectcraftApp) {
    let rt = &mut app.updates_rt;
    rt.open = true;
    rt.checked = true;
    // Help-menu checks must not hide an installation that already owns a verified candidate.
    #[cfg(target_os = "macos")]
    if rt.installing.is_some() || rt.ready.is_some() {
        return;
    }
    rt.version = None;
    #[cfg(not(target_arch = "wasm32"))]
    {
        if rt.checking.is_some() {
            return;
        }
        rt.message = "Checking for PixaEffects updates…".into();
        let (tx, rx) = std::sync::mpsc::channel();
        match std::thread::Builder::new().name("PixaEffects release check".into()).spawn(move || {
            let _ = tx.send(latest());
        }) {
            Ok(_) => rt.checking = Some(rx),
            Err(e) => rt.message = format!("Could not check for updates: {e}"),
        }
    }
    #[cfg(target_arch = "wasm32")]
    {
        rt.message = "Reload the website for browser updates.".into();
    }
}
#[cfg(not(target_arch = "wasm32"))]
fn latest() -> Result<Option<String>, String> {
    let agent = ureq::Agent::config_builder().timeout_global(Some(std::time::Duration::from_secs(20))).max_redirects(0).build().new_agent();
    let mut response = agent
        .get("https://api.github.com/repos/lukeybird/pixaeffects/releases/latest")
        .header("User-Agent", "PixaEffects")
        .call()
        .map_err(|_| "No published release could be checked. Try again later.".to_string())?;
    let data: serde_json::Value = response.body_mut().with_config().limit(1_000_000).read_json().map_err(|_| "Invalid release metadata".to_string())?;
    if data["draft"].as_bool() != Some(false) || data["prerelease"].as_bool() != Some(false) {
        return Err("No stable release is available".into());
    }
    let version = data["tag_name"]
        .as_str()
        .and_then(|s| s.strip_prefix('v'))
        .filter(|s| crate::pixaeffects_updater::valid_version(s))
        .ok_or("Invalid release version")?;
    let parse = |s: &str| s.split('.').map(str::parse::<u64>).collect::<Result<Vec<_>, _>>();
    let newer = parse(version).map_err(|_| "Invalid version")? > parse(env!("CARGO_PKG_VERSION")).map_err(|_| "Invalid app version")?;
    Ok(newer.then(|| version.to_owned()))
}
pub(crate) fn tick(app: &mut EffectcraftApp, ctx: &egui::Context) {
    if app.updates_rt.startup_check && !app.updates_rt.checked {
        check(app);
        app.updates_rt.open = false;
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let rt = &mut app.updates_rt;
        if let Some(rx) = &rt.checking {
            let result = match rx.try_recv() {
                Ok(v) => Some(v),
                Err(std::sync::mpsc::TryRecvError::Empty) => None,
                Err(_) => Some(Err("Release check stopped".into())),
            };
            if let Some(result) = result {
                rt.checking = None;
                rt.finish_check(result);
            }
            ctx.request_repaint_after(std::time::Duration::from_millis(250));
        }
    }
    let _ = ctx;
}
pub(crate) fn show(app: &mut EffectcraftApp, ctx: &egui::Context) {
    let clean = !app.session.is_dirty();
    #[cfg(target_os = "macos")]
    {
        let rt = &mut app.updates_rt;
        if let Some(rx) = &rt.installing {
            let result = match rx.try_recv() {
                Ok(v) => Some(v),
                Err(std::sync::mpsc::TryRecvError::Empty) => None,
                Err(_) => Some(Err("Installer stopped".into())),
            };
            if let Some(result) = result {
                rt.installing = None;
                match result {
                    Ok(v) => rt.ready = Some(v),
                    Err(e) => rt.message = e,
                }
            }
            ctx.request_repaint_after(std::time::Duration::from_millis(250));
        }
        if clean {
            if let Some(ready) = rt.ready.take() {
                match crate::pixaeffects_updater::restart(ready) {
                    Ok(()) => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
                    Err(e) => rt.message = e,
                }
            }
        }
    }
    if !app.updates_rt.open {
        return;
    }
    let mut open = true;
    let mut again = false;
    #[cfg(target_os = "macos")]
    let mut install = None;
    egui::Window::new("PixaEffects updates").open(&mut open).show(ctx, |ui| {
        ui.label("by AIWebDesignFirm");
        let rt = &app.updates_rt;
        if let Some(version) = &rt.version {
            ui.heading(format!("Update available — {version}"));
            #[cfg(target_os = "macos")]
            {
                let busy = rt.installing.is_some() || rt.ready.is_some();
                if !clean {
                    ui.label("Save your project before installing and restarting.");
                }
                if busy {
                    ui.spinner();
                    ui.label("Downloading and verifying publisher signature…");
                }
                if ui.add_enabled(clean && !busy, egui::Button::new("Install and restart")).clicked() {
                    install = Some(version.clone());
                }
            }
            #[cfg(not(target_os = "macos"))]
            if ui.button("Download update").clicked() {
                ctx.open_url(egui::OpenUrl::new_tab("https://aiwebdesignfirm.com/pixaeffects"));
            }
        }
        // Verification/download failures must remain visible when an update is available.
        ui.label(&rt.message);
        #[cfg(target_os = "macos")]
        let busy = rt.installing.is_some() || rt.ready.is_some();
        #[cfg(not(target_os = "macos"))]
        let busy = false;
        if ui.add_enabled(!busy, egui::Button::new("Check again")).clicked() {
            again = true;
        }
    });
    #[cfg(target_os = "macos")]
    if let Some(version) = install {
        let (tx, rx) = std::sync::mpsc::channel();
        match std::thread::Builder::new().name("PixaEffects installer".into()).spawn(move || {
            let _ = tx.send(crate::pixaeffects_updater::prepare(&version));
        }) {
            Ok(_) => app.updates_rt.installing = Some(rx),
            Err(e) => app.updates_rt.message = e.to_string(),
        }
    }
    let _ = clean;
    app.updates_rt.open = open;
    if again {
        check(app);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn available_release_does_not_claim_latest() {
        let mut runtime = Runtime::default();
        runtime.finish_check(Ok(Some("0.1.3".into())));
        assert_eq!(runtime.version.as_deref(), Some("0.1.3"));
        assert!(runtime.open);
        assert_eq!(runtime.message, "PixaEffects 0.1.3 is available.");
    }

    #[test]
    fn failed_or_empty_recheck_clears_stale_offer() {
        let mut runtime = Runtime::default();
        runtime.finish_check(Ok(Some("0.1.3".into())));
        runtime.finish_check(Err("Offline".into()));
        assert!(runtime.version.is_none());
        assert_eq!(runtime.message, "Offline");
        runtime.finish_check(Ok(Some("0.1.3".into())));
        runtime.finish_check(Ok(None));
        assert!(runtime.version.is_none());
        assert!(runtime.message.contains("latest published"));
    }
    #[test]
    fn verification_error_is_visible_beside_update_offer() {
        use egui_kittest::{Harness, kittest::Queryable};
        let mut app = EffectcraftApp::new(effectcraft_engine::Session::default());
        app.updates_rt.open = true;
        app.updates_rt.version = Some("0.1.3".into());
        app.updates_rt.message = "The update checksum does not match.".into();
        let mut harness = Harness::builder().with_size(egui::vec2(900.0, 700.0)).build_ui_state(|ui, app| show(app, ui.ctx()), app);
        harness.run_steps(3);
        harness.get_by_label("The update checksum does not match.");
        harness.get_by_label("Update available — 0.1.3");
    }
}
