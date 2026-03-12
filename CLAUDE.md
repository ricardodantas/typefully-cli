# CLAUDE.md

Claude Code instructions for the typefully-cli project.

## Read First

Read `AGENTS.md` for full project structure, conventions, and patterns.

## Key Rules

1. **Rust 2024 edition.** Do not use deprecated syntax or pre-2024 patterns.
2. **Async only.** All API interactions use tokio async/await. Never use `reqwest::blocking`.
3. **thiserror for errors.** Do not use `anyhow` or manual `impl Display` on error types.
4. **tracing for logging.** Use `tracing::debug!`, `tracing::info!`, etc. Never `println!` for diagnostics.
5. **secrecy for API keys.** Wrap in `SecretString`. Only call `expose_secret()` at the HTTP auth boundary.
6. **Trait-based API client.** Command handlers accept `&impl TypefullyApi`, not concrete `TypefullyClient`.
7. **Clippy clean.** Run `cargo clippy -- -D warnings` before every commit. Zero warnings required.
8. **No em dashes.** In any text content (docs, comments, help strings, README), never use em dashes. Use colons, commas, periods, or parentheses.

## Before Committing

```bash
cargo check
cargo clippy -- -D warnings
cargo build --release
```

All three must pass without errors or warnings.

## Module Guide

| Module | Purpose |
|--------|---------|
| `src/api/mod.rs` | `TypefullyApi` trait definition |
| `src/api/client.rs` | HTTP client implementation |
| `src/api/types.rs` | Platform, DraftStatus, DraftListParams |
| `src/cli/mod.rs` | Clap argument definitions |
| `src/config/mod.rs` | Config file + key/set resolution |
| `src/output/mod.rs` | Tables, JSON output, formatting |
| `src/error.rs` | AppError, ApiError, ConfigError |
| `src/main.rs` | Tokio entrypoint, command dispatch |
| `src/lib.rs` | Public re-exports |

## Patterns to Follow

- Add `#[must_use]` on pure public functions
- Add doc comments on all public items
- Use `#[non_exhaustive]` on public enums
- Derive `Debug`, `Clone` on structs where sensible
- Prefer `&str` over `String` in function signatures where ownership is not needed
- Use `Option<&str>` over `Option<String>` when borrowing is sufficient
