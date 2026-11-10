//! Offline branch/history recovery fixture; no collaboration or account servers.
use figcraft_engine::{Session,branches,pixa_recovery};
use figcraft_format::{Version,Branch};
use figcraft_doc::Document;
use serde_json::{Value,json};
fn versions(list:&[Version])->Result<Value,Box<dyn std::error::Error>>{
 let mut rows=Vec::new();for v in list {rows.push(json!({"id":v.id,"name":v.name,"description":v.description,"time":v.time,"auto":v.auto,"doc":figcraft_format::to_json(v.document()?.as_ref())?}));}Ok(json!(rows))
}
fn signature(doc:&Document,history:&[Version],list:&[Branch])->Result<Value,Box<dyn std::error::Error>>{
 let mut rows=Vec::new();for b in list{rows.push(json!({"id":b.id,"name":b.name,"time":b.time,"client":b.client,"state":format!("{:?}",b.state),"base":figcraft_format::to_json(&b.base)?,"head":figcraft_format::to_json(&b.head)?,"versions":versions(&b.versions)?}));}
 Ok(json!({"doc":figcraft_format::to_json(doc)?,"versions":versions(history)?,"branches":rows}))
}
fn main()->Result<(),Box<dyn std::error::Error>>{
 let mut folder=None;for n in 0..64{let d=std::env::temp_dir().join(format!("pixa-interface-recovery-{}-{n}",std::process::id()));match std::fs::create_dir(&d){Ok(())=>{folder=Some(d);break},Err(e) if e.kind()==std::io::ErrorKind::AlreadyExists=>continue,Err(e)=>return Err(e.into())}}
 let dir=folder.ok_or("No free fixture folder")?;let copies=dir.join("copies");std::fs::create_dir(&copies)?;
 let mut s=Session::empty();let mut originals=Vec::new();
 for n in 0..2{s.execute("file.new",&json!({"name":format!("Pixa fixture {n}")}))?;let path=dir.join(format!("original-{n}.figcraft"));s.execute("file.saveAs",&json!({"path":path}))?;originals.push((path.clone(),std::fs::read(&path)?));
 s.execute("shape.rectangle",&json!({"x":10+n*20,"y":0,"width":20,"height":20}))?;s.execute("file.saveVersion",&json!({"name":format!("Main version {n}")}))?;}
 s.execute("branch.create",&json!({"name":"Pixa branch"}))?;s.execute("shape.rectangle",&json!({"x":100,"y":0,"width":30,"height":30}))?;s.execute("file.saveVersion",&json!({"name":"Branch version"}))?;
 // Leave an additional change only in the live branch tab: not yet synchronized into main.
 s.execute("shape.rectangle",&json!({"x":200,"y":0,"width":40,"height":40}))?;
 let before:Vec<_>=s.docs.iter().map(|d|(d.path.clone(),d.is_dirty(),d.branch_of.clone())).collect();
 let expected=s.docs.iter().filter(|d|d.branch_of.is_none()).map(|d|{let (doc,v,b)=branches::recovery_copy(&s,d);Ok((d.uid,signature(&doc,&v,&b)?))}).collect::<Result<Vec<_>,Box<dyn std::error::Error>>>()?;
 if pixa_recovery::save(&s,&copies)?!=2{return Err("Missing main-file copies".into())}if pixa_recovery::save(&s,&copies).is_ok(){return Err("Recovery overwrote files".into())}
 let after:Vec<_>=s.docs.iter().map(|d|(d.path.clone(),d.is_dirty(),d.branch_of.clone())).collect();if before!=after{return Err("Recovery changed live tab state".into())}
 for (p,b) in originals{if std::fs::read(p)?!=b{return Err("Original changed".into())}}
 for (id,wanted) in expected {let loaded=figcraft_format::load_bytes_with_versions(&std::fs::read(copies.join(format!("document-{id}.figcraft")))?)?;if signature(&loaded.doc,&loaded.versions,&loaded.branches)?!=wanted{return Err("Recovery lost document, branch or version history".into())}}
 std::fs::remove_dir_all(dir)?;println!("PixaInterface live branch and version-history recovery passed");Ok(())
}
