"""Install shared desktop modules; only verified native copy commands become recovery pilots."""
from pathlib import Path
import json, os
from manuals import guide
ROOT=Path(__file__).resolve().parent
PILOTS={'pixafilm','pixatexture','pixadesign','pixadeck','pixavector','pixagrid','pixaword','pixapdf','pixamaterial','pixacad','pixalight','pixasound','pixainterface','pixaanything'}

def dependency_path(runtime, parent):
 try:return Path(os.path.relpath(runtime,parent)).as_posix()
 except ValueError:return Path(runtime).resolve().as_posix()

def install(root,app):
 from branding import install as brand
 brand(root,app)
 main=root/'apps'/app['source']/'src/main.rs'
 editor={'pixafilm':'filmcraft_ui_egui::FilmcraftApp','pixatexture':'crate::Shell','pixadesign':'crate::App','pixadeck':'crate::App','pixavector':'crate::App','pixagrid':'crate::App','pixaword':'crate::App','pixapdf':'pdfcraft_ui_egui::PixaPDFApp','pixamaterial':'materialcraft_ui_egui::PixaMaterialApp','pixacad':'crate::App','pixalight':'crate::App','pixasound':'crate::App','pixainterface':'crate::App','pixaanything':'craft_anything_ui_egui::CraftApp'}.get(app['id'],'dyn eframe::App')
 on_exit='fn on_exit(&mut self){eframe::App::on_exit(self.inner.as_mut());}'
 if app['source'] in {'lightcraft','pdfcraft'}:on_exit='fn on_exit(&mut self,gl:Option<&eframe::glow::Context>){eframe::App::on_exit(self.inner.as_mut(),gl);}'
 text=(ROOT/'templates/shell.rs').read_text(encoding='utf-8')
 for key,value in {'NAME':app['name'],'ID':app['id'],'EDITOR_TYPE':editor,'ON_EXIT':on_exit}.items():text=text.replace('@'+key+'@',value)
 if app['id']=='pixaanything':text=text.replace('egui::Id::unique(','egui::Id::new(')
 (main.parent/'pixa_shell.rs').write_text(text,encoding='utf-8')
 for template,destination in [('account_ui.rs','pixa_account.rs'),('help_ui.rs','pixa_help.rs')]:
  text=(ROOT/'templates'/template).read_text(encoding='utf-8').replace('@NAME@',app['name']).replace('@ID@',app['id'])
  if app['id']=='pixaanything':text=text.replace('egui::Id::unique(','egui::Id::new(')
  (main.parent/destination).write_text(text,encoding='utf-8')
 (main.parent/'pixa-guide.json').write_text(json.dumps(guide(app,root),indent=2)+'\n',encoding='utf-8')
 if app['id']=='pixafilm':
  body='''let file=directory.join("recovery.fcproj");let path=file.to_str().ok_or("Choose a folder with a Unicode path.")?;
 editor.session.execute("file.saveCopy",serde_json::json!({"path":path})).map_err(|e|e.to_string())?;Ok(())'''
 elif app['id']=='pixatexture':
  body='''editor.app.session.end_stroke().map_err(|e|e.to_string())?;
 let file=directory.join("recovery.tcraft");let path=file.to_str().ok_or("Choose a folder with a Unicode path.")?;
 texturecraft_engine::execute(&mut editor.app.session,"file.saveCopy",&serde_json::json!({"path":path})).map_err(|e|e.to_string())?;Ok(())'''
 elif app['id']=='pixaanything':
  body='craft_anything_ui_egui::pixa_recovery::save(editor,directory)?;Ok(())'
 elif app['id']=='pixainterface':
  body='let _=editor;figcraft_ui_egui::windows::with_app(|app|figcraft_engine::pixa_recovery::save(&app.session,directory)).ok_or("The editor is busy. Keep it open and retry recovery.")??;Ok(())'
 elif app['id'] in {'pixamaterial','pixacad','pixalight','pixasound'}:
  session={'pixamaterial':'editor.session','pixacad':'editor.cad.session','pixalight':'editor.0.session','pixasound':'editor.0.engine'}[app['id']]
  body=app['source'].replace('-','_')+'_engine::pixa_recovery::save(&'+session+',directory).map_err(|e|e.to_string())?;Ok(())'
 elif app['id'] in {'pixaword','pixapdf'}:
  session='editor.0.session' if app['id']=='pixaword' else 'editor.session'
  body=app['source']+'_engine::pixa_recovery::save(&'+session+',directory).map_err(|e|e.to_string())?;Ok(())'
 elif app['id'] in {'pixavector','pixagrid'}:
  session='editor.app.session' if app['id']=='pixavector' else 'editor.0.session'
  body=app['source']+'_engine::pixa_recovery::save(&'+session+',directory).map_err(|e|e.to_string())?;Ok(())'
 elif app['id'] in {'pixadesign','pixadeck'}:
  session='editor.app.session' if app['id']=='pixadesign' else 'editor.0.session'
  body=app['source']+'_engine::recovery::save(&'+session+',directory).map_err(|e|e.to_string())?;Ok(())'
 else:body='let _=(editor,directory);Err("Recovery adapter is not configured for this editor.".into())'
 ready=app['id'] in PILOTS
 recovery='use super::Editor;\nuse std::path::{Path,PathBuf};\npub fn ready()->bool{'+str(ready).lower()+'}\n'
 recovery+='pub fn choose_folder()->Option<PathBuf>{'+('rfd::FileDialog::new().pick_folder()' if ready else 'None')+'}\n'
 recovery+='pub fn pause(editor:&mut Editor)->Option<String>{'+('editor.0.pixa_pause_for_access()' if app['id']=='pixasound' else 'let _=editor;None')+'}\n'
 recovery+='pub fn save(editor:&mut Editor,directory:&Path)->Result<(),String>{'+body+'}\n'
 recovery+='pub fn can_restore()->bool{'+str(app['id']=='pixaanything').lower()+'}\n'
 recovery+='pub fn choose_file()->Option<PathBuf>{'+('rfd::FileDialog::new().add_filter("Pixa session",&["pixasession"]).pick_file()' if app['id']=='pixaanything' else 'None')+'}\n'
 recovery+='pub fn restore(editor:&mut Editor,file:&Path)->Result<(),String>{'+('craft_anything_ui_egui::pixa_recovery::restore(editor,file)' if app['id']=='pixaanything' else 'let _=(editor,file);Err("Draft restoration is not configured for this editor.".into())')+'}\n'
 (main.parent/'pixa_recovery.rs').write_text(recovery,encoding='utf-8')
 manifest=main.parent.parent/'Cargo.toml';text=manifest.read_text(encoding='utf-8')
 runtime_path=dependency_path(ROOT/'runtime',manifest.parent)
 if 'pixa-runtime =' not in text:text=text.replace('[dependencies]','[dependencies]\npixa-runtime = { path = '+json.dumps(runtime_path)+' }',1)
 if app['id']=='pixapdf' and 'rfd =' not in text:text=text.replace('[dependencies]','[dependencies]\nrfd = { workspace = true }',1)
 manifest.write_text(text,encoding='utf-8')
 text=main.read_text(encoding='utf-8')
 if app['id']=='pixafilm' and 'if pixa_shell::subscription_build()' not in text:
  old='let control = control_port.map(';new='let control = (if pixa_shell::subscription_build() { None } else { control_port }).map('
  if old not in text:raise RuntimeError('Film automation integration point changed')
  text=text.replace(old,new,1)
 if app['id'] in {'pixadesign','pixadeck','pixavector','pixagrid','pixacad','pixalight','pixasound'} and 'if pixa_shell::subscription_build()' not in text:
  old='if let Some(port) = control_port {';new='if let Some(port) = if pixa_shell::subscription_build() { None } else { control_port } {'
  if old not in text:raise RuntimeError('Control integration point changed')
  text=text.replace(old,new,1)
 if app['id'] in {'pixatexture','pixamaterial'} and 'if pixa_shell::subscription_build()' not in text:
  old='let control = args.control;';new='let control = if pixa_shell::subscription_build() { None } else { args.control };'
  if old not in text:raise RuntimeError('Texture automation integration point changed')
  text=text.replace(old,new,1)
 if app['id']=='pixainterface' and '.filter(|_| !pixa_shell::subscription_build())' not in text:
  for old,new in [('if let Some(p) = port {','if let Some(p) = port.filter(|_| !pixa_shell::subscription_build()) {'),('if let Some(link) = &args.join {','if let Some(link) = args.join.as_ref().filter(|_| !pixa_shell::subscription_build()) {')]:
   if old not in text:raise RuntimeError('Interface startup integration point changed')
   text=text.replace(old,new,1)
 if app['id']=='pixaword' and '.filter(|_| !pixa_shell::subscription_build())' not in text:
  marker='if let Some(port) = control_port\n'
  if marker not in text:raise RuntimeError('Word control integration point changed')
  text=text.replace(marker,'if let Some(port) = control_port.filter(|_| !pixa_shell::subscription_build())\n',1)
 if app['id']=='pixapdf' and '.filter(|_| !pixa_shell::subscription_build())' not in text:
  marker='if let Some(file) = &control_file {'
  if marker not in text:raise RuntimeError('PDF control integration point changed')
  text=text.replace(marker,'if let Some(file) = control_file.as_ref().filter(|_| !pixa_shell::subscription_build()) {',1)
 text=text.replace('if let Some(port) = (if pixa_shell::subscription_build() { None } else { control_port }) {','if let Some(port) = if pixa_shell::subscription_build() { None } else { control_port } {')
 main.write_text(text,encoding='utf-8')

 if app['id']=='pixadesign':
  run_file=root/'crates/ui-egui/src/menus/run.rs';text=run_file.read_text(encoding='utf-8')
  if 'pixa:manual' not in text:
   marker='        h if h.starts_with("help.")'
   if marker not in text:raise RuntimeError('Help integration point changed')
   arms='        "help.github" => { app.ui.pending_urls.push("pixa:manual".into()); Ok(json!({"opened":"manual"})) },\n        "help.appPage" => { app.ui.pending_urls.push("pixa:tool-tour".into()); Ok(json!({"opened":"tool-tour"})) },\n'
   run_file.write_text(text.replace(marker,arms+marker,1),encoding='utf-8')
  app_file=root/'crates/ui-egui/src/app.rs';text=app_file.read_text(encoding='utf-8')
  if 'pixa-help-request' not in text:
   marker='            ctx.open_url(egui::OpenUrl::new_tab(url));'
   if marker not in text:raise RuntimeError('Help UI integration point changed')
   handler='            if url=="pixa:manual" || url=="pixa:tool-tour" { ctx.data_mut(|d|d.insert_temp(egui::Id::unique("pixa-help-request"),url));continue; }\n'
   app_file.write_text(text.replace(marker,handler+marker,1),encoding='utf-8')
  commands=root/'crates/ui-egui/src/menus/ui_commands.rs';text=commands.read_text(encoding='utf-8')
  import re
  text=re.sub(r'\("help.github", "[^"]+", None, "[^"]+"\)', '("help.github", "PixaDesign Manual…", None, "{} — opens the offline manual")',text)
  text=re.sub(r'\("help.appPage", "[^"]+", None, "[^"]+"\)', '("help.appPage", "Tool tutorial…", None, "{} — explains every toolbar tool without changing the document")',text)
  commands.write_text(text,encoding='utf-8')
 if app['id']=='pixadesign':
  toolbar=root/'crates/ui-egui/src/toolbar.rs';text=toolbar.read_text(encoding='utf-8')
  text=text.replace('egui::Id::new(("pixa-tool-anchor",tool.id))','egui::Id::unique(("pixa-tool-anchor",tool.id))')
  toolbar.write_text(text,encoding='utf-8')
  if 'pixa-tool-anchor' not in text:
   old='let resp = tool_button(ui, shown.icon, active, &tip);'
   anchor='\n                                if resp.rect.intersects(ui.clip_rect()) {\n                                    let now=ui.input(|i| i.time);\n                                    for tool in g { ui.data_mut(|d| d.insert_temp(egui::Id::unique(("pixa-tool-anchor",tool.id)),(now,resp.rect))); }\n                                }'
   if old not in text:raise RuntimeError('Tool tutorial anchor changed')
   toolbar.write_text(text.replace(old,old+anchor,1),encoding='utf-8')
 if app['id']=='pixainterface':
  ui_manifest=root/'crates/ui-egui/Cargo.toml';text=ui_manifest.read_text(encoding='utf-8')
  if 'pixa-runtime =' not in text:text=text.replace('[dependencies]','[dependencies]\npixa-runtime = { path = '+json.dumps(dependency_path(ROOT/'runtime',ui_manifest.parent))+' }',1)
  ui_manifest.write_text(text,encoding='utf-8')
  windows=root/'crates/ui-egui/src/windows.rs';text=windows.read_text(encoding='utf-8')
  if 'pixa-editor-permit' not in text:
   marker='ctx.show_viewport_deferred(viewport(id), b, move |ui, _| {\n            let ctx = ui.ctx().clone();'
   if marker not in text:raise RuntimeError('Detached window guard integration point changed')
   guard='\n            if option_env!("PIXA_ACCESS_MODE").is_some_and(|mode|mode!="development") {\n                let allowed=ctx.data(|d|d.get_temp::<pixa_runtime::guard::ViewportPermit>(egui::Id::unique("pixa-editor-permit")))\n                    .is_some_and(|permit|permit.allows(std::time::Instant::now()));\n                if !allowed {\n                    if ctx.input(|i|i.viewport().close_requested()) {ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);}\n                    ui.vertical_centered(|ui|{ui.heading("PixaInterface");ui.label("Access is paused. Reconnect using the main window to continue.");ui.label("Your documents remain open. Use the main window to save a recovery copy.");});\n                    ctx.request_repaint_after(std::time::Duration::from_millis(500));return;\n                }\n            }\n'
   windows.write_text(text.replace(marker,marker+guard,1),encoding='utf-8')
 if app['id']=='pixalight':
  engine=root/'crates/engine/src';library=engine/'library.rs';text=library.read_text(encoding='utf-8')
  if 'pub mod pixa_recovery;' not in text:library.write_text(text+'\npub mod pixa_recovery;\n',encoding='utf-8')
  lib=engine/'lib.rs';text=lib.read_text(encoding='utf-8')
  if 'pub use library::pixa_recovery;' not in text:lib.write_text(text+'\npub use library::pixa_recovery;\n',encoding='utf-8')
  child=engine/'library/pixa_recovery.rs';child.parent.mkdir(exist_ok=True)
  child.write_text((ROOT/'recovery-checks/light-save.rs').read_text(encoding='utf-8'),encoding='utf-8')
 if app['id']=='pixasound':
  ui=root/'crates/ui-egui/src/lib.rs';text=ui.read_text(encoding='utf-8')
  if 'pub fn pixa_pause_for_access(' not in text:
   text+='\nimpl SoundApp {\n /// Stop playback and finish recording through the native capture path before locking.\n pub fn pixa_pause_for_access(&mut self)->Option<String>{let recording=self.engine.transport.recording;self.stop_play();if recording{Some(self.ui.status.clone())}else{None}}\n}\n'
   ui.write_text(text,encoding='utf-8')
 if app['id']=='pixaword':
  engine=root/'crates/engine/src';session=engine/'session.rs';text=session.read_text(encoding='utf-8')
  if 'pub mod pixa_recovery;' not in text:session.write_text(text+'\npub mod pixa_recovery;\n',encoding='utf-8')
  lib=engine/'lib.rs';text=lib.read_text(encoding='utf-8')
  if 'pub use session::pixa_recovery;' not in text:lib.write_text(text+'\npub use session::pixa_recovery;\n',encoding='utf-8')
  (engine/'session/pixa_recovery.rs').write_text((ROOT/'recovery-checks/word-save.rs').read_text(encoding='utf-8'),encoding='utf-8')
 if app['id'] in {'pixavector','pixagrid','pixapdf','pixamaterial','pixacad','pixasound','pixainterface'}:
  engine=root/'crates/engine/src';lib=engine/'lib.rs';text=lib.read_text(encoding='utf-8')
  if 'pub mod pixa_recovery;' not in text:lib.write_text(text+'\npub mod pixa_recovery;\n',encoding='utf-8')
  filename={'pixavector':'vector-save.rs','pixagrid':'grid-save.rs','pixapdf':'pdf-save.rs','pixamaterial':'material-save.rs','pixacad':'cad-save.rs','pixasound':'sound-save.rs','pixainterface':'interface-save.rs'}[app['id']]
  (engine/'pixa_recovery.rs').write_text((ROOT/'recovery-checks'/filename).read_text(encoding='utf-8'),encoding='utf-8')
 if app['id']=='pixaanything':
  ui=root/'crates/ui-egui/src';lib=ui/'lib.rs';text=lib.read_text(encoding='utf-8')
  if 'pub mod pixa_recovery;' not in text:lib.write_text(text+'\npub mod pixa_recovery;\n',encoding='utf-8')
  (ui/'pixa_recovery.rs').write_text((ROOT/'recovery-checks/anything-save.rs').read_text(encoding='utf-8'),encoding='utf-8')
 if app['id'] in PILOTS:
  example=root/('crates/ui-egui/examples/pixa_recovery_check.rs' if app['id']=='pixaanything' else 'crates/engine/examples/pixa_recovery_check.rs');example.parent.mkdir(parents=True,exist_ok=True)
  source={'pixafilm':'film.rs','pixatexture':'texture.rs','pixadesign':'design.rs','pixadeck':'deck.rs','pixavector':'vector.rs','pixagrid':'grid.rs','pixaword':'word.rs','pixapdf':'pdf.rs','pixamaterial':'material.rs','pixacad':'cad.rs','pixalight':'light.rs','pixasound':'sound.rs','pixainterface':'interface.rs','pixaanything':'anything.rs'}[app['id']]
  example.write_text((ROOT/'recovery-checks'/source).read_text(encoding='utf-8'),encoding='utf-8')
