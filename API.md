# JSON API

This branch adds a JSON API to Redlib. Prefix any Reddit path with `/api` to get Reddit's raw JSON.

```
GET /api/r/rust/hot?limit=10
GET /api/r/rust/comments/<post_id>
GET /api/user/<username>/submitted
GET /api/search?q=docker
GET /api/r/rust/about
```

Query parameters (`limit`, `after`, `sort`, `t`, ...) are passed through to Reddit.

## Responses

- `200`: Reddit's JSON. The `X-Redlib-Backend` header names the backend that served it.
- `502` / `503`: `{"error": "...", "path": "..."}`. `503` means Reddit blocked the request.

CORS is open (`Access-Control-Allow-Origin: *`).

## Run

```bash
docker build -f Dockerfile.distroless -t redlib-api .
docker run -d -p 127.0.0.1:8080:8080 --env-file .env redlib-api
```

All regular Redlib settings from `.env.example` apply.
