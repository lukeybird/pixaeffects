//! Actual native UI draft capture/restore with isolated settings and no account or AI requests.
use craft_anything_ui_egui::{CraftApp,pixa_recovery};
use craft_anything_engine::{Engine,Session,settings::Settings};
use craft_anything_ai::{AiHub,KeyStore};
use std::path::Path;
use std::sync::Arc;
fn app(dir:&Path)->CraftApp{
 let engine=Engine::shared(Session::new(Settings::default()),Vec::new());
 let hub=Arc::new(AiHub::with_stores(engine.clone(),dir.join("settings.toml"),KeyStore::file_only(dir.join("unused-keys.toml"))));
 let mut app=CraftApp::new(engine,hub);app.persist=false;app.headless=true;app
}
fn main()->Result<(),Box<dyn std::error::Error>>{
 let mut folder=None;for n in 0..64{let d=std::env::temp_dir().join(format!("pixa-anything-recovery-{}-{n}",std::process::id()));match std::fs::create_dir(&d){Ok(())=>{folder=Some(d);break},Err(e) if e.kind()==std::io::ErrorKind::AlreadyExists=>continue,Err(e)=>return Err(e.into())}}
 let dir=folder.ok_or("No free fixture folder")?;let copies=dir.join("copies");std::fs::create_dir(&copies)?;
 let attachment=dir.join("attached.txt");std::fs::write(&attachment,b"synthetic attachment")?;
 let mut source=app(&dir);let workspace=craft_anything_engine::workspace::Workspace::create(&dir.join("projects"),"Pixa fixture","offline recovery test",craft_anything_engine::workspace::TargetRef::None)?;source.workspace=Some(workspace);source.state.screen=craft_anything_ui_egui::state::Screen::Project;source.state.make.request="Build a blue button".into();source.state.make.pictures=vec![attachment.clone()];source.state.input="Unsent workbench request".into();source.state.project_input="Unsent project request".into();source.state.project_attachments=vec![attachment.clone()];source.key_inputs.insert("fixture".into(),"TEST-KEY-MUST-NOT-BE-COPIED".into());
 let before=serde_json::to_value(source.prefs())?;pixa_recovery::save(&source,&copies)?;
 if pixa_recovery::save(&source,&copies).is_ok(){return Err("Recovery overwrote files".into())}
 if serde_json::to_value(source.prefs())?!=before{return Err("Recovery changed live drafts".into())}
 let recovery=copies.join("session.pixasession");let text=std::fs::read_to_string(&recovery)?;
 if text.contains("TEST-KEY-MUST-NOT-BE-COPIED") || dir.join("unused-keys.toml").exists(){return Err("Credentials leaked into recovery or touched a key store".into())}
 let mut restored=app(&dir);pixa_recovery::restore(&mut restored,&recovery)?;if restored.workspace.as_ref().map(|w|&w.root)!=source.workspace.as_ref().map(|w|&w.root){return Err("Native project reference was not restored".into())}
 if restored.state.make.request!="Build a blue button" || restored.state.input!="Unsent workbench request" || restored.state.project_input!="Unsent project request"{return Err("Recovered drafts differ".into())}
 if std::fs::read(&restored.state.make.pictures[0])?!=b"synthetic attachment" || !restored.state.make.pictures[0].starts_with(&copies){return Err("Recovered attachment was not independently copied".into())}
 std::fs::remove_file(attachment)?;if std::fs::read(&restored.state.project_attachments[0])?!=b"synthetic attachment"{return Err("Recovery still depends on the temporary original attachment".into())}
 if pixa_recovery::restore(&mut restored,&recovery).is_ok(){return Err("Recovery replaced existing live drafts".into())}
 let wrong=dir.join("wrong.pixasession");let mut value:serde_json::Value=serde_json::from_str(&text)?;value["product"]=serde_json::json!("pixashop");std::fs::write(&wrong,serde_json::to_vec(&value)?)?;
 let mut untouched=app(&dir);if pixa_recovery::restore(&mut untouched,&wrong).is_ok() || !untouched.state.input.is_empty(){return Err("Wrong-product recovery was accepted or changed the editor".into())}
 drop(untouched);drop(restored);drop(source);std::fs::remove_dir_all(dir)?;println!("PixaAnything draft, attachment and credential-exclusion recovery passed");Ok(())
}
