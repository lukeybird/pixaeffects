//! Synthetic multi-package archive recovery and overwrite protection.
use materialcraft_engine::{Session,core,commands,pixa_recovery};
use serde_json::json;
fn main()->Result<(),Box<dyn std::error::Error>>{
 let mut folder=None;for n in 0..64{let d=std::env::temp_dir().join(format!("pixa-material-recovery-{}-{n}",std::process::id()));match std::fs::create_dir(&d){Ok(())=>{folder=Some(d);break},Err(e) if e.kind()==std::io::ErrorKind::AlreadyExists=>continue,Err(e)=>return Err(e.into())}}
 let dir=folder.ok_or("No free fixture folder")?;let copies=dir.join("copies");std::fs::create_dir(&copies)?;
 let mut s=Session::new(core::Package::new("Original 0"));let mut originals=Vec::new();
 for n in 0..2{if n>0{commands::execute(&mut s,"file.newPackage",&json!({"name":format!("Original {n}")}))?;}
 let path=dir.join(format!("original-{n}.mcraft"));commands::execute(&mut s,"file.saveAs",&json!({"path":path}))?;originals.push((path.clone(),std::fs::read(&path)?));
 s.pkg.name=format!("Edited package {n}");s.pkg.graphs.push(core::Graph::new(&format!("Pixa graph {n}")));s.modified=true;}
 let before=s.open_packages().into_iter().map(serde_json::to_value).collect::<Result<Vec<_>,_>>()?;
 if pixa_recovery::save(&s,&copies)?!=2{return Err("Missing a parked package".into())}
 if pixa_recovery::save(&s,&copies).is_ok(){return Err("Recovery overwrote existing files".into())}
 let after=s.open_packages().into_iter().map(serde_json::to_value).collect::<Result<Vec<_>,_>>()?;
 if before!=after{return Err("Recovery changed live packages".into())}
 for (path,bytes) in originals{if std::fs::read(path)?!=bytes{return Err("Original changed".into())}}
 for entry in before{let id=entry[0].as_u64().ok_or("Invalid document id")?;let mut expected=entry[1].clone();
 // The native SBS reader adds its empty Resources folder to packages with no folders.
 if expected["folders"]==json!([]){expected["folders"]=json!(["Resources"]);}if entry[3]!=true{return Err("Fixture is not unsaved".into())}let name=format!("package-{id}.mcraft");let bytes=std::fs::read(copies.join(&name))?;let restored=materialcraft_formats::project::read(&bytes,&name)?;if serde_json::to_value(&restored.pkg)?!=expected{std::fs::write(dir.join("expected.json"),serde_json::to_vec_pretty(&expected)?)?;std::fs::write(dir.join("actual.json"),serde_json::to_vec_pretty(&restored.pkg)?)?;return Err(format!("Recovered package differs: {}",dir.display()).into())}}
 std::fs::remove_dir_all(dir)?;println!("PixaMaterial active and parked recovery archives passed");Ok(())
}
