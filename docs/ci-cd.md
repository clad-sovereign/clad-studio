# CI/CD Pipeline Documentation

This document describes the Continuous Integration and Continuous Deployment (CI/CD) setup for Clad Studio, including how pipelines are configured, which checks must pass before merging, and how to test changes locally.

## Overview

Clad Studio uses GitHub Actions for automated testing, building, and validation. The CI/CD pipeline ensures code quality, prevents regressions, and validates that all components compile correctly before merging to the main branch.

**CI Status Badge:** [![CI](https://github.com/clad-sovereign/clad-studio/actions/workflows/ci.yml/badge.svg)](https://github.com/clad-sovereign/clad-studio/actions)

## Workflows

### 1. CI Workflow (`.github/workflows/ci.yml`)

**Triggers:**
- Push to `main` or `develop` branches
- Pull requests to `main` or `develop` branches

**Jobs:**

#### `check` - Code Quality Checks
Validates code formatting and linting standards.

**Steps:**
1. Install Rust toolchain (stable) with rustfmt, clippy, rust-src
2. Install system dependencies (build-essential, clang, protobuf, etc.)
3. Cache cargo registry, index, and build artifacts
4. Check code formatting: `cargo fmt -- --check`
5. Run clippy linter: `cargo clippy --all-targets --all-features --locked -- -D warnings`
6. Verify pallet compiles: `cargo check -p pallet-clad-token --locked`

**Purpose:** Ensures code follows formatting standards and passes all linting rules.

#### `test` - Test Suite
Runs the complete test suite for all workspace members.

**Steps:**
1. Install Rust toolchain (stable) with rust-src and wasm target
2. Install system dependencies
3. Cache cargo artifacts
4. Run tests: `cargo test --all --locked`

**Test Coverage:**
- Runtime integrity tests
- Pallet unit tests (22 tests for `pallet-clad-token`)
- Genesis configuration tests
- Mock runtime tests

**Purpose:** Validates all business logic and ensures no regressions.

#### `build` - Pallet Build
Builds the core `pallet-clad-token` in release mode.

**Steps:**
1. Install Rust toolchain
2. Install dependencies
3. Cache artifacts
4. Build: `cargo build --release -p pallet-clad-token --locked`

**Purpose:** Ensures the pallet compiles successfully in release mode.

#### `build-runtime` - Runtime Build
Builds the complete runtime with WASM compilation.

**Steps:**
1. Install Rust toolchain with wasm32-unknown-unknown target
2. Install dependencies
3. Cache artifacts
4. Build: `cargo build --release -p clad-runtime --locked`
5. Verify WASM binary exists: `target/release/wbuild/clad-runtime/clad_runtime.compact.compressed.wasm`

**Purpose:** Validates runtime compiles correctly and generates valid WASM binary for production deployment.

#### `build-benchmarks` - Benchmark Build
Builds runtime with benchmarking features enabled.

**Steps:**
1. Install Rust toolchain with wasm target
2. Install dependencies
3. Cache artifacts (separate cache key: `cargo-build-target-benchmarks`)
4. Build: `cargo build --features runtime-benchmarks --release -p clad-runtime --locked`

**Purpose:** Ensures benchmarking code stays compilable for future weight calculations (required for issue #12).

**Cache Strategy:**
- Each job caches cargo registry, index, and build artifacts separately
- Benchmark builds use dedicated cache key to avoid conflicts
- Caches are keyed by `Cargo.lock` hash for consistency

**Total Duration:** ~3-5 minutes (with warm caches)

### 2. Docker Workflow (`.github/workflows/docker.yml`)

**Triggers:**
- Push to `main` branch only (expensive operation)
- Pull requests to `main` branch

**Jobs:**

#### `docker-build` - Docker Integration Test
Validates Docker containerization and multi-validator testnet.

**Steps:**
1. Set up Docker Buildx
2. Build Docker image with layer caching (GitHub Actions cache)
3. Start multi-validator testnet: `docker-compose up -d`
4. Wait 30 seconds for nodes to initialize
5. Check Alice node health and logs
6. Check Bob node health and logs
7. Verify block production (20-second observation window)
8. Test RPC connectivity: `curl http://localhost:9944`
9. Cleanup: `docker-compose down -v`
10. Upload logs on failure (alice-logs.txt, bob-logs.txt)

**Purpose:** Ensures Docker setup works end-to-end, validators communicate properly, and blocks are produced.

**Note:** This workflow only runs on the main branch to conserve CI resources (Docker builds are time-intensive).

**Total Duration:** ~15-25 minutes (Docker build + testnet validation)

## Required Checks for Merge

Before any pull request can be merged to `main`, the following checks **must pass**:

✅ **Code Quality:**
- `cargo fmt -- --check` (formatting)
- `cargo clippy` (linting, -D warnings)

✅ **Compilation:**
- `cargo check -p pallet-clad-token --locked`
- `cargo build --release -p clad-runtime --locked` (runtime + WASM)
- `cargo build --features runtime-benchmarks --release -p clad-runtime --locked`

✅ **Testing:**
- `cargo test --all --locked` (all 22+ tests pass)

✅ **Docker (main branch only):**
- Docker image builds successfully
- Multi-validator testnet starts and produces blocks
- RPC endpoints accessible

## Branch Protection Settings

### Recommended Configuration

Navigate to **Settings → Branches → Branch protection rules** for `main`:

**Require a pull request before merging:**
- ✅ Enable
- Require approvals: 1 (can be 0 for solo development)
- Dismiss stale pull request approvals when new commits are pushed: ✅

**Require status checks to pass before merging:**
- ✅ Enable
- Require branches to be up to date before merging: ✅
- **Required status checks:**
  - `check` (formatting + clippy + pallet check)
  - `test` (test suite)
  - `build` (pallet release build)
  - `build-runtime` (runtime + WASM)
  - `build-benchmarks` (benchmark build)
  - `docker-build` (Docker integration test) - Optional for faster iteration

**Require conversation resolution before merging:**
- ✅ Enable (recommended)

**Do not allow bypassing the above settings:**
- ✅ Enable (for team environments)
- Can be disabled for solo development

### GitHub Settings Path

```
Repository → Settings → Branches → Add rule
```

**Branch name pattern:** `main`

Then configure the protection rules as described above.

## Testing CI Changes Locally

Before pushing changes that modify CI workflows or core build configurations, test them locally to avoid breaking the pipeline.

### Prerequisites

Install required dependencies:

**macOS:**
```bash
brew install cmake pkg-config openssl git curl protobuf
rustup target add wasm32-unknown-unknown
```

**Linux (Debian/Ubuntu):**
```bash
sudo apt install build-essential git clang curl libssl-dev llvm \
  libudev-dev make protobuf-compiler pkg-config
rustup target add wasm32-unknown-unknown
```

### Local Testing Commands

Run these commands **in order** before committing:

#### 1. Format Check
```bash
cargo fmt
cargo fmt -- --check
```

**Expected output:** No output (formatting is correct)

#### 2. Linting
```bash
cargo clippy --all-targets --all-features --locked -- -D warnings
```

**Expected output:** No warnings or errors

#### 3. Tests
```bash
cargo test --all --locked
```

**Expected output:**
```
running 22 tests
...
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

#### 4. Pallet Build
```bash
cargo build --release -p pallet-clad-token --locked
```

**Expected output:** `Finished release [optimized] target(s) in X.XXs`

#### 5. Runtime Build
```bash
cargo build --release -p clad-runtime --locked
```

**Expected output:**
- `Finished release [optimized] target(s) in X.XXs`
- Verify WASM: `ls -lh target/release/wbuild/clad-runtime/clad_runtime.compact.compressed.wasm`

#### 6. Benchmark Build
```bash
cargo build --features runtime-benchmarks --release -p clad-runtime --locked
```

**Expected output:** `Finished release [optimized] target(s) in X.XXs`

#### 7. Docker Test (Optional)
```bash
docker-compose build
docker-compose up -d
docker-compose logs -f alice  # Watch for block production
curl http://localhost:9933/health  # Test RPC
docker-compose down -v
```

**Expected output:**
- Nodes start successfully
- Logs show "Imported #N" messages (block production)
- RPC returns health status

### Quick Pre-Commit Script

Create a script to run all checks:

```bash
#!/bin/bash
# pre-commit-check.sh

set -e

echo "🔍 Running pre-commit checks..."

echo "1️⃣ Formatting..."
cargo fmt
cargo fmt -- --check

echo "2️⃣ Clippy..."
cargo clippy --all-targets --all-features --locked -- -D warnings

echo "3️⃣ Tests..."
cargo test --locked

echo "4️⃣ Release build..."
cargo build --release --locked

echo "✅ All checks passed!"
```

Make it executable:
```bash
chmod +x pre-commit-check.sh
./pre-commit-check.sh
```

## Troubleshooting CI Failures

### Formatting Failures

**Error:** `cargo fmt -- --check` fails

**Solution:**
```bash
cargo fmt
git add .
git commit --amend --no-edit
git push --force-with-lease
```

### Clippy Warnings

**Error:** `error: aborting due to previous error` from clippy

**Solution:**
1. Run `cargo clippy --all-targets --all-features --locked -- -D warnings` locally
2. Fix all warnings shown
3. Commit and push

**Common issues:**
- Unused variables: Prefix with `_` (e.g., `_unused_var`)
- Unused imports: Remove them
- Missing documentation: Add doc comments for public items

### Test Failures

**Error:** Tests fail in CI but pass locally

**Possible causes:**
1. **Cargo.lock out of sync:** Run `cargo update` then commit `Cargo.lock`
2. **Different Rust version:** Check CI uses same version: `rustup show`
3. **Platform differences:** Tests may behave differently on Linux (CI) vs macOS/Windows (local)

**Solution:**
```bash
cargo clean
cargo test --locked
```

If tests still fail, check the CI logs for the exact error.

### Build Failures

**Error:** `cargo build` fails with linker errors

**Possible causes:**
- Missing system dependencies
- Cargo.lock out of sync

**Solution:**
```bash
# Install dependencies (see Prerequisites above)
cargo clean
cargo build --release --locked
```

### Docker Build Failures

**Error:** Docker image fails to build

**Common issues:**
1. **Cargo registry corruption:** Retry the build or use `--no-cache`
2. **Out of disk space:** Clean Docker: `docker system prune -a`
3. **Network issues:** Check internet connection, retry

**Solution:**
```bash
docker-compose build --no-cache
```

### WASM Binary Missing

**Error:** `target/release/wbuild/clad-runtime/clad_runtime.compact.compressed.wasm` not found

**Cause:** Runtime didn't compile with WASM target

**Solution:**
```bash
rustup target add wasm32-unknown-unknown
cargo clean
cargo build --release -p clad-runtime --locked
```

## Monitoring CI Status

### GitHub Actions Dashboard

View all workflow runs:
```
Repository → Actions
```

**Useful filters:**
- `branch:main` - Show only main branch runs
- `event:pull_request` - Show PR runs
- `status:failure` - Show failed runs

### Status Badge

The CI status badge in README.md shows the current state of the main branch:

[![CI](https://github.com/clad-sovereign/clad-studio/actions/workflows/ci.yml/badge.svg)](https://github.com/clad-sovereign/clad-studio/actions)

**Badge states:**
- 🟢 **Passing:** All checks pass on main
- 🔴 **Failing:** At least one check failed
- 🟡 **In progress:** CI is currently running

Click the badge to view detailed workflow results.

### Email Notifications

GitHub sends email notifications for:
- CI failures on your branches
- Required checks failing on PRs you authored
- Successful runs after previous failures

Configure in: **Settings → Notifications → Actions**

## Performance Optimization

### Cache Strategy

The CI pipeline uses aggressive caching to minimize build times:

**Cargo Registry Cache:**
- Key: `${{ runner.os }}-cargo-registry-${{ hashFiles('**/Cargo.lock') }}`
- Invalidates when dependencies change

**Cargo Index Cache:**
- Key: `${{ runner.os }}-cargo-index-${{ hashFiles('**/Cargo.lock') }}`
- Invalidates when dependencies change

**Build Artifacts Cache:**
- Key: `${{ runner.os }}-cargo-build-target-${{ hashFiles('**/Cargo.lock') }}`
- Separate key for benchmark builds: `-benchmarks` suffix
- Invalidates when dependencies change

**Docker Layer Cache:**
- Uses GitHub Actions cache backend
- Caches intermediate Docker layers
- Significantly speeds up subsequent builds

### Typical Build Times

**With warm caches:**
- Check job: ~1 min
- Test job: ~1.5 min
- Build jobs: ~2 min each
- Docker job: ~10-15 min (main branch only)

**With cold caches:**
- Check job: ~3 min
- Test job: ~4 min
- Build jobs: ~8-10 min each
- Docker job: ~20-25 min

**Total CI time:** ~3-5 min (warm) / ~15-20 min (cold)

## Security Considerations

### Locked Dependencies

All CI builds use `--locked` flag:
```bash
cargo build --locked
```

**Benefits:**
- Ensures exact dependency versions from `Cargo.lock`
- Prevents supply chain attacks via dependency substitution
- Guarantees reproducible builds
- Fails if `Cargo.lock` is out of sync with `Cargo.toml`

**Critical for blockchain code** where reproducibility and security are paramount.

### RPC Safety in Docker

The Docker workflow tests with `--rpc-methods Safe`:
- Blocks potentially dangerous RPC methods
- Suitable for production deployments
- Aligns with Polkadot SDK security best practices

For local development, you can use `--rpc-methods Unsafe` but **never in production**.

## Future Enhancements

Planned improvements to the CI/CD pipeline:

1. **Coverage Reporting:** Integrate `cargo-tarpaulin` for test coverage metrics
2. **Automated Releases:** Tag-based releases with GitHub Releases
3. **Parachain Testing:** Add Zombienet tests for parachain functionality
4. **Benchmark Automation:** Automatically update weights on benchmark changes
5. **Security Scanning:** Add `cargo-audit` for dependency vulnerability checks
6. **Nightly Builds:** Test against Rust nightly for early warning of issues

## Resources

- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [Rust CI Best Practices](https://doc.rust-lang.org/cargo/guide/continuous-integration.html)
- [Polkadot SDK CI Examples](https://github.com/paritytech/polkadot-sdk/tree/master/.github/workflows)
- [Docker Best Practices](https://docs.docker.com/develop/dev-best-practices/)

## Support

If you encounter CI issues:

1. Check this documentation first
2. Review CI logs in GitHub Actions dashboard
3. Test locally using commands in "Testing CI Changes Locally" section
4. Open an issue with:
   - CI run URL
   - Error logs
   - Steps to reproduce locally
   - Your environment (OS, Rust version)

For urgent CI failures blocking merges, contact the maintainers via GitHub issues or the project's communication channels.
