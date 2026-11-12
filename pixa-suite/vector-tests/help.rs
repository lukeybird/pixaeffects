#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
use super::*;
use egui_kittest::{Harness, kittest::Queryable};

fn harness(tour: bool) -> Harness<'static, HelpUi> {
    let mut help = HelpUi::default();
    if tour { help.start_tour(); } else { help.open = true; }
    let mut harness = Harness::builder()
        .with_size(egui::vec2(1100.0, 800.0))
        .with_context_setup(vectorcraft_ui_egui::theme::install_fonts)
        .build_ui_state(|ui, help: &mut HelpUi| help.show(ui.ctx()), help);
    harness.run_steps(3);
    harness
}

#[test]
fn all_registered_tools_are_available_in_the_tour() {
    let mut harness = harness(true);
    let guide = harness.state().guide.as_ref().unwrap();
    assert_eq!(guide.chapters.len(), 12);
    assert_eq!(guide.tools.len(), 92);
    let ids: std::collections::BTreeSet<_> = guide.tools.iter().map(|tool| tool.tool_id.as_deref().unwrap()).collect();
    assert_eq!(ids.len(), 92);
    for index in 0..92 {
        harness.get_by_label(&format!("{} / 92", index + 1));
        if index == 1 {
            harness.get_by_label("Back").click(); harness.run_steps(3);
            harness.get_by_label("1 / 92");
            harness.get_by_label("Next").click(); harness.run_steps(3);
        }
        if index < 91 { harness.get_by_label("Next").click(); harness.run_steps(3); }
    }
    harness.get_by_label("Finish").click(); harness.run_steps(3);
    assert!(!harness.state().tour_open);
}

#[test]
fn manual_search_uses_the_visible_input() {
    let mut harness = harness(false);
    harness.get_by_role_and_label(egui::accesskit::Role::TextInput, "Search the manual").click();
    harness.run_steps(2);
    harness.input_mut().events.push(egui::Event::Text("no scheduled account expiration".into()));
    harness.run_steps(3);
    harness.get_by_label("Your account");
    assert!(harness.query_by_label("Save and export").is_none());
}

#[test]
fn escape_closes_the_read_only_tour() {
    let mut harness = harness(true);
    harness.input_mut().events.push(egui::Event::Key { key: egui::Key::Escape, physical_key: None, pressed: true, repeat: false, modifiers: egui::Modifiers::NONE });
    harness.run_steps(3);
    assert!(!harness.state().tour_open);
}
