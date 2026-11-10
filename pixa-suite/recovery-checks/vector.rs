//! Offline recovery roundtrip and collision checks; synthetic projects only.
use vectorcraft_engine::{Session,pixa_recovery};
use serde_json::json;
fn main()->Result<(),Box<dyn std::error::Error>> {
 let mut folder=None;for n in 0..64 {let d=std::env::temp_dir().join(format!("pixa-vector-recovery-{}-{n}",std::process::id()));match std::fs::create_dir(&d){Ok(())=>{folder=Some(d);break},Err(e) if e.kind()==std::io::ErrorKind::AlreadyExists=>continue,Err(e)=>return Err(e.into())}}
 let dir=folder.ok_or("No free fixture directory")?;let copies=dir.join("copies");std::fs::create_dir(&copies)?;
 let mut s=Session::new();let mut originals=Vec::new();
 for n in 0..2 {s.execute("file.new",&json!({"width":200,"height":100}))?; let path=dir.join(format!("original-{n}.vectorcraft"));
 s.execute("document.save",&json!({"path":path}))?;originals.push((path.clone(),std::fs::read(&path)?));s.execute("shape.rectangle",&json!({"x":10+n*20,"y":10,"width":40,"height":30}))?;}
 let expected:Vec<_>=s.documents().iter().map(|doc|serde_json::to_value(&doc.doc)).collect::<Result<_,_>>()?;
 let before:Vec<_>=s.documents().iter().map(|d|(d.path.clone(),d.is_dirty())).collect();
 if before.len()!=2 || before.iter().any(|(_,dirty)|!*dirty){return Err("Fixture did not contain two unsaved documents".into())}
 if pixa_recovery::save(&s,&copies)?!=2{return Err("Not all documents were saved".into())}
 let bytes_before=std::fs::read(copies.join("document-0.vectorcraft"))?;
 if pixa_recovery::save(&s,&copies).is_ok(){return Err("Recovery overwrote an existing copy".into())}
 if std::fs::read(copies.join("document-0.vectorcraft"))?!=bytes_before{return Err("Collision modified the recovery copy".into())}
 let after:Vec<_>=s.documents().iter().map(|d|(d.path.clone(),d.is_dirty())).collect();if before!=after{return Err("Recovery changed original paths or dirty flags".into())}
 for (path,bytes) in originals {if std::fs::read(path)?!=bytes{return Err("Original modified".into())}}
 for (n,wanted) in expected.into_iter().enumerate(){let mut restored=Session::new();restored.execute("document.open",&json!({"path":copies.join(format!("document-{n}.vectorcraft"))}))?;let doc=restored.doc()?;if serde_json::to_value(&doc.doc)?!=wanted{return Err("Recovered document differs".into())}}
 std::fs::remove_dir_all(dir)?;println!("vectorcraft recovery roundtrip passed");Ok(())
}
