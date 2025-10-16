# GitHub Actions Workflows

This directory contains the CI/CD workflows for the project.

## Workflows

### CI (`ci.yml`)

**Triggers:**
- Push to `main` or `dev` branches
- Pull requests to any branch

**Jobs:**
- **test**: Runs tests on Linux, macOS, and Windows
  - Runs `cargo test`
  - Runs `cargo clippy` with warnings as errors
  - Checks code formatting with `cargo fmt`
- **build**: Builds the project in release mode

**Purpose:** Ensures code quality and compatibility across platforms on every change.

### Release (`release.yml`)

**Triggers:**
- Push to `main` branch (excluding changes to `.md`, `LICENSE`, `.gitignore`)

**Jobs:**
- **create-release**: 
  - Extracts version from `Cargo.toml`
  - Checks if a tag for that version already exists
  - Creates a new GitHub release with tag `vX.Y.Z`
  - Skips if tag already exists
  
- **build**:
  - Builds binaries for multiple platforms:
    - Linux: x86_64 (glibc and musl)
    - macOS: x86_64 (Intel) and aarch64 (Apple Silicon)
    - Windows: x86_64
  - Strips binaries (Linux/macOS only)
  - Uploads binaries as release assets

**Purpose:** Automatically creates releases and distributes pre-built binaries.

## How Releases Work

1. **Version Bump**: Update `version` in `Cargo.toml`
   ```toml
   [package]
   version = "0.2.0"
   ```

2. **Commit and Push**:
   ```bash
   git add Cargo.toml CHANGELOG.md
   git commit -m "Release v0.2.0"
   git push origin main
   ```

3. **Automatic Process**:
   - GitHub Actions reads the version from `Cargo.toml`
   - Creates a git tag (e.g., `v0.2.0`)
   - Creates a GitHub release
   - Builds binaries for all platforms in parallel
   - Attaches binaries to the release

4. **Result**: Users can download pre-built binaries from the releases page

## Binary Naming Convention

- Linux (glibc): `rbxbuild-linux-x86_64`
- Linux (musl): `rbxbuild-linux-x86_64-musl`
- macOS (Intel): `rbxbuild-macos-x86_64`
- macOS (Apple Silicon): `rbxbuild-macos-aarch64`
- Windows: `rbxbuild-windows-x86_64.exe`

## Caching

The CI workflow uses GitHub Actions cache for:
- Cargo registry
- Cargo git index
- Build artifacts

This speeds up builds by avoiding redundant downloads and compilations.

## Secrets Required

- `GITHUB_TOKEN`: Automatically provided by GitHub Actions (no setup needed)

## Testing Locally

To test the release build process locally, use the provided script:

```bash
./scripts/build-release.sh
```

This will build binaries for all targets that are supported on your platform and place them in the `dist/` directory.
