# ZombieNet Setup Guide

ZombieNet is Polkadot's official tool for spawning ephemeral test networks and running automated assertions against them.

## Installation

### macOS (Apple Silicon)

```bash
curl -L -o zombienet https://github.com/paritytech/zombienet/releases/download/v1.3.133/zombienet-macos-arm64
chmod +x zombienet
sudo mv zombienet /usr/local/bin/

# Verify installation
zombienet version
```

### macOS (Intel)

```bash
curl -L -o zombienet https://github.com/paritytech/zombienet/releases/download/v1.3.133/zombienet-macos-x64
chmod +x zombienet
sudo mv zombienet /usr/local/bin/
```

### Linux

```bash
curl -L -o zombienet https://github.com/paritytech/zombienet/releases/download/v1.3.133/zombienet-linux-x64
chmod +x zombienet
sudo mv zombienet /usr/local/bin/
```

### NPM (Alternative)

```bash
npm i @zombienet/cli -g
```

## Quick Start

### 1. Build the node

```bash
cargo build --release --locked -p clad-node
```

### 2. Spawn a test network (interactive)

```bash
zombienet -p native spawn zombienet/network.toml
```

This starts a 2-validator network (Alice + Bob) and displays connection URLs:
- Polkadot.js Apps: `https://polkadot.js.org/apps/?rpc=ws://127.0.0.1:{PORT}`
- Prometheus metrics: `http://127.0.0.1:{PORT}/metrics`

Press `Ctrl+C` to stop the network.

### 3. Run automated tests

```bash
zombienet -p native test zombienet/basic.zndsl
```

This runs the test assertions and reports pass/fail status.

## Configuration Reference

### Network Configuration (`zombienet/network.toml`)

```toml
[settings]
timeout = 120           # Max seconds to wait for network to start
provider = "native"     # native, podman, kubernetes, docker

[relaychain]
default_command = "./target/release/clad-node"
chain = "local"

[[relaychain.nodes]]
name = "alice"
validator = true
args = ["--rpc-port", "9944", "--prometheus-port", "9615"]

[[relaychain.nodes]]
name = "bob"
validator = true
args = ["--rpc-port", "9945", "--prometheus-port", "9616"]
```

### Test File (`zombienet/basic.zndsl`)

```
Description: Basic clad-node network test
Network: ./network.toml
Creds: config

alice: is up
bob: is up
alice: reports peers is at least 1 within 30 seconds
alice: reports block height is at least 5 within 60 seconds
```

## Provider Options

| Provider | Platform | Use Case |
|----------|----------|----------|
| `native` | Linux, macOS | Local development, CI (recommended) |
| `podman` | Linux only | Rootless containers |
| `docker` | Linux, macOS | Container-based testing |
| `kubernetes` | Any | Production-like environments |

## DSL Assertions

### Node Status
- `alice: is up` - Node process is running

### Prometheus Metrics
- `alice: reports peers is at least 1` - Peer count
- `alice: reports block height is at least 5` - Block production
- `alice: reports node_roles is 4` - Authority role (validator)

### Timeouts
Add `within X seconds` to any assertion:
```
alice: reports block height is at least 10 within 120 seconds
```

## Troubleshooting

### Binary not found

Ensure `clad-node` is built and the path in `network.toml` is correct:
```bash
ls -la ./target/release/clad-node
```

### Port conflicts

If default ports are in use, ZombieNet will assign random ports. Check the output for actual port numbers.

### macOS Gatekeeper

If macOS blocks the binary:
```bash
xattr -d com.apple.quarantine zombienet
```

## References

- [ZombieNet Documentation](https://docs.polkadot.com/develop/toolkit/parachains/spawn-chains/zombienet/)
- [ZombieNet GitHub](https://github.com/paritytech/zombienet)
- [DSL Reference](https://paritytech.github.io/zombienet/)
