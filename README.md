# typefully-cli

A full-featured CLI client for the [Typefully](https://typefully.com) API v2. Create, schedule, and manage your social media drafts from the terminal.

## Installation

### From source

```bash
git clone https://github.com/ricardodantas/typefully-cli.git
cd typefully-cli
cargo install --path .
```

## Configuration

### Quick setup

```bash
typefully config init
```

This will interactively prompt for your API key and default social set.

### Manual setup

Create `~/.config/typefully/config.toml`:

```toml
api_key = "your-api-key-here"
default_social_set_id = "your-social-set-id"
```

### API key precedence

1. `--api-key` flag
2. `TYPEFULLY_API_KEY` environment variable
3. `~/.config/typefully/config.toml`

## Usage

### Authentication

```bash
# Verify your API key and show account info
typefully auth
```

### Social Sets

```bash
# List all social sets
typefully sets
```

### Drafts

```bash
# Create a simple draft
typefully draft create --content "Hello world!"

# Create a draft for multiple platforms
typefully draft create --content "Hello!" --platform x --platform linkedin

# Create and publish immediately
typefully draft create --content "Going live now!" --publish-at now

# Schedule for next free slot
typefully draft create --content "Scheduled post" --publish-at next-free-slot

# Schedule for a specific time
typefully draft create --content "Timed post" --publish-at "2025-01-15T10:00:00Z"

# Create a thread (split on ---)
typefully draft create --content "First post
---
Second post
---
Third post"

# Pipe content from stdin
echo "Hello from a pipe" | typefully draft create

# Create with tags
typefully draft create --content "Tagged post" --tag marketing --tag launch

# Attach media
typefully draft create --content "Check this out" --media media_abc123

# List drafts
typefully draft list
typefully draft list --status scheduled --limit 10
typefully draft list --tag marketing --sort scheduled_date

# Get a specific draft
typefully draft get abc123

# Edit a draft
typefully draft edit abc123 --content "Updated content"
typefully draft edit abc123 --publish-at "2025-02-01T12:00:00Z"

# Delete a draft (with confirmation)
typefully draft delete abc123

# Delete without confirmation
typefully draft delete abc123 --force

# Publish immediately
typefully draft publish abc123

# Schedule a draft
typefully draft schedule abc123 next-free-slot
typefully draft schedule abc123 "2025-01-20T15:00:00Z"
```

### Media

```bash
# Upload a media file
typefully media upload photo.jpg

# Use the returned media_id with draft create
typefully draft create --content "Look at this!" --media <media_id>
```

### Tags

```bash
# List all tags
typefully tags list

# Create a new tag
typefully tags create "product-launch"
```

### Specifying a social set

All commands that need a social set accept `--set <id>`. If omitted, the default from your config is used.

```bash
typefully draft list --set set_abc123
typefully tags list --set set_abc123
```

## Global Flags

| Flag | Description |
|------|-------------|
| `--json` | Output as JSON |
| `--no-color` | Disable colored output (also respects `NO_COLOR` env) |
| `-q, --quiet` | Minimal output |
| `-v, --verbose` | Debug output |
| `--api-key <key>` | Override API key |
| `-h, --help` | Show help |
| `--version` | Show version |

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | API or runtime error |
| 2 | Invalid usage |

## License

MIT
