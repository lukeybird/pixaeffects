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
    pub fn deadline(&self)->Option<Instant>{self.state.deadline()}
    pub fn allowed(&self)->bool{self.state.allowed(Instant::now())}
    pub fn needs_tick(&self)->bool{self.pending || self.next_check.is_some()}
    pub fn tick(&mut self){
        if self.worker.is_none() && !self.pending && !self.disconnecting && self.message.is_empty(){self.start();}
        let response=self.worker.as_ref().map(Worker::poll);
        match response{
            Some(Ok(Some(Reply::Checked{generation,result}))) if generation==self.state.generation()=>{
                self.pending=false;
                self.message=match result.and_then(|checked|{
                    self.state.apply_checked(generation,checked.token,checked.answer,checked.verified_at)?;Ok(checked.warning)
                }){
                    Ok(warning)=>{
                        let status=if self.allowed(){"Your account reports active access."}else if !self.state.code().is_empty(){"Enter this code on your account page, then check again."}else{"Approve this device on the website, then check again."};
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
            self.activation_controls(ui);
            if ui.button("Manage account").clicked(){let _=open_browser(account::ACCOUNT_URL);}
            ui.separator();
            ui.small("Sign-in uses OS credential storage when available. Passwords and payment details stay in your browser.");
            if self.pending{ctx.request_repaint_after(Duration::from_millis(250));}
        });
        self.open=open;
    }
    pub fn show_easy(&mut self, ui:&mut egui::Ui){
        if self.worker.is_none() && !self.pending && !self.disconnecting { self.start(); }
        ui.label("Your creative workspace, connected to your account.");
        if !self.message.is_empty(){ui.label(&self.message);}
        self.activation_controls(ui);
        if ui.button("Your free Pixa account").clicked(){
            if !open_browser("https://aiwebdesignfirm.com/account"){self.message="Could not open the account page. Go to aiwebdesignfirm.com/start.".into();}
        }
    }
    fn activation_controls(&mut self, ui:&mut egui::Ui){
        if !self.state.code().is_empty(){
            ui.add_space(18.0);
            ui.label(egui::RichText::new(self.state.code()).monospace().size(28.0));
            ui.label("Open your account and enter this code. Only approve codes shown on your own device.");
            if ui.button("Activate this device").clicked(){
                if !open_browser(account::ACTIVATE_URL){self.message="Could not open the activation page. Go to aiwebdesignfirm.com/pixashop/activate.".into();}
            }
        }
        if ui.add_enabled(!self.pending,egui::Button::new("Check again")).clicked(){self.start();}
        if ui.add_enabled(!self.disconnecting,egui::Button::new("Use another account")).clicked(){self.disconnect();}
        if self.pending{ui.spinner();}
    }
}

fn open_browser(url:&str)->bool{
    #[cfg(target_os="macos")]
    let mut command=std::process::Command::new("open");
    #[cfg(target_os="windows")]
    let mut command=std::process::Command::new("cmd");
    #[cfg(all(unix,not(target_os="macos")))]
    let mut command=std::process::Command::new("xdg-open");
    #[cfg(target_os="windows")]
    {command.args(["/C","start","",url]);}
    #[cfg(not(target_os="windows"))]
    {command.arg(url);}
    command.spawn().is_ok()
}
