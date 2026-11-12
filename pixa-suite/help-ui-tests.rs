#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
use super::*;
use egui_kittest::{Harness, kittest::Queryable};

fn harness(tour: bool) -> Harness<'static, HelpUi> {
    let mut help = HelpUi::default();
    if tour { help.start_tour(); } else { help.open = true; }
    let mut harness = Harness::builder().with_size(egui::vec2(1100.0, 800.0))
        // Use the same openly licensed Source Sans fixture as the editors.
        // Native font installation and visual appearance remain separate checks.
        .with_context_setup(|ctx| {
            let mut fonts = egui::FontDefinitions::default();
            fonts.font_data.insert("pixa-help-test".into(), egui::FontData::from_static(include_bytes!("../help-font.ttf")).into());
            fonts.families.entry(egui::FontFamily::Proportional).or_default().insert(0, "pixa-help-test".into());
            ctx.set_fonts(fonts);
        })
        .build_ui_state(|ui, help: &mut HelpUi| help.show(ui.ctx()), help);
    harness.run_steps(3);
    harness
}

#[test]
fn every_configured_tool_step_supports_next_back_and_finish() {
    let mut h = harness(true);
    let guide = h.state().guide.as_ref().unwrap();
    assert!(!guide.tools.is_empty(), "A first-project orientation is not a tool tour");
    let count = guide.tools.len();
    for step in 0..count {
        h.get_by_label(&format!("{} / {}", step + 1, count));
        if step == 1 {
            h.get_by_label("Back").click(); h.run_steps(3);
            h.get_by_label(&format!("1 / {count}"));
            h.get_by_label("Next").click(); h.run_steps(3);
        }
        if step + 1 < count { h.get_by_label("Next").click(); h.run_steps(3); }
    }
    h.get_by_label("Finish").click(); h.run_steps(3);
    assert!(!h.state().tour_open);
}

#[test]
fn offline_manual_search_has_matching_and_empty_results() {
    let mut h = harness(false);
    h.get_by_role_and_label(egui::accesskit::Role::TextInput, "Search the manual").click();
    h.run_steps(2);
    h.input_mut().events.push(egui::Event::Text("no scheduled account expiration".into()));
    h.run_steps(3);
    h.get_by_label("Your account");
    assert!(h.query_by_label("Save and export").is_none());
    h.state_mut().query = "zz-no-such-tool-88901".into(); h.run_steps(3);
    h.get_by_label("No matching sections. Try a shorter search.");
}

#[test]
fn escape_closes_the_tool_bubble() {
    let mut h = harness(true);
    h.input_mut().events.push(egui::Event::Key { key: egui::Key::Escape, physical_key: None, pressed: true, repeat: false, modifiers: egui::Modifiers::NONE });
    h.run_steps(3);
    assert!(!h.state().tour_open);
}

#[test]
fn f1_and_shift_f1_are_consumed_without_swallowing_other_shortcuts() {
    let mut help = HelpUi::default();
    let mut input = egui::RawInput::default();
    input.events.push(egui::Event::Key { key: egui::Key::F1, physical_key: None, pressed: true, repeat: false, modifiers: egui::Modifiers::NONE });
    help.handle_shortcuts(&mut input);
    assert!(help.open); assert!(input.events.is_empty());
    input.events.push(egui::Event::Key { key: egui::Key::F1, physical_key: None, pressed: true, repeat: false, modifiers: egui::Modifiers::SHIFT });
    help.handle_shortcuts(&mut input);
    assert!(help.tour_open); assert!(input.events.is_empty());
    input.events.push(egui::Event::Text("keep this project text".into()));
    help.handle_shortcuts(&mut input);
    assert_eq!(input.events.len(), 1);
}

#[test]
fn owned_manual_links_open_offline_and_preserve_unrelated_urls() {
    let mut help = HelpUi::default();
    let ctx = egui::Context::default();
    ctx.open_url(egui::OpenUrl::new_tab("pixa:manual"));
    ctx.open_url(egui::OpenUrl::new_tab("https://example.com/document-link"));
    help.capture_requests(&ctx);
    assert!(help.open);
    let urls: Vec<_> = ctx.output(|output| output.commands.iter().filter_map(|command| match command {
        egui::OutputCommand::OpenUrl(request) => Some(request.url.clone()), _ => None
    }).collect());
    assert_eq!(urls, vec!["https://example.com/document-link"]);
    ctx.open_url(egui::OpenUrl::new_tab("pixa:tool-tour"));
    help.capture_requests(&ctx);
    assert!(help.tour_open);
}
