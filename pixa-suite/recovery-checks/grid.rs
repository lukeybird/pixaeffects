//! Offline recovery roundtrip and collision checks; synthetic projects only.
use gridcraft_engine::{Session,pixa_recovery};
use serde_json::json;
fn main()->Result<(),Box<dyn std::error::Error>> {
 let mut folder=None;for n in 0..64 {let d=std::env::temp_dir().join(format!("pixa-grid-recovery-{}-{n}",std::process::id()));match std::fs::create_dir(&d){Ok(())=>{folder=Some(d);break},Err(e) if e.kind()==std::io::ErrorKind::AlreadyExists=>continue,Err(e)=>return Err(e.into())}}
 let dir=folder.ok_or("No free fixture directory")?;let copies=dir.join("copies");std::fs::create_dir(&copies)?;
 let mut s=Session::new();let mut originals=Vec::new();
 for n in 0..2 {s.new_workbook(); let path=dir.join(format!("original-{n}.json"));
 s.execute("file.saveAs",json!({"path":path}))?;originals.push((path.clone(),std::fs::read(&path)?));s.execute("cell.set",json!({"cell":"A1","input":format!("Pixa recovery {n}")}))?;s.execute("cell.set",json!({"cell":"B1","input":format!("=1+2+{n}")}))?;}
 let expected:Vec<_>=s.documents().iter().map(|doc|serde_json::to_value(&doc.wb)).collect::<Result<_,_>>()?;
 let before:Vec<_>=s.documents().iter().map(|d|(d.path.clone(),d.is_dirty())).collect();
 if before.len()!=2 || before.iter().any(|(_,dirty)|!*dirty){return Err("Fixture did not contain two unsaved documents".into())}
 if pixa_recovery::save(&s,&copies)?!=2{return Err("Not all documents were saved".into())}
 let bytes_before=std::fs::read(copies.join("document-0.json"))?;
 if pixa_recovery::save(&s,&copies).is_ok(){return Err("Recovery overwrote an existing copy".into())}
 if std::fs::read(copies.join("document-0.json"))?!=bytes_before{return Err("Collision modified the recovery copy".into())}
 let after:Vec<_>=s.documents().iter().map(|d|(d.path.clone(),d.is_dirty())).collect();if before!=after{return Err("Recovery changed original paths or dirty flags".into())}
 for (path,bytes) in originals {if std::fs::read(path)?!=bytes{return Err("Original modified".into())}}
 for (n,wanted) in expected.into_iter().enumerate(){let mut restored=Session::new();restored.execute("file.open",json!({"path":copies.join(format!("document-{n}.json"))}))?;let doc=restored.doc()?;if serde_json::to_value(&doc.wb)?!=wanted{return Err("Recovered document differs".into())}}
 std::fs::remove_dir_all(dir)?;println!("gridcraft recovery roundtrip passed");Ok(())
}
