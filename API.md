# JSON API fork

This fork is [Nachtalb/redlib](https://github.com/Nachtalb/redlib) plus a JSON API at `/api/*`.
All of our work lives on the `feat/json-api` branch.

- [Using the API](#using-the-api)
- [How this fork is set up](#how-this-fork-is-set-up)
- [Taking new code from upstream](#taking-new-code-from-upstream)
- [Building](#building)
- [Running a build locally](#running-a-build-locally)
- [Deploying to a server](#deploying-to-a-server)
- [Troubleshooting](#troubleshooting)

## Using the API

Prefix any Reddit path with `/api` to get Reddit's raw JSON. Query parameters (`limit`, `after`, `sort`, `t`, ...) are passed through.

```
GET /api/r/rust/hot?limit=10
GET /api/r/rust/comments/<post_id>
GET /api/user/<username>/submitted
GET /api/search?q=docker
GET /api/r/rust/about
```

| Status | Body | Meaning |
|---|---|---|
| `200` | Reddit's JSON | The `X-Redlib-Backend` header names the backend that served it |
| `502` | `{"error": "...", "path": "..."}` | Not found, private, banned, rate limited and similar |
| `503` | `{"error": "...", "path": "..."}` | Reddit blocked the request |

CORS is open (`Access-Control-Allow-Origin: *`). Check that an instance can reach Reddit with `GET /healthz/reddit`.

### Keep the stale cache off

Set `REDLIB_STALE_CACHE_MB=0` on every instance. With the cache on, a blocked `/api/*` request can return a copy up to 6 hours old as a normal `200`, with nothing in the response saying so. With it off, you get the `503` instead.

## How this fork is set up

| Remote | Repository | Used for |
|---|---|---|
| `origin` | https://github.com/exitdump/redlib | Our fork. Push here. |
| `upstream` | https://github.com/nachtalb/redlib | Where new code comes from. Never push here. |

The image is `ghcr.io/exitdump/redlib-api`, tagged `latest` and `sha-<commit>`.

Only these files differ from upstream, so they are the only ones to look after when merging:

| File | What it holds |
|---|---|
| `src/api.rs` | The API handler. It calls `backends::fetch` and `backends::is_blocked_error`. |
| `src/lib.rs`, `src/main.rs` | One line each, registering the module and the route |
| `API.md` | This file |

GitHub's **Sync fork** button only updates `main`, which does not contain the API. Use the steps below.

## Taking new code from upstream

Run these from the repository root.

**1. See what is new.** If this prints nothing, you are up to date and can stop.

```bash
git fetch upstream
git log --oneline feat/json-api..upstream/main
```

**2. Read what changed before merging.** Pay attention to `.env.example` and `CONFIGURATION.md` for new settings, and to `src/backends/` for changes that reach the API.

```bash
git diff feat/json-api...upstream/main --stat
git diff feat/json-api...upstream/main -- .env.example CONFIGURATION.md FORK.md src/backends/mod.rs
```

**3. Merge.**

```bash
git switch feat/json-api
git merge upstream/main
```

**4. Build and test it** using one of the options under [Building](#building), then call the API:

```bash
curl "http://localhost:8090/api/r/rust/new?limit=1"
curl "http://localhost:8090/healthz/reddit"
```

**5. Push the code.**

```bash
git push origin feat/json-api
```

To back out of a merge that has conflicts, run `git merge --abort`.

## Building

### Option A: Docker (recommended)

Needs only Docker. On PowerShell:

```powershell
$sha = git rev-parse --short HEAD
docker build -f Dockerfile.distroless `
  --build-arg GIT_HASH=$sha `
  --label org.opencontainers.image.source=https://github.com/exitdump/redlib `
  -t redlib-api:dev `
  -t ghcr.io/exitdump/redlib-api:latest `
  -t ghcr.io/exitdump/redlib-api:sha-$sha .
```

To publish the image, for a server to pull:

```powershell
docker push ghcr.io/exitdump/redlib-api:latest
docker push "ghcr.io/exitdump/redlib-api:sha-$sha"
```

Log in once first with `docker login ghcr.io -u exitdump`, using a token with the `write:packages` scope as the password.

### Option B: Local build, without Docker

Needs Rust, plus MSVC, CMake, NASM and LLVM on Windows. Upstream does not test Windows builds, so prefer Docker if this gives you trouble. See the build section of [README.md](README.md) for other platforms.

```bash
winget install -i Kitware.CMake
winget install -i NASM.NASM
winget install -i LLVM.LLVM
cargo build --release
cargo test
```

The binary ends up in `target/release/`.

## Running a build locally

### With Docker

Save this as `docker-compose.yml` anywhere outside the repository:

```yaml
services:
  redlib:
    image: redlib-api:dev
    ports: ["8090:8080"]
    environment:
      REDLIB_STALE_CACHE_MB: "0"
    restart: unless-stopped
```

Then start it, or restart it onto a newly built image, with:

```bash
docker compose -f path/to/docker-compose.yml up -d
```

### Without Docker

On PowerShell:

```powershell
$env:REDLIB_STALE_CACHE_MB = "0"
.\target\release\redlib.exe -p 8090
```

## Deploying to a server

Log in once, with a separate token that only has the `read:packages` scope:

```bash
docker login ghcr.io -u exitdump
```

`docker-compose.yml`:

```yaml
services:
  redlib:
    image: ghcr.io/exitdump/redlib-api:latest
    restart: unless-stopped
    ports: ["127.0.0.1:8090:8080"]
    env_file: .env
    environment:
      REDLIB_STALE_CACHE_MB: "0"
    user: nobody
    read_only: true
    security_opt:
      - no-new-privileges:true
    cap_drop:
      - ALL
```

Create `.env` from `.env.example`. All regular Redlib settings apply, and [CONFIGURATION.md](CONFIGURATION.md) describes them. Never commit `.env`, cookies, tokens or proxy passwords.

To update:

```bash
docker compose pull
docker compose up -d
curl http://127.0.0.1:8090/healthz/reddit
```

To roll back, replace `latest` with an older tag such as `sha-1980cb8` and run `docker compose up -d`.

## Troubleshooting

| Problem | Cause and fix |
|---|---|
| `CONFLICT` during a merge | Fix the files by hand, or run `git merge --abort`. Conflicts are most likely in `src/lib.rs` or `src/main.rs`; keep upstream's lines and our one line. |
| `src/api.rs` stops compiling after a merge | Upstream changed `backends::fetch` or `backends::is_blocked_error`. Update the handler to match. |
| `denied` when pushing to ghcr.io | The `--label` flag was left out of `docker build`, so the image still claims upstream's repository. Rebuild with it. |
| `is cmake not installed?` from `cargo build` | CMake is missing or not on `PATH`. Install it, or use the Docker build. |
| Docker says it can't reach the API on `dockerDesktopLinuxEngine` | Docker Desktop isn't running. Start it. |
| `/api/*` keeps returning `503` | Reddit is blocking this IP. See `REDLIB_REDDIT_COOKIE` in [CONFIGURATION.md](CONFIGURATION.md). |
