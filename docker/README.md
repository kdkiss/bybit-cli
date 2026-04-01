# Docker

This directory contains the Docker setup for running `bybit-cli` as an MCP server.

Files:

- `docker/mcp/Dockerfile`
- `docker/mcp/compose.yaml`

Run commands from the repo root:

```bash
cd /path/to/bybit-cli
```

Use testnet first:

```bash
export BYBIT_TESTNET=true
```

or in PowerShell:

```powershell
$env:BYBIT_TESTNET="true"
```

## Choose A Mode

Use **stdio** when your MCP client launches a local command and talks over stdin/stdout.

Use **HTTP** when your MCP client connects to a URL such as `http://localhost:8811/mcp`.

## Build

```bash
docker build -f docker/mcp/Dockerfile -t bybit-mcp .
```

or:

```bash
docker compose -f docker/mcp/compose.yaml build
```

Normal rebuilds reuse Docker and Cargo caches. Avoid `--no-cache` unless you actually need a full rebuild.

## Stdio MCP

### Compose

```bash
docker compose -f docker/mcp/compose.yaml run --rm -T mcp -s all
```

### Docker

```bash
docker run --rm -i \
  -e BYBIT_API_KEY \
  -e BYBIT_API_SECRET \
  -e BYBIT_TESTNET=true \
  -v bybit-mcp-data:/data/bybit \
  bybit-mcp -s all
```

Notes:

- no port mapping is needed
- `-T` is recommended for manual Compose stdio runs
- this is the best mode for clients that expect `command` plus `args`

## HTTP MCP

### Compose

```bash
docker compose -f docker/mcp/compose.yaml up mcp-http
```

Default URL:

```text
http://localhost:8811/mcp
```

### Docker

```bash
docker run --rm -p 8811:8811 \
  -e BYBIT_API_KEY \
  -e BYBIT_API_SECRET \
  -e BYBIT_TESTNET=true \
  -v bybit-mcp-data:/data/bybit \
  bybit-mcp --transport http --host 0.0.0.0 --port 8811 --path /mcp -s all
```

Notes:

- this is the best mode for clients that connect to a URL
- the container listens on `/mcp`
- if you change the port in Compose, set `BYBIT_MCP_PORT`
- if you change the path in Compose, set `BYBIT_MCP_PATH`

### Kilo Code

For project-level Kilo Code config in `.kilocode/mcp.json`:

```json
{
  "mcpServers": {
    "bybit-cli": {
      "type": "streamable-http",
      "url": "http://localhost:8811/mcp",
      "alwaysAllow": [],
      "disabled": false
    }
  }
}
```

If you later add auth in front of the MCP endpoint, add a `headers` object here too.

## Environment Variables

Common runtime variables:

- `BYBIT_API_KEY`
- `BYBIT_API_SECRET`
- `BYBIT_TESTNET=true`
- `BYBIT_MCP_SERVICES`
- `BYBIT_MCP_PORT`
- `BYBIT_MCP_PATH`

Examples:

```bash
BYBIT_TESTNET=true
BYBIT_MCP_SERVICES=market,account,paper
BYBIT_MCP_PORT=8811
BYBIT_MCP_PATH=/mcp
```

Do not bake secrets into the image.

## Persisted State

The container stores local state in `/data/bybit` through `BYBIT_CONFIG_DIR`.

That includes:

- saved config
- saved credentials if you store them
- paper trading state
- shell history
- anonymous instance ID

Compose and the Docker examples both mount a persistent volume for this path.

## Security

- prefer `BYBIT_TESTNET=true` while setting things up
- keep dangerous tools guarded unless you explicitly want autonomous execution
- HTTP mode binds publicly inside the container, so only publish ports intentionally
- this image does not add bearer-token auth by itself

If you expose the HTTP endpoint beyond your machine, put auth and network controls in front of it.

## Troubleshooting

Container looks idle:

- stdio mode is probably waiting for the MCP handshake

HTTP client cannot connect:

- make sure you started `mcp-http` or passed `--transport http`
- make sure port `8811` is published
- make sure the client URL includes `/mcp`

State is not persisting:

- make sure the `/data/bybit` volume mount is present

Client cannot launch Docker:

- use a native `bybit` install instead
