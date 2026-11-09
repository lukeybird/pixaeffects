//! PixaEffects desktop activation. Passwords and card details stay in the system browser.
//! Tokens grant a short online lease; a local clock change cannot extend one.
use crate::Desktop;
use egui::{Context, Ui};
use serde::{Deserialize, Serialize};
use std::{
    path::PathBuf,
    sync::mpsc,
    time::{Duration, Instant},
};

// Desktop automation stays disabled until it has its own entitlement enforcement.
pub(crate) fn automation_enabled() -> bool {
    false
}

const API: &str = "https://aiwebdesignfirm.com/api/pixaeffects/device";
const ACTIVATE: &str = "https://aiwebdesignfirm.com/pixaeffects/activate";

#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Answer {
    #[serde(default)]
    token: String,
    #[serde(default)]
    code: String,
    #[serde(default)]
    access: bool,
    #[serde(default)]
    access_seconds: u64,
    #[serde(default)]
    message: String,
}
#[derive(Serialize, Deserialize)]
struct Stored {
    token: String,
}
type Pending = mpsc::Receiver<Result<Answer, String>>;

pub(crate) struct AccessApp {
    inner: Desktop,
    token: String,
    code: String,
    message: String,
    allowed_until: Option<Instant>,
    next_check: Instant,
    pending: Option<Pending>,
}
impl AccessApp {
    pub(crate) fn new(inner: Desktop) -> Self {
        let token = token_path()
            .and_then(|p| std::fs::read_to_string(p).ok())
            .and_then(|s| serde_json::from_str::<Stored>(&s).ok())
            .filter(|s| valid_token(&s.token))
            .map(|s| s.token)
            .unwrap_or_default();
        let mut app = Self {
            inner,
            token,
            code: String::new(),
            message: "Checking your PixaEffects access…".into(),
            allowed_until: None,
            next_check: Instant::now(),
            pending: None,
        };
        app.start();
        app
    }
    fn allowed(&self) -> bool {
        self.allowed_until.is_some_and(|until| Instant::now() < until)
    }
    fn start(&mut self) {
        if self.pending.is_some() {
            return;
        }
        let token = self.token.clone();
        let (tx, rx) = mpsc::channel();
        let spawned = std::thread::Builder::new().name("PixaEffects activation".into()).spawn(move || {
            let answer = request(&token);
            let _ = tx.send(answer);
        });
        match spawned {
            Ok(_) => self.pending = Some(rx),
            Err(_) => self.message = "Could not start the access check. Please retry.".into(),
        }
        self.next_check = Instant::now() + Duration::from_secs(10);
    }
    fn tick(&mut self) {
        if let Some(rx) = &self.pending {
            let answer = match rx.try_recv() {
                Ok(v) => Some(v),
                Err(mpsc::TryRecvError::Empty) => None,
                Err(mpsc::TryRecvError::Disconnected) => Some(Err("The access check stopped. Please retry.".into())),
            };
            if let Some(answer) = answer {
                self.pending = None;
                match answer {
                    Ok(a) => {
                        if valid_token(&a.token) {
                            self.token = a.token;
                            if let Err(error) = store_token(&self.token) {
                                self.message = error;
                            }
                        }
                        if !a.code.is_empty() {
                            self.code = a.code;
                        }
                        if a.access {
                            self.code.clear();
                        }
                        self.allowed_until = lease(a.access, a.access_seconds, Instant::now());
                        self.message = if a.access {
                            "Your PixaEffects access is active.".into()
                        } else if a.message.is_empty() {
                            "Sign in with an active trial or subscription to continue.".into()
                        } else {
                            a.message
                        };
                        self.next_check = Instant::now() + Duration::from_secs(if a.access { 240 } else { 10 });
                    }
                    Err(error) => {
                        self.message = error;
                        self.next_check = Instant::now() + Duration::from_secs(30);
                    }
                }
            }
        }
        if Instant::now() >= self.next_check {
            self.start();
        }
    }
    fn locked_ui(&mut self, ui: &mut Ui) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(70.0);
                ui.heading("PixaEffects");
                ui.label("by AIWebDesignFirm");
                ui.add_space(24.0);
                ui.label("Your creative workspace, connected to your account.");
                ui.label(&self.message);
                if !self.code.is_empty() {
                    ui.add_space(18.0);
                    ui.monospace(&self.code);
                    ui.label("Open your account and enter this code. Only approve codes shown on your own device.");
                    if ui.button("Activate this device").clicked() {
                        ui.ctx().open_url(egui::OpenUrl::new_tab(ACTIVATE));
                    }
                }
                if ui.button("Start your 7-day trial / Manage billing").clicked() {
                    ui.ctx().open_url(egui::OpenUrl::new_tab("https://aiwebdesignfirm.com/start"));
                }
                if ui.button("Check again").clicked() {
                    self.start();
                }
                if ui.button("Use another account").clicked() {
                    self.token.clear();
                    self.code.clear();
                    self.allowed_until = None;
                    if let Some(path) = token_path() {
                        let _ = std::fs::remove_file(path);
                    }
                    // Discard a pending old-account response before creating a new pairing.
                    self.pending = None;
                    self.start();
                }
                ui.add_space(24.0);
                ui.small(
                    "An internet connection is required. Your projects and footage stay on your computer; access checks send only this app's device token.",
                );
                ui.small("Open documents remain in memory while access is checked. Your local files are not removed.");
                if self.inner.app.session.is_dirty() && ui.button("Save current project").clicked() {
                    let _ = effectcraft_ui_egui::menus::invoke(&mut self.inner.app, ui.ctx(), "file.save", serde_json::json!({}));
                }
            });
        });
    }
}
impl eframe::App for AccessApp {
    fn logic(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
        self.tick();
        // Continue saves/background jobs even if a lease ends, so work is not lost.
        if self.allowed() {
            self.inner.logic(ctx, frame);
        }
        ctx.request_repaint_after(Duration::from_millis(250));
    }
    fn ui(&mut self, ui: &mut Ui, frame: &mut eframe::Frame) {
        egui::Panel::bottom("pixaeffects-publisher").exact_size(24.0).show(ui, |ui| {
            ui.horizontal_centered(|ui| {
                ui.small("by AIWebDesignFirm");
            });
        });
        if self.allowed() {
            self.inner.ui(ui, frame);
        } else {
            self.locked_ui(ui);
        }
    }
    fn raw_input_hook(&mut self, ctx: &Context, raw: &mut egui::RawInput) {
        if self.allowed() {
            self.inner.raw_input_hook(ctx, raw);
        }
    }
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        self.inner.save(storage);
    }
    fn on_exit(&mut self) {
        self.inner.on_exit();
    }
}
fn valid_token(token: &str) -> bool {
    token.len() == 64 && token.bytes().all(|b| b.is_ascii_hexdigit())
}
fn lease(access: bool, seconds: u64, now: Instant) -> Option<Instant> {
    if access && seconds > 0 { now.checked_add(Duration::from_secs(seconds.min(300))) } else { None }
}
fn token_path() -> Option<PathBuf> {
    effectcraft_host::config_dir().map(|p| p.join("pixaeffects-device.json"))
}
fn store_token(token: &str) -> Result<(), String> {
    let path = token_path().ok_or("Your device could not save its activation. Sign in again next launch.")?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|_| "Could not create the activation folder.")?;
    }
    let bytes = serde_json::to_vec(&Stored { token: token.into() }).map_err(|_| "Could not save activation.")?;
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(&path).map_err(|_| "Could not save activation. Check folder permissions.")?;
    use std::io::Write;
    file.write_all(&bytes).map_err(|_| "Could not finish saving activation.".to_owned())
}
fn request(token: &str) -> Result<Answer, String> {
    let agent = ureq::Agent::config_builder().timeout_global(Some(Duration::from_secs(15))).max_redirects(0).build().new_agent();
    let body = if token.is_empty() { serde_json::json!({"action":"begin"}) } else { serde_json::json!({"action":"check","token":token}) };
    let mut response = agent.post(API).send_json(body).map_err(|_| "Could not verify access. Check your connection and retry.")?;
    let a: Answer = response.body_mut().with_config().limit(16_384).read_json().map_err(|_| "The access service returned an invalid response.")?;
    Ok(a)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn untrusted_duration_cannot_extend_a_lease() {
        let now = Instant::now();
        assert_eq!(lease(true, u64::MAX, now), now.checked_add(Duration::from_secs(300)));
        assert!(lease(false, 300, now).is_none());
        assert!(lease(true, 0, now).is_none());
    }
    #[test]
    fn tokens_are_bounded() {
        assert!(valid_token(&"a".repeat(64)));
        assert!(!valid_token("hello"));
        assert!(!valid_token(&"z".repeat(64)));
    }
}
