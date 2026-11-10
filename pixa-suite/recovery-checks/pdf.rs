//! Two edited PDFs, original preservation and recovery overwrite refusal; entirely offline.
use pdfcraft_engine::{Session,Edit,pixa_recovery};
use std::sync::Arc;
fn main()->Result<(),Box<dyn std::error::Error>>{
 let mut folder=None;for n in 0..64{let d=std::env::temp_dir().join(format!("pixa-pdf-recovery-{}-{n}",std::process::id()));match std::fs::create_dir(&d){Ok(())=>{folder=Some(d);break},Err(e) if e.kind()==std::io::ErrorKind::AlreadyExists=>continue,Err(e)=>return Err(e.into())}}
 let dir=folder.ok_or("No free fixture folder")?;let copies=dir.join("copies");std::fs::create_dir(&copies)?;
 let mut session=Session::new().with_clock(||1_700_000_000);let mut originals=Vec::new();let mut wanted=Vec::new();
 for n in 0..2{
  let bytes=session.create_blank(200.0,300.0,n+1)?;let path=dir.join(format!("original-{n}.pdf"));std::fs::write(&path,&*bytes)?;
  let id=session.open_new(format!("original-{n}.pdf"),bytes.clone())?;session.mark_saved(id,bytes.clone(),Some(path.to_str().ok_or("Non-Unicode fixture path")?.into()))?;
  originals.push((path,bytes));session.apply(id,Edit::RotatePages{pages:vec![0],degrees:90*(n as i64+1)})?;wanted.push(session.save_bytes(id)?);
 }
 let before:Vec<_>=session.docs().iter().map(|d|(d.path.clone(),d.dirty)).collect();
 if before.iter().any(|(_,dirty)|!*dirty){return Err("Fixture is not unsaved".into())}
 if pixa_recovery::save(&session,&copies)?!=2{return Err("Missing PDF recovery copies".into())}
 if pixa_recovery::save(&session,&copies).is_ok(){return Err("Recovery overwrote existing copies".into())}
 let after:Vec<_>=session.docs().iter().map(|d|(d.path.clone(),d.dirty)).collect();if before!=after{return Err("Live PDF state changed".into())}
 for (path,bytes) in originals{if std::fs::read(path)?!=*bytes{return Err("Original PDF changed".into())}}
 for (n,expected) in wanted.into_iter().enumerate(){let bytes=std::fs::read(copies.join(format!("document-{n}.pdf")))?;if bytes!=*expected{return Err("Copy differs from native save pipeline".into())}let mut restored=Session::new();restored.open_new("recovered.pdf",Arc::new(bytes))?;if restored.docs()[0].info.pages.len()!=n+1{return Err("Recovered PDF lost pages".into())}}
 std::fs::remove_dir_all(dir)?;println!("PixaPDF native recovery roundtrip passed");Ok(())
}
