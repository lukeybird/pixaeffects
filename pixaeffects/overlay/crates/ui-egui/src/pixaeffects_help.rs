//! Offline product manual and a read-only tour of every registered toolbar tool.
use crate::state::Tool;
#[derive(Clone, Default)]
struct State {
    manual: bool,
    tour: bool,
    index: usize,
    search: String,
}
fn id() -> egui::Id {
    egui::Id::unique("pixaeffects-help")
}
pub(crate) fn open_manual(ctx: &egui::Context) {
    ctx.data_mut(|d| {
        let mut s = d.get_temp::<State>(id()).unwrap_or_default();
        s.manual = true;
        s.tour = false;
        d.insert_temp(id(), s);
    });
}
pub(crate) fn open_tour(ctx: &egui::Context) {
    ctx.data_mut(|d| {
        let s = State { tour: true, ..Default::default() };
        d.insert_temp(id(), s);
    });
}
pub(crate) fn record_tool(ctx: &egui::Context, tool: Tool, rect: egui::Rect) {
    ctx.data_mut(|d| d.insert_temp(egui::Id::unique(("pe-tool", format!("{tool:?}"))), rect));
}
pub(crate) fn show(ctx: &egui::Context) {
    let mut state = ctx.data(|d| d.get_temp::<State>(id())).unwrap_or_default();
    if state.manual {
        egui::Window::new("PixaEffects User Manual").open(&mut state.manual).default_width(650.0).default_height(550.0).show(ctx, |ui| {
            ui.label("by AIWebDesignFirm");
            let search_label = ui.label("Search the manual");
            ui.text_edit_singleline(&mut state.search).labelled_by(search_label.id);
            let query = state.search.to_lowercase();
            egui::ScrollArea::vertical().show(ui, |ui| {
                if let Ok(chapters) = serde_json::from_str::<Vec<(String, String)>>(include_str!("../../../pixaeffects/manual/guide.json")) {
                    for (title, body) in chapters {
                        if query.is_empty() || format!("{title} {body}").to_lowercase().contains(&query) {
                            egui::CollapsingHeader::new(title).default_open(!query.is_empty()).show(ui, |ui| {
                                ui.label(body);
                            });
                        }
                    }
                } else {
                    ui.label("The bundled manual could not be loaded.");
                }
            });
            if ui.button("Quick Tool Tour").clicked() {
                state.tour = true;
                state.index = 0;
            }
        });
    }
    if state.tour {
        state.manual = false;
        let mut finished = false;
        state.index = state.index.min(Tool::ALL.len().saturating_sub(1));
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            state.tour = false;
        }
        egui::Window::new("PixaEffects Quick Tool Tour")
            .open(&mut state.tour)
            .default_width(390.0)
            .resizable(false)
            .anchor(egui::Align2::LEFT_TOP, egui::vec2(20.0, 80.0))
            .show(ctx, |ui| {
                if let Some(tool) = Tool::ALL.get(state.index) {
                    if let Some(rect) = ctx.data(|d| d.get_temp::<egui::Rect>(egui::Id::unique(("pe-tool", format!("{tool:?}"))))) {
                        ctx.layer_painter(egui::LayerId::new(egui::Order::Foreground, egui::Id::unique("pe-tour-highlight"))).rect_stroke(
                            rect.expand(3.0),
                            4.0,
                            egui::Stroke::new(2.0, egui::Color32::from_rgb(189, 155, 250)),
                            egui::StrokeKind::Outside,
                        );
                    }
                    ui.label(format!("Tool {} of {}", state.index + 1, Tool::ALL.len()));
                    ui.heading(tool.label());
                    if let Some(shortcut) = tool.shortcut() {
                        ui.label(format!("Shortcut: {shortcut}"));
                    }
                    egui::Frame::new().fill(egui::Color32::from_rgb(33, 21, 51)).corner_radius(12.0).inner_margin(14.0).show(ui, |ui| {
                        let key = format!("{tool:?}");
                        let descriptions =
                            serde_json::from_str::<std::collections::BTreeMap<String, String>>(include_str!("../../../pixaeffects/manual/tools.json"))
                                .unwrap_or_default();
                        if let Some(text) = descriptions.get(&key) {
                            ui.label(egui::RichText::new(text).color(egui::Color32::WHITE));
                        }
                    });
                    ui.small("Grouped tools are in the toolbar fly-outs. This tour does not edit your project or change your active tool.");
                }
                ui.horizontal(|ui| {
                    if ui.add_enabled(state.index > 0, egui::Button::new("Back")).clicked() {
                        state.index = state.index.saturating_sub(1);
                    }
                    if state.index + 1 < Tool::ALL.len() {
                        if ui.button("Next").clicked() {
                            state.index += 1;
                        }
                    } else if ui.button("Finish").clicked() {
                        finished = true;
                    }
                });
            });
        if finished {
            state.tour = false;
        }
    }
    ctx.data_mut(|d| d.insert_temp(id(), state));
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_registered_tool_has_help() {
        let map: std::collections::BTreeMap<String, String> = serde_json::from_str(include_str!("../../../pixaeffects/manual/tools.json")).unwrap();
        assert_eq!(map.len(), Tool::ALL.len());
        for tool in Tool::ALL {
            assert!(map.get(&format!("{tool:?}")).is_some_and(|s| !s.is_empty()));
        }
    }
    #[test]
    fn manual_has_chapters() {
        let chapters: Vec<(String, String)> = serde_json::from_str(include_str!("../../../pixaeffects/manual/guide.json")).unwrap();
        assert_eq!(chapters.len(), 12);
    }
    #[test]
    fn tour_walks_every_tool_and_finishes() {
        use egui_kittest::{Harness, kittest::Queryable};
        let mut harness = Harness::builder().with_size(egui::vec2(900.0, 700.0)).build_ui_state(
            |ui, started: &mut bool| {
                if !*started {
                    open_tour(ui.ctx());
                    *started = true;
                }
                show(ui.ctx());
            },
            false,
        );
        harness.run_steps(3);
        for index in 0..Tool::ALL.len() {
            harness.get_by_label(&format!("Tool {} of {}", index + 1, Tool::ALL.len()));
            if index == 1 {
                harness.get_by_label("Back").click();
                harness.run_steps(3);
                harness.get_by_label(&format!("Tool 1 of {}", Tool::ALL.len()));
                harness.get_by_label("Next").click();
                harness.run_steps(3);
            }
            if index + 1 < Tool::ALL.len() {
                harness.get_by_label("Next").click();
                harness.run_steps(3);
            }
        }
        harness.get_by_label("Finish").click();
        harness.run_steps(3);
        assert!(harness.query_by_label("Finish").is_none());
    }

    #[test]
    fn bundled_manual_search_displays_matching_chapter() {
        use egui_kittest::{Harness, kittest::Queryable};
        let mut harness = Harness::builder().with_size(egui::vec2(900.0, 700.0)).build_ui_state(
            |ui, started: &mut bool| {
                if !*started {
                    open_manual(ui.ctx());
                    *started = true;
                }
                show(ui.ctx());
            },
            false,
        );
        harness.run_steps(3);
        harness.get_by_role_and_label(egui::accesskit::Role::TextInput, "Search the manual").click();
        harness.run_steps(2);
        harness.input_mut().events.push(egui::Event::Text("scheduled expiration".into()));
        harness.run_steps(3);
        harness.get_by_label("Account, updates and privacy");
        assert!(harness.query_by_label("Create a composition").is_none());
    }
    #[test]
    fn escape_closes_tour() {
        let ctx = egui::Context::default();
        open_tour(&ctx);
        let input = egui::RawInput {
            events: vec![egui::Event::Key { key: egui::Key::Escape, physical_key: None, pressed: true, repeat: false, modifiers: egui::Modifiers::NONE }],
            ..Default::default()
        };
        ctx.run_ui(input, |ui| show(ui.ctx())).drop_without_applying_deltas();
        assert!(!ctx.data(|data| data.get_temp::<State>(id()).unwrap()).tour);
    }
}
