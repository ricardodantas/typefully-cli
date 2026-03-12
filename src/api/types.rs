//! Shared API types.

use std::fmt;

/// Parameters for listing drafts.
#[derive(Debug, Clone, Default)]
pub struct DraftListParams {
    /// Filter by status.
    pub status: Option<String>,
    /// Filter by tag name.
    pub tag: Option<String>,
    /// Sort field.
    pub sort: String,
    /// Maximum results.
    pub limit: u32,
    /// Offset for pagination.
    pub offset: u32,
}

impl DraftListParams {
    /// Build query string for the request.
    #[must_use]
    pub fn to_query_string(&self) -> String {
        let mut parts = vec![
            format!("sort={}", self.sort),
            format!("limit={}", self.limit),
            format!("offset={}", self.offset),
        ];
        if let Some(ref s) = self.status {
            parts.push(format!("status={s}"));
        }
        if let Some(ref t) = self.tag {
            parts.push(format!("tag={t}"));
        }
        parts.join("&")
    }
}

/// Supported social media platforms.
#[derive(Debug, Clone, clap::ValueEnum)]
#[non_exhaustive]
pub enum Platform {
    /// X (formerly Twitter).
    X,
    /// `LinkedIn`.
    Linkedin,
    /// Threads.
    Threads,
    /// Bluesky.
    Bluesky,
    /// Mastodon.
    Mastodon,
}

impl fmt::Display for Platform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::X => write!(f, "x"),
            Self::Linkedin => write!(f, "linkedin"),
            Self::Threads => write!(f, "threads"),
            Self::Bluesky => write!(f, "bluesky"),
            Self::Mastodon => write!(f, "mastodon"),
        }
    }
}

/// Draft status filter.
#[derive(Debug, Clone, clap::ValueEnum)]
#[non_exhaustive]
pub enum DraftStatus {
    /// Unpublished draft.
    Draft,
    /// Scheduled for future publishing.
    Scheduled,
    /// Already published.
    Published,
}

impl fmt::Display for DraftStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Draft => write!(f, "draft"),
            Self::Scheduled => write!(f, "scheduled"),
            Self::Published => write!(f, "published"),
        }
    }
}
