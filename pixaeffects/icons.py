"""Original PixaEffects pixel-grid mark. Pillow is packaging tooling only."""
from pathlib import Path

def generate(root: Path):
    from PIL import Image, ImageDraw
    base = Image.new('RGBA', (1024, 1024))
    d = ImageDraw.Draw(base)
    d.rounded_rectangle((8, 8, 1016, 1016), radius=220, fill='#101e2c')
    # A geometric P made from a grid: no dependency on a licensed typeface.
    for x, y in [(0,0),(1,0),(2,0),(0,1),(0,2),(1,2),(2,2),(0,3),(0,4),(1,4),(2,4)]:
        left, top = 270 + x * 145, 175 + y * 140
        d.rounded_rectangle((left, top, left + 116, top + 112), radius=18, fill='#54d5ee')
    folder = root / 'assets/app-icon'
    for p in folder.rglob('*.png'):
        with Image.open(p) as old:
            size = old.size
        base.resize(size, Image.Resampling.LANCZOS).save(p)
        branded = p.with_name(p.name.replace('ai.storyteller.effectcraft', 'com.aiwebdesignfirm.pixaeffects'))
        if branded != p:
            base.resize(size, Image.Resampling.LANCZOS).save(branded)
    base.save(folder / 'effectcraft.ico', sizes=[(16,16),(32,32),(48,48),(64,64),(128,128),(256,256)])
    base.save(folder / 'effectcraft.icns')
    svg = '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100"><rect width="100" height="100" rx="22" fill="#101e2c"/>'
    for x, y in [(0,0),(1,0),(2,0),(0,1),(0,2),(1,2),(2,2),(0,3),(0,4),(1,4),(2,4)]:
        svg += f'<rect x="{27+x*14}" y="{17+y*14}" width="11" height="11" rx="2" fill="#54d5ee"/>'
    svg += '</svg>'
    for p in folder.rglob('*.svg'):
        p.write_text(svg)

    for f in folder.rglob('*'):
        if f.is_file() and f.suffix in {'.png','.svg','.ico','.icns'}:
            f.with_name(f.name+'.attribution').write_text(f'asset: {f.name}\nauthor: AIWebDesignFirm\nsource: original work (pixaeffects/icons.py)\nlicense: MIT OR Apache-2.0\nadded: 2026-11-09\n')
