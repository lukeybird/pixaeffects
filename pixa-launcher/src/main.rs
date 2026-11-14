#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]
use eframe::egui::{self, Color32, RichText};
use pixa_launcher::{
    catalog::{self, Product},
    installer,
};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc, Arc,
};

enum Event {
    Progress(String, u64),
    Finished(Result<String, String>),
    Catalog(Result<Vec<Product>, String>),
}
struct Launcher {
    products: Vec<Product>,
    query: String,
    message: String,
    error: bool,
    active: Option<String>,
    progress: u64,
    phase: String,
    tx: mpsc::Sender<Event>,
    rx: mpsc::Receiver<Event>,
    cancel: Arc<AtomicBool>,
    checking: bool,
    installed: std::collections::HashMap<String, installer::Installed>,
    confirm: Option<Product>,
    help: bool,
    only_installed: bool,
}
impl Launcher {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self::from_context(&cc.egui_ctx)
    }
    fn from_context(ctx: &egui::Context) -> Self {
        ctx.set_visuals(egui::Visuals::dark());
        let mut style = (*ctx.style()).clone();
        style.visuals.panel_fill = Color32::from_rgb(11, 16, 26);
        style.spacing.item_spacing = egui::vec2(12., 12.);
        ctx.set_style(style);
        let (tx, rx) = mpsc::channel();
        let catalog = catalog::embedded_catalog();
        let (products, message, error) = match catalog {
            Ok(p) => (p, "Choose an app to get started. Apps handle activation with your Pixa account.".into(), false),
            Err(e) => (vec![], e, true),
        };
        let mut app = Self {
            products,
            query: String::new(),
            message,
            error,
            active: None,
            progress: 0,
            phase: String::new(),
            tx,
            rx,
            cancel: Arc::new(AtomicBool::new(false)),
            checking: false,
            installed: Default::default(),
            confirm: None,
            help: false,
            only_installed: false,
        };
        app.scan();
        app
    }
    fn scan(&mut self) {
        self.installed = self.products.iter().filter_map(|p| installer::installed(p).map(|i| (p.id.clone(), i))).collect();
    }
    fn check(&mut self, ctx: &egui::Context) {
        self.checking = true;
        let products = self.products.clone();
        let tx = self.tx.clone();
        let ctx = ctx.clone();
        if let Err(e) = std::thread::Builder::new().spawn(move || {
            let _ = tx.send(Event::Catalog(catalog::refresh(&products)));
            ctx.request_repaint();
        }) {
            self.checking = false;
            self.message = e.to_string();
            self.error = true;
        }
    }
    fn start(&mut self, p: Product, ctx: &egui::Context) {
        self.active = Some(p.id.clone());
        self.progress = 0;
        self.phase = "Preparing".into();
        self.error = false;
        self.cancel.store(false, Ordering::Relaxed);
        let cancel = self.cancel.clone();
        let tx = self.tx.clone();
        let ctx = ctx.clone();
        if let Err(e) = std::thread::Builder::new().spawn(move || {
            let result = installer::install(&p, &cancel, |phase, n| {
                let _ = tx.send(Event::Progress(phase, n));
                ctx.request_repaint();
            });
            let _ = tx.send(Event::Finished(result));
            ctx.request_repaint();
        }) {
            self.active = None;
            self.message = e.to_string();
            self.error = true;
        }
    }
    fn open(&mut self, p: Product, ctx: &egui::Context) {
        self.active = Some(p.id.clone());
        self.phase = "Verifying and opening".into();
        self.progress = 0;
        let tx = self.tx.clone();
        let ctx = ctx.clone();
        if let Err(e) = std::thread::Builder::new().spawn(move || {
            let result = installer::open_app(&p).map(|()| format!("Opened {}", p.name));
            let _ = tx.send(Event::Finished(result));
            ctx.request_repaint();
        }) {
            self.active = None;
            self.message = e.to_string();
            self.error = true;
        }
    }
    fn link(&mut self, url: &str) {
        if let Err(e) = installer::open_url(url) {
            self.message = e;
            self.error = true;
        }
    }
}
fn color(s: &str) -> Color32 {
    let n = u32::from_str_radix(s.trim_start_matches('#'), 16).unwrap_or(0x72d4ed);
    Color32::from_rgb((n >> 16) as u8, (n >> 8) as u8, n as u8)
}
fn icon(ui: &mut egui::Ui, p: &Product) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(58., 58.), egui::Sense::hover());
    for r in &p.icon {
        let pos = rect.min + egui::vec2(r.x, r.y) * (58. / 1024.);
        ui.painter().rect_filled(egui::Rect::from_min_size(pos, egui::vec2(r.w, r.h) * (58. / 1024.)), (r.radius * 58. / 1024.) as u8, color(&r.color));
    }
}
impl Launcher {
    fn ui(&mut self, ctx: &egui::Context) {
        while let Ok(e) = self.rx.try_recv() {
            match e {
                Event::Progress(phase, n) => {
                    self.phase = phase;
                    self.progress = n;
                }
                Event::Finished(result) => {
                    self.active = None;
                    self.scan();
                    match result {
                        Ok(s) => {
                            self.message = s;
                            self.error = false
                        }
                        Err(e) => {
                            self.message = e;
                            self.error = true
                        }
                    }
                }
                Event::Catalog(result) => {
                    self.checking = false;
                    match result {
                        Ok(p) => {
                            self.products = p;
                            self.message = "Release check complete. Install or update the apps you choose.".into();
                            self.error = false
                        }
                        Err(e) => {
                            self.message = format!("{e}. The included releases remain available; try again later.");
                            self.error = true
                        }
                    }
                }
            }
        }
        egui::TopBottomPanel::top("header").show(ctx, |ui| {
            ui.add_space(14.);
            ui.horizontal(|ui| {
                ui.heading(RichText::new("Pixa").size(29.).color(Color32::from_rgb(114, 212, 237)));
                ui.label("LAUNCHER");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Your account ↗").clicked() {
                        self.link("https://aiwebdesignfirm.com/settings");
                    }
                    if ui
                        .add_enabled(
                            !self.checking && self.active.is_none(),
                            egui::Button::new(if self.checking { "Checking…" } else { "Check for app updates" }),
                        )
                        .clicked()
                    {
                        self.check(ctx);
                    }
                });
            });
            ui.add_space(12.);
        });
        egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
            ui.add_space(8.);
            if let Some(id) = &self.active {
                let total = self.products.iter().find(|p| &p.id == id).map(|p| p.size).unwrap_or(1);
                ui.horizontal(|ui| {
                    ui.add(
                        egui::ProgressBar::new(self.progress as f32 / total as f32)
                            .text(format!("{} · {:.1} / {:.1} MB", self.phase, self.progress as f64 / 1e6, total as f64 / 1e6))
                            .desired_width((ui.available_width() - 90.).max(100.)),
                    );
                    if ui.button("Cancel").clicked() {
                        self.cancel.store(true, Ordering::Relaxed);
                    }
                });
            }
            ui.label(RichText::new(&self.message).color(if self.error { Color32::from_rgb(255, 160, 170) } else { Color32::from_rgb(180, 194, 210) }));
            ui.horizontal(|ui| {
                ui.small("Mac early access · Your files stay in the editors · Pixa Launcher 0.2.0");
                if ui.small_button("Help").clicked() {
                    self.help = true;
                }
            });
            ui.add_space(6.);
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(14.);
            ui.heading("Your creative toolkit.");
            ui.label("Install what you need. Open it here. Make something yours.");
            ui.add_space(8.);
            ui.add(egui::TextEdit::singleline(&mut self.query).hint_text("Search apps or categories…").desired_width(360.));
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.only_installed, "Installed apps only");
                if ui.button("Refresh installed apps").clicked() {
                    self.scan();
                }
            });
            ui.add_space(12.);
            if !cfg!(target_os = "macos") {
                ui.colored_label(Color32::YELLOW, "This preview can browse the library. Native installation requires macOS.");
            }
            let query = self.query.to_lowercase();
            let products: Vec<_> = self
                .products
                .iter()
                .filter(|p| {
                    (!self.only_installed || self.installed.contains_key(&p.id))
                        && format!("{} {} {}", p.name, p.category, p.summary).to_lowercase().contains(&query)
                })
                .cloned()
                .collect();
            egui::ScrollArea::vertical().show(ui, |ui| {
                if products.is_empty() {
                    ui.label("No apps match your search.");
                }
                for p in products {
                    egui::Frame::new().fill(Color32::from_rgb(17, 25, 38)).corner_radius(12.).inner_margin(18.).show(ui, |ui| {
                        ui.horizontal(|ui| {
                            icon(ui, &p);
                            ui.vertical(|ui| {
                                ui.strong(RichText::new(&p.name).size(19.));
                                ui.label(RichText::new(&p.category).small().color(Color32::from_rgb(150, 167, 187)));
                            });
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label(RichText::new(format!("Ready · v{}", p.version)).color(Color32::from_rgb(137, 230, 166)));
                            });
                        });
                        ui.label(&p.summary);
                        ui.horizontal(|ui| {
                            let existing = self.installed.get(&p.id).cloned();
                            let action = if let Some(i) = &existing {
                                match catalog::compare_versions(&p.version, &i.version) {
                                    Ok(std::cmp::Ordering::Greater) => "Update",
                                    Ok(std::cmp::Ordering::Less) => "Newer installed",
                                    _ => "Reinstall",
                                }
                            } else {
                                "Install"
                            };
                            if ui
                                .add_enabled(
                                    cfg!(target_os = "macos") && self.active.is_none() && !self.checking && action != "Newer installed",
                                    egui::Button::new(action).fill(Color32::from_rgb(21, 103, 128)),
                                )
                                .clicked()
                            {
                                self.confirm = Some(p.clone());
                            }
                            if let Some(i) = existing {
                                if ui.add_enabled(self.active.is_none(), egui::Button::new("Open")).clicked() {
                                    self.open(p.clone(), ctx);
                                }
                                ui.small(format!("Installed {}", i.version));
                            }
                            if ui.button("Product page ↗").clicked() {
                                self.link(&format!("https://aiwebdesignfirm.com/products/{}#downloads", p.id));
                            }
                        });
                    });
                    ui.add_space(10.);
                }
            });
        });
        if let Some(p) = self.confirm.clone() {
            egui::Window::new(format!("Install {}?",p.name)).collapsible(false).resizable(false).anchor(egui::Align2::CENTER_CENTER,[0.,0.]).show(ctx,|ui|{
   ui.set_max_width(430.);ui.label(format!("Download {} v{} ({:.1} MB) and install into your Applications folder. No administrator password is needed.",p.name,p.version,p.size as f64/1e6));
   ui.label("Quit the app before updating. Downloads are checked against the published checksum, Apple signature, publisher and Gatekeeper. Any previous app is preserved. Documents are not removed.");
   ui.label("Use your Pixa account to activate inside each editor.");
   ui.horizontal(|ui|{if ui.button("Install").clicked(){self.confirm=None;self.start(p,ctx);}
if ui.button("Cancel").clicked(){self.confirm=None;}});
  });
        }
        if self.help {
            egui::Window::new("Using Pixa Launcher").open(&mut self.help).resizable(false).show(ctx,|ui|{
          ui.set_max_width(480.);ui.heading("One home for your Pixa apps.");
          for text in ["1. Search or browse the library, then choose Install. Confirm the download to begin.","2. Wait for download, Apple verification and installation. Cancel stops before replacement; a network request may need to time out first.","3. Choose Open. Activate within the editor using your Pixa account. Your account button opens account settings in your browser.","4. Choose Check for app updates. New verified release candidates show Update. Quit the editor before installing.","Previous app bundles are preserved in hidden .pixa-previous-* folders within your home Applications folder. Your projects are never removed. To restore one, quit the editor, move the current app aside in Finder, then move the preserved app back.","If a checksum, signature, publisher, version or Gatekeeper check fails, the app is not installed. Retry later or use the product page for help. The launcher never requests your Apple password.","The launcher itself does not auto-update yet. Obtain a newer signed launcher from the Pixa website when available."] {ui.label(text);}
        });
        }
        if ctx.input(|i| i.viewport().close_requested()) && self.active.is_some() {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.message = "An installation is in progress. Cancel it and wait for it to stop before closing.".into();
        }
        ctx.request_repaint_after(std::time::Duration::from_millis(150));
    }
}
impl eframe::App for Launcher {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        self.ui(ctx);
    }
}
fn main() -> eframe::Result {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|s| s == "--version") {
        println!("Pixa Launcher 0.2.0");
        return Ok(());
    }
    if args.iter().any(|s| s == "--check-releases") {
        let result = catalog::embedded_catalog().and_then(|p| catalog::refresh(&p));
        match result {
            Ok(p) => {
                for p in p {
                    println!("{} {} {}", p.name, p.version, p.url);
                }
            }
            Err(e) => {
                eprintln!("{e}");
                std::process::exit(1);
            }
        }
        return Ok(());
    }
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([970., 760.])
            .with_min_inner_size([640., 480.])
            .with_app_id("com.aiwebdesignfirm.pixalauncher"),
        ..Default::default()
    };
    eframe::run_native("Pixa Launcher", options, Box::new(|cc| Ok(Box::new(Launcher::new(cc)))))
}

#[cfg(test)]
mod ui_tests {
    use super::*;
    fn frame(app: &mut Launcher, ctx: &egui::Context, w: f32, h: f32) {
        let input = egui::RawInput { screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(w, h))), ..Default::default() };
        let output = ctx.run(input, |ctx| app.ui(ctx));
        assert!(!output.shapes.is_empty());
    }
    #[test]
    fn library_search_empty_and_help_render_at_two_sizes() {
        let ctx = egui::Context::default();
        let mut app = Launcher::from_context(&ctx);
        frame(&mut app, &ctx, 970., 760.);
        app.query = "no-such-app".into();
        frame(&mut app, &ctx, 640., 480.);
        app.query = "video".into();
        app.help = true;
        frame(&mut app, &ctx, 970., 760.);
        app.help = false;
        app.confirm = app.products.first().cloned();
        frame(&mut app, &ctx, 640., 480.);
    }
    #[test]
    fn capture_native_ui_mesh_when_requested() {
        let Ok(path) = std::env::var("PIXA_LAUNCHER_FRAME_JSON") else {
            return;
        };
        let ctx = egui::Context::default();
        let mut app = Launcher::from_context(&ctx);
        let input = egui::RawInput { screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(970., 760.))), ..Default::default() };
        let first = ctx.run(input.clone(), |ctx| app.ui(ctx));
        let out = ctx.run(egui::RawInput { time: Some(1.0), ..input }, |ctx| app.ui(ctx));
        let textures: Vec<_> = first
            .textures_delta
            .set
            .iter()
            .map(|(_, d)| {
                let pixels: Vec<[u8; 4]> = match &d.image {
                    egui::ImageData::Color(c) => c.pixels.iter().map(|p| p.to_array()).collect(),
                    egui::ImageData::Font(f) => f.srgba_pixels(None).map(|p| p.to_array()).collect(),
                };
                serde_json::json!({"size":d.image.size(),"pixels":pixels})
            })
            .collect();
        let mesh:Vec<_>=ctx.tessellate(out.shapes,out.pixels_per_point).into_iter().filter_map(|p|match p.primitive{egui::epaint::Primitive::Mesh(m)=>Some(serde_json::json!({"clip":[p.clip_rect.min.x,p.clip_rect.min.y,p.clip_rect.max.x,p.clip_rect.max.y],"vertices":m.vertices.iter().map(|v|serde_json::json!([v.pos.x,v.pos.y,v.uv.x,v.uv.y,v.color.to_array()])).collect::<Vec<_>>(),"indices":m.indices})),_=>None}).collect();
        std::fs::write(path, serde_json::to_vec(&serde_json::json!({"textures":textures,"meshes":mesh})).unwrap()).unwrap();
    }
    #[test]
    fn download_failure_clears_busy_and_preserves_error() {
        let ctx = egui::Context::default();
        let mut app = Launcher::from_context(&ctx);
        app.active = Some("pixashop".into());
        app.tx.send(Event::Finished(Err("checksum mismatch".into()))).unwrap();
        frame(&mut app, &ctx, 970., 760.);
        assert!(app.active.is_none());
        assert!(app.error);
        assert_eq!(app.message, "checksum mismatch");
    }
}
