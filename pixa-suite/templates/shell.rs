//! Shared Pixa development shell. Editor state and shutdown behavior remain owned by the editor.
use eframe::egui;
#[path = "pixa_account.rs"]
mod account_ui;
#[path = "pixa_help.rs"]
mod help_ui;
struct PixaShell {
    inner: Box<dyn eframe::App>,
    help: help_ui::HelpUi,
    account: account_ui::AccountUi,
    updates_open: bool,
}
pub fn wrap(inner: Box<dyn eframe::App>) -> Box<dyn eframe::App> {
    Box::new(PixaShell { inner, help: help_ui::HelpUi::default(), account: account_ui::AccountUi::default(), updates_open: false })
}
impl eframe::App for PixaShell {
    fn logic(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) { self.account.tick(); self.inner.logic(ctx, frame); }
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        egui::Panel::bottom("pixa-publisher").exact_size(26.0).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.small("@NAME@ by AIWebDesignFirm"); ui.separator();
                if ui.small_button("Manual").clicked() { self.help.open = true; }
                if ui.small_button("Quick orientation").clicked() { self.help.start_tour(); }
                if ui.small_button("Your account").clicked() { self.account.open = true; }
                if ui.small_button("Updates").clicked() { self.updates_open = true; }
                ui.small("Development preview");
            });
        });
        self.inner.ui(ui, frame);
        self.account.show(ui.ctx()); self.help.show(ui.ctx());
        egui::Window::new("@NAME@ — Updates").open(&mut self.updates_open).show(ui.ctx(), |ui| {
            ui.label("Pixa updates are not published for this development build.");
            ui.label("The publisher must configure a signed Pixa release feed before installation can be enabled.");
            ui.small("Unsigned update feeds and upstream ArtCraft installers are not accepted.");
        });
    }
    fn raw_input_hook(&mut self, ctx: &egui::Context, raw: &mut egui::RawInput) { self.inner.raw_input_hook(ctx, raw); }
    fn save(&mut self, storage: &mut dyn eframe::Storage) { self.inner.save(storage); }
    @ON_EXIT@
    fn auto_save_interval(&self) -> std::time::Duration { self.inner.auto_save_interval() }
    fn clear_color(&self, visuals: &egui::Visuals) -> [f32; 4] { self.inner.clear_color(visuals) }
    fn persist_egui_memory(&self) -> bool { self.inner.persist_egui_memory() }
}
