//! Shared Pixa development shell. Editor state and shutdown behavior remain owned by the editor.
use eframe::egui;
struct PixaShell {
    inner: Box<dyn eframe::App>,
    help_open: bool,
}
pub fn wrap(inner: Box<dyn eframe::App>) -> Box<dyn eframe::App> {
    Box::new(PixaShell { inner, help_open: false })
}
impl eframe::App for PixaShell {
    fn logic(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) { self.inner.logic(ctx, frame); }
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        egui::Panel::bottom("pixa-publisher").exact_size(26.0).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.small("@NAME@ by AIWebDesignFirm");
                ui.separator();
                if ui.small_button("Getting started").clicked() { self.help_open = true; }
                ui.small("Development preview");
            });
        });
        self.inner.ui(ui, frame);
        egui::Window::new("@NAME@ — Getting started").open(&mut self.help_open).resizable(true).show(ui.ctx(), |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading("@TAGLINE@");
                ui.label("Create or open a document from the editor's File menu. Save a separate copy before trying an unfamiliar operation.");
                ui.label("Use the editor's Help menu for its existing command reference and keyboard shortcuts.");
                ui.separator();
                ui.label("This is an unpublished development build. Subscription enforcement, verified Pixa updates and platform testing must pass before distribution.");
                ui.label("Project formats and editing behavior come from the open-source @UPSTREAM@ engine. Original licenses and contributor credits are retained.");
            });
        });
    }
    fn raw_input_hook(&mut self, ctx: &egui::Context, raw: &mut egui::RawInput) { self.inner.raw_input_hook(ctx, raw); }
    fn save(&mut self, storage: &mut dyn eframe::Storage) { self.inner.save(storage); }
    @ON_EXIT@
    fn auto_save_interval(&self) -> std::time::Duration { self.inner.auto_save_interval() }
    fn clear_color(&self, visuals: &egui::Visuals) -> [f32; 4] { self.inner.clear_color(visuals) }
    fn persist_egui_memory(&self) -> bool { self.inner.persist_egui_memory() }
}
