#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod pixa_help_acceptance {
    use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};
    #[test]
    fn pixa_native_help_and_update_commands_stay_inside_the_app() {
        let mut app = crate::FilmcraftApp::new(filmcraft_engine::Session::default());
        let ctx = egui::Context::default();
        let calls = Arc::new(AtomicUsize::new(0));
        let observed = calls.clone();
        app.hooks.open_url = Some(Box::new(move |_| { observed.fetch_add(1, Ordering::Relaxed); Ok(()) }));
        for (command, request) in [("help.github", "pixa:manual"), ("help.filmcraftHelp", "pixa:manual"), ("help.appPage", "pixa:tool-tour"), ("app.checkForUpdates", "pixa:updates")] {
            crate::menus::invoke_command(&mut app, &ctx, command, serde_json::json!({})).unwrap();
            let routed = ctx.data_mut(|data| data.remove_temp::<String>(egui::Id::unique("pixa-help-request")));
            assert_eq!(routed.as_deref(), Some(request));
        }
        assert_eq!(calls.load(Ordering::Relaxed), 0);
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod pixa_header_acceptance {
    #[test]
    fn pixa_header_has_no_duplicate_account_or_discord_hit_target() {
        let mut app = crate::FilmcraftApp::new(filmcraft_engine::Session::default());
        let ctx = egui::Context::default();
        crate::theme::install(&ctx, &app.tokens);
        let _ = ctx.run_ui(egui::RawInput::default(), |ui| {
            crate::header::show(&mut app, ui, egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1200.0, crate::header::HEIGHT)));
        });
        assert!(!app.auto.elements.iter().any(|element| element.id == "header.discord"));
        assert!(app.auto.elements.iter().any(|element| element.id == "header.workspaceName"));
    }
}
