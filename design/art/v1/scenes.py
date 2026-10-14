import math
from art import C
from icon import bindle24

TRAVELLER=[ # 9x13, faces right, bindle over left shoulder handled separately
"...kkk...",
"..kkkkk..",
"..kllll..",
"..kllkl..",
"...lll...",
"..mmmmm..",
".mmmmmmm.",
".lmmmmml.",
"..mmmmm..",
"..kk.kk..",
"..kk.kk..",
"..k...k..",
".kk...kk.",
]
def traveller(cv,x,y,bag=True):
    cv.ascii(x,y,TRAVELLER)
    if bag:
        cv.line(x+1,y+7,x-5,y+1,'b')
        cv.ascii(x-8,y-2,[".kkk.","kclck","kcccck"[:5],".kkk."])

def hills():
    cv=C(96,40)
    # sky bands with dither
    for y in range(40):
        for x in range(96):
            t=y/26
            col='p'
            if t>0.45: col='e'
            if t>0.7: col='y' if (x+y)%2==0 and t<0.8 else ('y' if t>=0.8 else 'e')
            cv.set(x,y,col)
    # sun
    cv.ell(70,23,8,8,'h'); cv.ell(70,23,5,5,'y')
    # clouds
    for cx,cy in [(18,8),(44,5)]:
        cv.ell(cx,cy,6,2.2,'w'); cv.ell(cx+4,cy-1.5,3.5,2.2,'w'); cv.rect(cx-5,cy+1,cx+7,cy+1,'n')
    # birds
    for bx,by in [(54,10),(60,7),(58,13)]:
        cv.ascii(bx,by,["m.m",".m."])
    # far hills
    for x in range(96):
        h=int(25+3*math.sin(x/9)+2*math.sin(x/4.3))
        for y in range(h,40): cv.set(x,y,'q')
    # near hill
    for x in range(96):
        h=int(31+3*math.sin((x+20)/12)-2*math.cos(x/7))
        for y in range(h,40): cv.set(x,y,'s')
        cv.set(x,h,'g' if x%3 else 's')
    # grass tufts & dither shade
    for x in range(0,96,5):
        y=int(31+3*math.sin((x+20)/12)-2*math.cos(x/7))
        cv.set(x,y-1,'g')
    for y in range(35,40):
        for x in range(96):
            if (x+y)%2==0: cv.set(x,y,'g')
    # path
    for y in range(30,40):
        cx=int(30+ (y-30)*1.6)
        w=1+(y-30)//3
        for x in range(cx-w,cx+w+1): cv.set(x,y,'n')
    # traveller on hill top-left
    traveller(cv,24,17)
    # tiny flowers
    for fx,fy in [(8,36),(48,37),(80,34),(88,38)]:
        cv.set(fx,fy,'h'); cv.set(fx+1,fy,'c')
    return cv

def desk():
    cv=C(96,40)
    cv.rect(0,0,95,39,'e')
    # window
    cv.rect(62,2,89,17,'k'); cv.rect(63,3,88,16,'y'); cv.ell(82,9,3.5,3.5,'h')
    for y in range(3,17):
        for x in range(63,89):
            if y>12 and (x+y)%2==0: cv.set(x,y,'q')
            if y>14: cv.set(x,y,'s')
    cv.rect(75,3,75,16,'k'); cv.rect(63,9,88,9,'k')
    # sticky notes
    for i,(sx,sy,c) in enumerate([(8,4,'h'),(18,6,'l'),(28,3,'q')]):
        cv.rect(sx,sy,sx+7,sy+7,c); cv.rect(sx+1,sy+2,sx+5,sy+2,'m'); cv.rect(sx+1,sy+4,sx+4,sy+4,'m'); cv.set(sx+3,sy,'k')
    # desk
    cv.rect(0,28,95,30,'b'); cv.rect(0,28,95,28,'h'); cv.rect(0,31,95,39,'d')
    for x in range(0,96,2): cv.set(x,33,'b')
    # laptop
    cv.rect(22,12,52,27,'k'); cv.rect(24,14,50,25,'m')
    lines=[(26,16,36,'s'),(28,18,44,'h'),(28,20,40,'p'),(26,22,34,'l'),(28,24,46,'s')]
    for x0,y,x1,c in lines: cv.rect(x0,y,x1,y,c)
    cv.rect(18,27,56,28,'k'); cv.rect(19,27,55,27,'n')
    # chai glass
    cv.ascii(62,21,["kkkkkk","knnnnk","kyhhyk","khhhhk","khhhhk",".khhk.",".kkkk."])
    cv.ascii(63,15,[".n..",".n.n","n..n","n.n.",".n.."])
    # plant
    cv.ascii(78,18,["..s.s..",".sgsgs.","sgsgsgs",".sgsgs.","..sgs..","kkkkkkk",".kccck.",".kccck.",".kdddk.","..kkk.."])
    # bindle leaning
    return cv

def ours():
    cv=C(64,32)
    for y in range(32):
        for x in range(64):
            cv.set(x,y,'p' if y<16 else 'e')
    for x in range(64):
        h=int(25+2*math.sin(x/8))
        for y in range(h,32): cv.set(x,y,'s')
        cv.set(x,h,'g')
    # two travellers
    T2=[r.replace('kkk','kkk') for r in TRAVELLER]
    W=[ # wife with long hair
    "...kkk...",
    "..kkkkk..",
    ".kkllllk.",
    ".kkllklk.",
    ".kklllk..",
    "..ccccc..",
    ".ccccccc.",
    ".lcccccl.",
    "..ccccc..",
    "..ccccc..",
    "..kk.kk..",
    "..k...k..",
    ".kk...kk.",
    ]
    cv.ascii(18,12,TRAVELLER)
    cv.ascii(37,12,W)
    # shared stick across shoulders with bag in middle hanging
    cv.line(10,18,54,18,'b'); cv.line(10,19,54,19,'k')
    cv.ascii(28,17,[".kk.kk.","kclkck","kccccck"[:7],"kclcpck","kcccdck",".kdddk.","..kkk.."])
    # heart
    cv.ascii(29,2,[".hh.hh.","hyhhhhh","hhhhhhh",".hhhhh.","..hhh..","...h..."])
    return cv

def env():
    cv=C(32,32)
    cv.ascii(4,8,[
    "kkkkkkkkkkkkkkkkkkkkkkkk",
    "kwkwwwwwwwwwwwwwwwwwwkwk"[:24],
    "kwwkwwwwwwwwwwwwwwwwkwwk",
    "kwwwkwwwwwwwwwwwwwwkwwwk",
    "kwwwwkkwwwwwwwwwwkkwwwwk",
    "kwwwwwwkkwwwwwwkkwwwwwwk",
    "kwwwwwwwwkkkkkkwwwwwwwwk",
    "kwwwwwwwwwwwwwwwwwwwwwwk",
    "kwwwwwwwwwwwwwwwwwwwwwwk",
    "kwwwwwwwwwwwwwwwwwwwwwwk",
    "kwwwwwwwwwwwwwwwwwwwwwwk",
    "kwwwwwwwwwwwwwwwwwwwwwwk",
    "knnnnnnnnnnnnnnnnnnnnnnk",
    "kkkkkkkkkkkkkkkkkkkkkkkk"])
    cv.ell(25,22,5.5,5.5,'s','g'); cv.ascii(22,20,["....p","p..p.",".pp.."])
    return cv

def crate():
    cv=C(32,32)
    cv.rect(3,6,28,25,'k'); cv.rect(4,7,27,9,'n'); cv.set(5,8,'c'); cv.set(7,8,'h'); cv.set(9,8,'s')
    cv.rect(4,10,27,24,'m')
    cv.ascii(6,12,["k...","sk..",".sk.","sk..","k..."])
    cv.ascii(11,16,["hhhhh"])
    cv.ascii(6,19,["pppppppp..","ppppp....."][0:1])
    cv.rect(6,21,15,21,'l'); cv.rect(6,23,12,23,'q')
    return cv

def board():
    cv=C(32,32)
    cv.rect(2,4,29,27,'k'); cv.rect(3,5,28,26,'n')
    for i,c in enumerate(['h','l','q']):
        x=4+i*8
        cv.rect(x,7,x+6,12,c); cv.rect(x+1,9,x+4,9,'m')
    cv.rect(4,15,10,20,'h'); cv.rect(12,15,18,20,'l')
    cv.rect(5,17,8,17,'m'); cv.rect(13,17,16,17,'m')
    cv.rect(2,28,3,30,'k'); cv.rect(28,28,29,30,'k')
    return cv

def camera():
    cv=C(32,32)
    cv.ascii(3,8,[
    "....kkkkkk..............",
    "...kmmmmmmk.............",
    "kkkkkkkkkkkkkkkkkkkkkkkk",
    "kmmmmmmmmmmmmmmmmmmmmmhk",
    "kmmmmmmmkkkkkkkkmmmmmmmk",
    "kmmmmmmkkccccckkmmmmmmmk",
    "kmmmmmkccclllcckkmmmmmmk",
    "kmmmmmkcclkkklcckmmmmmmk",
    "kmmmmmkcclkpklcckmmmmmmk",
    "kmmmmmkccllllccckmmmmmmk",
    "kmmmmmmkkcccccckkmmmmmmk",
    "kmmmmmmmkkkkkkkkmmmmmmmk",
    "kmmmmmmmmmmmmmmmmmmmmmmk",
    "kkkkkkkkkkkkkkkkkkkkkkkk"])
    return cv

def avatar(kind):
    cv=C(16,16)
    cv.ell(8,8,7.8,7.8,'y' if kind=='s' else 'q')
    if kind=='s':
        cv.ascii(4,3,["..kkkkk.",".kkkkkkk",".kllllkk",".klklkl.",".kllllll","..lldll.","..llll..",".mmmmmm.","mmmmmmmm"])
    else:
        cv.ascii(3,2,["...kkkkk..","..kkkkkkk.",".kkllllkkk",".kklklkkkk",".kkllllkkk",".kklddlkk.","..kllllk..","..cccccc..",".cccccccc."])
    return cv

if __name__=='__main__':
    for name,fn in [('hills',hills),('desk',desk),('ours',ours),('env',env),('crate',crate),('board',board),('camera',camera)]:
        fn().save(f'{name}.png',8)
    avatar('s').save('avatar-s.png',8); avatar('w').save('avatar-w.png',8)
