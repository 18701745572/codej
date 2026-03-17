---
name: "zed-workflow-reuse"
description: "Reuse Zed editor's development workflow patterns including GPUI cross-platform setup, CI/CD best practices, and Rust project structure. Invoke when setting up new Rust projects inspired by Zed's architecture."
---

# Zed Workflow Reuse Skill

This skill captures Zed editor's development patterns and best practices for reuse in your own projects.

## Project Architecture Patterns

### 1. Workspace Structure

Zed uses a Cargo workspace with multiple crates:

```
project-root/
├── Cargo.toml              # Workspace definition
├── crates/
│   ├── app/                # Main application crate
│   ├── ui/                 # UI framework (like GPUI)
│   ├── platform/           # Platform abstraction
│   ├── platform_macos/     # macOS implementation
│   ├── platform_linux/     # Linux implementation
│   ├── platform_windows/   # Windows implementation
│   ├── editor/             # Editor core
│   ├── project/            # Project management
│   └── ...
├── script/                 # Build scripts
│   ├── bundle-linux
│   ├── bundle-windows.ps1
│   └── generate-licenses
└── .github/workflows/      # CI/CD workflows
```

### 2. Workspace Cargo.toml

```toml
[workspace]
resolver = "2"
members = [
    "crates/app",
    "crates/ui",
    "crates/platform",
    "crates/platform_macos",
    "crates/platform_linux",
    "crates/platform_windows",
]

[workspace.package]
edition = "2021"
version = "0.1.0"
authors = ["Your Name <email@example.com>"]
license = "MIT OR Apache-2.0"

[workspace.dependencies]
# Common dependencies
gpui = { path = "crates/gpui" }
anyhow = "1.0"
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1.0", features = ["full"] }
```

### 3. Platform Abstraction Pattern

Zed uses conditional compilation for platform-specific code:

**In `platform/Cargo.toml`:**
```toml
[target.'cfg(target_os = "macos")'.dependencies]
gpui_macos = { path = "../gpui_macos" }

[target.'cfg(target_os = "windows")'.dependencies]
gpui_windows = { path = "../gpui_windows" }

[target.'cfg(any(target_os = "linux", target_os = "freebsd"))'.dependencies]
gpui_linux = { path = "../gpui_linux" }
```

**In `platform/src/lib.rs`:**
```rust
#[cfg(target_os = "macos")]
pub use gpui_macos::*;

#[cfg(target_os = "windows")]
pub use gpui_windows::*;

#[cfg(any(target_os = "linux", target_os = "freebsd"))]
pub use gpui_linux::*;
```

## CI/CD Best Practices from Zed

### 1. Test Workflow Structure

```yaml
name: Run Tests

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: '1'

jobs:
  test_macos:
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-action@stable
      - uses: taiki-e/install-action@nextest
      - run: cargo nextest run --workspace

  test_linux:
    runs-on: ubuntu-22.04
    env:
      CC: clang
      CXX: clang++
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-action@stable
      - name: Install dependencies
        run: |
          sudo apt-get update
          sudo apt-get install -y libwayland-dev libxkbcommon-dev clang
      - uses: taiki-e/install-action@nextest
      - run: cargo nextest run --workspace

  test_windows:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-action@stable
      - uses: taiki-e/install-action@nextest
      - run: cargo nextest run --workspace
```

### 2. Build Optimization with sccache

```yaml
- name: Setup sccache
  run: |
    cargo install sccache --locked
    echo "RUSTC_WRAPPER=sccache" >> $GITHUB_ENV
    echo "SCCACHE_GHA_ENABLED=true" >> $GITHUB_ENV
  env:
    SCCACHE_GHA_ENABLED: "true"
```

### 3. Rust Cache Configuration

Create `.cargo/config.toml`:

```toml
[build]
rustflags = ["-C", "target-cpu=native"]

[profile.release]
opt-level = 3
lto = "thin"
codegen-units = 1
strip = true

[profile.dev]
opt-level = 0
debug = true
```

## GPUI-Inspired UI Framework Setup

### 1. Core UI Framework Structure

```rust
// ui/src/lib.rs
pub mod app;
pub mod window;
pub mod element;
pub mod style;

pub use app::App;
pub use window::Window;
```

### 2. Platform-Specific Window Implementation

**macOS (`platform_macos/src/window.rs`):**
```rust
use cocoa::base::id;
use cocoa::appkit::NSWindow;

pub struct PlatformWindow {
    native_window: id,
}

impl PlatformWindow {
    pub fn new(title: &str, width: u32, height: u32) -> Self {
        // NSWindow creation code
    }
}
```

**Linux (`platform_linux/src/window.rs`):**
```rust
use wayland_client::Display;

pub struct PlatformWindow {
    display: Display,
    surface: /* wayland surface */,
}
```

**Windows (`platform_windows/src/window.rs`):**
```rust
use windows::Win32::Foundation::HWND;

pub struct PlatformWindow {
    hwnd: HWND,
}
```

### 3. GPU Rendering Setup with wgpu

```rust
// ui/src/renderer.rs
use wgpu::{Device, Queue, Surface};

pub struct Renderer {
    device: Device,
    queue: Queue,
    surface: Surface,
}

impl Renderer {
    pub async fn new(window: &impl HasRawWindowHandle) -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        
        let surface = unsafe { instance.create_surface(window) }.unwrap();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();
        
        // Create device and queue...
    }
}
```

## Build Scripts

### 1. Linux Bundle Script

```bash
#!/usr/bin/env bash
# script/bundle-linux
set -euxo pipefail

export BUNDLE=true
channel=$(<RELEASE_CHANNEL)
version=$(cargo pkgid | cut -d# -f2 | cut -d: -f2)

target_dir="target/release"
mkdir -p "${target_dir}/bundle"

# Build
cargo build --release

# Create tarball
tar -czf "${target_dir}/bundle/your-app-${version}-linux-x86_64.tar.gz" \
  -C "${target_dir}" \
  your-binary \
  --transform 's|^|your-app/|'
```

### 2. Windows Bundle Script (PowerShell)

```powershell
# script/bundle-windows.ps1
param(
    [string]$Architecture = "x86_64"
)

$ErrorActionPreference = 'Stop'

$target = "$Architecture-pc-windows-msvc"
$CargoOutDir = "./target/$target/release"

# Build
rustup target add $target
cargo build --release --target $target

# Create installer with Inno Setup
& "C:\Program Files (x86)\Inno Setup 6\ISCC.exe" `
  /DAppVersion=$version `
  /DOutputDir=$pwd\target `
  installer.iss
```

## Development Tools Setup

### 1. Recommended VS Code/Trae Settings

`.vscode/settings.json`:
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

### 2. Clippy Configuration

`clippy.toml`:
```toml
avoid-breaking-exported-api = false
msrv = "1.75"
```

### 3. GitHub Actions Reusable Workflows

`.github/workflows/build-reusable.yml`:
```yaml
name: Reusable Build

on:
  workflow_call:
    inputs:
      platform:
        required: true
        type: string
      target:
        required: true
        type: string

jobs:
  build:
    runs-on: ${{ inputs.platform }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-action@stable
        with:
          targets: ${{ inputs.target }}
      - run: cargo build --release --target ${{ inputs.target }}
```

## Key Takeaways

1. **Modular Architecture**: Separate platform-specific code into different crates
2. **Conditional Compilation**: Use `#[cfg(target_os = "...")]` for platform differences
3. **Workspace Organization**: Group related functionality into workspace members
4. **CI/CD Parallelism**: Build each platform in parallel jobs
5. **Caching**: Use sccache and GitHub Actions cache for faster builds
6. **Cross-Compilation**: Use rustup targets for building on different architectures
