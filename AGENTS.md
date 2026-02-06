# Zero-Copy Wayland Wallpaper (Rust)

A high-performance Wayland wallpaper renderer using Rust, GStreamer, and WGPU.
The project strictly enforces hardware-accelerated paths (DMA-BUF) without
software fallbacks.

## Project Structure

Codebase is organized strictly by function. Do not deviate from the module structure.

    src/
    ├──main.rs — Entry point and glue logic only.
    ├──cli/ — Argument parsing using clap.
    ├──decode/ — GStreamer pipeline management and DMABUF extraction.
    ├──renderer/ — WGPU instance, HAL imports, and rendering logic.
    ├──transitions/ — Visual effects and shader transitions.
    └──utils/ — Shared utility code, math, and type conversions.

## Commands

- `cargo build` — Build the project (debug)
- `cargo build --release` — Build optimized release binary
- `cargo check` — Fast check without building artifacts
- `cargo test` — Run all tests
- `cargo test <test_name>` — Run specific test by name
- `cargo test -- --list` — List all tests
- `cargo clippy` — Run linter for additional checks
- `cargo clippy --fix` — Auto-apply lint suggestions
- `cargo fmt` — Format all source files
- `cargo fmt --check` — Check formatting without changes

## Code Style

### Formatting

- Use **hard tabs** for indentation (configured in rustfmt.toml)
- Edition 2021 Rust
- Max line width 100 chars default

### Type Conversions

Use external traits to extend library types. Implement TryFrom with W<T> wrapper:

```rs
use crate::prelude::*;

impl TryFrom<W<&DirEntry>> for String {
    type Error = Error;
    fn try_from(val: W<&DirEntry>) -> Result<String> {
        val.0.path().to_str().map(String::from)
            .ok_or_else(|| Error::Generic(f!("Invalid path {:?}", val.0)))
    }
}
```

### Prelude Pattern

Every module imports from crate prelude: `use crate::prelude::*;`
The prelude provides: custom Error, Result<T>, W<T> wrapper, and `f` alias.

### Error Handling

Use `thiserror` for custom errors. Define errors in src/error.rs, export via prelude:

```rs
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error("Generic error: {0}")]
    Generic(String),
}
```

### Logging

Never use `println!`/`eprintln!`. Use `log` crate with `env_logger` initialization.

### Imports

- Group imports: std, external crates, then crate modules
- Use prelude for common crate types
- Prefer specific imports over glob imports (except prelude)

### Naming

- Modules: snake_case
- Types: PascalCase
- Functions/methods: snake_case
- Constants: SCREAMING_SNAKE_CASE

## Core Constraints

1. No Software Fallbacks
   GStreamer: Caps must enforce memory:DMABUF and video/x-raw
   WGPU: Never use queue.write_texture for video frames
   Fail fast with descriptive errors, no silent fallbacks

2. Modularity
   Shared code in utils.rs only. Pass raw handles (DMABUF FDs) between modules.
   Decoder unaware of renderer; decoupling enforced.

3. GStreamer Caps: `video/x-raw(memory:DMABUF),format=NV12` (native format preferred)

## CLI Commands

Break clap commands into modules under cli/. Implement AllwallCommand trait:

```rs
pub trait AllwallCommand {
    fn execute(&self) -> Result<()>;
}
```
