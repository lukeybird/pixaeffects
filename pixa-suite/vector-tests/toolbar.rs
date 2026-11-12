#[test]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
fn pixa_visible_tool_groups_have_read_only_tour_anchors() {
    let mut app = VectorcraftApp::new(Session::new(), Default::default());
    app.run("file.new", json!({"width":300,"height":300})).unwrap();
    let selected = app.session.tool_id().to_owned();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, 0.0, vec![]);
    frame(&mut app, &ctx, 0.1, vec![]);
    let anchor = ctx.data(|data| data.get_temp::<(f64, egui::Rect)>(egui::Id::unique(("pixa-tool-anchor", "selection"))));
    let (time, rect) = anchor.expect("visible selection tool has a tour anchor");
    assert_eq!(time, 0.1);
    assert!(rect.is_positive());
    assert_eq!(app.session.tool_id(), selected);
}

#[test]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
fn pixa_help_and_updates_do_not_open_external_or_upstream_pages() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    let mut app = VectorcraftApp::new(Session::new(), Default::default());
    let calls = Arc::new(AtomicUsize::new(0));
    let recorded = calls.clone();
    app.services.open_url = Some(Box::new(move |_| { recorded.fetch_add(1, Ordering::Relaxed); }));
    for (command, request) in [("help.github", "pixa:manual"), ("help.appPage", "pixa:tool-tour")] {
        app.open_link(command);
        assert_eq!(app.pending_url.take().as_deref(), Some(request));
    }
    app.run("help.checkForUpdates", json!({})).unwrap();
    assert_eq!(app.pending_url.as_deref(), Some("pixa:updates"));
    assert_eq!(calls.load(Ordering::Relaxed), 0);
}
