//! Offline drawing and unsaved Block Editor fixture using the real native codecs.
use cadcraft_engine::{Session,pixa_recovery};
use serde_json::json;
fn geometry(d:&cadcraft_doc::Drawing)->serde_json::Value{
 let blocks:std::collections::BTreeMap<_,_>=d.blocks.iter().filter(|(name,b)|!name.starts_with('*')&&!b.entities.is_empty()).map(|(name,b)|(name,b.entities.iter().map(|e|&e.kind).collect::<Vec<_>>())).collect();
 json!({"model":d.model.iter().map(|e|&e.kind).collect::<Vec<_>>(),"blocks":blocks})
}
fn main()->Result<(),Box<dyn std::error::Error>>{
 cadcraft_engine::cmd::file::set_io(cadcraft_engine::cmd::file::IoHooks{read:|b,n|cadcraft_io::read(b,n).map_err(|e|e.to_string()),write:|d,n|cadcraft_io::write(d,n).map_err(|e|e.to_string()),plot:None});
 let mut folder=None;for n in 0..64{let d=std::env::temp_dir().join(format!("pixa-cad-recovery-{}-{n}",std::process::id()));match std::fs::create_dir(&d){Ok(())=>{folder=Some(d);break},Err(e) if e.kind()==std::io::ErrorKind::AlreadyExists=>continue,Err(e)=>return Err(e.into())}}
 let dir=folder.ok_or("No free fixture folder")?;let copies=dir.join("copies");std::fs::create_dir(&copies)?;
 let mut s=Session::empty();let mut originals=Vec::new();
 for n in 0..2{s.new_drawing(false);let path=dir.join(format!("original-{n}.dwg"));s.execute("saveas",&json!({"path":path}))?;originals.push((path.clone(),std::fs::read(&path)?));
 if n==1{s.state_mut()?.open_block_editor("PixaRecoveryBlock",true)?;}
 s.execute("line",&json!({"points":[[0,0],[10+n*10,5+n]]}))?;}
 if !s.state()?.block_unsaved(){return Err("Fixture has no pending block edits".into())}
 let before:Vec<_>=s.docs.iter().map(|d|(d.path.clone(),d.is_dirty(),d.block_editor().map(str::to_owned))).collect();
 let expected=s.docs.iter().map(|d|d.recovery_drawing().map(|v|geometry(&v))).collect::<Result<Vec<_>,_>>()?;
 if pixa_recovery::save(&s,&copies)?!=2{return Err("Missing drawing copies".into())}
 if pixa_recovery::save(&s,&copies).is_ok(){return Err("Recovery overwrote files".into())}
 let after:Vec<_>=s.docs.iter().map(|d|(d.path.clone(),d.is_dirty(),d.block_editor().map(str::to_owned))).collect();if before!=after{return Err("Live CAD state changed".into())}
 for (p,b) in originals{if std::fs::read(p)?!=b{return Err("Original drawing changed".into())}}
 for (n,wanted) in expected.into_iter().enumerate(){let name=format!("drawing-{n}.dwg");let actual=cadcraft_io::read(&std::fs::read(copies.join(&name))?,&name)?;if geometry(&actual)!=wanted{return Err("Recovered geometry or pending block edits differ".into())}}
 std::fs::remove_dir_all(dir)?;println!("PixaCAD drawing and pending block recovery passed");Ok(())
}
