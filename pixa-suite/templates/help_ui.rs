//! Searchable offline starter manual and a read-only orientation. Never changes an editor document.
use eframe::egui;
use pixa_runtime::help::{self, Guide};
pub struct HelpUi {
    pub open: bool,
    pub tour_open: bool,
    guide: Result<Guide, String>,
    query: String,
    step: usize,
    tools_mode: bool,
}
impl Default for HelpUi {
    fn default() -> Self { Self { open: false, tour_open: false, guide: help::read(include_str!("pixa-guide.json"), "@ID@"), query: String::new(), step: 0, tools_mode: false } }
}
impl HelpUi {
    pub fn start_tour(&mut self) { self.step = 0; self.tools_mode=self.guide.as_ref().is_ok_and(|g|!g.tools.is_empty()); self.tour_open = true; }
    pub fn handle_shortcuts(&mut self, input: &mut egui::RawInput) {
        input.events.retain(|event| {
            if let egui::Event::Key { key: egui::Key::F1, pressed, repeat, modifiers, .. } = event {
                if !modifiers.ctrl && !modifiers.alt && !modifiers.command && !modifiers.mac_cmd {
                    if *pressed && !*repeat {
                        if modifiers.shift { self.start_tour(); } else { self.open = true; }
                    }
                    // Consume both press and release so the underlying editor
                    // cannot open an unrelated online help page as well.
                    return false;
                }
            }
            true
        });
    }
    pub fn capture_requests(&mut self, ctx: &egui::Context) {
        let mut manual = false;
        let mut tour = false;
        ctx.output_mut(|output| output.commands.retain(|command| {
            if let egui::OutputCommand::OpenUrl(request) = command {
                if matches!(request.url.as_str(), "pixa:manual" | "https://aiwebdesignfirm.com/pixa-showcase/manuals/@ID@" | "https://aiwebdesignfirm.com/@ID@/manual") {
                    manual = true; return false;
                }
                if request.url == "pixa:tool-tour" { tour = true; return false; }
            }
            true
        }));
        if manual { self.open = true; }
        if tour { self.start_tour(); }
    }
    pub fn show(&mut self, ctx: &egui::Context) {
        egui::Window::new("@NAME@ — Manual").open(&mut self.open).default_width(580.0).default_height(500.0).show(ctx, |ui| {
            match &self.guide {
                Err(error) => { ui.label(error); }
                Ok(guide) => {
                    ui.heading(&guide.title); ui.small(&guide.scope);
                    let label = ui.label("Search the manual");
                    ui.add(egui::TextEdit::singleline(&mut self.query).hint_text("Search the manual")).labelled_by(label.id);
                    let query = self.query.trim().to_lowercase(); let mut shown = 0;
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for chapter in guide.chapters.iter().chain(&guide.tools) {
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
        let mut highlight=None;
        let bubble=egui::Window::new("@NAME@ — Quick tour").open(&mut open).resizable(false)
            .anchor(egui::Align2::RIGHT_BOTTOM, [-24.0,-60.0]).default_width(360.0).show(ctx, |ui| {
                match &self.guide {
                    Err(error) => { ui.label(error); }
                    Ok(guide) => {
                        let steps=if self.tools_mode { &guide.tools } else { &guide.tour };
                        if let Some(chapter) = steps.get(self.step) {
                            if let Some(id)=&chapter.tool_id {
                                let anchor=ctx.data(|d|d.get_temp::<(f64,egui::Rect)>(egui::Id::unique(("pixa-tool-anchor",id.as_str()))));
                                if let Some((time,rect))=anchor {
                                    if ctx.input(|i|i.time)-time<0.25 { highlight=Some(rect); }
                                }
                            }
                            ui.small(format!("{} / {}",self.step.saturating_add(1),steps.len()));
                            ui.heading(&chapter.title); ui.label(&chapter.body); ui.add_space(10.0);
                            ui.horizontal(|ui| {
                                if ui.add_enabled(self.step > 0,egui::Button::new("Back")).clicked() { self.step = self.step.saturating_sub(1); }
                                if self.step.saturating_add(1) < steps.len() {
                                    if ui.button("Next").clicked() { self.step = self.step.saturating_add(1); }
                                } else if ui.button("Finish").clicked() { self.tour_open = false; }
                            });
                            if self.tools_mode {ui.small(if highlight.is_some(){"The outline marks the tool group. Open its flyout for grouped tools."}else{"Find this control in the editor toolbar or tool panel. Grouped tools may share a flyout."});ui.small("No tools are selected automatically.");}
                            ui.small("Read-only tour. Your document is not changed.");
                        }
                    }
                }
            });
        if let (Some(rect),Some(window))=(highlight,bubble) {
            let painter=ctx.layer_painter(egui::LayerId::new(egui::Order::Foreground,egui::Id::unique("pixa-tour-highlight")));
            let color=egui::Color32::from_rgb(76,213,242);
            painter.rect_stroke(rect.expand(3.0),4.0,egui::Stroke::new(2.0,color),egui::StrokeKind::Outside);
            painter.line_segment([rect.right_center(),window.response.rect.left_center()],egui::Stroke::new(1.0,color));
        }
        self.tour_open = self.tour_open && open;
    }
}
