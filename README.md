<p align="center">
  <h1 align="center">typefully</h1>
  <p align="center">A powerful CLI client for the Typefully API v2. Create, schedule, and manage your social media content from the terminal.</p>
</p>

<p align="center">
  <a href="https://github.com/ricardodantas/typefully-cli/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-GPL--3.0-blue.svg" alt="License: GPL-3.0"></a>
  <a href="https://crates.io/crates/typefully"><img src="https://img.shields.io/crates/v/typefully.svg" alt="crates.io"></a>
  <a href="https://github.com/ricardodantas/typefully-cli/actions"><img src="https://img.shields.io/github/actions/workflow/status/ricardodantas/typefully-cli/ci.yml?branch=main" alt="CI"></a>
  <img src="https://img.shields.io/badge/rust-1.91%2B-orange.svg" alt="Rust 1.91+">
</p>

---

## ✨ Features

- 📝 **Create drafts** with rich content, tags, and media attachments
- 🧵 **Thread support** with simple `---` separators
- 📅 **Schedule posts** for specific times or the next free slot
- 🌐 **Multi-platform** publishing (X, LinkedIn, Threads, Bluesky, Mastodon)
- 🖼️ **Media uploads** with automatic S3 presigned URL handling
- 🏷️ **Tag management** for organizing your content
- 📊 **Flexible output** with human-friendly tables or JSON
- 🔐 **Secure** API key handling (never logged or displayed)
- ⚡ **Fast** async runtime powered by tokio

## 📦 Installation

### Homebrew

```bash
brew tap ricardodantas/tap
brew install typefully
```

### From crates.io

```bash
cargo install typefully
```

### From source

```bash
git clone https://github.com/ricardodantas/typefully-cli.git
cd typefully-cli
cargo install --path .
```

## 🚀 Quick Start

```bash
# 1. Set up your API key and default social set
typefully config init

# 2. Verify authentication
typefully auth

# 3. Create your first draft
typefully draft create --content "Hello from the terminal! 🚀"

# 4. Publish it immediately
typefully draft create --content "Going live now!" --publish-at now
```

## ⚙️ Configuration

### Interactive Setup

The easiest way to get started:

```bash
typefully config init
```

This prompts for your API key and lets you pick a default social set from your account.

### Manual Setup

Create `~/.config/typefully/config.toml`:

```toml
api_key = "your-api-key-here"
default_social_set_id = "your-social-set-id"
```

### API Key Precedence

The CLI looks for your API key in this order:

1. `--api-key` command-line flag
2. `TYPEFULLY_API_KEY` environment variable
3. `~/.config/typefully/config.toml` file

## 📖 Command Reference

### Authentication

Verify your API key and display account information:

```bash
typefully auth
typefully auth --json
```

### Social Sets

List all social sets with their connected platforms:

```bash
typefully sets
typefully sets --json
```

### Drafts

#### Create a Draft

```bash
# Simple draft (saved as draft, not published)
typefully draft create --content "Hello world!"

# Pipe content from stdin
echo "Hello from a pipe" | typefully draft create

# Publish immediately
typefully draft create --content "Live now!" --publish-at now

# Schedule for the next free slot
typefully draft create --content "Queued up" --publish-at next-free-slot

# Schedule for a specific time (ISO-8601)
typefully draft create --content "Timed post" --publish-at "2025-06-15T10:00:00Z"

# Add tags
typefully draft create --content "Product update" --tag launch --tag product

# Attach uploaded media
typefully draft create --content "Check this out!" --media media_abc123

# Target specific platforms
typefully draft create --content "Hello LinkedIn!" --platform linkedin

# Post to multiple platforms
typefully draft create --content "Everywhere at once" --platform x --platform linkedin --platform threads

# Use a specific social set
typefully draft create --set set_abc123 --content "Hello!"
```

#### Create a Thread

Split your content into multiple posts using `---` on its own line:

```bash
typefully draft create --content "1/ Here's a thread about something interesting.
---
2/ The second post continues the thought.
---
3/ And the conclusion wraps it all up."
```

You can also pipe a thread from a file:

```bash
cat my-thread.txt | typefully draft create
```

Where `my-thread.txt` contains:

```
First post of the thread.
---
Second post with more detail.
---
Final post with a call to action!
```

#### List Drafts

```bash
# List recent drafts (default: 20)
typefully draft list

# Filter by status
typefully draft list --status scheduled
typefully draft list --status published
typefully draft list --status draft

# Filter by tag
typefully draft list --tag marketing

# Custom sorting
typefully draft list --sort scheduled_date
typefully draft list --sort updated_at

# Pagination
typefully draft list --limit 10 --offset 20

# Combine filters
typefully draft list --status scheduled --tag launch --sort scheduled_date --limit 5
```

#### Get a Draft

```bash
typefully draft get abc123
typefully draft get abc123 --json
```

#### Edit a Draft

```bash
# Update content
typefully draft edit abc123 --content "Updated content here"

# Reschedule
typefully draft edit abc123 --publish-at "2025-07-01T09:00:00Z"

# Update tags
typefully draft edit abc123 --tag new-tag --tag another-tag

# Toggle sharing
typefully draft edit abc123 --share true
```

#### Delete a Draft

```bash
# With confirmation prompt
typefully draft delete abc123

# Skip confirmation
typefully draft delete abc123 --force
```

#### Publish Immediately

A shortcut to publish a draft right now:

```bash
typefully draft publish abc123
```

#### Schedule a Draft

```bash
# Schedule for the next free slot
typefully draft schedule abc123 next-free-slot

# Schedule for a specific time
typefully draft schedule abc123 "2025-06-20T14:00:00Z"
```

### Media

#### Upload Workflow

Media uploads use a three-step process (handled automatically by the CLI):

1. Request a presigned upload URL from Typefully
2. Upload the file to S3
3. Poll until processing is complete

```bash
# Upload an image
typefully media upload photo.jpg

# Upload and get the media ID as JSON (useful for scripting)
typefully media upload banner.png --json

# Then attach it to a draft
typefully draft create --content "New banner!" --media <media_id_from_above>
```

Supported formats: PNG, JPEG, GIF, WebP, MP4.

### Tags

#### List Tags

```bash
typefully tags list
typefully tags list --json
```

#### Create a Tag

```bash
typefully tags create "product-launch"
typefully tags create "weekly-update" --set set_abc123
```

### Self-Update

Update typefully to the latest version. Automatically detects whether you installed via Homebrew or cargo:

```bash
typefully update
```

## 🌍 Global Flags

These flags work with every command:

| Flag | Description |
|------|-------------|
| `--json` | Output raw JSON to stdout |
| `--no-color` | Disable colored output (also respects `NO_COLOR` env) |
| `-q, --quiet` | Minimal output (only errors) |
| `-v, --verbose` | Debug logging to stderr |
| `--api-key <key>` | Override the API key for this invocation |
| `-h, --help` | Show help text |
| `--version` | Show version |

## 🔢 Exit Codes

| Code | Meaning |
|------|---------|
| `0` | Success |
| `1` | API error or runtime error |
| `2` | Invalid usage (bad flags, missing arguments) |

## 🔧 Scripting Examples

```bash
# List scheduled draft IDs as plain text
typefully draft list --status scheduled --json | jq -r '.[].id'

# Publish all drafts tagged "ready"
for id in $(typefully draft list --tag ready --json | jq -r '.[].id'); do
  typefully draft publish "$id"
done

# Upload media and create a draft in one pipeline
MEDIA_ID=$(typefully media upload photo.jpg --json | jq -r '.media_id')
typefully draft create --content "New photo!" --media "$MEDIA_ID"
```

## 🤝 Contributing

Contributions are welcome! Please read the following before submitting a PR:

1. **Read `AGENTS.md`** for project conventions and architecture
2. **Rust 2024 edition** with MSRV 1.91
3. **All checks must pass:**
   ```bash
   cargo check
   cargo clippy -- -D warnings
   cargo build --release
   ```
4. **Follow existing patterns:** trait-based API client, thiserror for errors, tracing for logging
5. **Add doc comments** on all public items
6. **No em dashes** in any text content

## 📄 License

This project is licensed under the [GNU General Public License v3.0](LICENSE).
