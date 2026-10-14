# Design sources

Everything needed to open, change and re-render the Bindle screens.

| Folder | What is in it |
|---|---|
| `v2/project/` | The current design. One `.dc.html` file per screen plus `canvas.json`, which lays the screens out on the canvas. |
| `v1/project/` | The first design, kept exactly as it was. |
| `art/v1/`, `art/v2/` | The pixel art. `art.py` holds the palette and drawing helpers. `scenes.py` (v1) and `sprites.py` (v2) draw each sprite. `png/` has the finished sprites at true 1x size. |
| `tools/shoot.py` | Renders screens to PNG with Playwright, including the Creator account views. |

## Screens in v2

| Screen | Scene in the header |
|---|---|
| Life Home | Morning hills |
| Ours | A two-room house |
| Work Desk | A railway platform at dusk |
| Inbox | A small post office |
| Code | A cosy arcade at night |
| Board | A warm workshop |
| Creator | A tiny creator studio |
| Brand | Palette, type, the pixel cast and the app icon |

Creator is interactive. It opens on all accounts. Click an account to see its own numbers, growth, top posts and best posting times, then use the tabs to switch accounts or go back.

## Re-render the screenshots

Images in the `.dc.html` files point at uploaded assets (`/_blob/<id>`). `v2/blobs.json` maps each id to its sprite in this folder.

```sh
pip install playwright
python design/tools/shoot.py design/v2/project out Desk.dc.html 'Creator.dc.html:cte={"ch":"cte"}'
```

Set `BLOBMAP=design/v2/blobs.json`, plus `DPR=2` for 2x. The script looks for Inter, Newsreader and JetBrains Mono in `~/fonts`.

## Pixel art rules

- Draw at 1x. Show only at whole-number scales, with `image-rendering: pixelated`.
- Use the palette in `art.py`. The darkest pixel is ink `#2B2622`, never pure black.
- Regenerate with `cd design/art/v2 && mkdir -p out && python sprites.py`.
