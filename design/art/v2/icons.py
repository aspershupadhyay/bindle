# SPDX-License-Identifier: Apache-2.0
# Copyright 2026 Sparsh Upadhyay and the Bindle contributors.
"""Week 7 pixel art: icons for Sort, Promises, People and Wrap up (32x32),
the people pictures (16x16), and the logo on dark.

Run from this folder: python3 icons.py, then copy png/ files to app/src/art/.
Each picture names what is drawn (short hair, a beard, a turban), never who
someone is: anyone can pick any of them.
"""
from art import C
from PIL import Image

# ---------- screen icons, 32x32 ----------
def sort_icon():
    """A wooden tray with notes dropping in."""
    cv = C(32, 32)
    # a falling note, tilted by a step
    cv.rect(13, 1, 21, 7, 'k'); cv.rect(14, 2, 20, 6, 'w')
    cv.line(15, 4, 19, 4, 'c')
    cv.set(11, 3, 'y'); cv.set(23, 2, 'y'); cv.set(24, 6, 'y')
    # notes already in the tray
    cv.rect(6, 10, 17, 19, 'k'); cv.rect(7, 11, 16, 19, 'y'); cv.line(8, 13, 14, 13, 'b'); cv.line(8, 15, 12, 15, 'b')
    cv.rect(15, 12, 26, 19, 'k'); cv.rect(16, 13, 25, 19, 'l'); cv.line(17, 15, 23, 15, 'd'); cv.line(17, 17, 21, 17, 'd')
    # the tray
    cv.rect(2, 18, 29, 28, 'k')
    cv.rect(3, 19, 28, 27, 'b')
    cv.line(3, 19, 28, 19, 'h')
    cv.rect(5, 21, 26, 25, 'A')
    cv.line(5, 21, 26, 21, 'k')
    cv.rect(13, 23, 18, 23, 'h')
    cv.line(3, 29, 28, 29, 'n')
    return cv

def promises_icon():
    """A ledger page with three promises: one kept, two open."""
    cv = C(32, 32)
    cv.rect(5, 3, 26, 29, 'k'); cv.rect(6, 4, 25, 28, 'w')
    cv.line(6, 28, 25, 28, 'n'); cv.line(25, 4, 25, 28, 'n')
    # rings
    for x in (9, 15, 21):
        cv.rect(x, 1, x + 1, 5, 'k'); cv.set(x, 2, 'T'); cv.set(x + 1, 2, 'T')
    # row 1: kept
    cv.rect(8, 8, 13, 13, 'k'); cv.rect(9, 9, 12, 12, 's')
    for x, y in [(9, 10), (10, 11), (11, 10), (12, 9)]:
        cv.set(x, y, 'w')
    cv.line(15, 11, 23, 11, 'T')
    # row 2: open, clay
    cv.rect(8, 15, 13, 20, 'k'); cv.rect(9, 16, 12, 19, 'w')
    cv.line(15, 18, 22, 18, 'c')
    # row 3: open, honey
    cv.rect(8, 22, 13, 27, 'k'); cv.rect(9, 23, 12, 26, 'w')
    cv.line(15, 25, 20, 25, 'h')
    # a pencil leaning on the page
    cv.line(27, 12, 21, 26, 'k'); cv.line(28, 13, 22, 27, 'h'); cv.line(29, 13, 23, 27, 'k')
    cv.set(22, 28, 'k'); cv.set(28, 11, 'r'); cv.set(29, 12, 'r')
    return cv

def figure(cv, x, y, hair, skin, shirt, long_hair=False):
    """A head-and-shoulders person, 10 wide, top-left at x, y."""
    cv.ascii(x, y, [
        "..kkkkkk..",
        ".kHHHHHHk.",
        ".kHSSSSHk.",
        ".kSkSSkSk." if not long_hair else ".kSkSSkSk.",
        ".kSSSSSSk.",
        "..kSddSk..",
        "...kSSk...",
        ".kkMMMMkk.",
        "kMMMMMMMMk",
        "kMMMMMMMMk",
        "kMMMMMMMMk",
    ])
    for j in range(11):
        for i in range(10):
            c = cv.get(x + i, y + j)
            if c in ('H', 'S', 'M'):
                cv.set(x + i, y + j, {'H': hair, 'S': skin, 'M': shirt}[c])
    if long_hair:
        for j in range(2, 7):
            cv.set(x, y + j, 'k'); cv.set(x + 1, y + j, hair)
            cv.set(x + 9, y + j, 'k'); cv.set(x + 8, y + j, hair)

def people_icon():
    """Two people side by side, with a small heart between them."""
    cv = C(32, 32)
    figure(cv, 4, 18, 'k', 'l', 'c', long_hair=True)
    figure(cv, 18, 18, 'b', 'l', 's')
    cv.ascii(12, 9, [".hh.hh.", "hhhhhhh", "hyhhhhh", ".hhhhh.", "..hhh..", "...h..."])
    cv.line(2, 29, 29, 29, 'g'); cv.line(4, 30, 27, 30, 'n')
    return cv

def wrap_up_icon():
    """A window at night: the moon, two stars and a lamp on the sill."""
    cv = C(32, 32)
    cv.rect(3, 2, 28, 25, 'k')
    cv.rect(4, 3, 27, 24, 'O')
    # dithered dusk at the bottom of the glass
    for y in range(19, 25):
        for x in range(4, 28):
            if y > 21 or (x + y) % 2 == 0: cv.set(x, y, 'u')
    cv.line(15, 3, 15, 24, 'b'); cv.line(16, 3, 16, 24, 'b'); cv.line(4, 13, 27, 13, 'b')
    # moon
    cv.ell(9.5, 8.5, 3.6, 3.6, 'y'); cv.ell(11.2, 7.4, 2.6, 2.6, 'O')
    cv.set(22, 6, 'y'); cv.set(25, 9, 'y'); cv.set(20, 17, 'y'); cv.set(8, 17, 'U')
    # sill and lamp
    cv.rect(1, 25, 30, 27, 'k'); cv.rect(2, 26, 29, 26, 'b')
    # a candle, its glow on the glass
    for x, y in [(21, 15), (23, 15), (20, 17), (24, 17), (22, 14)]:
        cv.set(x, y, 'A')
    cv.ascii(20, 15, [
        "..a..",
        ".aya.",
        "..y..",
        ".kkk.",
        ".kwk.",
        ".kwk.",
        ".kwk.",
        "kbbbk",
        "kkkkk",
    ])
    cv.line(2, 28, 29, 28, 'n')
    return cv

def ai_icon():
    """Two speech bubbles: the person and an AI, with a small spark."""
    cv = C(32, 32)
    cv.rect(2, 4, 19, 16, 'k'); cv.rect(3, 5, 18, 15, 'w')
    cv.line(5, 8, 15, 8, 'T'); cv.line(5, 11, 12, 11, 'T')
    cv.ascii(4, 17, ["kk", "k."])
    cv.rect(11, 14, 29, 26, 'k'); cv.rect(12, 15, 28, 25, 'q')
    cv.line(14, 18, 25, 18, 'g'); cv.line(14, 21, 22, 21, 'g')
    cv.ascii(26, 27, ["kk", ".k"])
    cv.ascii(24, 1, ["..h..", "..h..", "hhyhh", "..h..", "..h.."])
    return cv

# ---------- people pictures, 16x16 ----------
DISC = [
    "......####......",
    "....########....",
    "..############..",
    "..############..",
    ".##############.",
    ".##############.",
    "################",
    "################",
    "################",
    "################",
    ".##############.",
    ".##############.",
    "..############..",
    "..############..",
    "....########....",
    "......####......",
]
FACE = {
    5:  "....kllllllk....",
    6:  ".....lkllkl.....",
    7:  ".....llllll.....",
    8:  ".....llddll.....",
    9:  "......llll......",
    10: "....MMMMMMMM....",
    11: "...MMMMMMMMMM...",
}
AVATARS = {
    "short":  ('y', 'm', {3: ".....kkkkkk.....", 4: "....kkkkkkkk...."}),
    "long":   ('q', 'c', {3: ".....kkkkkk.....", 4: "....kkkkkkkk....", 6: "....klkllklk....",
                          7: "....kllllllk....", 8: "....kllddllk....", 9: "....kkllllkk...."}),
    "curly":  ('U', 's', {2: ".....b.bb.b.....", 3: "....bbbbbbbb....", 4: "...bbbbbbbbbb...",
                          5: "...bbllllllbb...", 6: "...b.lkllkl.b..."}),
    "bun":    ('B', 's', {1: ".......kk.......", 2: "......kkkk......", 3: ".....kkkkkk.....", 4: "....kkkkkkkk...."}),
    "beard":  ('h', 'g', {3: ".....kkkkkk.....", 4: "....kkkkkkkk....", 8: ".....kkddkk.....", 9: "......kkkk......"}),
    "turban": ('B', 'm', {2: "......cccc......", 3: ".....ccddcc.....", 4: "....cdccccdc....", 5: "....cllllllc...."}),
    "scarf":  ('y', 's', {2: ".....ssssss.....", 3: "....ssssssss....", 4: "...ssggggggss...",
                          5: "...ssllllllss...", 6: "...sslkllklss...", 7: "...ssllllllss...",
                          8: "...ssllddllss...", 9: "...sssllllsss...", 10: "...ssssssssss...",
                          11: "..ssssssssssss.."}),
    "cap":    ('q', 'm', {3: ".....cccccc.....", 4: "....cccccccc....", 5: "....dddddddddd.."}),
    "grey":   ('U', 'T', {3: ".....tttttt.....", 4: "....tttttttt....", 5: "....tllllllt....",
                          6: "....tlkllklt....", 7: ".....llllll....."}),
}

def avatar(name):
    bg, shirt, rows = AVATARS[name]
    cv = C(16, 16)
    cv.ascii(0, 0, [r.replace('#', bg) for r in DISC])
    merged = dict(FACE); merged.update(rows)
    for y, row in merged.items():
        cv.ascii(0, y, [row.replace('M', shirt)])
    return cv

# ---------- the logo on dark ----------
def logo_on_dark(src, out):
    """The brand sheet's dark tile: ink outline pixels drop away, so the bag
    and stick read on a dark background."""
    im = Image.open(src).convert('RGBA')
    ink = (43, 38, 34, 255)
    for y in range(im.height):
        for x in range(im.width):
            if im.getpixel((x, y)) == ink:
                im.putpixel((x, y), (0, 0, 0, 0))
    im.save(out)

if __name__ == '__main__':
    import os
    os.makedirs('png', exist_ok=True)
    for name, fn in [('sort', sort_icon), ('promises', promises_icon), ('people', people_icon), ('wrap-up', wrap_up_icon), ('ai', ai_icon)]:
        fn().save(f'png/{name}.png')
    for name in AVATARS:
        avatar(name).save(f'png/av-{name}.png')
    logo_on_dark('../../../app/src/art/bindle-24.png', 'png/bindle-24-dark.png')
