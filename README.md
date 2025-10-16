# rbxbuild

A lightweight CLI that converts Rojo project JSON strings into Roblox place XML files.

## About

This project leverages the [Rojo](https://rojo.space/) ecosystem, specifically the `rbx_dom_weak` and related crates, to provide a minimal build tool. It uses Rojo's robust DOM implementation to parse project files and serialize them into Roblox XML format, without the full overhead of the complete Rojo toolchain.

## Usage

Pass a Rojo project JSON string to the tool:

```bash
rbxbuild '{"name": "MyProject", "tree": {...}}'
```

The tool will output the corresponding Roblox place XML to stdout.

## Building

Build the project using Cargo:

```bash
cargo build --release
```

The compiled binary will be available at `target/release/rojo-build-lite` (or `rbxbuild` depending on your Cargo configuration).

## Installation

### Pre-built Binaries

Download the latest pre-built binaries from the [releases page](https://github.com/paralov/rbxbuild/releases).

Available platforms:
- **Linux**: `rbxbuild-linux-x86_64`, `rbxbuild-linux-x86_64-musl`
- **macOS**: `rbxbuild-macos-x86_64` (Intel), `rbxbuild-macos-aarch64` (Apple Silicon)
- **Windows**: `rbxbuild-windows-x86_64.exe`

After downloading, make the binary executable (Linux/macOS):
```bash
chmod +x rbxbuild-*
```

### From Source

Build from source using Cargo:
```bash
cargo install --git https://github.com/paralov/rbxbuild
```

## Development

### Testing

Run the test suite:
```bash
cargo test
```

### CI/CD

This project uses GitHub Actions for continuous integration and deployment:

- **CI Pipeline** (`.github/workflows/ci.yml`): Runs on every push and PR
  - Runs tests on Linux, macOS, and Windows
  - Runs clippy linter
  - Checks code formatting
  
- **Release Pipeline** (`.github/workflows/release.yml`): Runs on pushes to `main`
  - Automatically creates a release based on version in `Cargo.toml`
  - Builds binaries for multiple platforms
  - Attaches binaries to the GitHub release

### Creating a Release

To create a new release:

1. Update the version in `Cargo.toml`:
   ```toml
   [package]
   version = "0.2.0"  # Bump this version
   ```

2. Commit and push to `main`:
   ```bash
   git add Cargo.toml
   git commit -m "Bump version to 0.2.0"
   git push origin main
   ```

3. The GitHub Actions workflow will automatically:
   - Create a git tag (e.g., `v0.2.0`)
   - Create a GitHub release
   - Build binaries for all platforms
   - Attach binaries to the release

**Note**: If a tag for the version already exists, the release workflow will skip creating a new release.
