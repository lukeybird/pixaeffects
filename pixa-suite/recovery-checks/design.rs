//! Offline multi-document recovery fixture; never reads user projects.
use designcraft_engine::{Session,recovery};
use serde_json::json;
fn main()->Result<(),Box<dyn std::error::Error>> {
 let mut directory=None;
 for n in 0..64 {let p=std::env::temp_dir().join(format!("pixa-designcraft-check-{}-{n}",std::process::id()));match std::fs::create_dir(&p){Ok(())=>{directory=Some(p);break},Err(e) if e.kind()==std::io::ErrorKind::AlreadyExists=>continue,Err(e)=>return Err(e.into())}}
 let dir=directory.ok_or("No free fixture directory")?;let copies=dir.join("copies");
 let mut s=Session::new();let mut originals=Vec::new();let mut expected=Vec::new();
 for n in 0..2 {
  s.execute("file.new",&json!({}))?;let path=dir.join(format!("original-{n}.designcraft"));
  s.execute("file.saveAs",&json!({"path":path}))?;
  originals.push((path.clone(),std::fs::read(&path)?));
  for _ in 0..=n { s.execute("frame.create",&json!({"rect":[10,10,100,100]}))?; }
  expected.push(serde_json::to_value(&s.doc()?.doc)?);
 }
 let before:Vec<_>=s.documents().iter().map(|d|(d.path.clone(),d.is_dirty())).collect();
 if before.len()!=2 || before.iter().any(|(_,dirty)| !dirty){return Err("Fixture should contain two unsaved documents".into())}
 if recovery::save(&s,&copies)?!=2{return Err("Not every unsaved document was copied".into())}
 let blocker=dir.join("not-a-folder");std::fs::write(&blocker,b"fixture")?;
 if recovery::save(&s,&blocker).is_ok(){return Err("Recovery unexpectedly accepted a file as a directory".into())}
 let after:Vec<_>=s.documents().iter().map(|d|(d.path.clone(),d.is_dirty())).collect();
 if before!=after{return Err("Recovery changed paths or dirty state".into())}
 for (path,bytes) in &originals {if std::fs::read(path)?!=*bytes{return Err("Original was overwritten".into())}}
 let entries=recovery::list(&copies);if entries.len()!=2{return Err("Missing recovery files".into())}
 let mut actual=Vec::new();
 for (uid,_) in entries {let mut restored=Session::new();restored.execute("file.open",&json!({"path":copies.join(format!("{uid}.designcraft"))}))?;actual.push(serde_json::to_value(&restored.doc()?.doc)?);}
 for value in expected {let i=actual.iter().position(|v|v==&value).ok_or("Recovered document differs from edited document")?;actual.remove(i);}
 // A second checkpoint must not overwrite the first independent recovery folder.
 let first:Vec<_>=std::fs::read_dir(&copies)?.filter_map(Result::ok).filter(|e|e.path().extension().is_some_and(|x|x=="designcraft")).map(|e|std::fs::read(e.path())).collect::<Result<_,_>>()?;
 recovery::save(&s,&dir.join("copies-2"))?;
 let second:Vec<_>=std::fs::read_dir(&copies)?.filter_map(Result::ok).filter(|e|e.path().extension().is_some_and(|x|x=="designcraft")).map(|e|std::fs::read(e.path())).collect::<Result<_,_>>()?;
 if first!=second{return Err("Independent checkpoint changed previous copies".into())}
 println!("designcraft multi-document recovery roundtrip passed");
 // Recovery holds directory handles on Windows until process exit: cleanup best effort, only this fixture directory.
 let _=std::fs::remove_dir_all(&dir);Ok(())
}
