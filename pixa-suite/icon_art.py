"""Original Pixa pixel letterforms; no font or third-party artwork. MIT OR Apache-2.0."""
from PIL import Image, ImageDraw

GLYPHS = {
    'A': ['010','101','111','101','101'],
    'C': ['111','100','100','100','111'],
    'D': ['110','101','101','101','110'],
    'E': ['111','100','110','100','111'],
    'F': ['111','100','110','100','100'],
    'G': ['111','100','101','101','111'],
    'I': ['111','010','010','010','111'],
    'L': ['100','100','100','100','111'],
    'M': ['10001','11011','10101','10001','10001'],
    'P': ['111','101','111','100','100'],
    'S': ['111','100','111','001','111'],
    'T': ['111','010','010','010','010'],
    'V': ['101','101','101','101','010'],
    'W': ['10001','10001','10101','11011','10001'],
}

def rectangles(mark):
    """Return the same centered pixel geometry for raster, SVG and inline marks."""
    if not mark or len(mark)>3 or any(c not in GLYPHS for c in mark):
        raise ValueError('Unsupported Pixa icon mark')
    width=sum(len(GLYPHS[c][0]) for c in mark)+len(mark)-1
    step=min(140,680/width)
    left=(1024-width*step+step*.18)/2
    top=(1024-5*step+step*.18)/2
    result=[]; offset=0
    for c in mark:
        rows=GLYPHS[c]
        for y,row in enumerate(rows):
            for x,value in enumerate(row):
                if value=='1':result.append((left+(offset+x)*step,top+y*step,step*.82,step*.82,step*.12))
        offset+=len(rows[0])+1
    return result

def render(mark,color,background,size):
    if not 16<=size<=1024:raise ValueError('Icon size must be 16–1024')
    image=Image.new('RGBA',(1024,1024))
    draw=ImageDraw.Draw(image)
    draw.rounded_rectangle((8,8,1016,1016),radius=220,fill=background)
    for x,y,w,h,r in rectangles(mark):draw.rounded_rectangle((x,y,x+w,y+h),radius=r,fill=color)
    return image.resize((size,size),Image.Resampling.LANCZOS)

def svg(mark,color,background):
    pixels=''.join(f'<rect x="{x:.3f}" y="{y:.3f}" width="{w:.3f}" height="{h:.3f}" rx="{r:.3f}" fill="{color}"/>' for x,y,w,h,r in rectangles(mark))
    return f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1024 1024"><rect x="8" y="8" width="1008" height="1008" rx="220" fill="{background}"/>{pixels}</svg>\n'

def inline_path(mark):
    return ''.join(f'M{x/10.24:.3f},{y/10.24:.3f}h{w/10.24:.3f}v{h/10.24:.3f}h{-w/10.24:.3f}Z' for x,y,w,h,_ in rectangles(mark))
