"""Install shared desktop modules; only verified native copy commands become recovery pilots."""
from pathlib import Path
import json, os
from manuals import guide
ROOT=Path(__file__).resolve().parent
PILOTS={'pixafilm','pixatexture','pixadesign','pixadeck'}

def dependency_path(runtime, parent):
 try:return Path(os.path.relpath(runtime,parent)).as_posix()
 except ValueError:return Path(runtime).resolve().as_posix()

def install(root,app):
 from branding import install as brand
 brand(root,app)
 main=root/'apps'/app['source']/'src/main.rs'
 editor={'pixafilm':'filmcraft_ui_egui::FilmcraftApp','pixatexture':'crate::Shell','pixadesign':'crate::App','pixadeck':'crate::App'}.get(app['id'],'dyn eframe::App')
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
 elif app['id'] in {'pixadesign','pixadeck'}:
  session='editor.app.session' if app['id']=='pixadesign' else 'editor.0.session'
  body=app['source']+'_engine::recovery::save(&'+session+',directory).map_err(|e|e.to_string())?;Ok(())'
 else:body='let _=(editor,directory);Err("Recovery adapter is not configured for this editor.".into())'
 ready=app['id'] in PILOTS
 recovery='use super::Editor;\nuse std::path::{Path,PathBuf};\npub fn ready()->bool{'+str(ready).lower()+'}\n'
 recovery+='pub fn choose_folder()->Option<PathBuf>{'+('rfd::FileDialog::new().pick_folder()' if ready else 'None')+'}\n'
 recovery+='pub fn save(editor:&mut Editor,directory:&Path)->Result<(),String>{'+body+'}\n'
 (main.parent/'pixa_recovery.rs').write_text(recovery,encoding='utf-8')
 manifest=main.parent.parent/'Cargo.toml';text=manifest.read_text(encoding='utf-8')
 runtime_path=dependency_path(ROOT/'runtime',manifest.parent)
 if 'pixa-runtime =' not in text:text=text.replace('[dependencies]','[dependencies]\npixa-runtime = { path = '+json.dumps(runtime_path)+' }',1)
 manifest.write_text(text,encoding='utf-8')
 text=main.read_text(encoding='utf-8')
 if app['id']=='pixafilm' and 'if pixa_shell::subscription_build()' not in text:
  old='let control = control_port.map(';new='let control = (if pixa_shell::subscription_build() { None } else { control_port }).map('
  if old not in text:raise RuntimeError('Film automation integration point changed')
  text=text.replace(old,new,1)
 if app['id'] in {'pixadesign','pixadeck'} and 'if pixa_shell::subscription_build()' not in text:
  old='if let Some(port) = control_port {';new='if let Some(port) = if pixa_shell::subscription_build() { None } else { control_port } {'
  if old not in text:raise RuntimeError('Control integration point changed')
  text=text.replace(old,new,1)
 if app['id']=='pixatexture' and 'if pixa_shell::subscription_build()' not in text:
  old='let control = args.control;';new='let control = if pixa_shell::subscription_build() { None } else { args.control };'
  if old not in text:raise RuntimeError('Texture automation integration point changed')
  text=text.replace(old,new,1)
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
 if app['id'] in PILOTS:
  example=root/'crates/engine/examples/pixa_recovery_check.rs';example.parent.mkdir(parents=True,exist_ok=True)
  source={'pixafilm':'film.rs','pixatexture':'texture.rs','pixadesign':'design.rs','pixadeck':'deck.rs'}[app['id']]
  example.write_text((ROOT/'recovery-checks'/source).read_text(encoding='utf-8'),encoding='utf-8')
