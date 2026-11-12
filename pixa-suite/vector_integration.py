"""PixaVector-specific help, tour and publisher-link integration."""
from pathlib import Path

ROOT = Path(__file__).resolve().parent


def replace_once(path, old, new):
    text = path.read_text(encoding='utf-8')
    if old not in text:
        raise RuntimeError(f'PixaVector integration point changed: {path.name}: {old[:60]}')
    path.write_text(text.replace(old, new, 1), encoding='utf-8')


def install(root, app):
    if app['id'] != 'pixavector':
        return
    main = root / 'apps/vectorcraft/src/main.rs'
    main_text = main.read_text(encoding='utf-8').replace('println!("vectorcraft {}"', 'println!("PixaVector {}"').replace('eprintln!("vectorcraft:', 'eprintln!("PixaVector:')
    main.write_text(main_text, encoding='utf-8')
    ui = root / 'crates/ui-egui/src'
    desktop = root / 'apps/vectorcraft'
    help_file = root / 'crates/engine/src/cmd/help.rs'
    text = help_file.read_text(encoding='utf-8')
    text = text.replace('"vectorcraft"', '"pixavector"')
    text = text.replace('https://aiwebdesignfirm.com/settings', 'https://aiwebdesignfirm.com/account')
    text = text.replace('{WEBSITE_URL}/apps/{APP_ID}', '{WEBSITE_URL}/products/{APP_ID}')
    text = text.replace('https://github.com/storytold/{APP_ID}', '{WEBSITE_URL}/products/{APP_ID}')
    text = text.replace('https://aiwebdesignfirm.com/apps/vectorcraft', 'https://aiwebdesignfirm.com/products/pixavector')
    # Preserve legacy command IDs and JSON keys, while removing misleading visible promotion.
    text = text.replace('"Join Our Discord"', '"Your Pixa account"')
    text = text.replace('"PixaVector on aiwebdesignfirm.com"', '"Quick Tool Tour"')
    text = text.replace('"PixaVector help"', '"PixaVector User Manual"')
    text = text.replace('the AIWebDesignFirm community Discord', 'your Pixa account')
    text = text.replace("/// This app's source repository.", '/// Product-page fallback for the legacy help command.')
    text = text.replace('source code, issues and releases', 'the PixaVector offline manual in the editor')
    text = text.replace('this app\'s page', 'the PixaVector tool tour in the editor')
    # The old unit test's legacy repository URL now resolves to the product page.
    text = text.replace('s.execute("help.github", &json!({})).unwrap()["url"], "https://aiwebdesignfirm.com"', 's.execute("help.github", &json!({})).unwrap()["url"], "https://aiwebdesignfirm.com/products/pixavector"')
    help_file.write_text(text, encoding='utf-8')
    # Align the existing browser-routing regression with native offline Help.
    community = ui / 'community.rs'
    text = community.read_text(encoding='utf-8')
    text = text.replace('https://aiwebdesignfirm.com/settings', 'https://aiwebdesignfirm.com/account')
    text = text.replace('        assert_eq!(\n            *opened.lock().unwrap(),', '        #[cfg(target_arch = "wasm32")]\n        assert_eq!(\n            *opened.lock().unwrap(),', 1)
    text = text.replace('https://aiwebdesignfirm.com/apps/vectorcraft', 'https://aiwebdesignfirm.com/products/pixavector')
    text = text.replace('"https://aiwebdesignfirm.com/products/pixavector",\n                "https://aiwebdesignfirm.com"', '"https://aiwebdesignfirm.com/products/pixavector",\n                "https://aiwebdesignfirm.com/products/pixavector"')
    text = text.replace('        // The control channel', '        #[cfg(not(target_arch = "wasm32"))]\n        assert_eq!(*opened.lock().unwrap(), ["https://aiwebdesignfirm.com/account", "https://aiwebdesignfirm.com"]);\n        // The control channel', 1)
    text = text.replace('assert_eq!(opened.lock().unwrap().len(), 4);', 'assert_eq!(opened.lock().unwrap().len(), if cfg!(target_arch = "wasm32") { 4 } else { 2 });')
    community.write_text(text, encoding='utf-8')
    tree = ui / 'menus/tree.rs'
    text = tree.read_text(encoding='utf-8').replace('"Join Our Discord"', '"Your Pixa account"').replace('"PixaVector on aiwebdesignfirm.com"', '"Quick Tool Tour"').replace('"PixaVector help"', '"PixaVector User Manual"')
    tree.write_text(text, encoding='utf-8')
    # Intercept before the native browser callback, so Help stays inside the editor.
    replace_once(ui / 'app/links.rs', '    pub fn open_link(&mut self, id: &str) {', '''    pub fn open_link(&mut self, id: &str) {
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(request) = match id {
            "help.github" => Some("pixa:manual"),
            "help.appPage" => Some("pixa:tool-tour"),
            _ => None,
        } {
            self.pending_url = Some(request.into());
            return;
        }''')
    replace_once(ui / 'app.rs', '            ctx.open_url(egui::OpenUrl::new_tab(u));', '''            if cfg!(not(target_arch = "wasm32")) && matches!(u.as_str(), "pixa:manual" | "pixa:tool-tour" | "pixa:updates") {
                ctx.data_mut(|data| data.insert_temp(egui::Id::unique("pixa-help-request"), u));
            } else {
                ctx.open_url(egui::OpenUrl::new_tab(u));
            }''')
    replace_once(ui / 'menus/run.rs', '"help.checkForUpdates" => crate::updates_ui::check(app),', '''"help.checkForUpdates" => {
            #[cfg(not(target_arch = "wasm32"))]
            { app.pending_url = Some("pixa:updates".into()); Ok(Value::Null) }
            #[cfg(target_arch = "wasm32")]
            { crate::updates_ui::check(app) }
        },''')
    legacy_updates = ui / 'updates_ui.rs'
    text = legacy_updates.read_text(encoding='utf-8')
    text = text.replace('use serde_json::Value;', '#[cfg(any(test, target_arch = "wasm32"))]\nuse serde_json::Value;')
    text = text.replace('pub(crate) fn check(app:', '#[cfg(target_arch = "wasm32")]\npub(crate) fn check(app:')
    text = text.replace('self.updates_rt.startup_check = on;', 'let _ = on;\n        self.updates_rt.startup_check = false; // The Pixa shell owns native release checks.')
    legacy_updates.write_text(text, encoding='utf-8')
    shell = desktop / 'src/pixa_shell.rs'
    replace_once(shell, 'else if request=="pixa:tool-tour" {self.help.start_tour();}', 'else if request=="pixa:tool-tour" {self.help.start_tour();} else if request=="pixa:updates" {self.updates.open=true;self.updates.check();}')
    # Record visible tool groups, without selecting tools or changing document state.
    marker = 'let (rect, resp) = ui.allocate_exact_size(vec2(36.0, PITCH - 1.0), Sense::click_and_drag());'
    replace_once(ui / 'toolbar.rs', marker, marker + '''
                    if rect.intersects(ui.clip_rect()) {
                        let now = ui.input(|input| input.time);
                        for tool_id in slot {
                            ui.data_mut(|data| data.insert_temp(egui::Id::unique(("pixa-tool-anchor", *tool_id)), (now, rect)));
                        }
                    }''')
    tests = ui / 'toolbar/tests.rs'
    tests.write_text(tests.read_text(encoding='utf-8') + '\n' + (ROOT / 'vector-tests/toolbar.rs').read_text(encoding='utf-8'), encoding='utf-8')
    manifest = desktop / 'Cargo.toml'
    replace_once(manifest, '[dev-dependencies]', '[dev-dependencies]\negui_kittest = { workspace = true }')
    help_ui = desktop / 'src/pixa_help.rs'
    help_ui.write_text(help_ui.read_text(encoding='utf-8') + '\n#[cfg(test)]\n#[path = "pixa_help_acceptance.rs"]\nmod acceptance;\n', encoding='utf-8')
    (desktop / 'src/pixa_help_acceptance.rs').write_text((ROOT / 'vector-tests/help.rs').read_text(encoding='utf-8'), encoding='utf-8')
    updates = desktop / 'src/pixa_updates.rs'
    text = updates.read_text(encoding='utf-8')
    text = text.replace(' pub open:bool,', ' pub open:bool,\n checked_once:bool,')
    text = text.replace(' pub fn check(&mut self){', ' pub fn check(&mut self){\n  self.checked_once=true;')
    text = text.replace(' pub fn tick(&mut self,ctx:&egui::Context){', ' pub fn tick(&mut self,ctx:&egui::Context){\n  if !self.checked_once {self.check();}')
    text = text.replace('self.version=r.map(|r|r.version);self.message=', 'self.version=r.map(|r|r.version);if self.version.is_some(){self.open=true;}self.message=')
    text = text.replace('self.message="Checking for Pixa updates…".into();self.saved=false;', 'self.version=None;self.message="Checking for Pixa updates…".into();self.saved=false;')
    text = text.replace('Err(e)=>self.message=e}', 'Err(e)=>{self.version=None;self.message=e}}', 1)
    text = text.replace('self.checking=None;self.message="Update check stopped.".into();', 'self.checking=None;self.version=None;self.message="Update check stopped.".into();')
    updates.write_text(text + '\n' + (ROOT / 'vector-tests/updates.rs').read_text(encoding='utf-8'), encoding='utf-8')
