//! Development account connection. No production entitlement enforcement is claimed by this UI.
use eframe::egui;
use pixa_runtime::account::{self, Account, Answer};
use std::{sync::mpsc, time::Instant};
#[derive(Default)]
pub struct AccountUi {
    pub open: bool,
    state: Account,
    pending: Option<mpsc::Receiver<(u64, Result<Answer, String>)>>,
    message: String,
}
impl AccountUi {
    fn start(&mut self) {
        if self.pending.is_some() { return; }
        let token = self.state.token(); let generation = self.state.generation();
        let (tx, rx) = mpsc::channel();
        match std::thread::Builder::new().name("Pixa account check".into()).spawn(move || {
            let _ = tx.send((generation, account::check(&token)));
        }) {
            Ok(_) => { self.pending = Some(rx); self.message = "Checking your account…".into(); }
            Err(_) => self.message = "Could not start the account check. Please retry.".into(),
        }
    }
    pub fn tick(&mut self) {
        let result = self.pending.as_ref().and_then(|rx| match rx.try_recv() {
            Ok(v) => Some(v),
            Err(mpsc::TryRecvError::Empty) => None,
            Err(mpsc::TryRecvError::Disconnected) => Some((self.state.generation(), Err("The account check stopped. Please retry.".into()))),
        });
        if let Some((generation, result)) = result {
            self.pending = None;
            self.message = match result.and_then(|answer| self.state.apply(generation, answer, Instant::now())) {
                Ok(()) if self.state.allowed(Instant::now()) => "Your account reports an active trial or subscription.".into(),
                Ok(()) => "Approve this device on the website, then check again.".into(),
                Err(error) => error,
            };
        }
    }
    pub fn show(&mut self, ctx: &egui::Context) {
        let mut open = self.open;
        egui::Window::new("@NAME@ — Your account").open(&mut open).resizable(false).show(ctx, |ui| {
            ui.label("Passwords and payment details stay in your browser.");
            ui.label(if self.state.allowed(Instant::now()) { "Current account confirmation is active." } else { "No current account confirmation." });
            if !self.message.is_empty() { ui.label(&self.message); }
            if !self.state.code().is_empty() {
                ui.monospace(self.state.code());
                ui.label("Only approve a code displayed on your own device.");
                if ui.button("Approve this device").clicked() { ctx.open_url(egui::OpenUrl::new_tab(account::ACTIVATE_URL)); }
            }
            if ui.add_enabled(self.pending.is_none(), egui::Button::new("Connect / Check account")).clicked() { self.start(); }
            if ui.button("Manage account").clicked() { ctx.open_url(egui::OpenUrl::new_tab(account::ACCOUNT_URL)); }
            if ui.button("Disconnect").clicked() {
                self.state.forget(); self.pending = None; self.message = "Account disconnected from this session.".into();
            }
            ui.separator();
            ui.small("Development connection test. Editor access enforcement and secure persistent sign-in still require the per-editor integration before release.");
            if self.pending.is_some() { ui.spinner(); ctx.request_repaint_after(std::time::Duration::from_millis(250)); }
        });
        self.open = open;
    }
}
