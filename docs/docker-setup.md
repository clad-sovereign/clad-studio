# Docker Setup Guide

This guide explains how to run a local multi-validator Clad Studio testnet using Docker or Podman.

## Prerequisites

- Docker (version 20.10 or later) OR Podman (version 3.0 or later)
- Docker Compose (version 2.0 or later) OR podman-compose
- At least 4GB of free disk space
- At least 4GB of RAM

### Installation

**macOS (Podman - Recommended):**
```bash
brew install podman podman-compose
podman machine init
podman machine start
```

**macOS (Docker):**
```bash
brew install docker docker-compose
```

**Ubuntu/Debian (Podman - Recommended):**
```bash
sudo apt-get update
sudo apt-get install podman podman-compose
```

**Ubuntu/Debian (Docker):**
```bash
sudo apt-get update
sudo apt-get install docker.io docker-compose-v2
sudo systemctl start docker
sudo systemctl enable docker
```

> **Note**: Podman is a drop-in replacement for Docker that runs daemonless (less resource-hungry). If using Podman, replace `docker` with `podman` and `docker-compose` with `podman-compose` in all commands below.

## Quick Start

### Option 1: Use Pre-Built Image (Recommended)

This is the fastest way to get started - pulls a pre-built image from GitHub Container Registry:

```bash
# Pull and start the testnet
export CLAD_IMAGE=ghcr.io/clad-sovereign/clad-node:latest
podman-compose up -d

# View logs
podman-compose logs -f
```

### Option 2: Build from Source (Slow)

Only use this if you need to test local code changes:

```bash
# Build from source (~30-60 minutes)
podman-compose up -d --build

# View logs
podman-compose logs -f
```

### Verify Block Production

Check that blocks are being produced and finalized:

```bash
# Watch Alice's logs for block production
podman-compose logs -f alice | grep "Imported"

# Expected output (approximately every 6 seconds):
# Imported #1 (0x1234...)
# Imported #2 (0x5678...)
# Finalized #1 (0x1234...)
```

### Connect with Polkadot.js Apps

The easiest way to interact with your testnet is via **Polkadot.js Apps**:

1. Open https://polkadot.js.org/apps/
2. Click the network selector (top-left)
3. Choose "Development" → "Local Node"
4. Or directly: https://polkadot.js.org/apps/?rpc=ws://127.0.0.1:9944

From here you can:
- View blocks and events
- Submit extrinsics (mint, transfer, freeze, etc.)
- Query chain state
- Manage accounts

## Network Architecture

```
┌─────────────────┐         ┌─────────────────┐
│  Alice Node     │◄───────►│   Bob Node      │
│  (Validator)    │   P2P   │  (Validator)    │
│                 │         │                 │
│ WS:  9944       │         │ WS:  9945       │
│ HTTP: 9933      │         │ HTTP: 9934      │
│ P2P:  30333     │         │ P2P:  30334     │
│ Metrics: 9615   │         │ Metrics: 9616   │
└────────┬────────┘         └────────┬────────┘
         │                           │
         └──────────┬────────────────┘
                    │
                    ▼
         clad-network (bridge)
                    │
                    ▼
         External Access (localhost)
                    │
         ┌──────────┴──────────┐
         │                     │
         ▼                     ▼
   Polkadot.js Apps      Mobile App
   (browser)             (future)
```

## Configuration

### Ports

| Service | WebSocket | HTTP | P2P   | Prometheus |
|---------|-----------|------|-------|------------|
| Alice   | 9944      | 9933 | 30333 | 9615       |
| Bob     | 9945      | 9934 | 30334 | 9616       |

**Port Descriptions:**
- **WebSocket (9944/9945)**: JSON-RPC over WebSocket for real-time blockchain interaction
- **HTTP (9933/9934)**: JSON-RPC over HTTP for standard API calls
- **P2P (30333/30334)**: Peer-to-peer networking for block propagation
- **Prometheus (9615/9616)**: Metrics endpoint for monitoring and observability

### Chain Specification

The testnet uses the built-in `local` chain spec with:
- **Consensus**: AURA (6-second block time) + GRANDPA (finality)
- **Validators**: Alice and Bob (SR25519 keys)
- **Network**: Private local network

### Data Persistence

Blockchain data is stored in container volumes:
- `alice-data` - Alice's blockchain database
- `bob-data` - Bob's blockchain database

To reset the chain:

```bash
# Stop and remove containers and volumes
podman-compose down -v

# Restart fresh
podman-compose up -d
```

## RPC Examples

```bash
# Health check
curl http://localhost:9933/health

# Get chain name
curl -H "Content-Type: application/json" \
     -d '{"id":1, "jsonrpc":"2.0", "method": "system_chain"}' \
     http://localhost:9933

# Get latest block number
curl -H "Content-Type: application/json" \
     -d '{"id":1, "jsonrpc":"2.0", "method": "chain_getHeader"}' \
     http://localhost:9933

# Check Prometheus metrics
curl http://localhost:9615/metrics
```

## Common Operations

### Start the Network

```bash
# Using pre-built image (fast)
export CLAD_IMAGE=ghcr.io/clad-sovereign/clad-node:latest
podman-compose up -d

# Building from source (slow)
podman-compose up -d --build
```

### Stop the Network

```bash
podman-compose stop
```

### Restart a Specific Node

```bash
podman-compose restart alice
```

### View Live Logs

```bash
# All nodes
podman-compose logs -f

# Specific node
podman-compose logs -f bob

# Last 100 lines
podman-compose logs --tail=100 alice
```

### Execute Commands Inside a Container

```bash
# Open shell in Alice's container
podman-compose exec alice /bin/sh

# Check node version
podman-compose exec alice clad-node --version
```

### Complete Reset

```bash
# Stop containers and remove volumes
podman-compose down -v

# Restart fresh
podman-compose up -d
```

## Connecting Mobile App

To connect the Clad mobile app to your local testnet:

1. Ensure your mobile device is on the same network as your host machine
2. Find your machine's local IP address:

```bash
# macOS
ipconfig getifaddr en0

# Linux
hostname -I | awk '{print $1}'
```

3. In the mobile app, connect to:
   - WebSocket: `ws://<YOUR_IP>:9944`
   - Example: `ws://192.168.1.100:9944`

> **Note**: Using `localhost` or `127.0.0.1` from a mobile device won't work - you must use your machine's actual IP address.

## Troubleshooting

### No Blocks Being Produced

**Symptom**: Logs show nodes starting but no "Imported" messages appear.

**Solutions**:
1. Check both nodes are running: `podman-compose ps`
2. Check Alice's peer count: `podman-compose logs alice | grep "peers"`
3. Wait for Bob to connect (can take 30-60 seconds)
4. Restart both nodes: `podman-compose restart`

### Bob Can't Connect to Alice

**Symptom**: Bob's logs show "No peers available" or connection timeouts.

**Solutions**:
1. Verify Alice started first and is healthy: `podman-compose ps`
2. Check the bootnode peer ID matches: `podman-compose logs alice | grep "Local node identity"`
3. Restart both nodes: `podman-compose down && podman-compose up -d`

### RPC Not Accessible

**Symptom**: `curl` commands fail or apps can't connect.

**Solutions**:
1. Verify containers are running: `podman-compose ps`
2. Check port mappings: `podman-compose port alice 9944`
3. Test from container first: `podman-compose exec alice curl http://localhost:9933/health`
4. Check firewall isn't blocking ports 9933/9944

### Out of Disk Space

**Symptom**: Build fails with "no space left on device".

**Solutions**:
```bash
# Remove old images
podman image prune -a

# Remove unused volumes
podman volume prune

# Remove all stopped containers
podman container prune
```

### Build from Source Fails (Cargo Registry Corruption)

**Symptom**: Build fails with cargo registry errors or manifest parsing errors.

**Why it happens**: Building Substrate inside Docker is resource-intensive and prone to network/registry issues during the ~30-60 minute build.

**Solution**: Use the pre-built image instead:

```bash
export CLAD_IMAGE=ghcr.io/clad-sovereign/clad-node:latest
podman-compose up -d
```

If you must build from source (e.g., testing local changes), try:
1. Retry the build (often resolves itself)
2. Build with no cache: `podman-compose build --no-cache`
3. Use the original Dockerfile which builds inside Linux:
   ```bash
   # This builds from source inside a Linux container (~30-60 min)
   podman-compose up -d --build
   ```

> **Note for macOS/Windows users**: The `Dockerfile.runtime` approach only works with Linux binaries. Since `cargo build` on macOS produces macOS binaries, they won't run in Linux containers. Use `--build` to compile inside the container, or pull the pre-built image from CI.

### Chain State Corrupted

**Symptom**: Nodes crash on startup or blocks don't finalize.

**Solution**:
```bash
# Complete reset
podman-compose down -v
podman-compose up -d
```

## Advanced Configuration

### Custom Chain Spec

To use a custom chain specification:

1. Generate chain spec:
```bash
podman-compose run --rm alice build-spec --chain local > custom-spec.json
```

2. Convert to raw format:
```bash
podman-compose run --rm alice build-spec --chain custom-spec.json --raw > custom-spec-raw.json
```

3. Update `docker-compose.yml` to use custom spec:
```yaml
command: [
  "--alice",
  "--validator",
  "--chain", "/clad-node/custom-spec-raw.json"
]
volumes:
  - ./custom-spec-raw.json:/clad-node/custom-spec-raw.json:ro
```

### Running a Single Dev Node

For quick testing with instant block sealing:

```bash
podman run --rm -p 9944:9944 -p 9933:9933 \
  ghcr.io/clad-sovereign/clad-node:latest \
  --dev --rpc-external --rpc-cors all
```

### Production Deployment

For production use:

1. Remove unsafe RPC flags (`--rpc-cors all`)
2. Use proper chain specification with real validators
3. Configure TLS/HTTPS for RPC endpoints
4. Set up monitoring (Prometheus/Grafana)
5. Implement proper key management (never use dev keys)
6. Use dedicated hardware with appropriate resources

## Testing Checklist

Before submitting a PR with Docker changes:

- [ ] Pre-built image starts: `CLAD_IMAGE=ghcr.io/clad-sovereign/clad-node:latest podman-compose up -d`
- [ ] Both nodes show as healthy: `podman-compose ps`
- [ ] Alice produces blocks within 60 seconds
- [ ] Bob connects to Alice and syncs blocks
- [ ] Blocks are finalized (GRANDPA)
- [ ] RPC endpoint responds: `curl http://localhost:9933/health`
- [ ] Polkadot.js Apps connects via `ws://localhost:9944`
- [ ] `podman-compose down -v` cleans up successfully

## Resources

- [Polkadot.js Apps](https://polkadot.js.org/apps/) - Browser-based chain interaction
- [Podman Documentation](https://docs.podman.io/)
- [Docker Documentation](https://docs.docker.com/)
- [Polkadot SDK Docs](https://docs.polkadot.com/)

## Support

If you encounter issues not covered here:

1. Check container logs: `podman-compose logs`
2. Verify setup: `podman info`
3. Open an issue on GitHub with:
   - Podman/Docker version: `podman --version`
   - OS version
   - Full error logs
   - Steps to reproduce
