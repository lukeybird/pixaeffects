//! Independent native library snapshot from the live catalogue, including unflushed edits.
use super::{Session,Journal,FsStore};
use std::path::Path;
use std::io::Write;
pub fn save(session:&Session,directory:&Path)->Result<usize,String>{
 if !directory.is_dir(){return Err("Choose an existing recovery folder.".into())}
 let library=directory.join("library");std::fs::create_dir(&library).map_err(|e|e.to_string())?;
 let store=FsStore::open(&library).map_err(|e|e.to_string())?;
 let (mut journal,_,_)=Journal::open(Box::new(store)).map_err(|e|e.to_string())?;
 journal.snapshot(&session.catalog).map_err(|e|e.to_string())?;
 drop(journal);
 let presets=super::presets_json(session);
 serde_json::from_str::<serde_json::Value>(&presets).map_err(|e|e.to_string())?;
 write_new(&library.join("presets.json"),presets.as_bytes())?;
 let view=session.view_json();serde_json::from_slice::<serde_json::Value>(&view).map_err(|e|e.to_string())?;
 write_new(&library.join("view.json"),&view)?;
 let prefs=serde_json::to_vec_pretty(&super::PrefsFile {
            xmp: session.xmp,
            last_export: session.last_export.clone(),
            last_merge: session.last_merge.clone(),
            export_presets: session.export_presets.clone(),
            print_templates: session.print_templates.clone(),
            metadata_presets: session.metadata_presets.clone(),
            filter_presets: session.filter_presets.clone(),
            curve_presets: session.curve_presets.clone(),
            mask_effects: session.mask_effects.clone(),
            mask_tool_effects: session.mask_tool_effects.clone(),
            label_sets: session.label_sets.clone(),
            lut_profiles: session.lut_profiles.clone(),
            dcp_profiles: session.dcp_profiles.clone(),
            keyword_sets: session.keyword_sets.clone(),
            keyword_set: session.keyword_set.clone(),
            recent_keywords: session.recent_keywords.clone(),
            import: session.import_defaults.clone(),
            cache_mb: session.cache_mb,
            prep: session.prep.library,
            smart_previews_dir: session.smart_previews_dir.as_ref().map(|d| d.to_string_lossy().to_string()),
            forget_local_days: Some(session.forget_local_days),
            slideshow: session.slideshow.clone(),
            slide_layout: session.slide_layout.clone(),
            slide_overlays: session.slide_overlays.clone(),
            slide_titles: session.slide_titles.clone(),
            hold_view: session.hold_view,
        }).map_err(|e|e.to_string())?;
 write_new(&library.join("prefs.json"),&prefs)?;
 write_new(&directory.join("READ-ME.txt"),b"Open the library subfolder in PixaLight. Catalogue edits, presets and preferences are copied. Original photos are referenced in place; this is not a backup of the photo files themselves. Keep your original photos.")?;
 Ok(session.catalog.len())
}
fn write_new(path:&Path,bytes:&[u8])->Result<(),String>{
 let mut options=std::fs::OpenOptions::new();options.write(true).create_new(true);
 #[cfg(unix)]{use std::os::unix::fs::OpenOptionsExt;options.mode(0o600);}
 let mut file=options.open(path).map_err(|e|e.to_string())?;file.write_all(bytes).and_then(|()|file.sync_all()).map_err(|e|e.to_string())
}
