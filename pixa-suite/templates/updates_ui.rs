//! Publisher updates, explicit save acknowledgement, and restart only after normal editor shutdown.
use eframe::egui;
#[path="pixa_updater.rs"] mod installer;
#[derive(Default)]
pub struct Updates {
 pub open:bool,
 message:String,
 version:Option<String>,
 checking:Option<std::sync::mpsc::Receiver<Result<Option<pixa_runtime::release_feed::Release>,String>>>,
 saved:bool,
 #[cfg(target_os="macos")] downloading:Option<std::sync::mpsc::Receiver<Result<installer::Prepared,String>>>,
 #[cfg(target_os="macos")] ready:Option<installer::Prepared>,
 #[cfg(target_os="macos")] restart_requested:bool,
 #[cfg(target_os="macos")] close_frames:u8,
}
impl Updates {
 pub fn check(&mut self){
  if self.checking.is_some(){return;}
  #[cfg(target_os="macos")] if self.downloading.is_some() || self.ready.is_some(){return;}
  self.message="Checking for Pixa updates…".into();self.saved=false;
  let(tx,rx)=std::sync::mpsc::channel();
  match std::thread::Builder::new().name("@NAME@ update check".into()).spawn(move||{let _=tx.send(pixa_runtime::release_feed::latest("@ID@","@VERSION@",std::env::consts::OS));}){
   Ok(_)=>self.checking=Some(rx),Err(e)=>self.message=e.to_string(),
  }
 }
 pub fn tick(&mut self,ctx:&egui::Context){
  if let Some(rx)=&self.checking{
   match rx.try_recv(){Ok(result)=>{self.checking=None;match result{Ok(r)=>{self.version=r.map(|r|r.version);self.message=if self.version.is_some(){"A newer Pixa release is available."}else{"No newer published Pixa update is available."}.into();},Err(e)=>self.message=e}},Err(std::sync::mpsc::TryRecvError::Disconnected)=>{self.checking=None;self.message="Update check stopped.".into();},Err(_)=>ctx.request_repaint_after(std::time::Duration::from_millis(250))}
  }
  #[cfg(target_os="macos")] {
   if let Some(rx)=&self.downloading{
    match rx.try_recv(){Ok(result)=>{self.downloading=None;match result{Ok(ready)=>{self.ready=Some(ready);self.message="Publisher signature verified. Save your work, then install and restart.".into();},Err(e)=>self.message=e}},Err(std::sync::mpsc::TryRecvError::Disconnected)=>{self.downloading=None;self.message="Download stopped.".into();},Err(_)=>ctx.request_repaint_after(std::time::Duration::from_millis(250))}
   }
   // A canceled native close leaves the editor running; do not unexpectedly update on a later unrelated exit.
   if self.restart_requested {if self.close_frames>0{self.close_frames-=1;}else{self.restart_requested=false;self.message="Restart was canceled. Save your work and try again when ready.".into();}}
  }
 }
 pub fn show(&mut self,ctx:&egui::Context){
  if !self.open{return;}let mut open=true;let mut check=false;
  egui::Window::new("@NAME@ — Updates").open(&mut open).show(ctx,|ui|{
   if let Some(version)=&self.version{ui.heading(format!("Update available — {version}"));}ui.label(&self.message);
   if self.checking.is_some(){ui.spinner();}
   #[cfg(target_os="macos")] {
    if self.downloading.is_some(){ui.spinner();ui.label("Downloading and verifying the Pixa publisher signature…");}
    if self.ready.is_some(){
     ui.checkbox(&mut self.saved,"I have saved my projects and stopped active exports or recording.");
     if ui.add_enabled(self.saved&&!self.restart_requested,egui::Button::new("Install and restart")).clicked(){self.restart_requested=true;self.close_frames=2;ctx.send_viewport_cmd(egui::ViewportCommand::Close);}
     if ui.button("Cancel update").clicked(){self.ready=None;self.restart_requested=false;self.saved=false;}
    }else if self.downloading.is_none(){
     if let Some(version)=self.version.clone(){if ui.button("Download update").clicked(){let(tx,rx)=std::sync::mpsc::channel();match std::thread::Builder::new().name("@NAME@ installer".into()).spawn(move||{let _=tx.send(installer::prepare(&version));}){Ok(_)=>self.downloading=Some(rx),Err(e)=>self.message=e.to_string()}}}
    }
   }
   #[cfg(not(target_os="macos"))] if self.version.is_some() && ui.button("Open Pixa downloads").clicked(){ctx.open_url(egui::OpenUrl::new_tab("https://aiwebdesignfirm.com/products/@ID@"));}
   if ui.button("Check for updates").clicked(){check=true;}
  });self.open=open;if check{self.check();}
 }
 pub fn on_exit(&mut self){
  #[cfg(target_os="macos")] if self.restart_requested{if let Some(ready)=self.ready.take(){if let Err(error)=installer::restart(ready){eprintln!("@NAME@ update was not installed: {error}");}}}
 }
}
#[cfg(test)]
mod tests {
 use super::*;
 #[test]
 fn available_release_and_empty_feed_have_distinct_messages() {
  let mut updates=Updates::default();
  let(tx,rx)=std::sync::mpsc::channel();updates.checking=Some(rx);
  assert!(tx.send(Ok(Some(pixa_runtime::release_feed::Release{version:"0.2.0".into()}))).is_ok());
  updates.tick(&egui::Context::default());
  assert_eq!(updates.version.as_deref(),Some("0.2.0"));
  assert_eq!(updates.message,"A newer Pixa release is available.");
  let(tx,rx)=std::sync::mpsc::channel();updates.checking=Some(rx);
  assert!(tx.send(Ok(None)).is_ok());updates.tick(&egui::Context::default());
  assert!(updates.version.is_none());
  assert_eq!(updates.message,"No newer published Pixa update is available.");
 }
}
