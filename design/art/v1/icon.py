from art import C
def bindle24(cv,ox=0,oy=0):
    cv.line(ox+1,oy+23,ox+16,oy+8,'k'); cv.line(ox+1,oy+22,ox+15,oy+8,'b'); cv.line(ox+0,oy+22,ox+15,oy+7,'y'); cv.line(ox+0,oy+23,ox+0,oy+23,'k')
    S=cv.ell(ox+17.5,oy+15.5,6.2,5.9,'c','k')
    for (x,y) in S:
        lx,ly=x-ox,y-oy
        if cv.get(x,y)=='c':
            if ((lx-17.5)/6.2)**2+((ly-15.5)/5.9)**2<1 and lx<17 and ly<15 and (lx-12)**2+(ly-11)**2<14: cv.set(x,y,'l')
            if ly>=19 or (lx>=21 and ly>=17): cv.set(x,y,'d')
    for (x,y) in [(15,13),(19,12),(21,16),(17,17),(14,18),(20,20)]:
        if cv.get(ox+x,oy+y) in ('c','d','l'): cv.set(ox+x,oy+y,'p')
    cv.ascii(ox+14,oy+3,["kk...kk","klk.kck",".kkckk.","..kdk..","..kdk..",".kdddk."])
if __name__=='__main__':
    ic=C(24,24); bindle24(ic); ic.save('bindle-24.png',10)
