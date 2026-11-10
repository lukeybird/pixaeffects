//! Account actions and OS credentials stay off the editor/UI thread.
use eframe::egui;
use pixa_runtime::{account::{self,Account},account_worker::{Worker,Reply}};
use std::time::{Instant,Duration};
#[derive(Default)]
pub struct AccountUi {
    pub open:bool,
    state:Account,
    worker:Option<Worker>,
    pending:bool,
    disconnecting:bool,
    message:String,
    next_check:Option<Instant>,
}
impl AccountUi {
    fn ensure_worker(&mut self)->Result<&Worker,String>{
        if self.worker.is_none(){self.worker=Some(Worker::native("@ID@")?);}
        self.worker.as_ref().ok_or_else(||"Could not start account storage.".into())
    }
    fn start(&mut self){
        if self.pending{return;}
        let token=self.state.token();let generation=self.state.generation();
        match self.ensure_worker().and_then(|worker|worker.check(generation,token)){
            Ok(())=>{self.pending=true;self.message="Checking your account…".into();}
            Err(error)=>{self.message=error;self.next_check=None;}
        }
    }
    fn disconnect(&mut self){
        self.state.forget();self.next_check=None;self.disconnecting=true;
        let generation=self.state.generation();
        match self.ensure_worker().and_then(|worker|worker.forget(generation)){
            Ok(())=>{self.pending=true;self.message="Disconnecting and removing saved sign-in…".into();}
            Err(error)=>{self.pending=false;self.disconnecting=false;self.message=error;}
        }
    }
    pub fn allowed(&self)->bool{self.state.allowed(Instant::now())}
    pub fn needs_tick(&self)->bool{self.pending || self.next_check.is_some()}
    pub fn tick(&mut self){
        let response=self.worker.as_ref().map(Worker::poll);
        match response{
            Some(Ok(Some(Reply::Checked{generation,result}))) if generation==self.state.generation()=>{
                self.pending=false;
                self.message=match result.and_then(|checked|{
                    self.state.apply_checked(generation,checked.token,checked.answer,checked.verified_at)?;Ok(checked.warning)
                }){
                    Ok(warning)=>{
                        let status=if self.allowed(){"Your account reports active access."}else{"Approve this device on the website, then check again."};
                        warning.map_or_else(||status.to_owned(),|warning|format!("{status} {warning}"))
                    }
                    Err(error)=>error,
                };
                self.next_check=Instant::now().checked_add(Duration::from_secs(if self.allowed(){240}else{30}));
            }
            Some(Ok(Some(Reply::Forgotten{generation,result}))) if generation==self.state.generation()=>{
                self.pending=false;self.disconnecting=false;
                self.message=match result{Ok(())=>"Disconnected. Saved sign-in was removed.".into(),Err(error)=>error};
            }
            Some(Err(error))=>{self.pending=false;self.disconnecting=false;self.next_check=None;self.message=error;self.worker=None;}
            _=>{}
        }
        if !self.pending && self.next_check.is_some_and(|t|Instant::now()>=t){self.start();}
    }
    pub fn show(&mut self,ctx:&egui::Context){
        let mut open=self.open;
        egui::Window::new("@NAME@ — Your account").open(&mut open).resizable(false).show(ctx,|ui|{
            ui.label("Passwords and payment details stay in your browser.");
            ui.label(if self.allowed(){"Current account confirmation is active."}else{"No current account confirmation."});
            if !self.message.is_empty(){ui.label(&self.message);}
            if !self.state.code().is_empty(){
                ui.monospace(self.state.code());ui.label("Only approve a code displayed on your own device.");
                if ui.button("Approve this device").clicked(){ctx.open_url(egui::OpenUrl::new_tab(account::ACTIVATE_URL));}
            }
            if ui.add_enabled(!self.pending,egui::Button::new("Connect / Check account")).clicked(){self.start();}
            if ui.button("Manage account").clicked(){ctx.open_url(egui::OpenUrl::new_tab(account::ACCOUNT_URL));}
            if ui.add_enabled(!self.disconnecting,egui::Button::new("Disconnect")).clicked(){self.disconnect();}
            ui.separator();
            ui.small("Sign-in uses OS credential storage when available. Restricted previews are experimental; save and recovery testing is required before release.");
            if self.pending{ui.spinner();ctx.request_repaint_after(Duration::from_millis(250));}
        });
        self.open=open;
    }
}
