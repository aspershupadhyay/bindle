from PIL import Image
import math, random
P = {
 'k':(43,38,34),'m':(110,101,92),'c':(194,101,61),'d':(164,80,44),'l':(232,154,114),
 's':(107,143,113),'g':(79,110,85),'h':(217,164,65),'y':(240,207,133),'b':(122,83,38),
 'p':(250,247,242),'w':(255,253,249),'n':(232,225,214),'e':(242,236,227),'q':(150,180,155),
 'r':(178,58,42),'R':(120,36,26),'o':(31,36,33),'O':(58,52,46),'a':(255,176,59),'A':(122,80,32),'G':(126,214,140),'u':(122,74,110),'U':(196,140,180),'t':(188,184,176),'T':(140,134,126),'B':(160,196,206),
}
class C:
    def __init__(s,w,h,bg=None):
        s.w,s.h=w,h; s.px=[[bg]*w for _ in range(h)]
    def set(s,x,y,c):
        if 0<=x<s.w and 0<=y<s.h: s.px[y][x]=c
    def get(s,x,y):
        return s.px[y][x] if 0<=x<s.w and 0<=y<s.h else None
    def rect(s,x0,y0,x1,y1,c):
        for y in range(y0,y1+1):
            for x in range(x0,x1+1): s.set(x,y,c)
    def ell(s,cx,cy,rx,ry,c,out=None):
        pts=[]
        for y in range(s.h):
            for x in range(s.w):
                if ((x+.5-cx)/rx)**2+((y+.5-cy)/ry)**2<=1: pts.append((x,y))
        for x,y in pts: s.set(x,y,c)
        if out:
            S=set(pts)
            for x,y in pts:
                if any((x+dx,y+dy) not in S for dx,dy in((1,0),(-1,0),(0,1),(0,-1))): s.set(x,y,out)
        return set(pts)
    def line(s,x0,y0,x1,y1,c):
        dx=abs(x1-x0); dy=-abs(y1-y0); sx=1 if x0<x1 else -1; sy=1 if y0<y1 else -1; e=dx+dy
        while True:
            s.set(x0,y0,c)
            if x0==x1 and y0==y1: break
            e2=2*e
            if e2>=dy: e+=dy; x0+=sx
            if e2<=dx: e+=dx; y0+=sy
    def ascii(s,ox,oy,rows):
        for j,r in enumerate(rows):
            for i,ch in enumerate(r):
                if ch!='.': s.set(ox+i,oy+j,ch)
    def save(s,path,scale=1):
        im=Image.new('RGBA',(s.w,s.h),(0,0,0,0))
        for y in range(s.h):
            for x in range(s.w):
                c=s.px[y][x]
                if c: im.putpixel((x,y),P[c]+(255,))
        im.save(path)
        if scale>1: im.resize((s.w*scale,s.h*scale),Image.NEAREST).save(path.replace('.png',f'@{scale}x.png'))

