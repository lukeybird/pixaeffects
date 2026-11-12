"""PixaFilm-specific offline Help, tool highlights and publisher update routing."""
from pathlib import Path
from vector_integration import replace_once

ROOT = Path(__file__).resolve().parent


def rebrand_packaging(root):
    """Separate publisher installer identity, retaining internal binary/format names and lawfully required notices."""
    for file in (root / 'packaging').rglob('*'):
        if not file.is_file() or not (file.suffix in {'.sh','.ps1','.wxi','.wxs','.json','.yml','.yaml','.in'} or file.name=='generate.py'):
            continue
        if any(word in file.name.lower() for word in ['license','notice','attribution']):
            continue
        text = file.read_text()
        lines = []
        for line in text.splitlines(keepends=True):
            if 'copyright' not in line.lower():
                line = line.replace('FilmCraft','PixaFilm').replace('ArtCraft','AIWebDesignFirm')
                line = line.replace('Manufacturer="Learning Machines LLC"','Manufacturer="AIWebDesignFirm"')
                line = line.replace('PixaFilm maintainers <filmcraft@storyteller.ai>','AIWebDesignFirm')
                line = line.replace(r'App Paths\\filmcraft.exe',r'App Paths\\PixaFilm.exe')
                line = line.replace(r'App Paths\filmcraft.exe',r'App Paths\PixaFilm.exe')
                # An independent product must never upgrade/remove the upstream MSI.
                line = line.replace('31E6FB14-9DBC-41DD-B24B-26B71FB31B62','2029AEB3-50B9-5F36-A7A8-A6B59605C416')
            lines.append(line)
        file.write_text(''.join(lines))


def install(root, app):
    if app['id'] != 'pixafilm':
        return
    rebrand_packaging(root)
    ui = root / 'crates/ui-egui/src'
    desktop = root / 'apps/filmcraft'
    main = desktop / 'src/main.rs'
    text = main.read_text().replace('format!("filmcraft {}', 'format!("PixaFilm {}').replace('format!("filmcraft: ', 'format!("PixaFilm: ')
    main.write_text(text)
    import_mode = ui / 'panels/import_mode.rs'
    text = import_mode.read_text().replace('tl!("Join us on Discord")', 'tl!("Your Pixa account")').replace('tl!("Community")', 'tl!("Pixa resources")')
    import_mode.write_text(text)
    settings = ui / 'panels/settings.rs'
    settings.write_text(settings.read_text().replace('&format!("{}/blob/main/docs/project-files.md#settings", crate::links::GITHUB)', 'crate::links::GITHUB'))
    dialogs = ui / 'panels/dialogs.rs'
    dialogs.write_text(dialogs.read_text().replace('"PixaFilm on aiwebdesignfirm.com"', '"Quick Tool Tour"').replace('"Product help"', '"PixaFilm User Manual"'))
    # The shell footer already owns account access. Remove the duplicate header button,
    # retaining the title and workspace controls, rather than leaving a dead hit target.
    header = ui / 'header.rs'
    text = header.read_text()
    start = text.index('    // Community: a labelled Discord button, always one click away.')
    end = text.index('    let popup_id = egui::Id::unique("workspaces-popup");', start)
    title = text[text.index('        // Localized menus', start):text.index('        if resp.clicked() {', start)]
    title = title.replace('let right = r.min.x - 12.0;', 'let right = wr.min.x - 12.0;')
    text = text[:start] + '    // The Pixa shell footer owns account access.\n    {\n' + title + '    }\n' + text[end:]
    header.write_text(text)
    links = ui / 'links.rs'
    text = links.read_text()
    text = text.replace('pub const APP: &str = "filmcraft"', 'pub const APP: &str = "pixafilm"')
    text = text.replace('https://aiwebdesignfirm.com/apps/filmcraft', 'https://aiwebdesignfirm.com/products/pixafilm')
    text = text.replace('pub const GITHUB: &str = "https://aiwebdesignfirm.com"', 'pub const GITHUB: &str = "https://aiwebdesignfirm.com/pixa-showcase/manuals/pixafilm"')
    text = text.replace('pub const ISSUES: &str = "https://aiwebdesignfirm.com"', 'pub const ISSUES: &str = "https://aiwebdesignfirm.com/settings"')
    text = text.replace('"PixaFilm on aiwebdesignfirm.com"', '"Quick Tool Tour"').replace('"PixaFilm help"', '"PixaFilm User Manual"')
    text = text.replace('assert_eq!(APP_PAGE, format!("{WEBSITE}/apps/{APP}"));', 'assert_eq!(APP_PAGE, format!("{WEBSITE}/products/{APP}"));')
    text = text.replace('assert_eq!(GITHUB, format!("https://github.com/storytold/{APP}"));', 'assert_eq!(GITHUB, format!("{WEBSITE}/pixa-showcase/manuals/{APP}"));')
    text = text.replace('            assert_eq!(opened.borrow().last().map(String::as_str), Some(url), "{id}");', '            if cfg!(target_arch = "wasm32") || ![APP_PAGE, GITHUB].contains(&url) {\n                assert_eq!(opened.borrow().last().map(String::as_str), Some(url), "{id}");\n            }')
    text = text.replace('assert_eq!(opened.borrow().len(), ALL.len() + 1);', 'assert_eq!(opened.borrow().len(), if cfg!(target_arch = "wasm32") { ALL.len() + 1 } else { ALL.len() - 1 });')
    # The manual is offline now; retain the failed-browser dialog regression
    # against an actual external website, including its copy/close assertions.
    start = text.index('    fn failed_link_dialog_keeps_the_address_until_closed()')
    end = text.index('    #[test]', start)
    text = text[:start] + text[start:end].replace('GITHUB', 'WEBSITE') + text[end:]
    links.write_text(text)
    replace_once(links, 'pub fn open(app: &mut crate::FilmcraftApp, ctx: &egui::Context, url: &str) {', '''pub fn open(app: &mut crate::FilmcraftApp, ctx: &egui::Context, url: &str) {
    #[cfg(not(target_arch = "wasm32"))]
    if let Some(request) = match url {
        GITHUB => Some("pixa:manual"),
        APP_PAGE => Some("pixa:tool-tour"),
        _ => None,
    } {
        ctx.data_mut(|data| data.insert_temp(egui::Id::unique("pixa-help-request"), request.to_owned()));
        return;
    }''')
    menus = ui / 'menus.rs'
    text = menus.read_text().replace('"PixaFilm on aiwebdesignfirm.com"', '"Quick Tool Tour"').replace('"PixaFilm help"', '"PixaFilm User Manual"')
    menus.write_text(text)
    replace_once(menus, '            let _ = crate::updates_ui::check(app);', '''            #[cfg(not(target_arch = "wasm32"))]
            ctx.data_mut(|data| data.insert_temp(egui::Id::unique("pixa-help-request"), "pixa:updates".to_owned()));
            #[cfg(target_arch = "wasm32")]
            let _ = crate::updates_ui::check(app);''')
    keyboard = ui / 'panels/keyboard.rs'
    text = keyboard.read_text().replace('https://aiwebdesignfirm.com/tree/main/docs', 'https://aiwebdesignfirm.com/pixa-showcase/manuals/pixafilm').replace('"PixaFilm Help…"', '"PixaFilm User Manual"')
    keyboard.write_text(text)
    legacy = ui / 'updates_ui.rs'
    text = legacy.read_text().replace('self.updates_rt.startup_check = on;', 'let _ = on;\n        self.updates_rt.startup_check = false; // The Pixa shell owns native checks.')
    text = text.replace('use serde_json::Value;', '#[cfg(target_arch = "wasm32")]\nuse serde_json::Value;')
    text = text.replace('pub(crate) fn check(app:', '#[cfg(target_arch = "wasm32")]\npub(crate) fn check(app:')
    legacy.write_text(text)
    shell = desktop / 'src/pixa_shell.rs'
    replace_once(shell, 'else if request=="pixa:tool-tour" {self.help.start_tour();}', 'else if request=="pixa:tool-tour" {self.help.start_tour();} else if request=="pixa:updates" {self.updates.open=true;self.updates.check();}')
    marker = '        let r = Rect::from_min_size(at, vec2(SIZE, SIZE));'
    replace_once(ui / 'panels/tools.rs', marker, marker + '''
        if r.intersects(ui.clip_rect()) {
            let now = ui.input(|input| input.time);
            for tool in &group {
                ui.data_mut(|data| data.insert_temp(egui::Id::unique(("pixa-tool-anchor", format!("{tool:?}"))), (now, r)));
            }
        }''')
    tools = ui / 'panels/tools.rs'
    tools.write_text(tools.read_text() + '\n' + (ROOT / 'film-tests/tools.rs').read_text())
    links.write_text(links.read_text() + '\n' + (ROOT / 'film-tests/links.rs').read_text())
    manifest = desktop / 'Cargo.toml'
    replace_once(manifest, '[dev-dependencies]', '[dev-dependencies]\negui_kittest = { workspace = true }')
    help_ui = desktop / 'src/pixa_help.rs'
    replace_once(help_ui, 'ui.add(egui::TextEdit::singleline(&mut self.query).hint_text("Search the manual"));', 'let label = ui.label("Search the manual");\n                    ui.add(egui::TextEdit::singleline(&mut self.query).hint_text("Search the manual")).labelled_by(label.id);')
    help_ui.write_text(help_ui.read_text() + '\n#[cfg(test)]\n#[path = "pixa_help_acceptance.rs"]\nmod acceptance;\n')
    (desktop / 'src/pixa_help_acceptance.rs').write_text((ROOT / 'film-tests/help.rs').read_text())
    updates = desktop / 'src/pixa_updates.rs'
    text = updates.read_text().replace(' pub open:bool,', ' pub open:bool,\n checked_once:bool,')
    text = text.replace(' pub fn check(&mut self){', ' pub fn check(&mut self){\n  self.checked_once=true;')
    text = text.replace(' pub fn tick(&mut self,ctx:&egui::Context){', ' pub fn tick(&mut self,ctx:&egui::Context){\n  if !self.checked_once {self.check();}')
    text = text.replace('self.version=r.map(|r|r.version);self.message=', 'self.version=r.map(|r|r.version);if self.version.is_some(){self.open=true;}self.message=')
    text = text.replace('self.message="Checking for Pixa updates…".into();self.saved=false;', 'self.version=None;self.message="Checking for Pixa updates…".into();self.saved=false;')
    text = text.replace('Err(e)=>self.message=e}', 'Err(e)=>{self.version=None;self.message=e}}', 1)
    text = text.replace('self.checking=None;self.message="Update check stopped.".into();', 'self.checking=None;self.version=None;self.message="Update check stopped.".into();')
    updates.write_text(text + '\n' + (ROOT / 'film-tests/updates.rs').read_text())
