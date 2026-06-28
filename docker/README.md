# Docker

## Quick start

```bash
cp .env.example .env
# fill in KLEOS_API_KEY and KLEOS_BOOTSTRAP_SECRET
docker compose up -d
```

Server is available at `http://localhost:4200`.

## Profiles

| Command | What runs |
|---|---|
| `docker compose up` | server + sidecar |
| `docker compose -f compose.yaml -f docker/gui.yml up` | server with web UI (requires `KLEOS_GUI_PASSWORD`) |
| `docker compose --profile full up` | + credd + phylaxd |

## Build targets

The Dockerfile has named targets so you can build individual images:

| Target | Description |
|---|---|
| `runtime` | Server + CLI, no GUI (default) |
| `runtime-gui` | Server + CLI + pre-built web UI |
| `sidecar` | Sidecar proxy only |
| `credd` | Credential daemon only |
| `phylaxd` | Phylax security daemon only |

```bash
# server without GUI
docker build --target runtime -t kleos:latest .

# server with GUI
docker build --target runtime-gui -t kleos:gui .
```

## Override files

Place compose override files in this directory and reference them with `-f`:

```bash
docker compose -f compose.yaml -f docker/my-override.yml up
```
