import math, random
from art import C

random.seed(7)

TRAV = [
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

def person(cv, x, y, shirt='m', hair='k'):
    rows = [r.replace('m', shirt).replace('k', hair, 0) for r in TRAV]
    cv.ascii(x, y, rows)

def dither_sky(cv, w, h, top, mid, low, split1, split2):
    for y in range(h):
        for x in range(w):
            if y < split1: c = top
            elif y < split1 + 2: c = mid if (x + y) % 2 == 0 else top
            elif y < split2: c = mid
            elif y < split2 + 2: c = low if (x + y) % 2 == 0 else mid
            else: c = low
            cv.set(x, y, c)

# ---------- Desk: railway platform at dusk ----------
def station():
    cv = C(96, 40)
    dither_sky(cv, 96, 40, 'O', 'u', 'l', 6, 14)
    # stars
    for sx, sy in [(8, 2), (30, 4), (51, 1), (77, 3), (88, 5), (20, 7)]:
        cv.set(sx, sy, 'y')
    # far city
    for bx, bw, bh in [(0, 7, 9), (8, 5, 13), (14, 8, 7), (60, 6, 11), (67, 9, 8), (78, 5, 14), (84, 12, 9)]:
        cv.rect(bx, 26 - bh, bx + bw - 1, 25, 'O')
        for wy in range(26 - bh + 2, 25, 3):
            for wx in range(bx + 1, bx + bw - 1, 2):
                if random.random() < 0.45: cv.set(wx, wy, 'h')
    # platform roof
    cv.rect(0, 0, 95, 2, 'k'); cv.rect(0, 3, 95, 3, 'd')
    for x in range(0, 96, 4): cv.set(x, 3, 'c')
    # pillars
    for px in (4, 46, 90):
        cv.rect(px, 4, px + 1, 31, 'k'); cv.set(px, 4, 'm')
    # dot matrix board hanging
    cv.line(17, 4, 17, 6, 'k'); cv.line(38, 4, 38, 6, 'k')
    cv.rect(12, 7, 43, 16, 'k'); cv.rect(13, 8, 42, 15, 'o')
    # amber text dots
    rows = [".a.aaa...aaa.aaa..a..",
            "aa.a.a.a...a.a...aaa.",
            ".a.a.a.....a.aa..a.a.",
            ".a.a.a.a.aaa...a.a.a.",
            ".a.aaa...aaa.aa......"]
    for j, r in enumerate(rows):
        for i, ch in enumerate(r):
            if ch == 'a': cv.set(15 + i, 9 + j, 'a')
    cv.rect(37, 9, 40, 9, 'A'); cv.rect(37, 11, 40, 11, 'A'); cv.rect(37, 13, 39, 13, 'a')
    # clock
    cv.ell(56, 9.5, 4, 4, 'w', 'k'); cv.set(56, 8, 'k'); cv.set(56, 9, 'k'); cv.set(57, 9, 'k'); cv.set(58, 9, 'k')
    cv.line(56, 4, 56, 5, 'k')
    # train on the right
    cv.rect(62, 17, 95, 31, 'c'); cv.rect(62, 17, 95, 17, 'k'); cv.rect(62, 31, 95, 31, 'k')
    cv.line(62, 17, 62, 31, 'k')
    cv.rect(63, 18, 95, 19, 'd')
    for wx in range(66, 95, 8):
        cv.rect(wx, 21, wx + 4, 25, 'k'); cv.rect(wx + 1, 22, wx + 3, 24, 'y')
    cv.rect(63, 27, 95, 28, 'h')
    cv.ell(66, 23, 1.5, 1.5, 'y')
    # platform
    cv.rect(0, 32, 95, 39, 'n')
    for x in range(96): cv.set(x, 32, 'h' if x % 4 < 2 else 'k')
    for y in range(34, 40):
        for x in range(96):
            if (x + y) % 6 == 0: cv.set(x, y, 'e')
    # bench
    cv.rect(26, 27, 38, 28, 'b'); cv.rect(27, 29, 27, 31, 'k'); cv.rect(37, 29, 37, 31, 'k')
    # traveller with bindle
    person(cv, 47, 19, shirt='s')
    cv.line(48, 26, 43, 21, 'b')
    cv.ascii(40, 18, [".kkk.", "kclck", "kccck", ".kkk."])
    # chai cup on bench
    cv.ascii(30, 25, ["ww", "nn"])
    return cv

# ---------- Inbox: the dak ghar ----------
def dakghar():
    cv = C(96, 40)
    cv.rect(0, 0, 95, 39, 'e')
    # brick wall
    for y in range(0, 30):
        for x in range(96):
            off = 3 if (y // 3) % 2 else 0
            if y % 3 == 2 or (x + off) % 8 == 0: cv.set(x, y, 'n')
            else: cv.set(x, y, 'l' if (x * 7 + y * 3) % 11 else 'c')
    # pigeonhole cabinet
    cv.rect(4, 3, 41, 27, 'b'); cv.rect(5, 4, 40, 26, 'k')
    for r in range(4):
        for c in range(6):
            x0 = 6 + c * 6; y0 = 5 + r * 5
            cv.rect(x0, y0, x0 + 4, y0 + 3, 'O')
            k = (r * 6 + c) % 5
            if k in (0, 2, 3):
                cv.rect(x0 + 1, y0 + 1, x0 + 3, y0 + 3, 'w' if k else 'y')
                cv.set(x0 + 3, y0 + 1, 'c')
    # sign
    cv.rect(50, 3, 76, 9, 'k'); cv.rect(51, 4, 75, 8, 'h')
    for i, ch in enumerate("k.kk.k.k..kk.k.k.kk"):
        if ch == 'k': cv.set(54 + i, 6, 'k')
    # floor
    cv.rect(0, 30, 95, 39, 'n')
    for x in range(0, 96, 2): cv.set(x, 30, 'm')
    for y in range(32, 40):
        for x in range(96):
            if (x - y) % 7 == 0: cv.set(x, y, 'e')
    # pillar post box
    cv.ell(62, 16, 7, 4, 'r', 'k')
    cv.rect(55, 16, 69, 34, 'r'); cv.line(55, 16, 55, 34, 'k'); cv.line(69, 16, 69, 34, 'k')
    cv.rect(54, 14, 70, 15, 'R'); cv.rect(54, 14, 70, 14, 'k')
    cv.rect(58, 20, 66, 21, 'k')  # slot
    cv.rect(58, 24, 66, 28, 'w'); cv.rect(59, 25, 65, 25, 'm'); cv.rect(59, 27, 63, 27, 'm')
    cv.line(56, 17, 56, 33, 'R'); cv.line(68, 17, 68, 33, 'l')
    cv.rect(53, 34, 71, 36, 'k')
    cv.rect(53, 34, 71, 36, 'k')
    # flying envelopes
    for ex, ey in [(78, 12), (86, 6), (82, 20)]:
        cv.ascii(ex, ey, ["kkkkkk", "kwkkwk", "kwwwwk", "kkkkkk"])
    cv.ascii(80, 15, ["m.", ".m"]); cv.ascii(84, 10, ["m.", ".m"])
    # postman with cycle bag
    person(cv, 38, 21, shirt='T')
    cv.rect(36, 27, 39, 31, 'b'); cv.set(37, 28, 'w'); cv.set(38, 28, 'w')
    # pigeon on box
    cv.ascii(63, 8, [".ww.", "wnwwk", ".ww.h", ".h.h."])
    return cv

# ---------- Board: workshop pegboard ----------
def workshop():
    cv = C(96, 40)
    cv.rect(0, 0, 95, 39, 'y')
    for y in range(1, 30, 3):
        for x in range(1, 96, 3): cv.set(x, y, 'h')
    # shelves of tags (cards)
    tags = [(8, 4, 'w'), (16, 4, 'w'), (8, 12, 'q'), (24, 4, 'l'), (24, 12, 'w'), (16, 12, 'w')]
    for tx, ty, col in tags:
        cv.set(tx + 2, ty - 1, 'k')
        cv.rect(tx, ty, tx + 5, ty + 6, col); cv.rect(tx, ty, tx + 5, ty, 'k') if False else None
        for (a, b) in [(tx, ty), (tx + 5, ty), (tx, ty + 6), (tx + 5, ty + 6)]: pass
        cv.line(tx, ty + 6, tx + 5, ty + 6, 'n')
        cv.set(tx + 2, ty + 1, 'k')
        cv.line(tx + 1, ty + 3, tx + 4, ty + 3, 'm'); cv.line(tx + 1, ty + 4, tx + 3, ty + 4, 'm')
    # tool outlines: hammer, wrench, saw
    cv.rect(40, 4, 41, 18, 'b'); cv.rect(37, 3, 44, 5, 'O')
    cv.ascii(50, 3, [".OO.", "O..O", ".OO.", ".OO.", ".OO.", ".OO.", ".OO.", ".OO.", "O..O", ".OO."])
    cv.rect(58, 4, 60, 16, 'b'); cv.rect(61, 4, 68, 12, 't')
    for x in range(61, 69, 2): cv.set(x, 13, 't')
    # lamp
    cv.line(84, 0, 84, 6, 'k'); cv.ell(84, 8, 5, 2.5, 'c', 'k'); cv.rect(81, 10, 87, 10, 'y')
    for y in range(11, 22):
        for x in range(80 - (y - 11) // 2, 89 + (y - 11) // 2):
            if (x + y) % 2 == 0 and cv.get(x, y) in ('y', 'h'): cv.set(x, y, 'w')
    # workbench
    cv.rect(0, 26, 95, 28, 'b'); cv.rect(0, 26, 95, 26, 'l'); cv.rect(0, 29, 95, 39, 'O')
    cv.rect(6, 29, 9, 39, 'k'); cv.rect(86, 29, 89, 39, 'k')
    for x in range(0, 96, 2): cv.set(x, 29, 'k')
    # things on bench: jar of pencils, box, laptop
    cv.rect(12, 20, 16, 25, 'B'); cv.rect(12, 20, 16, 20, 'k')
    cv.line(13, 17, 13, 20, 'c'); cv.line(15, 16, 15, 20, 'h'); cv.line(14, 18, 14, 20, 's')
    cv.rect(66, 21, 79, 25, 'b'); cv.rect(66, 21, 79, 21, 'k'); cv.rect(70, 22, 75, 23, 'h')
    # person working
    person(cv, 44, 13, shirt='c')
    cv.ascii(51, 22, ["ww.", "wkw", ".w."])
    return cv

# ---------- Code: the arcade ----------
def arcade():
    cv = C(96, 40)
    cv.rect(0, 0, 95, 39, 'o')
    for y in range(0, 30):
        for x in range(96):
            if (x * 3 + y * 5) % 23 == 0: cv.set(x, y, 'O')
    # neon sign </>
    neon = ["...g.....g..g...",
            "..g.....g....g..",
            ".g.....g......g.",
            "..g...g......g..",
            "...g.g......g..."]
    for j, r in enumerate(neon):
        for i, ch in enumerate(r):
            if ch == 'g': cv.set(8 + i, 4 + j, 'G')
    # cabinet
    cv.rect(40, 4, 63, 37, 'u'); cv.line(40, 4, 40, 37, 'k'); cv.line(63, 4, 63, 37, 'k'); cv.line(40, 4, 63, 4, 'k')
    cv.rect(42, 6, 61, 8, 'h')
    for i in range(43, 61, 3): cv.set(i, 7, 'k')
    cv.rect(42, 10, 61, 23, 'k'); cv.rect(43, 11, 60, 22, 'o')
    # screen content: invaders & green code lines
    inv = [".g...g.", "..ggg..", ".gg.gg.", "ggggggg", "g.g.g.g"]
    cv.ascii(48, 12, inv)
    for i, ln in enumerate([7, 11, 5]):
        cv.line(45, 18 + i * 1, 45 + ln, 18 + i * 1, 'G') if i != 1 else cv.line(47, 19, 47 + ln, 19, 'G')
    cv.set(51, 21, 'a')
    for y in range(11, 23, 2):
        for x in range(43, 61):
            if cv.get(x, y) == 'o': cv.set(x, y, 'O')
    # control deck
    cv.rect(39, 24, 64, 27, 'U'); cv.line(39, 24, 64, 24, 'k')
    cv.set(45, 25, 'k'); cv.set(45, 26, 'r'); cv.set(52, 25, 'h'); cv.set(55, 25, 's'); cv.set(58, 25, 'c')
    cv.rect(44, 30, 59, 31, 'k'); cv.rect(48, 30, 55, 31, 'h')
    # second cabinet silhouette
    cv.rect(72, 8, 91, 37, 'O'); cv.rect(74, 13, 89, 23, 'k'); cv.rect(75, 14, 88, 22, 'A')
    for y in range(14, 23, 2): cv.line(75, y, 88, y, 'b')
    # floor
    cv.rect(0, 36, 95, 39, 'k')
    for x in range(0, 96, 6):
        cv.rect(x, 37, x + 2, 37, 'O')
    # player
    person(cv, 26, 23, shirt='s')
    # bug enemy
    cv.ascii(18, 30, [".k.k.", "rrrrr", "rkrkr", "rrrrr", "k.k.k"])
    # coins
    cv.set(31, 33, 'h'); cv.set(33, 34, 'h')
    return cv

# ---------- Creator: the studio ----------
def studio():
    cv = C(96, 40)
    cv.rect(0, 0, 95, 39, 'O')
    for x in range(0, 96, 6):
        cv.rect(x, 0, x + 2, 29, 'k')
    # TV wall 3x2
    cols = ['c', 's', 'h', 'U', 'B', 'l']
    for i in range(6):
        r, c = divmod(i, 3)
        x0 = 44 + c * 16; y0 = 3 + r * 12
        cv.rect(x0, y0, x0 + 14, y0 + 10, 'k')
        cv.rect(x0 + 1, y0 + 1, x0 + 11, y0 + 9, cols[i])
        for yy in range(y0 + 2, y0 + 9, 2): cv.line(x0 + 1, yy, x0 + 11, yy, 'w' if i == 4 else cols[i])
        if i == 5:  # static
            for yy in range(y0 + 1, y0 + 10):
                for xx in range(x0 + 1, x0 + 12):
                    cv.set(xx, yy, random.choice('wnmT'))
        cv.set(x0 + 13, y0 + 3, 'h'); cv.set(x0 + 13, y0 + 6, 'm')
    # play glyph on first
    cv.ascii(48, 6, ["w..", "ww.", "www", "ww.", "w.."])
    # ring light
    cv.ell(14, 11, 7, 7, 'y', 'h')
    cv.ell(14, 11, 4, 4, 'O')
    cv.line(14, 18, 14, 34, 'k'); cv.line(10, 34, 18, 34, 'k')
    # camera on tripod
    cv.rect(26, 14, 35, 20, 'k'); cv.rect(27, 15, 34, 19, 'T'); cv.rect(28, 16, 30, 17, 't'); cv.ell(37, 17, 2.5, 2.5, 'k'); cv.set(37, 17, 'B')
    cv.set(28, 13, 'r')
    cv.line(30, 21, 26, 34, 'T'); cv.line(31, 21, 31, 34, 'T'); cv.line(32, 21, 36, 34, 'T')
    # floor
    cv.rect(0, 35, 95, 39, 'b')
    for x in range(0, 96, 5): cv.set(x, 35, 'l')
    # creator on stool
    person(cv, 18, 21, shirt='c')
    cv.rect(18, 34, 26, 34, 'k')
    cv.rect(40, 30, 43, 34, 's'); cv.ascii(39, 27, [".s.s", "ss.s", ".sss"])
    return cv

# ---------- Ours: two-room house ----------
def house():
    cv = C(128, 56)
    cv.rect(0, 0, 127, 55, 'B')
    cv.ell(110, 10, 5, 5, 'h')
    for cx, cy in [(20, 8), (70, 5)]:
        cv.ell(cx, cy, 6, 2.2, 'w'); cv.ell(cx + 4, cy - 1.5, 3.5, 2.2, 'w')
    # roof
    for y in range(10, 22):
        half = (y - 10) * 4 + 6
        cv.line(64 - half, y, 64 + half, y, 'd' if y % 2 else 'c')
    cv.line(16, 21, 112, 21, 'k')
    for y in range(10, 22):
        half = (y - 10) * 4 + 6
        cv.set(64 - half, y, 'k'); cv.set(64 + half, y, 'k')
    cv.rect(84, 8, 88, 14, 'R')
    # walls (cutaway)
    cv.rect(18, 22, 110, 48, 'k')
    cv.rect(19, 23, 63, 47, 'e')   # left room
    cv.rect(65, 23, 109, 47, 'y')  # right room
    for y in range(23, 48):
        for x in range(19, 64):
            if (x + y) % 9 == 0: cv.set(x, y, 'n')
        for x in range(65, 110):
            if (x * 2 + y) % 11 == 0: cv.set(x, y, 'h')
    # floors
    cv.rect(19, 45, 109, 47, 'b'); cv.rect(19, 45, 109, 45, 'l')
    # left: desk + laptop + him
    cv.rect(24, 36, 42, 37, 'b'); cv.rect(25, 38, 25, 44, 'k'); cv.rect(41, 38, 41, 44, 'k')
    cv.rect(28, 31, 35, 35, 'k'); cv.rect(29, 32, 34, 34, 'G'); cv.rect(27, 35, 37, 35, 'm')
    cv.ascii(38, 33, ["ww", "nn"])
    person(cv, 46, 32, shirt='s')
    # shelf
    cv.rect(22, 27, 36, 27, 'b'); cv.rect(23, 24, 24, 26, 'c'); cv.rect(25, 25, 26, 26, 's'); cv.rect(27, 24, 28, 26, 'h')
    # right: plants, sofa, her
    cv.rect(70, 37, 92, 41, 'c'); cv.rect(70, 35, 92, 37, 'l'); cv.rect(70, 37, 71, 44, 'd'); cv.rect(91, 37, 92, 44, 'd')
    cv.rect(98, 39, 102, 44, 'b'); cv.ascii(96, 31, [".s.s.s.", "ssqsqss", ".sssss.", "..sgs..", "...g..."])
    person(cv, 76, 26, shirt='U')
    # window right room
    cv.rect(96, 25, 106, 31, 'k'); cv.rect(97, 26, 105, 30, 'B'); cv.line(101, 26, 101, 30, 'k')
    # shared door / heart in the middle wall
    cv.rect(63, 33, 65, 44, 'h')
    cv.ascii(61, 26, [".c.c.", "ccccc", ".ccc.", "..c.."])
    # ground
    cv.rect(0, 49, 127, 55, 's')
    for x in range(128):
        if x % 3 == 0: cv.set(x, 49, 'g')
    for y in range(51, 56):
        for x in range(128):
            if (x + y) % 2 == 0: cv.set(x, y, 'g')
    for y in range(49, 56):
        w = 3 + (y - 49)
        for x in range(64 - w, 64 + w): cv.set(x, y, 'n')
    return cv

# ---------- channel avatars 16x16 ----------
def av_elite():
    cv = C(16, 16)
    cv.ell(8, 8, 7.5, 7.5, 'O', 'k')
    cv.ell(8, 8, 5, 5, 'B', 'k')
    cv.line(3, 8, 13, 8, 'k'); cv.line(8, 3, 8, 13, 'k')
    cv.ell(8, 8, 2.5, 5, 'B')
    cv.line(8, 3, 8, 13, 'k'); cv.line(4, 6, 12, 6, 's'); cv.line(4, 10, 12, 10, 's')
    cv.set(6, 5, 'h'); cv.set(10, 11, 'h')
    return cv

def av_cte():
    cv = C(16, 16)
    cv.rect(0, 0, 15, 15, 'y')
    cv.ell(6.5, 6.5, 5, 5, 'w', 'k'); cv.ell(6.5, 6.5, 3, 3, 'B')
    cv.line(10, 10, 14, 14, 'k'); cv.line(11, 10, 15, 14, 'b')
    cv.ascii(4, 5, ["r.r", "rrr", "k.k"])
    return cv

def av_yt():
    cv = C(16, 16)
    cv.rect(1, 3, 14, 12, 'r'); cv.rect(1, 3, 14, 3, 'k'); cv.rect(1, 12, 14, 12, 'k'); cv.line(1, 3, 1, 12, 'k'); cv.line(14, 3, 14, 12, 'k')
    cv.ascii(6, 5, ["w...", "ww..", "www.", "ww..", "w..."])
    cv.line(5, 13, 3, 15, 'k'); cv.line(10, 13, 12, 15, 'k')
    return cv

def av_x():
    cv = C(16, 16)
    cv.rect(0, 0, 15, 15, 'o')
    for i in range(16):
        for j in range(16):
            if random.random() < 0.45: cv.set(i, j, random.choice('mTn'))
    return cv

def boss():
    cv = C(20, 16)
    cv.ascii(0, 0, [
        "....k........k......",
        ".....k......k.......",
        "....rrrrrrrrrr......",
        "...rrrrrrrrrrrr.....",
        "..rrwwrrrrrwwrrr....",
        "..rrwkrrrrrwkrrr....",
        "kkrrrrrrrrrrrrrrkk..",
        "..rrRRRRRRRRRRrr....",
        "kkrrRrrrrrrrrRrrkk..",
        "..rrRRRRRRRRRRrr....",
        "kkrrrrrrrrrrrrrrkk..",
        "...rrrrrrrrrrrr.....",
        "....rrrrrrrrrr......",
        "....k..k..k..k......",
        "...k..k....k..k.....",
        "....................",
    ])
    return cv

if __name__ == '__main__':
    out = {}
    for name, fn in [('station', station), ('dakghar', dakghar), ('workshop', workshop), ('arcade', arcade),
                     ('studio', studio), ('house', house), ('av-elite', av_elite), ('av-cte', av_cte),
                     ('av-yt', av_yt), ('av-x', av_x), ('boss', boss)]:
        cv = fn(); cv.save(f'out/{name}.png', 8)
