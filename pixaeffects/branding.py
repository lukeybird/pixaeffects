"""Finish product display branding without changing licenses or crate/file formats."""
from pathlib import Path
import re

PAGE = 'https://aiwebdesignfirm.com/pixaeffects'


def clean_metadata(text, filename):
    for old, new in [('Learning Machines LLC', 'AIWebDesignFirm'), ('https://getartcraft.com/apps/effectcraft', PAGE), ('https://getartcraft.com', 'https://aiwebdesignfirm.com'), ('https://discord.gg/artcraft', PAGE), ('ai.storyteller.effectcraft', 'com.aiwebdesignfirm.pixaeffects')]:
        text = text.replace(old, new)
    text = text.replace('<developer id="ai.storyteller">', '<developer id="com.aiwebdesignfirm">')
    if '.metainfo.xml' in filename:
        text = re.sub(r'\s*<screenshots>[\s\S]*?</screenshots>', '', text)
    return text


def finish_branding(root: Path):
    help_file = root / 'crates/engine/src/cmd/help.rs'
    text = help_file.read_text()
    text = '\n'.join(line for line in text.split('\n') if not any('cmd!("' + key + '"' in line for key in ['help.discord', 'help.github']))
    help_file.write_text(text)
    for relative in ['crates/ui-egui/src/panels/dialogs.rs', 'crates/ui-egui/src/panels/home.rs']:
        file = root / relative
        file.write_text(file.read_text().replace('"help.discord"', '"help.account"'))
    # Remove obsolete translated promotion entries, leaving other translations and credits.
    for relative in ['crates/ui-egui/src/i18n.rs', 'crates/ui-egui/src/i18n/ui.rs']:
        file = root / relative
        lines = []
        for line in file.read_text().splitlines(keepends=True):
            if 'getartcraft.com' in line or re.search(r'^\s*\("(?:Discord|PixaEffects on GitHub|Join[^"\n]*Discord)[" ,]', line):
                continue
            lines.append(line)
        file.write_text(''.join(lines))
    for file in (root / 'packaging').rglob('*'):
        if file.is_file() and file.suffix in {'.in', '.xml', '.yaml', '.yml'}:
            file.write_text(clean_metadata(file.read_text(), file.name))
    demo = root / 'crates/engine/src/home/demo.rs'
    demo.write_text(demo.read_text().replace('"EFFECTCRAFT"', '"PIXAEFFECTS"'))
    main = root / 'apps/effectcraft/src/main.rs'
    main.write_text(main.read_text().replace('println!("effectcraft {}"', 'println!("PixaEffects {}"').replace('eprintln!("effectcraft: {e}")', 'eprintln!("PixaEffects: {e}")'))
