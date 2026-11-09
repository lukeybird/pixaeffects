#!/usr/bin/env python3
"""Prepare the PixaEffects product from its pinned, clean EffectCraft checkout."""
from pathlib import Path
import re, shutil
ROOT=Path(__file__).resolve().parent.parent

def edit(path,old,new):
 p=ROOT/path;s=p.read_text()
 if old not in s: raise RuntimeError(f'Integration point changed: {path}: {old[:70]}')
 p.write_text(s.replace(old,new,1))

def prepare():
 if (ROOT/'pixaeffects/.prepared').exists():
  print('PixaEffects already prepared');return
 # Display names only: crate names and project file compatibility remain unchanged.
 for parent in ['apps','crates','packaging']:
  for p in (ROOT/parent).rglob('*'):
   if not p.is_file() or p.suffix not in {'.rs','.tsv','.html','.in','.xml','.desktop','.wxs','.wxl','.ps1','.sh','.txt'}:continue
   if any(x in p.name.lower() for x in ['license','attribution','notice']) or 'credits' in p.parts:continue
   try:s=p.read_text()
   except UnicodeDecodeError:continue
   s='\n'.join(line if 'copyright' in line.lower() else line.replace('EffectCraft','PixaEffects') for line in s.split('\n'))
   s=s.replace('ai.storyteller.effectcraft','com.aiwebdesignfirm.pixaeffects')
   s=s.replace('https://github.com/storytold/effectcraft/issues','https://aiwebdesignfirm.com/pixaeffects').replace('https://github.com/storytold/effectcraft','https://aiwebdesignfirm.com/pixaeffects/manual')
   s=s.replace('https://getartcraft.com/apps/effectcraft','https://aiwebdesignfirm.com/pixaeffects').replace('https://getartcraft.com','https://aiwebdesignfirm.com').replace('https://discord.gg/artcraft','https://aiwebdesignfirm.com/account')
   p.write_text(s)
 for p in (ROOT/'pixaeffects/overlay').rglob('*'):
  if p.is_file():
   target=ROOT/p.relative_to(ROOT/'pixaeffects/overlay');target.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(p,target)
 edit('Cargo.toml','version = "0.21.0"','version = "0.1.0"')
 edit('apps/effectcraft/src/main.rs','struct Desktop {','mod access;\nstruct Desktop {')
 edit('apps/effectcraft/src/main.rs','Ok(Box::new(Desktop {','Ok(Box::new(access::AccessApp::new(Desktop {')
 edit('apps/effectcraft/src/main.rs','reporter: desktop_reporter,\n            }))','reporter: desktop_reporter,\n            })))')
 edit('apps/effectcraft/src/main.rs','    let Args { control_port, files, demo, home } = match parse_args','    let Args { control_port: _, files, demo, home } = match parse_args')
 edit('apps/effectcraft/src/main.rs','    if let Some(dir) = effectcraft_host::config_dir() {','    // Paid desktop does not expose a control server outside its entitlement gate.\n    let control_port: Option<u16> = None;\n    if let Some(dir) = effectcraft_host::config_dir() {')
 edit('apps/effectcraft/Cargo.toml','serde_json = { workspace = true }','serde_json = { workspace = true }\nureq = { version = "3.4", features = ["json"] }')
 edit('crates/ui-egui/Cargo.toml','craft-update = { workspace = true }','craft-update = { workspace = true }\nureq = { version = "3.4", features = ["json"] }')
 edit('crates/ui-egui/src/lib.rs','pub mod updates_ui;','pub mod updates_ui;\nmod pixaeffects_updater;\nmod pixaeffects_help;')
 # Windows upstream finish helper is not used: custom verified installation is Mac-only.
 p=ROOT/'apps/effectcraft/src/main.rs';s=p.read_text();s=re.sub(r'    #\[cfg\(windows\)\]\n    if let Some\(result\) = craft_update::self_update::finish[\s\S]*?\n    }\n','',s,count=1);p.write_text(s)
 p=ROOT/'crates/ui-egui/src/lib.rs';s=p.read_text().replace('..craft_update::suite::EFFECTCRAFT','developer: "AIWebDesignFirm",\n    ..craft_update::app::App::new("pixaeffects", "PixaEffects", "Motion graphics and compositing", "lukeybird/pixaeffects")');p.write_text(s)
 p=ROOT/'crates/host/src/app_dirs.rs';s=p.read_text().replace('"effectcraft"','"pixaeffects"').replace('EFFECTCRAFT_CONFIG_DIR','PIXAEFFECTS_CONFIG_DIR');p.write_text(s)
 # Replace promotional surfaces; retain truthful contributor and license notices.
 p=ROOT/'crates/engine/src/menus.rs';s=p.read_text();s='\n'.join(l for l in s.split('\n') if not any(t in l for t in ['| help.discord','| help.github']));s=s.replace('ArtCraft Website','AIWebDesignFirm Website').replace('In-App Tutorials... |','Quick Tool Tour... |').replace('PixaEffects Help... |','PixaEffects User Manual... |');p.write_text(s)
 p=ROOT/'crates/engine/src/cmd/help.rs';s=p.read_text();start=s.index('    let page = match');end=s.index('\n}',start);s=s[:start]+'    let _ = p;\n    open(s, "https://aiwebdesignfirm.com/pixaeffects/manual".into())'+s[end:];s=s.replace('ArtCraft Website','AIWebDesignFirm Website');p.write_text(s)
 p=ROOT/'crates/engine/src/home/links.rs';s=p.read_text();a=s.index('pub const SIBLINGS:');b=s.index('\n];',a)+3;s=s[:a]+'pub const SIBLINGS: &[(&str, &str)] = &[];'+s[b:];s=s.replace('format!("https://github.com/storytold/{slug}")','{ let _ = slug; "https://aiwebdesignfirm.com".into() }').replace('format!("https://aiwebdesignfirm.com/apps/{slug}")','{ let _ = slug; "https://aiwebdesignfirm.com".into() }');p.write_text(s)
 p=ROOT/'crates/ui-egui/src/header.rs';s=p.read_text();a=s.index('    let t = app.tokens;',s.index('fn community_buttons'));b=s.index('\n}\n',a);s=s[:a]+'''    let _ = (app, p, cy);
    if ui.button("Account").clicked() { ui.ctx().open_url(egui::OpenUrl::new_tab("https://aiwebdesignfirm.com/account")); }
    rx -= 70.0;
    rx'''+s[b:];s=s.replace('../../../docs/brand/artcraft-mark.png','../../../assets/app-icon/pixaeffects-mark.png');p.write_text(s)
 # Header-generated title, About and Home promotional labels.
 for p in (ROOT/'crates/ui-egui/src').rglob('*.rs'):
  if p.name=='i18n.rs' or 'credits' in p.parts:continue
  s=p.read_text().replace('ArtCraft Website','AIWebDesignFirm Website').replace('Join the ArtCraft Discord','Manage your account').replace('Join the ArtCraft community on Discord','Manage your account')
  p.write_text(s)
 # Product About/Home have account and website links, no source repository promotion.
 for rel in ['crates/ui-egui/src/panels/dialogs.rs','crates/ui-egui/src/panels/home.rs']:
  p=ROOT/rel;s=p.read_text();s='\n'.join(line for line in s.split('\n') if not ('Icon::Code' in line and 'help.github' in line))
  s=s.replace("let links: [(Icon, &str, &str, &'static str); 4]", "let links: [(Icon, &str, &str, &'static str); 3]")
  s=s.replace('Part of the ArtCraft family of creative apps.', 'PixaEffects by AIWebDesignFirm.').replace('ArtCraft website','AIWebDesignFirm website').replace('getartcraft.com','aiwebdesignfirm.com').replace('More ArtCraft apps','by AIWebDesignFirm').replace('tr("Community")','tr("Your workspace")')
  p.write_text(s)
 edit('crates/ui-egui/src/header.rs','        tool_slot(app, ui, p, si, slot, r, held_id);','        for tool in *slot { crate::pixaeffects_help::record_tool(ui.ctx(), *tool, r); }\n        tool_slot(app, ui, p, si, slot, r, held_id);')
 # Intercept existing Help commands without editing projects or adding engine state.
 edit('crates/ui-egui/src/menus/invoke.rs','    crate::control::check_script_render(app, id)?;','''    if id == "help.docs" { crate::pixaeffects_help::open_manual(ctx); return Ok(Value::Null); }
    if id == "help.inAppTutorials" { crate::pixaeffects_help::open_tour(ctx); return Ok(Value::Null); }
    crate::control::check_script_render(app, id)?;''')
 edit('crates/ui-egui/src/app/frame.rs','        updates_ui::show(self, &ctx);','        updates_ui::show(self, &ctx);\n        crate::pixaeffects_help::show(&ctx);')
 from icons import generate
 generate(ROOT)
 from PIL import Image
 Image.open(ROOT/'assets/app-icon/effectcraft.ico').resize((256,256)).save(ROOT/'assets/app-icon/pixaeffects-mark.png')
 # Generated mark is original pixel geometry, not inherited ArtCraft artwork.
 (ROOT/'assets/app-icon/pixaeffects-mark.png.attribution').write_text('asset: pixaeffects-mark.png\nauthor: AIWebDesignFirm\nsource: original work\nlicense: MIT OR Apache-2.0\nadded: 2026-11-09\n')
 with (ROOT/'ATTRIBUTION.md').open('a') as f: f.write('\nPixaEffects pixel-E app icons: original geometry by AIWebDesignFirm, MIT OR Apache-2.0; generated by pixaeffects/icons.py.\n')
 (ROOT/'pixaeffects/.prepared').write_text('PixaEffects by AIWebDesignFirm\n')
 print('Prepared PixaEffects. Build Cargo target effectcraft.')
if __name__=='__main__':prepare()
