<h1 align="center"> Rust API

Template For Building Rest APIs With Rust
     
![Rust](https://img.shields.io/badge/rust-stable-brightgreen)
![Actix Web](https://img.shields.io/badge/framework-actix_web-blue)
![License](https://img.shields.io/badge/license-MIT-blue.svg)
</h1>

## Getting Started

```bash
git clone https://github.com/stormfiber/rust_api
cd rust_api
```

## Build and Run

``` bash
cargo build
cargo run
```
- By default it will runs on port:3000
- http://127.0.0.1:3000
- Configure with `PORT`, `HOST`, `FONT_PATH`, `MAX_PAYLOAD_BYTES` env vars

## Test Routes

- Shorten a URL (POST)

```bash
curl -X POST http://127.0.0.1:3000/url/shorten \
     -H "Content-Type: application/json" \
     -d '{"url": "https://example.com"}'
```
- Response Example:

```json
{
  "short_url": "s19"
}
```

- Expand a short URL (GET)

```nginx
http://127.0.0.1:3000/url/expand?short=s19
```
- Response Example:

```json
{
  "url": "https://example.com"
}
```

- GET QR as PNG Browser

```nginx
http://127.0.0.1:3000/qr/generate_png?message=HelloWorld
```

- POST QR as Base64 JSON

```bash
curl -X POST http://127.0.0.1:3000/qr/generate_base64 \
     -H "Content-Type: application/json" \
     -d '{"message": "HelloWorld"}'
```
- Response Example:
```json
{
  "message": "HelloWorld",
  "qr_base64": "iVBORw0KGgoAAAANSUhEUgAA..."
}
```

## Text Tools

- Echo (GET)

```nginx
http://127.0.0.1:3000/text/echo?message=Hello
```

- Reverse (POST)

```bash
curl -X POST http://127.0.0.1:3000/text/reverse \
     -H "Content-Type: application/json" \
     -d '{"message": "Hello"}'
```

## Image Editors

Every editor accepts an image either as raw bytes (POST body) or base64 JSON (`{"image": "..."}`).
Raw endpoints return `image/png`; `_base64` endpoints return `{"image_base64": "..."}`.

- Resize (query `width`, `height`, `mode`=fit|exact|fill)

```bash
# raw bytes
curl -X POST http://127.0.0.1:3000/image/editors/resize?width=200&height=200 --data-binary @image.png -o out.png

# base64 JSON
curl -X POST http://127.0.0.1:3000/image/editors/resize_base64 \
     -H "Content-Type: application/json" \
     -d '{"image":"<base64>","width":200,"height":200,"mode":"fit"}'
```

- Rotate (query `deg`=90|180|270)

```bash
curl -X POST http://127.0.0.1:3000/image/editors/rotate?deg=90 --data-binary @image.png -o out.png
curl -X POST http://127.0.0.1:3000/image/editors/rotate_base64 \
     -H "Content-Type: application/json" \
     -d '{"image":"<base64>","deg":90}'
```

- Flip (query `direction`=horizontal|vertical)

```bash
curl -X POST http://127.0.0.1:3000/image/editors/flip?direction=vertical --data-binary @image.png -o out.png
curl -X POST http://127.0.0.1:3000/image/editors/flip_base64 \
     -H "Content-Type: application/json" \
     -d '{"image":"<base64>","direction":"vertical"}'
```

- Grayscale

```bash
curl -X POST http://127.0.0.1:3000/image/editors/grayscale --data-binary @image.png -o out.png
curl -X POST http://127.0.0.1:3000/image/editors/grayscale_base64 \
     -H "Content-Type: application/json" \
     -d '{"image":"<base64>"}'
```

- Blur (query `sigma`)

```bash
curl -X POST http://127.0.0.1:3000/image/editors/blur?sigma=3.0 --data-binary @image.png -o out.png
curl -X POST http://127.0.0.1:3000/image/editors/blur_base64 \
     -H "Content-Type: application/json" \
     -d '{"image":"<base64>","sigma":3.0}'
```

## Image Effects

Single endpoint `/image/effects` (raw PNG) and `/image/effects_base64` (JSON).
Image source: JSON `image` (base64), JSON `url` (fetch), query `url` (fetch), or multipart `image` file upload.
All parameters optional with sensible defaults.

Effects: `blur`, `brightness`, `contrast`, `sharpen`, `grayscale`, `invert`, `normalize`, `sepia`, `flip`, `resize`, `crop`, `scale`, `contain`, `cover`, `opacity`, `fade`, `saturation`, `hue`, `correction`, `threshold`, `tint`, `adjustment`, `median`, `pixelate`, `convolute`, `gaussian`, `posturize`, `dither`, `circle`, `fisheye`, `drop_shadow`, `displace`, `mirror`, `glitch`, `vignette`, `vintage`, `emboss`, `oil_painting`, `duo_tone`, `chromatic`, `halftone`, `ascii_art`, `scanlines`, `motion_blur`, `kaleidoscope`, `pixel_sort`, `wave_distortion`, `swirl`, `bulge`, `noise`, `denoise`, `cartoon`, `pencil_sketch`, `hdr`, `bokeh`, `thermal`, `xray`, `infrared`, `auto_enhance`, `remove_background`, `vhs_effect`, `neon_glow`, `polaroid_frame`, `watermark`

Common optional params: `effect`, `sigma`, `amount`, `x`, `y`, `width`, `height`, `color` (hex), `factor`, `threshold`, `angle`, `radius`, `strength`, `opacity`

```bash
# base64 JSON
curl -X POST http://127.0.0.1:3000/image/effects_base64 \
     -H "Content-Type: application/json" \
     -d '{"image":"<base64>","effect":"sepia"}'

# raw bytes
curl -X POST http://127.0.0.1:3000/image/effects?effect=vintage --data-binary @image.png -o out.png

# via URL fetch (GET)
curl "http://127.0.0.1:3000/image/effects_get?url=<image-url>&effect=blur" -o out.png

# multipart upload
curl -X POST http://127.0.0.1:3000/image/effects_multipart \
     -F "image=@image.png" -F "effect=sepia" -o out.png
```

Legacy individual endpoints (still available): `/image/effects/sepia`, `/image/effects/sepia_base64`, `/image/effects/invert`, `/image/effects/invert_base64`, `/image/effects/edge`, `/image/effects/edge_base64`.

## Canvas

- Blank canvas (POST, query `width`, `height`, `color`)

```bash
curl -X POST "http://127.0.0.1:3000/canvas/blank?width=200&height=100&color=blue" -o out.png
curl -X POST http://127.0.0.1:3000/canvas/blank_base64 \
     -H "Content-Type: application/json" \
     -d '{"width":200,"height":100,"color":"blue"}'
```

- Draw text (POST, query `width`, `height`, `text`, `x`, `y`, `size`, `color`; optional raw image body to draw on)

```bash
curl -X POST "http://127.0.0.1:3000/canvas/draw_text?text=Hello&width=200&height=100&size=24&color=black" -o out.png
curl -X POST http://127.0.0.1:3000/canvas/draw_text_base64 \
     -H "Content-Type: application/json" \
     -d '{"text":"Hello","width":200,"height":100,"size":24,"color":"black"}'
```

Requires a TTF font: set `FONT_PATH` or place `assets/fonts/DejaVuSans.ttf`.

## Cards Generator

18 card types: `welcome`, `rank`, `leave`, `profile`, `birthday`, `achievement`, `quote`, `spotify`, `github_stats`, `warning`, `ban`, `announcement`, `poll_vote`, `ship`, `magic8ball`, `weather`, `certificate`, `support_ticket`.

Each card has `_base64` (JSON) and raw (POST) variants, plus a `_multipart` variant for file uploads.
Optional overrides: `background` (hex), `image` (base64 or multipart file), `text` (string), `text_color` (hex), `width`, `height`.
Image can also be provided via `image_url` query param (fetched from URL).

```bash
# base64 JSON
curl -X POST http://127.0.0.1:3000/cards/welcome_base64 \
     -H "Content-Type: application/json" \
     -d '{"text":"Welcome aboard!","background":"#1a1a2e","text_color":"#00d2ff"}'

# raw (query params)
curl -X POST "http://127.0.0.1:3000/cards/rank?text=Rank+Card&background=#2c3e50" -o out.png

# multipart (file upload)
curl -X POST http://127.0.0.1:3000/cards/birthday_multipart \
     -F "image=@photo.png" -F "text=Happy+Birthday!" -o out.png
```

## Running Tests

```bash
cargo test
```
