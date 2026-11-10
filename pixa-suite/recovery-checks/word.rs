//! Synthetic parked/active and encrypted-copy fixture. No accounts or user files.
use wordcraft_engine::{Session,pixa_recovery,io};
use wordcraft_doc::{Document,StoryRef};
use serde_json::json;
fn main()->Result<(),Box<dyn std::error::Error>>{
 let mut folder=None;for n in 0..64{let d=std::env::temp_dir().join(format!("pixa-word-recovery-{}-{n}",std::process::id()));match std::fs::create_dir(&d){Ok(())=>{folder=Some(d);break},Err(e) if e.kind()==std::io::ErrorKind::AlreadyExists=>continue,Err(e)=>return Err(e.into())}}
 let dir=folder.ok_or("No free fixture folder")?;let copies=dir.join("copies");std::fs::create_dir(&copies)?;
 let mut s=Session::new(Document::new());let mut originals=Vec::new();let mut wanted=Vec::new();
 for n in 0..3{
  if n>0{s.run("file.new",&json!({}))?;}
  let path=dir.join(format!("original-{n}.json"));s.run("file.saveAs",&json!({"path":path}))?;originals.push((path.clone(),std::fs::read(&path)?));
  s.run("text.insert",&json!({"text":format!("Pixa offline recovery {n}")}))?;
  if n==2{s.password=Some(io::Password::new("fixture-password-only"));}
  wanted.push((s.document_id(),s.doc.plain_text(StoryRef::Body),serde_json::to_value(&s.doc)?));
 }
 let before=serde_json::to_value(s.documents())?;
 if pixa_recovery::save(&s,&copies)?!=3{return Err("Missing an active or parked document".into())}
 if serde_json::to_value(s.documents())?!=before{return Err("Recovery changed live document metadata".into())}
 if pixa_recovery::save(&s,&copies).is_ok(){return Err("Recovery overwrote existing files".into())}
 for (path,bytes) in originals{if std::fs::read(path)?!=bytes{return Err("Original file changed".into())}}
 for (id,text,doc) in wanted{
  let protected=copies.join(format!("document-{id}.docx"));
  let file=if protected.is_file(){protected}else{copies.join(format!("document-{id}.json"))};
  let password=if file.extension().is_some_and(|e|e=="docx"){Some("fixture-password-only")}else{None};
  let bytes=std::fs::read(&file)?;
  if password.is_some() && io::open_bytes_with("copy.docx",&bytes,None).is_ok(){return Err("Protected recovery copy lost encryption".into())}
  let (restored,_)=io::open_bytes_with(file.to_str().ok_or("Non-Unicode fixture path")?,&bytes,password)?;
  if restored.plain_text(StoryRef::Body)!=text{return Err("Recovered text differs".into())}
  if password.is_none() && serde_json::to_value(&restored)?!=doc{return Err("Native recovery document differs".into())}
 }
 std::fs::remove_dir_all(dir)?;println!("PixaWord active, parked and encrypted recovery copies passed");Ok(())
}
