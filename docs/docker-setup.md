# Docker Setup Guide

This guide explains how to run a local multi-validator Clad Studio testnet using Docker.

## Prerequisites

- Docker (version 20.10 or later) OR Podman (version 3.0 or later)
- Docker Compose (version 2.0 or later) OR podman-compose
- At least 4GB of free disk space
- At least 4GB of RAM

### Installation

**macOS (Docker):**
```bash
brew install docker docker-compose
```

**macOS (Podman):**
```bash
brew install podman podman-compose
podman machine init
podman machine start
```

**Ubuntu/Debian (Docker):**
```bash
sudo apt-get update
sudo apt-get install docker.io docker-compose-v2
sudo systemctl start docker
sudo systemctl enable docker
```

**Ubuntu/Debian (Podman):**
```bash
sudo apt-get update
sudo apt-get install podman podman-compose
```

**Note on Podman**: Podman is a drop-in replacement for Docker. If using Podman, simply replace `docker` with `podman` and `docker-compose` with `podman-compose` in all commands below.

## Quick Start

### 1. Build and Start the Testnet

From the repository root:

```bash
# Build Docker images and start the network
docker-compose up -d

# View logs from both validators
docker-compose logs -f

# View logs from a specific validator
docker-compose logs -f alice
docker-compose logs -f bob
```

This starts a 2-validator testnet with:
- **Alice** (primary validator) - Ports: 9944 (WS RPC), 9933 (HTTP RPC), 30333 (P2P)
- **Bob** (secondary validator) - Ports: 9945 (WS RPC), 9934 (HTTP RPC), 30334 (P2P)

### 2. Verify Block Production

Check that blocks are being produced and finalized:

```bash
# Watch Alice's logs for block production
docker-compose logs -f alice | grep "Imported"

# Expected output (approximately every 6 seconds):
# Imported #1 (0x1234...)
# Imported #2 (0x5678...)
# Finalized #1 (0x1234...)
```

### 3. Connect via RPC

The RPC endpoint is available at:
- WebSocket: `ws://localhost:9944`
- HTTP: `http://localhost:9933`

Test connectivity:

```bash
# Health check
curl -H "Content-Type: application/json" \
     -d '{"id":1, "jsonrpc":"2.0", "method": "system_health"}' \
     http://localhost:9933

# Get chain name
curl -H "Content-Type: application/json" \
     -d '{"id":1, "jsonrpc":"2.0", "method": "system_chain"}' \
     http://localhost:9933

# Get latest block number
curl -H "Content-Type: application/json" \
     -d '{"id":1, "jsonrpc":"2.0", "method": "chain_getHeader"}' \
     http://localhost:9933
```

## Network Architecture

```
┌─────────────────┐         ┌─────────────────┐
│  Alice Node     │◄───────►│   Bob Node      │
│  (Validator)    │   P2P   │  (Validator)    │
│                 │         │                 │
│ WS:  9944       │         │ WS:  9945       │
│ HTTP: 9933      │         │ HTTP: 9934      │
│ P2P:  30333     │         │ P2P:  30334     │
└────────┬────────┘         └────────┬────────┘
         │                           │
         └──────────┬────────────────┘
                    │
                    ▼
         clad-network (bridge)
                    │
                    ▼
         External Access (localhost)
```

## Configuration

### Chain Specification

The testnet uses the built-in `local` chain spec with:
- **Consensus**: AURA (6-second block time) + GRANDPA (finality)
- **Validators**: Alice and Bob (SR25519 keys)
- **Network**: Private local network

### Ports

| Service | WebSocket | HTTP | P2P   |
|---------|-----------|------|-------|
| Alice   | 9944      | 9933 | 30333 |
| Bob     | 9945      | 9934 | 30334 |

### Data Persistence

Blockchain data is stored in Docker volumes:
- `alice-data` - Alice's blockchain database
- `bob-data` - Bob's blockchain database

To reset the chain:

```bash
# Stop and remove containers and volumes
docker-compose down -v

# Restart fresh
docker-compose up -d
```

## Connecting Mobile App

To connect the Clad mobile app to your local testnet:

1. Ensure your mobile device is on the same network as your Docker host
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

**Note**: Using `localhost` or `127.0.0.1` from a mobile device won't work - you must use your machine's actual IP address.

## Common Operations

### Start the Network

```bash
docker-compose up -d
```

### Stop the Network

```bash
docker-compose stop
```

### Restart a Specific Node

```bash
docker-compose restart alice
```

### View Live Logs

```bash
# All nodes
docker-compose logs -f

# Specific node
docker-compose logs -f bob

# Last 100 lines
docker-compose logs --tail=100 alice
```

### Execute Commands Inside a Container

```bash
# Open shell in Alice's container
docker-compose exec alice /bin/sh

# Check node version
docker-compose exec alice clad-node --version
```

### Rebuild After Code Changes

```bash
# Rebuild images
docker-compose build

# Or rebuild and restart
docker-compose up -d --build
```

## Troubleshooting

### No Blocks Being Produced

**Symptom**: Logs show nodes starting but no "Imported" messages appear.

**Solutions**:
1. Check both nodes are running: `docker-compose ps`
2. Verify network connectivity: `docker-compose exec bob ping alice`
3. Check Alice's peer count: `docker-compose logs alice | grep "peers"`
4. Ensure ports aren't blocked by firewall

### Bob Can't Connect to Alice

**Symptom**: Bob's logs show "No peers available" or connection timeouts.

**Solutions**:
1. Verify Alice started first: `docker-compose up -d alice && sleep 10 && docker-compose up -d bob`
2. Check the bootnode peer ID matches: `docker-compose logs alice | grep "Local node identity"`
3. Restart both nodes: `docker-compose restart`

### RPC Not Accessible

**Symptom**: `curl` commands fail or mobile app can't connect.

**Solutions**:
1. Verify containers are running: `docker-compose ps`
2. Check port mappings: `docker-compose port alice 9944`
3. Test from container first: `docker-compose exec alice curl http://localhost:9933/health`
4. Check firewall isn't blocking ports 9933/9944

### Out of Disk Space

**Symptom**: Build fails with "no space left on device".

**Solutions**:
```bash
# Remove old Docker images
docker image prune -a

# Remove unused volumes
docker volume prune

# Remove all stopped containers
docker container prune
```

### Slow Build Times

**Symptom**: `docker-compose build` takes 10+ minutes.

**Solutions**:
1. Enable BuildKit: `export DOCKER_BUILDKIT=1`
2. Use more CPU cores: `docker-compose build --parallel`
3. Ensure `.dockerignore` exists to exclude `target/` directory

### Cargo Registry Corruption

**Symptom**: Build fails with `error: failed to parse manifest at .../Cargo.toml` for a dependency like `base64ct`.

**Solutions**:
1. Retry the build (often resolves itself)
2. Build with --no-cache: `docker-compose build --no-cache`
3. Clear cargo cache and rebuild:
```bash
# Remove existing images
docker rmi clad-studio_alice clad-studio_bob || true
# Rebuild
docker-compose build
```

This is a known transient issue with cargo's registry caching in containerized builds.

### Chain State Corrupted

**Symptom**: Nodes crash on startup or blocks don't finalize.

**Solution**:
```bash
# Complete reset
docker-compose down -v
docker-compose up -d
```

## Advanced Configuration

### Custom Chain Spec

To use a custom chain specification:

1. Generate chain spec:
```bash
docker-compose run --rm alice build-spec --chain local > custom-spec.json
```

2. Convert to raw format:
```bash
docker-compose run --rm alice build-spec --chain custom-spec.json --raw > custom-spec-raw.json
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

### Running in Development Mode

For quick testing with a single node:

```bash
docker run --rm -p 9944:9944 -p 9933:9933 \
  clad-studio:latest \
  --dev --rpc-external --rpc-cors all
```

### Production Deployment

For production use:

1. Remove unsafe RPC flags
2. Use proper chain specification
3. Configure TLS/HTTPS for RPC
4. Set up monitoring (Prometheus/Grafana)
5. Implement proper key management
6. Use non-root user (already configured in Dockerfile)

## Testing Checklist

Before submitting a PR with Docker changes:

- [ ] `docker-compose build` succeeds
- [ ] `docker-compose up -d` starts both nodes
- [ ] Alice produces blocks within 30 seconds
- [ ] Bob connects to Alice and syncs blocks
- [ ] Blocks are finalized (GRANDPA)
- [ ] RPC endpoint responds: `curl http://localhost:9933/health`
- [ ] WebSocket RPC works (test with mobile app or Polkadot.js)
- [ ] `docker-compose down -v` cleans up successfully

## Resources

- [Docker Documentation](https://docs.docker.com/)
- [Docker Compose Reference](https://docs.docker.com/compose/compose-file/)
- [Substrate Node Template](https://github.com/substrate-developer-hub/substrate-node-template)
- [Polkadot SDK Docs](https://docs.polkadot.com/)

## Support

If you encounter issues not covered here:

1. Check container logs: `docker-compose logs`
2. Verify Docker setup: `docker info`
3. Open an issue on GitHub with:
   - Docker version: `docker --version`
   - OS version
   - Full error logs
   - Steps to reproduce
