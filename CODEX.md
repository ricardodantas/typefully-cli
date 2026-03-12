# CODEX.md

OpenAI Codex instructions for the typefully-cli project.

## Read First

Read `AGENTS.md` for full project structure, conventions, and patterns.

## Quick Reference

- **Language:** Rust, edition 2024, MSRV 1.91
- **Async runtime:** tokio (full features)
- **Error handling:** thiserror (AppError, ApiError, ConfigError)
- **Logging:** tracing + tracing-subscriber (not println)
- **Secrets:** secrecy crate for API keys
- **HTTP:** reqwest (async, not blocking)
- **CLI parsing:** clap with derive macros
- **Output:** tabled for tables, serde_json for JSON mode
- **Config:** toml format at `~/.config/typefully/config.toml`

## Mandatory Checks

Before any commit, all of these must pass:

```bash
cargo check
cargo clippy -- -D warnings
cargo build --release
```

## Architecture

The API client is trait-based (`TypefullyApi`) for testability. Command handlers accept `&impl TypefullyApi`. The concrete implementation is `TypefullyClient`.

Error types form a hierarchy: `AppError` wraps `ApiError` and `ConfigError` via `#[from]`. All use thiserror derive.

API keys are never stored as plain `String` in runtime structs. They use `secrecy::SecretString` and are only exposed at the HTTP authorization point.

## Style

- No em dashes anywhere (use colons, commas, periods, parentheses)
- `#[must_use]` on pure public functions
- Doc comments on all public items
- `#[non_exhaustive]` on public enums
- Clippy pedantic warnings acknowledged (with targeted allows where needed)
