# AGENTS.md

Instructions for AI coding agents working on the typefully-cli project.

## Project Overview

A full-featured CLI client for the [Typefully API v2](https://typefully.com), written in Rust. Lets users create, schedule, manage, and publish social media drafts from the terminal.

## Project Structure

```
typefully-cli/
  src/
    main.rs          # Entry point, command dispatch, tokio runtime
    lib.rs           # Public module re-exports
    api/
      mod.rs         # TypefullyApi trait definition + re-exports
      client.rs      # TypefullyClient (concrete HTTP implementation)
      types.rs       # Shared types: Platform, DraftStatus, DraftListParams
    cli/
      mod.rs         # Clap derive structs (Cli, Commands, DraftCmd, etc.)
    config/
      mod.rs         # Config file loading/saving, API key + set ID resolution
    output/
      mod.rs         # Table rows, JSON output, formatting helpers
    error.rs         # Error hierarchy: AppError, ApiError, ConfigError
  .github/
    workflows/
      ci.yml         # PR checks: test, clippy, fmt
      release.yml    # Version bump, cross-compile, GitHub Release, crates.io, Homebrew
  Cargo.toml
  rustfmt.toml
  LICENSE            # GPL-3.0-only
  README.md
```

## Conventions

### Rust Edition and Toolchain
- **Edition:** 2024
- **MSRV:** 1.91
- **Lints:** `unsafe_code = "forbid"`, `clippy::all = "deny"`, `clippy::pedantic = "warn"`

### Async Runtime
- Use `tokio` with the `full` feature set
- All API calls are async. The `#[tokio::main]` macro is in `main.rs`
- Never use blocking reqwest

### Error Handling
- `thiserror` for all error type derivations
- Three error enums: `AppError` (top level), `ApiError` (HTTP/API layer), `ConfigError` (file/parse)
- All errors implement `Display` via thiserror
- Exit codes: 0 = success, 1 = API/runtime error, 2 = invalid usage

### Logging
- Use `tracing` macros (`debug!`, `info!`, `warn!`) instead of `eprintln!` for diagnostics
- `tracing-subscriber` with env-filter, writing to stderr
- `--verbose` sets the filter to `debug`; default is `warn`

### Secrets
- API keys are wrapped in `secrecy::SecretString`
- Never log or display API keys. Use `ExposeSecret` only at the point of HTTP auth

### API Client Pattern
- `TypefullyApi` trait in `src/api/mod.rs` defines all API operations
- `TypefullyClient` in `src/api/client.rs` is the concrete implementation
- Command handlers accept `&impl TypefullyApi` so they are testable with mocks
- `mockall` is available in dev-dependencies

### CLI Arguments
- `clap` with derive macros
- Global flags: `--json`, `--no-color`, `-q`/`--quiet`, `-v`/`--verbose`, `--api-key`
- Social set ID: `--set` flag, falls back to config default

### Output
- Human-friendly tables via `tabled` crate (default)
- `--json` flag outputs raw JSON to stdout
- Success/error messages go to stderr
- Respect `NO_COLOR` env var

### Config
- Config file at `~/.config/typefully/config.toml` (via `dirs` crate for XDG)
- Fields: `api_key`, `default_social_set_id`
- Key resolution order: `--api-key` flag > `TYPEFULLY_API_KEY` env > config file

### Branch
- Default branch: `main` (not master)
- All workflows and references use `main`

## Building and Testing

```bash
# Check compilation
cargo check

# Run clippy (must pass with zero warnings)
cargo clippy -- -D warnings

# Build release
cargo build --release

# Run
cargo run -- --help
```

## Writing Style

- Never use em dashes in any text (README, help text, comments, doc comments)
- Use colons, commas, periods, or parentheses instead
- Keep help text concise

## Adding a New Subcommand

1. Add the clap struct/variant in `src/cli/mod.rs`
2. Add the API trait method in `src/api/mod.rs`
3. Implement it in `src/api/client.rs`
4. Add a command handler function in `src/main.rs`
5. Wire it into the `run()` match in `src/main.rs`
6. Run `cargo clippy -- -D warnings` before committing

## Adding a New API Type

1. Define it in `src/api/types.rs`
2. Re-export it via `pub use types::*` in `src/api/mod.rs`
3. Add `#[non_exhaustive]` on public enums
4. Derive `Debug`, `Clone`, and implement `Display` where appropriate
