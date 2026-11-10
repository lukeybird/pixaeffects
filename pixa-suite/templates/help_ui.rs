//! Searchable offline starter manual and a read-only orientation. Never changes an editor document.
use eframe::egui;
use pixa_runtime::help::{self, Guide};
pub struct HelpUi {
    pub open: bool,
    pub tour_open: bool,
    guide: Result<Guide, String>,
    query: String,
    step: usize,
}
impl Default for HelpUi {
    fn default() -> Self { Self { open: false, tour_open: false, guide: help::read(include_str!("pixa-guide.json"), "@ID@"), query: String::new(), step: 0 } }
}
impl HelpUi {
    pub fn start_tour(&mut self) { self.step = 0; self.tour_open = true; }
    pub fn show(&mut self, ctx: &egui::Context) {
        egui::Window::new("@NAME@ — Manual").open(&mut self.open).default_width(580.0).default_height(500.0).show(ctx, |ui| {
            match &self.guide {
                Err(error) => { ui.label(error); }
                Ok(guide) => {
                    ui.heading(&guide.title); ui.small(&guide.scope);
                    ui.add(egui::TextEdit::singleline(&mut self.query).hint_text("Search the manual"));
                    let query = self.query.trim().to_lowercase(); let mut shown = 0;
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for chapter in &guide.chapters {
                            if !query.is_empty() && !format!("{} {}",chapter.title,chapter.body).to_lowercase().contains(&query) { continue; }
                            shown += 1;
                            ui.add_space(14.0); ui.strong(&chapter.title); ui.label(&chapter.body);
                        }
                        if shown == 0 { ui.label("No matching sections. Try a shorter search."); }
                    });
                }
            }
        });
        if !self.tour_open { return; }
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) { self.tour_open = false; return; }
        let mut open = self.tour_open;
        egui::Window::new("@NAME@ — Quick orientation").open(&mut open).resizable(false)
            .anchor(egui::Align2::RIGHT_BOTTOM, [-24.0,-60.0]).default_width(360.0).show(ctx, |ui| {
                match &self.guide {
                    Err(error) => { ui.label(error); }
                    Ok(guide) => {
                        if let Some(chapter) = guide.tour.get(self.step) {
                            ui.small(format!("{} / {}",self.step.saturating_add(1),guide.tour.len()));
                            ui.heading(&chapter.title); ui.label(&chapter.body); ui.add_space(10.0);
                            ui.horizontal(|ui| {
                                if ui.add_enabled(self.step > 0,egui::Button::new("Back")).clicked() { self.step = self.step.saturating_sub(1); }
                                if self.step.saturating_add(1) < guide.tour.len() {
                                    if ui.button("Next").clicked() { self.step = self.step.saturating_add(1); }
                                } else if ui.button("Finish").clicked() { self.tour_open = false; }
                            });
                            ui.small("Read-only orientation. Your document is not changed.");
                        }
                    }
                }
            });
        self.tour_open = self.tour_open && open;
    }
}
