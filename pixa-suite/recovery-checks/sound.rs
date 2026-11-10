//! A memory-only audio source must survive native recovery with sample-identical audio.
use soundcraft_engine::{Engine,io,pixa_recovery};
use soundcraft_model::Clip;
use soundcraft_audio_io::{AudioBuffer,FileFormat};
use serde_json::json;
fn main()->Result<(),Box<dyn std::error::Error>>{
 let mut folder=None;for n in 0..64{let d=std::env::temp_dir().join(format!("pixa-sound-recovery-{}-{n}",std::process::id()));match std::fs::create_dir(&d){Ok(())=>{folder=Some(d);break},Err(e) if e.kind()==std::io::ErrorKind::AlreadyExists=>continue,Err(e)=>return Err(e.into())}}
 let dir=folder.ok_or("No free fixture folder")?;let copies=dir.join("copies");std::fs::create_dir(&copies)?;
 let mut e=Engine::default();let original=dir.join("original.scraft");io::save_session(&mut e,original.to_str().ok_or("Non-Unicode fixture path")?)?;let original_bytes=std::fs::read(&original)?;
 e.execute("track.new",&json!({"count":1,"name":"Pixa recovery audio","format":"mono"}))?;let track=e.session().tracks[0].id;
 let tone:Vec<f32>=(0..4800).map(|i|(i as f32*0.05).sin()*0.5).collect();
 let source=io::add_source(e.session_mut(),"memory-only",AudioBuffer{sample_rate:48000,channels:vec![tone.clone()]},None,FileFormat::Wav);
 let clip=e.session_mut().new_clip_id();soundcraft_engine::edit::place_clip(e.session_mut(),track,Clip::audio(clip,"memory-only",source,0,0,4800));
 let before=e.session().to_json()?;let path=e.path.clone();let dirty=e.is_dirty();if !dirty{return Err("Fixture is not unsaved".into())}
 pixa_recovery::save(&e,&copies)?;
 if pixa_recovery::save(&e,&copies).is_ok(){return Err("Recovery overwrote existing files".into())}
 if e.path!=path || e.is_dirty()!=dirty || e.session().to_json()?!=before{return Err("Recovery changed live engine state".into())}
 if std::fs::read(original)?!=original_bytes{return Err("Recovery changed original session".into())}
 let mut restored=Engine::default();let missing=io::open_session(&mut restored,copies.join("recovery.scraft"))?;if !missing.is_empty(){return Err("Recovered session has missing media".into())}
 let restored_source=restored.session().sources.first().ok_or("Recovered session lost its source")?.id;
 let audio=restored.session().pool.get(restored_source).ok_or("Recovered audio is missing")?;
 if audio.buffer.channels[0]!=tone || restored.session().tracks[0].clips().len()!=1{return Err("Recovered samples or clips differ".into())}
 std::fs::remove_dir_all(dir)?;println!("PixaSound in-memory audio recovery passed");Ok(())
}
