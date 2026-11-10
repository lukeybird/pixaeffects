//! Shared Pixa shell. Protected previews never run an editor without access and a recovery adapter.
use eframe::egui;
use pixa_runtime::guard::{BuildMode, Guard};
#[path = "pixa_account.rs"] mod account_ui;
#[path = "pixa_help.rs"] mod help_ui;
#[path = "pixa_recovery.rs"] mod recovery;
type Editor = @EDITOR_TYPE@;
struct PixaShell {
    inner: Box<Editor>,
    help: help_ui::HelpUi,
    account: account_ui::AccountUi,
    guard: Guard,
    updates_open: bool,
    recovery_message: String,
    discard_prompt: bool,
    discard_confirmed: bool,
}
pub fn subscription_build() -> bool { BuildMode::parse(option_env!("PIXA_ACCESS_MODE")) != BuildMode::Development }
pub fn wrap(inner: Box<Editor>) -> Box<dyn eframe::App> {
    Box::new(PixaShell { inner, help: help_ui::HelpUi::default(), account: account_ui::AccountUi::default(),
        guard: Guard::new(BuildMode::parse(option_env!("PIXA_ACCESS_MODE")), recovery::ready()), updates_open: false,
        recovery_message: String::new(), discard_prompt: false, discard_confirmed: false })
}
impl PixaShell {
    fn can_edit(&self) -> bool { self.guard.can_edit(self.account.allowed()) }
    fn locked(&mut self, ui: &mut egui::Ui) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(56.0);ui.heading("@NAME@");ui.label("by AIWebDesignFirm");ui.add_space(18.0);
                if !self.guard.configuration_valid() {
                    ui.label("This preview has an invalid build configuration.");
                } else if !self.guard.adapter_ready() {
                    ui.label("This editor's restricted preview is not configured yet.");
                } else {
                    ui.label("Connect an account with active access to use the editor.");
                    if ui.button("Your account").clicked() { self.account.open=true; }
                }
                if self.guard.needs_close_review(self.account.allowed()) {
                    ui.label("Your project remains in memory. Reconnect to continue, or save a recovery copy before closing.");
                    if ui.button("Save recovery copy…").clicked() {
                        if let Some(parent)=recovery::choose_folder() {
                            self.recovery_message=match pixa_runtime::recovery::create_directory(&parent,"@ID@")
                                .and_then(|dir| recovery::save(self.inner.as_mut(),&dir).map(|()|dir)) {
                                Ok(dir)=>{self.guard.recovery_succeeded();format!("Recovery copy written to {}. Unfinished field edits may still require reconnecting.",dir.display())}
                                Err(error)=>error,
                            };
                        }
                    }
                    if ui.button(if self.guard.recovery_saved(){"Review recovery copy and close…"}else{"Close without a recovery copy…"}).clicked() { self.discard_prompt=true; }
                }
                if !self.recovery_message.is_empty() { ui.label(&self.recovery_message); }
                ui.small("Restricted development preview. Do not distribute before save, recovery and activation testing is complete.");
            });
        });
    }
}
impl eframe::App for PixaShell {
    fn logic(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.account.tick();
        if self.can_edit() {
            self.discard_confirmed=false;self.discard_prompt=false;self.recovery_message.clear();
            self.guard.note_editor_used(self.account.allowed());
            eframe::App::logic(self.inner.as_mut(),ctx,frame);
        } else if ctx.input(|i|i.viewport().close_requested()) && !self.guard.may_close(self.account.allowed(),self.discard_confirmed) {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
        }
        if self.account.needs_tick() { ctx.request_repaint_after(std::time::Duration::from_millis(500)); }
    }
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        egui::Panel::bottom("pixa-publisher").exact_size(26.0).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.small("@NAME@ by AIWebDesignFirm");ui.separator();
                if ui.small_button("Manual").clicked(){self.help.open=true;}
                if ui.small_button("Quick orientation").clicked(){self.help.start_tour();}
                if ui.small_button("Your account").clicked(){self.account.open=true;}
                if ui.small_button("Updates").clicked(){self.updates_open=true;}
                ui.small(if subscription_build(){"Restricted preview"}else{"Development preview"});
            });
        });
        if self.can_edit(){eframe::App::ui(self.inner.as_mut(),ui,frame);}else{self.locked(ui);}
        self.account.show(ui.ctx());self.help.show(ui.ctx());
        if self.discard_prompt {
            egui::Window::new("Close this editing session?").resizable(false).show(ui.ctx(),|ui|{
                ui.label(if self.guard.recovery_saved(){"The recovery copy contains committed project data. Reconnect before closing if you need unfinished fields or background jobs."}else{"Closing without a recovery copy can lose unsaved changes. Reconnect if you need to finish an edit first."});
                if ui.button("Keep the app open").clicked(){self.discard_prompt=false;}
                if ui.button("Discard and close").clicked(){self.discard_confirmed=true;ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);}
            });
        }
        egui::Window::new("@NAME@ — Updates").open(&mut self.updates_open).show(ui.ctx(),|ui|{
            ui.label("Pixa updates are not published for this development build.");
            ui.label("The publisher must configure a signed Pixa release feed before installation can be enabled.");
        });
    }
    fn raw_input_hook(&mut self,ctx:&egui::Context,raw:&mut egui::RawInput){
        if self.can_edit(){eframe::App::raw_input_hook(self.inner.as_mut(),ctx,raw);}else{raw.dropped_files.clear();raw.hovered_files.clear();}
    }
    fn save(&mut self,storage:&mut dyn eframe::Storage){eframe::App::save(self.inner.as_mut(),storage);}
    @ON_EXIT@
    fn auto_save_interval(&self)->std::time::Duration{eframe::App::auto_save_interval(self.inner.as_ref())}
    fn clear_color(&self,v:&egui::Visuals)->[f32;4]{eframe::App::clear_color(self.inner.as_ref(),v)}
    fn persist_egui_memory(&self)->bool{eframe::App::persist_egui_memory(self.inner.as_ref())}
}
