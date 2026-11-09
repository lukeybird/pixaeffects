"""Finder drag-to-install presentation shared by builds and repackaging."""
from pathlib import Path
import sys


def build_installer(app, destination):
    import dmgbuild
    app, destination = Path(app), Path(destination)
    if not (app / 'Contents/Info.plist').is_file():
        raise ValueError('Expected a complete PixaEffects.app bundle')
    dmgbuild.build_dmg(str(destination), 'PixaEffects', settings={
        'format': 'UDZO',
        'files': [str(app)],
        'symlinks': {'Applications': '/Applications'},
        'background': 'builtin-arrow',
        'icon_locations': {'PixaEffects.app': (140, 120), 'Applications': (500, 120)},
        'window_rect': ((200, 200), (640, 280)),
        'default_view': 'icon-view',
        'icon_size': 96,
        'text_size': 14,
        'show_toolbar': False,
        'show_status_bar': False,
        'show_sidebar': False,
        'show_pathbar': False,
        'show_tab_view': False,
        'badge_icon': str(app / 'Contents/Resources/PixaEffects.icns'),
    })


if __name__ == '__main__':
    build_installer(*sys.argv[1:])
