//! CLI argument definitions using clap derive.

use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::api::{DraftStatus, Platform};

/// CLI client for the Typefully API v2.
#[derive(Parser, Debug)]
#[command(
    name = "typefully",
    version,
    about = "CLI client for the Typefully API v2"
)]
#[allow(clippy::struct_excessive_bools)]
pub struct Cli {
    /// Subcommand to run.
    #[command(subcommand)]
    pub command: Commands,

    /// Output as JSON.
    #[arg(long, global = true)]
    pub json: bool,

    /// Disable colored output.
    #[arg(long, global = true)]
    pub no_color: bool,

    /// Minimal output.
    #[arg(short, long, global = true)]
    pub quiet: bool,

    /// Debug output (sets TRACE logging).
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Override API key.
    #[arg(long, global = true)]
    pub api_key: Option<String>,
}

/// Top-level subcommands.
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Verify authentication and show account info.
    Auth,
    /// Manage configuration.
    #[command(subcommand)]
    Config(ConfigCmd),
    /// List social sets.
    Sets,
    /// Manage drafts.
    #[command(subcommand)]
    Draft(DraftCmd),
    /// Upload media.
    #[command(subcommand)]
    Media(MediaCmd),
    /// Manage tags.
    #[command(subcommand)]
    Tags(TagsCmd),
    /// Update typefully to the latest version.
    Update,
}

/// Configuration subcommands.
#[derive(Subcommand, Debug)]
pub enum ConfigCmd {
    /// Interactive setup for API key and default social set.
    Init,
}

/// Draft subcommands.
#[derive(Subcommand, Debug)]
pub enum DraftCmd {
    /// Create a new draft.
    Create {
        /// Social set ID (uses default from config if not provided).
        #[arg(long)]
        set: Option<String>,
        /// Content text (reads from stdin if not provided).
        #[arg(long)]
        content: Option<String>,
        /// Target platforms.
        #[arg(long, value_delimiter = ',', default_value = "x")]
        platform: Vec<Platform>,
        /// When to publish: "now", "next-free-slot", or ISO-8601 datetime.
        #[arg(long)]
        publish_at: Option<String>,
        /// Tags to apply (repeatable).
        #[arg(long)]
        tag: Vec<String>,
        /// Media IDs to attach (repeatable).
        #[arg(long)]
        media: Vec<String>,
    },
    /// List drafts.
    List {
        /// Social set ID.
        #[arg(long)]
        set: Option<String>,
        /// Filter by status.
        #[arg(long)]
        status: Option<DraftStatus>,
        /// Filter by tag.
        #[arg(long)]
        tag: Option<String>,
        /// Sort field.
        #[arg(long, default_value = "created_at")]
        sort: String,
        /// Maximum results.
        #[arg(long, default_value_t = 20)]
        limit: u32,
        /// Offset for pagination.
        #[arg(long, default_value_t = 0)]
        offset: u32,
    },
    /// Get a draft by ID.
    Get {
        /// Draft ID.
        draft_id: String,
        /// Social set ID.
        #[arg(long)]
        set: Option<String>,
    },
    /// Edit a draft.
    Edit {
        /// Draft ID.
        draft_id: String,
        /// Social set ID.
        #[arg(long)]
        set: Option<String>,
        /// New content.
        #[arg(long)]
        content: Option<String>,
        /// New publish time.
        #[arg(long)]
        publish_at: Option<String>,
        /// Tags to set.
        #[arg(long)]
        tag: Vec<String>,
        /// Share flag.
        #[arg(long)]
        share: Option<bool>,
    },
    /// Delete a draft.
    Delete {
        /// Draft ID.
        draft_id: String,
        /// Social set ID.
        #[arg(long)]
        set: Option<String>,
        /// Skip confirmation prompt.
        #[arg(long)]
        force: bool,
    },
    /// Publish a draft immediately.
    Publish {
        /// Draft ID.
        draft_id: String,
        /// Social set ID.
        #[arg(long)]
        set: Option<String>,
    },
    /// Schedule a draft for a specific time.
    Schedule {
        /// Draft ID.
        draft_id: String,
        /// "next-free-slot" or ISO-8601 datetime.
        time: String,
        /// Social set ID.
        #[arg(long)]
        set: Option<String>,
    },
}

/// Media subcommands.
#[derive(Subcommand, Debug)]
pub enum MediaCmd {
    /// Upload a media file.
    Upload {
        /// Path to the file to upload.
        file: PathBuf,
        /// Social set ID.
        #[arg(long)]
        set: Option<String>,
    },
}

/// Tag subcommands.
#[derive(Subcommand, Debug)]
pub enum TagsCmd {
    /// List all tags.
    List {
        /// Social set ID.
        #[arg(long)]
        set: Option<String>,
    },
    /// Create a new tag.
    Create {
        /// Tag name.
        name: String,
        /// Social set ID.
        #[arg(long)]
        set: Option<String>,
    },
}
