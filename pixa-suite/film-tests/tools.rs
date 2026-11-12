#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod pixa_tool_acceptance {
    use super::*;
    #[test]
    fn pixa_visible_groups_have_read_only_tour_anchors() {
        let mut app = FilmcraftApp::new(filmcraft_engine::Session::default());
        let selected = app.ui.tool;
        let ctx = egui::Context::default();
        crate::theme::install(&ctx, &app.tokens);
        for time in [0.0, 0.1] {
            let _ = ctx.run_ui(egui::RawInput { time: Some(time), ..Default::default() }, |ui| {
                show(&mut app, ui, Rect::from_min_size(egui::Pos2::ZERO, vec2(42.0, 700.0)));
            });
        }
        for tool in Tool::ALL {
            let anchor = ctx.data(|data| data.get_temp::<(f64, egui::Rect)>(egui::Id::unique(("pixa-tool-anchor", format!("{tool:?}")))));
            let (time, rect) = anchor.expect("visible tool group has an anchor");
            assert_eq!(time, 0.1);
            assert!(rect.is_positive());
        }
        assert_eq!(app.ui.tool, selected);
    }
}
