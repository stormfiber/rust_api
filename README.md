<h1 align="center"> Template For Building Rest APIs With Rust </h1>

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
