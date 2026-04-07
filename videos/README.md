# Cortado Promo Video

Built with [Remotion](https://www.remotion.dev/) (React-based video creation).

## Commands

**Install dependencies**

```console
npm i
```

**Start Studio** (live preview)

```console
npm run dev
```

## Exporting

Remotion Studio runs in your browser at native Retina resolution (2x on Mac), so the preview looks sharp. The default `npx remotion render` outputs at 1x resolution with lossy defaults, which looks noticeably worse -- blurry text, color banding on dark backgrounds, and compression artifacts around fine edges.

To match what you see in Studio, you need to address three things:

1. **Resolution** -- render at 2x scale so the output matches Retina pixel density.
2. **Intermediate quality** -- Remotion captures each frame as JPEG (default quality 80). On dark UI with subtle gradients, this introduces banding before video encoding even starts.
3. **Final encoding** -- H.264's default CRF 18 adds visible compression noise around text on dark backgrounds.

### Full-quality render (4K, 41 MB)

For local viewing or YouTube upload:

```console
npx remotion render CortadoPromo \
  --output=out/CortadoPromo-final-hq.mp4 \
  --scale=2 \
  --jpeg-quality=100 \
  --color-space=bt709 \
  --crf=1
```

| Flag                  | Why                                                                                                                       |
| --------------------- | ------------------------------------------------------------------------------------------------------------------------- |
| `--scale=2`           | Outputs 3840x2160 from the 1920x1080 composition, matching Retina density. Single biggest improvement for text sharpness. |
| `--jpeg-quality=100`  | Eliminates intermediate compression in per-frame screenshots. Critical for dark backgrounds with fine text.               |
| `--color-space=bt709` | Matches browser color rendering (sRGB/bt709). Default `bt601` shifts colors slightly.                                     |
| `--crf=1`             | Near-lossless H.264. Dark backgrounds with white/colored text are especially sensitive to compression.                    |

### Web-optimized render (4K, ~10 MB)

For GitHub README or web embedding (under 25 MB):

```console
npx remotion render CortadoPromo \
  --output=out/CortadoPromo-final-hq.mp4 \
  --scale=2 \
  --jpeg-quality=100 \
  --color-space=bt709 \
  --crf=1

ffmpeg -i out/CortadoPromo-final-hq.mp4 \
  -c:v libx264 -crf 15 -preset slow \
  -pix_fmt yuv420p \
  -c:a aac -b:a 192k \
  -movflags +faststart \
  out/CortadoPromo-final-web.mp4
```

This renders at full quality first, then re-encodes with ffmpeg's `slow` preset (better quality per bit than Remotion's default encoder) at CRF 15. The `+faststart` flag moves metadata to the beginning so the video starts playing before fully downloading.
