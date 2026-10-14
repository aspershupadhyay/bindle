from PIL import Image
import math, random
P = {
 'k':(43,38,34),'m':(110,101,92),'c':(194,101,61),'d':(164,80,44),'l':(232,154,114),
 's':(107,143,113),'g':(79,110,85),'h':(217,164,65),'y':(240,207,133),'b':(122,83,38),
 'p':(250,247,242),'w':(255,253,249),'n':(232,225,214),'e':(242,236,227),'q':(150,180,155),
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

def bindle(cv,ox,oy,size=24):
    # stick
    f=size/24
    cv_line=lambda a,b,c,d,col: cv.line(ox+round(a*f),oy+round(b*f),ox+round(c*f),oy+round(d*f),col)
    cv_line(1,22,20,5,'b'); cv_line(2,22,21,5,'k') if size>=24 else None
    cv_line(1,23,2,22,'k')
    # bag
    S=cv.ell(ox+15.5*f,oy+15*f,6.2*f,5.6*f,'c','k')
    for (x,y) in S:
        lx,ly=x-ox,y-oy
        if cv.get(x,y)=='c':
            if (lx+ly)%4==0 and ((lx)%2==0): cv.set(x,y,'p')
    # highlight upper-left inside
    for (x,y) in S:
        lx,ly=x-ox,y-oy
        if cv.get(x,y)=='c' and lx<13*f and ly<13*f and (lx+ly)<24*f: cv.set(x,y,'l')
    # shade bottom-right
    for (x,y) in S:
        lx,ly=x-ox,y-oy
        if cv.get(x,y)=='c' and ly>17.5*f: cv.set(x,y,'d')
    # knot
    cv.ascii(ox+round(13*f),oy+round(6*f),["k...k","kck.kck"[:0]+"kckck",".kck.","..k.."] if True else [])

# 1) icon 24
ic=C(24,24); bindle(ic,0,0); ic.save('bindle-24.png',10)
