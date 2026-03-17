---
name: "rust-cross-platform-build"
description: "Build Rust projects for multiple platforms (Windows/Linux/macOS) using GitHub Actions CI/CD. Invoke when user needs to set up cross-platform compilation workflows for Rust applications, or when onboarding new developers to a Rust project with complex build requirements."
---

# Rust Cross-Platform Build Skill

This skill helps you set up cross-platform CI/CD pipelines for Rust projects, inspired by Zed's build system. It's designed for both beginners (getting started from scratch) and experienced developers (setting up production workflows).

## Overview

This skill provides patterns for:
- Building Rust applications for Windows, Linux, and macOS
- Creating platform-specific installers (DMG, MSI/EXE, tarball)
- Setting up GitHub Actions workflows for automated builds
- Cross-compilation strategies
- Environment setup for new developers

## Quick Start for Beginners

### Step 1: Environment Setup

#### macOS
```bash
# 1. Install Xcode from App Store
# 2. Install Xcode command line tools
xcode-select --install

# 3. Accept Xcode license
sudo xcode-select --switch /Applications/Xcode.app/Contents/Developer
sudo xcodebuild -license accept

# 4. Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# 5. Install cmake
brew install cmake

# 6. Verify installation
rustc --version
cargo --version
```

#### Linux (Ubuntu/Debian)
```bash
# 1. Install system dependencies
sudo apt-get update
sudo apt-get install -y \
  gcc g++ cmake clang \
  libasound2-dev libfontconfig-dev libssl-dev \
  libwayland-dev libxkbcommon-x11-dev \
  libzstd-dev git curl

# 2. Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# 3. (Optional but recommended) Install faster linker
# Option A: Mold
sudo apt-get install mold

# Option B: Wild (even faster)
cargo install wild-linker

# 4. Configure linker (add to ~/.cargo/config.toml)
mkdir -p ~/.cargo
cat >> ~/.cargo/config.toml << 'EOF'
[target.x86_64-unknown-linux-gnu]
linker = "clang"
rustflags = ["-C", "link-arg=--ld-path=mold"]
EOF

# 5. Verify installation
rustc --version
cargo --version
```

#### Windows
```powershell
# 1. Install Rust
# Download from https://rustup.rs/ and run rustup-init.exe

# 2. Install Visual Studio 2022 or Build Tools
# Required components:
# - MSVC v143 - VS 2022 C++ x64/x86 build tools
# - Windows 11 SDK (10.0.22621.0 or later)
# - C++ CMake tools for Windows

# 3. Install CMake
# Download from https://cmake.org/download/
# Or install via Visual Studio Installer

# 4. Verify installation (in a new terminal)
rustc --version
cargo --version
```

### Step 2: First Build

```bash
# Clone the repository
git clone <your-repo-url>
cd <your-project>

# Build in debug mode (faster compilation, slower runtime)
cargo build

# Or build in release mode (slower compilation, faster runtime)
cargo build --release

# Run the application
cargo run
```

### Step 3: Run Tests

```bash
# Run all tests
cargo test --workspace

# Run tests with output
cargo test --workspace -- --nocapture

# Run specific test
cargo test test_name
```

## Project Structure Template

For a new cross-platform Rust project, use this structure:

```
my-project/
├── Cargo.toml                 # Workspace root
├── crates/
│   ├── my-app/               # Main application
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   └── main.rs
│   │   └── resources/
│   │       └── icons/
│   └── my-lib/               # Shared library (optional)
│       ├── Cargo.toml
│       └── src/
│           └── lib.rs
├── script/
│   ├── bundle-linux          # Linux packaging script
│   ├── bundle-windows.ps1    # Windows packaging script
│   └── generate-licenses     # License generation
├── .github/
│   └── workflows/
│       └── build.yml         # CI/CD workflow
├── Cargo.lock
└── README.md
```

### Workspace Cargo.toml Template

```toml
[workspace]
resolver = "2"
members = ["crates/*"]

[workspace.package]
edition = "2021"
version = "0.1.0"
authors = ["Your Name <you@example.com>"]
license = "MIT OR Apache-2.0"
rust-version = "1.75"

[workspace.dependencies]
# Add shared dependencies here
anyhow = "1.0"
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1.0", features = ["full"] }
```

### Application Cargo.toml Template

```toml
[package]
name = "my-app"
version.workspace = true
edition.workspace = true
authors.workspace = true
license.workspace = true

[[bin]]
name = "my-app"
path = "src/main.rs"

[dependencies]
anyhow.workspace = true
serde.workspace = true
tokio.workspace = true

# macOS bundle configuration
[package.metadata.bundle]
name = "MyApp"
identifier = "com.yourcompany.myapp"
icon = ["resources/icons/32x32.png", "resources/icons/128x128.png", "resources/icons/icon.icns"]
version = "0.1.0"
copyright = "Copyright (c) 2024"
category = "Developer Tool"
short_description = "Your app description"
osx_minimum_system_version = "10.11"
```

## Supported Platforms

| Platform | Architectures | Output Format |
|----------|--------------|---------------|
| macOS | aarch64 (Apple Silicon), x86_64 (Intel) | .dmg, .app bundle |
| Linux | x86_64, aarch64 | .tar.gz |
| Windows | x86_64, aarch64 | .exe (Inno Setup) |

## GitHub Actions Workflow Template

### Complete Multi-Platform Build Workflow

```yaml
name: Build All Platforms

on:
  workflow_dispatch:
    inputs:
      release_channel:
        description: 'Release channel (stable/preview/nightly/dev)'
        required: true
        default: 'stable'
        type: choice
        options:
          - stable
          - preview
          - nightly
          - dev
  push:
    tags:
      - 'v*'

env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: '1'
  CARGO_INCREMENTAL: 0

jobs:
  # ==================== macOS aarch64 (Apple Silicon) ====================
  bundle_mac_aarch64:
    runs-on: macos-latest
    steps:
      - name: Checkout repository
        uses: actions/checkout@v4

      - name: Setup Rust
        uses: dtolnay/rust-action@stable
        with:
          targets: aarch64-apple-darwin

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'

      - name: Install cargo-bundle
        run: |
          cargo install cargo-bundle --git https://github.com/zed-industries/cargo-bundle.git --branch zed-deploy

      - name: Set release channel
        run: |
          channel="${{ github.event.inputs.release_channel || 'stable' }}"
          echo "$channel" > RELEASE_CHANNEL
          echo "RELEASE_CHANNEL=$channel" >> $GITHUB_ENV

      - name: Build (aarch64)
        run: |
          export BUNDLE=true
          export CXXFLAGS="-stdlib=libc++"
          cargo build --release --target aarch64-apple-darwin

      - name: Create app bundle
        run: |
          app_path=$(cargo bundle --release --target aarch64-apple-darwin | xargs)
          echo "Bundled $app_path"

      - name: Create DMG
        run: |
          mkdir -p target/aarch64-apple-darwin/release/dmg
          app_path=$(find target/aarch64-apple-darwin/release -name "*.app" -type d | head -n 1)
          cp -r "$app_path" target/aarch64-apple-darwin/release/dmg/
          ln -s /Applications target/aarch64-apple-darwin/release/dmg/
          hdiutil create -volname YourApp -srcfolder target/aarch64-apple-darwin/release/dmg -ov -format UDZO target/aarch64-apple-darwin/release/YourApp-aarch64.dmg

      - name: Upload artifacts
        uses: actions/upload-artifact@v4
        with:
          name: macos-aarch64
          path: target/aarch64-apple-darwin/release/YourApp-aarch64.dmg

  # ==================== macOS x86_64 (Intel) ====================
  bundle_mac_x86_64:
    runs-on: macos-13  # Intel runner
    steps:
      - name: Checkout repository
        uses: actions/checkout@v4

      - name: Setup Rust
        uses: dtolnay/rust-action@stable
        with:
          targets: x86_64-apple-darwin

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'

      - name: Install cargo-bundle
        run: |
          cargo install cargo-bundle --git https://github.com/zed-industries/cargo-bundle.git --branch zed-deploy

      - name: Set release channel
        run: |
          channel="${{ github.event.inputs.release_channel || 'stable' }}"
          echo "$channel" > RELEASE_CHANNEL
          echo "RELEASE_CHANNEL=$channel" >> $GITHUB_ENV

      - name: Build (x86_64)
        run: |
          export BUNDLE=true
          export CXXFLAGS="-stdlib=libc++"
          cargo build --release --target x86_64-apple-darwin

      - name: Create app bundle
        run: |
          app_path=$(cargo bundle --release --target x86_64-apple-darwin | xargs)
          echo "Bundled $app_path"

      - name: Create DMG
        run: |
          mkdir -p target/x86_64-apple-darwin/release/dmg
          app_path=$(find target/x86_64-apple-darwin/release -name "*.app" -type d | head -n 1)
          cp -r "$app_path" target/x86_64-apple-darwin/release/dmg/
          ln -s /Applications target/x86_64-apple-darwin/release/dmg/
          hdiutil create -volname YourApp -srcfolder target/x86_64-apple-darwin/release/dmg -ov -format UDZO target/x86_64-apple-darwin/release/YourApp-x86_64.dmg

      - name: Upload artifacts
        uses: actions/upload-artifact@v4
        with:
          name: macos-x86_64
          path: target/x86_64-apple-darwin/release/YourApp-x86_64.dmg

  # ==================== Linux x86_64 ====================
  bundle_linux_x86_64:
    runs-on: ubuntu-22.04
    env:
      CC: clang
      CXX: clang++
    steps:
      - name: Checkout repository
        uses: actions/checkout@v4

      - name: Setup Rust
        uses: dtolnay/rust-action@stable

      - name: Install dependencies
        run: |
          sudo apt-get update
          sudo apt-get install -y \
            libasound2-dev \
            libfontconfig1-dev \
            libwayland-dev \
            libxkbcommon-x11-dev \
            libssl-dev \
            libzstd-dev \
            clang \
            mold \
            curl

      - name: Set release channel
        run: |
          channel="${{ github.event.inputs.release_channel || 'stable' }}"
          echo "$channel" > RELEASE_CHANNEL

      - name: Build
        run: |
          export BUNDLE=true
          export RUSTFLAGS="-C link-args=-Wl,--disable-new-dtags,-rpath,\$ORIGIN/../lib"
          cargo build --release

      - name: Create tarball
        run: |
          mkdir -p target/release/bundle
          tar -czf target/release/YourApp-linux-x86_64.tar.gz -C target/release your-binary-name

      - name: Upload artifacts
        uses: actions/upload-artifact@v4
        with:
          name: linux-x86_64
          path: target/release/YourApp-linux-x86_64.tar.gz

  # ==================== Windows x86_64 ====================
  bundle_windows_x86_64:
    runs-on: windows-latest
    steps:
      - name: Checkout repository
        uses: actions/checkout@v4

      - name: Setup Rust
        uses: dtolnay/rust-action@stable
        with:
          targets: x86_64-pc-windows-msvc

      - name: Setup Visual Studio
        uses: microsoft/setup-msbuild@v2

      - name: Set release channel
        shell: pwsh
        run: |
          $channel = "${{ github.event.inputs.release_channel || 'stable' }}"
          $channel | Set-Content -Path "RELEASE_CHANNEL"
          "RELEASE_CHANNEL=$channel" | Out-File -FilePath $env:GITHUB_ENV -Append

      - name: Build
        shell: pwsh
        run: |
          $env:BUNDLE = "true"
          $target = "x86_64-pc-windows-msvc"
          rustup target add $target
          cargo build --release --target $target

      - name: Create installer (Inno Setup)
        shell: pwsh
        run: |
          # Download and install Inno Setup
          $url = "https://files.jrsoftware.org/is/6/innosetup-6.2.2.exe"
          Invoke-WebRequest -Uri $url -OutFile "innosetup.exe"
          Start-Process -FilePath "innosetup.exe" -ArgumentList "/SILENT", "/NORESTART" -Wait
          
          # Create installer using your .iss script
          & "C:\Program Files (x86)\Inno Setup 6\ISCC.exe" your-installer.iss

      - name: Upload artifacts
        uses: actions/upload-artifact@v4
        with:
          name: windows-x86_64
          path: target/YourApp-x86_64.exe
```

## Cargo.toml Bundle Configuration

Add to your `Cargo.toml` for macOS app bundling:

```toml
[package.metadata.bundle]
name = "YourApp"
identifier = "com.yourcompany.yourapp"
icon = ["icons/32x32.png", "icons/128x128.png", "icons/icon.icns"]
version = "1.0.0"
resources = ["assets", "config"]
copyright = "Copyright (c) 2024 Your Company"
category = "Developer Tool"
short_description = "Your app description"
long_description = """
Longer description of your application.
"""
```

## Key Patterns

### 1. Release Channel Management
```bash
# Set channel (stable/preview/nightly/dev)
echo "stable" > RELEASE_CHANNEL
```

### 2. Platform-Specific Build Flags
```bash
# macOS
export CXXFLAGS="-stdlib=libc++"

# Linux
export RUSTFLAGS="-C link-args=-Wl,--disable-new-dtags,-rpath,\$ORIGIN/../lib"

# Windows (PowerShell)
$env:BUNDLE = "true"
```

### 3. Cross-Compilation Setup
```bash
# Add target
rustup target add aarch64-apple-darwin
rustup target add x86_64-pc-windows-msvc
rustup target add aarch64-unknown-linux-gnu
```

## Best Practices

1. **Use sccache for faster builds**
   ```yaml
   - name: Setup sccache
     run: |
       cargo install sccache
       echo "RUSTC_WRAPPER=sccache" >> $GITHUB_ENV
   ```

2. **Cache dependencies**
   ```yaml
   - uses: Swatinem/rust-cache@v2
   ```

3. **Use mold linker on Linux for faster linking**
   ```bash
   sudo apt-get install mold
   export RUSTFLAGS="-C link-arg=-fuse-ld=mold"
   ```

4. **Strip binaries for smaller size**
   ```bash
   strip target/release/your-binary
   ```

## Local Development & Debugging

### Development Workflow

```bash
# 1. Check code formatting
cargo fmt --check

# 2. Run linter
cargo clippy --workspace

# 3. Run tests
cargo test --workspace

# 4. Build and run
cargo run

# 5. Build release version
cargo build --release
```

### Useful Cargo Commands

```bash
# Check without building (fast)
cargo check

# Check all targets
cargo check --all-targets

# Build specific package
cargo build -p my-app

# Run with specific features
cargo run --features feature-name

# Clean build artifacts
cargo clean

# Update dependencies
cargo update

# Generate documentation
cargo doc --open
```

### IDE Setup (VS Code / Trae)

Create `.vscode/settings.json`:
```json
{
  "rust-analyzer.cargo.features": "all",
  "rust-analyzer.checkOnSave.command": "clippy",
  "rust-analyzer.checkOnSave.allTargets": false,
  "editor.formatOnSave": true,
  "[rust]": {
    "editor.defaultFormatter": "rust-lang.rust-analyzer"
  }
}
```

Recommended extensions:
- rust-analyzer (Rust language support)
- CodeLLDB (Debugging)
- Better TOML (Cargo.toml editing)

## Troubleshooting

### Common Build Errors

#### 1. "linker `cc` not found" (Linux)
```bash
# Install build essentials
sudo apt-get install build-essential
```

#### 2. "could not find system library" (Linux)
```bash
# Install common dependencies
sudo apt-get install libssl-dev pkg-config
```

#### 3. "Windows SDK not found" (Windows)
- Install Windows SDK via Visual Studio Installer
- Or download from: https://developer.microsoft.com/windows/downloads/windows-sdk/

#### 4. "Xcode not found" (macOS)
```bash
# Reset Xcode path
sudo xcode-select --reset
# Or set manually
sudo xcode-select --switch /Applications/Xcode.app/Contents/Developer
```

#### 5. AWS-LC Linking Errors (GCC >= 14)
If you see errors about `__isoc23_sscanf` or `__isoc23_strtol`:
```bash
# Use clang instead of gcc
export CC=clang
export CXX=clang++
```

### macOS Code Signing (Optional)
For unsigned local builds:
```bash
codesign --force --deep --sign - target/release/bundle/osx/YourApp.app
```

### Windows DLL Dependencies
Use `dumpbin` or `Dependencies` tool to check required DLLs:
```powershell
dumpbin /dependents target\release\your-binary.exe
```

### Linux Runtime Dependencies
Use `ldd` to check shared library dependencies:
```bash
ldd target/release/your-binary
```

### Build Performance Issues

#### Slow Linking on Linux
```bash
# Install and use Mold linker
sudo apt-get install mold

# Add to ~/.cargo/config.toml
[target.x86_64-unknown-linux-gnu]
rustflags = ["-C", "link-arg=-fuse-ld=mold"]
```

#### Out of Memory During Build
```bash
# Reduce parallel jobs
cargo build --release -j 2

# Or set environment variable
export CARGO_BUILD_JOBS=2
```

### GitHub Actions Issues

#### 1. Workflow Not Triggering
- Check if workflow file is in `.github/workflows/`
- Verify YAML syntax is valid
- Check branch filters in `on:` section

#### 2. Build Timeout
```yaml
# Increase timeout
jobs:
  build:
    timeout-minutes: 120  # Default is 360
```

#### 3. Artifact Upload Fails
```yaml
# Use v4 of upload-artifact
- uses: actions/upload-artifact@v4
  with:
    name: my-artifact
    path: target/release/my-binary
```

## Next Steps

After setting up your build system:

1. **Add Tests**: Write unit and integration tests
2. **Documentation**: Generate docs with `cargo doc`
3. **Release Process**: Automate versioning and changelogs
4. **Code Signing**: Set up certificates for production releases
5. **Update System**: Implement auto-updater (like Zed's)
