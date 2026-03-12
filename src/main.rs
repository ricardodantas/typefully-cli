use clap::{Parser, Subcommand, ValueEnum};
use colored::Colorize;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::io::{self, IsTerminal, Read};
use std::path::PathBuf;
use tabled::{Table, Tabled};

// ── Config ──────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Default)]
struct Config {
    api_key: Option<String>,
    default_social_set_id: Option<String>,
}

impl Config {
    fn path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("typefully")
            .join("config.toml")
    }

    fn load() -> Self {
        std::fs::read_to_string(Self::path())
            .ok()
            .and_then(|s| toml::from_str(&s).ok())
            .unwrap_or_default()
    }

    fn save(&self) -> Result<(), AppError> {
        let path = Self::path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, toml::to_string_pretty(self).unwrap())?;
        Ok(())
    }
}

// ── Error ───────────────────────────────────────────────────────────

#[derive(Debug)]
enum AppError {
    Api(String, Option<u16>),
    Config(String),
    Io(io::Error),
    Usage(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::Api(msg, code) => {
                if let Some(c) = code {
                    write!(f, "API error ({}): {}", c, msg)
                } else {
                    write!(f, "API error: {}", msg)
                }
            }
            AppError::Config(msg) => write!(f, "Config error: {}", msg),
            AppError::Io(e) => write!(f, "IO error: {}", e),
            AppError::Usage(msg) => write!(f, "{}", msg),
        }
    }
}

impl From<io::Error> for AppError {
    fn from(e: io::Error) -> Self {
        AppError::Io(e)
    }
}

impl From<reqwest::Error> for AppError {
    fn from(e: reqwest::Error) -> Self {
        AppError::Api(e.to_string(), e.status().map(|s| s.as_u16()))
    }
}

// ── CLI ─────────────────────────────────────────────────────────────

#[derive(Parser)]
#[command(name = "typefully", version, about = "CLI client for the Typefully API v2")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Output as JSON
    #[arg(long, global = true)]
    json: bool,

    /// Disable colored output
    #[arg(long, global = true)]
    no_color: bool,

    /// Minimal output
    #[arg(short, long, global = true)]
    quiet: bool,

    /// Debug output
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Override API key
    #[arg(long, global = true, env = "TYPEFULLY_API_KEY")]
    api_key: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Verify authentication and show account info
    Auth,
    /// Manage configuration
    #[command(subcommand)]
    Config(ConfigCmd),
    /// List social sets
    Sets,
    /// Manage drafts
    #[command(subcommand)]
    Draft(DraftCmd),
    /// Upload media
    #[command(subcommand)]
    Media(MediaCmd),
    /// Manage tags
    #[command(subcommand)]
    Tags(TagsCmd),
}

#[derive(Subcommand)]
enum ConfigCmd {
    /// Interactive setup for API key and default social set
    Init,
}

#[derive(Subcommand)]
enum DraftCmd {
    /// Create a new draft
    Create {
        /// Social set ID (uses default from config if not provided)
        #[arg(long)]
        set: Option<String>,
        /// Content text (reads from stdin if not provided)
        #[arg(long)]
        content: Option<String>,
        /// Target platforms
        #[arg(long, value_delimiter = ',', default_value = "x")]
        platform: Vec<Platform>,
        /// When to publish: "now", "next-free-slot", or ISO-8601 datetime
        #[arg(long)]
        publish_at: Option<String>,
        /// Tags to apply (repeatable)
        #[arg(long)]
        tag: Vec<String>,
        /// Media IDs to attach (repeatable)
        #[arg(long)]
        media: Vec<String>,
    },
    /// List drafts
    List {
        #[arg(long)]
        set: Option<String>,
        #[arg(long)]
        status: Option<DraftStatus>,
        #[arg(long)]
        tag: Option<String>,
        #[arg(long, default_value = "created_at")]
        sort: String,
        #[arg(long, default_value_t = 20)]
        limit: u32,
        #[arg(long, default_value_t = 0)]
        offset: u32,
    },
    /// Get a draft by ID
    Get {
        draft_id: String,
        #[arg(long)]
        set: Option<String>,
    },
    /// Edit a draft
    Edit {
        draft_id: String,
        #[arg(long)]
        set: Option<String>,
        #[arg(long)]
        content: Option<String>,
        #[arg(long)]
        publish_at: Option<String>,
        #[arg(long)]
        tag: Vec<String>,
        #[arg(long)]
        share: Option<bool>,
    },
    /// Delete a draft
    Delete {
        draft_id: String,
        #[arg(long)]
        set: Option<String>,
        /// Skip confirmation
        #[arg(long)]
        force: bool,
    },
    /// Publish a draft immediately
    Publish {
        draft_id: String,
        #[arg(long)]
        set: Option<String>,
    },
    /// Schedule a draft
    Schedule {
        draft_id: String,
        /// "next-free-slot" or ISO-8601 datetime
        time: String,
        #[arg(long)]
        set: Option<String>,
    },
}

#[derive(Subcommand)]
enum MediaCmd {
    /// Upload a media file
    Upload {
        /// Path to the file to upload
        file: PathBuf,
        #[arg(long)]
        set: Option<String>,
    },
}

#[derive(Subcommand)]
enum TagsCmd {
    /// List all tags
    List {
        #[arg(long)]
        set: Option<String>,
    },
    /// Create a new tag
    Create {
        name: String,
        #[arg(long)]
        set: Option<String>,
    },
}

#[derive(Clone, ValueEnum)]
enum Platform {
    X,
    Linkedin,
    Threads,
    Bluesky,
    Mastodon,
}

impl std::fmt::Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Platform::X => write!(f, "x"),
            Platform::Linkedin => write!(f, "linkedin"),
            Platform::Threads => write!(f, "threads"),
            Platform::Bluesky => write!(f, "bluesky"),
            Platform::Mastodon => write!(f, "mastodon"),
        }
    }
}

#[derive(Clone, ValueEnum)]
enum DraftStatus {
    Draft,
    Scheduled,
    Published,
}

impl std::fmt::Display for DraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DraftStatus::Draft => write!(f, "draft"),
            DraftStatus::Scheduled => write!(f, "scheduled"),
            DraftStatus::Published => write!(f, "published"),
        }
    }
}

// ── API Client ──────────────────────────────────────────────────────

const BASE_URL: &str = "https://api.typefully.com/v2";

struct ApiClient {
    client: Client,
    api_key: String,
    json_output: bool,
    quiet: bool,
    verbose: bool,
}

impl ApiClient {
    fn new(api_key: String, json_output: bool, quiet: bool, verbose: bool) -> Self {
        Self {
            client: Client::new(),
            api_key,
            json_output,
            quiet,
            verbose,
        }
    }

    fn get(&self, path: &str) -> Result<serde_json::Value, AppError> {
        let url = format!("{}{}", BASE_URL, path);
        if self.verbose {
            eprintln!("{} GET {}", "DEBUG:".dimmed(), url);
        }
        let resp = self
            .client
            .get(&url)
            .bearer_auth(&self.api_key)
            .send()?;
        self.handle_response(resp)
    }

    fn post(&self, path: &str, body: &serde_json::Value) -> Result<serde_json::Value, AppError> {
        let url = format!("{}{}", BASE_URL, path);
        if self.verbose {
            eprintln!("{} POST {} {:?}", "DEBUG:".dimmed(), url, body);
        }
        let resp = self
            .client
            .post(&url)
            .bearer_auth(&self.api_key)
            .json(body)
            .send()?;
        self.handle_response(resp)
    }

    fn patch(&self, path: &str, body: &serde_json::Value) -> Result<serde_json::Value, AppError> {
        let url = format!("{}{}", BASE_URL, path);
        if self.verbose {
            eprintln!("{} PATCH {} {:?}", "DEBUG:".dimmed(), url, body);
        }
        let resp = self
            .client
            .patch(&url)
            .bearer_auth(&self.api_key)
            .json(body)
            .send()?;
        self.handle_response(resp)
    }

    fn delete(&self, path: &str) -> Result<serde_json::Value, AppError> {
        let url = format!("{}{}", BASE_URL, path);
        if self.verbose {
            eprintln!("{} DELETE {}", "DEBUG:".dimmed(), url);
        }
        let resp = self
            .client
            .delete(&url)
            .bearer_auth(&self.api_key)
            .send()?;
        self.handle_response(resp)
    }

    fn put_bytes(&self, url: &str, data: Vec<u8>, content_type: &str) -> Result<(), AppError> {
        if self.verbose {
            eprintln!("{} PUT {} ({} bytes)", "DEBUG:".dimmed(), url, data.len());
        }
        let resp = self
            .client
            .put(url)
            .header("Content-Type", content_type)
            .body(data)
            .send()?;
        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().unwrap_or_default();
            return Err(AppError::Api(text, Some(status.as_u16())));
        }
        Ok(())
    }

    fn handle_response(
        &self,
        resp: reqwest::blocking::Response,
    ) -> Result<serde_json::Value, AppError> {
        let status = resp.status();

        if let Some(retry_after) = resp.headers().get("retry-after") {
            if status == 429 {
                let retry = retry_after.to_str().unwrap_or("unknown");
                return Err(AppError::Api(
                    format!("Rate limited. Retry after {} seconds.", retry),
                    Some(429),
                ));
            }
        }

        let text = resp.text()?;

        if !status.is_success() {
            let msg = serde_json::from_str::<serde_json::Value>(&text)
                .ok()
                .and_then(|v| v.get("error").and_then(|e| e.as_str()).map(String::from))
                .unwrap_or(text);
            return Err(AppError::Api(msg, Some(status.as_u16())));
        }

        if text.is_empty() {
            return Ok(serde_json::Value::Null);
        }

        serde_json::from_str(&text).map_err(|e| AppError::Api(e.to_string(), None))
    }
}

// ── Table helpers ───────────────────────────────────────────────────

#[derive(Tabled)]
struct SocialSetRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Platforms")]
    platforms: String,
}

#[derive(Tabled)]
struct DraftRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "Status")]
    status: String,
    #[tabled(rename = "Content")]
    content: String,
    #[tabled(rename = "Scheduled")]
    scheduled: String,
}

#[derive(Tabled)]
struct TagRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "Name")]
    name: String,
}

// ── Helpers ─────────────────────────────────────────────────────────

fn resolve_set_id(set: Option<String>) -> Result<String, AppError> {
    if let Some(id) = set {
        return Ok(id);
    }
    let config = Config::load();
    config
        .default_social_set_id
        .ok_or_else(|| AppError::Usage(
            "No social set ID provided. Use --set or run 'typefully config init' to set a default.".into(),
        ))
}

fn resolve_api_key(cli_key: Option<String>) -> Result<String, AppError> {
    if let Some(key) = cli_key {
        return Ok(key);
    }
    // env is handled by clap's env feature, but let's also check config
    if let Ok(key) = std::env::var("TYPEFULLY_API_KEY") {
        if !key.is_empty() {
            return Ok(key);
        }
    }
    let config = Config::load();
    config
        .api_key
        .ok_or_else(|| AppError::Config(
            "No API key found. Set --api-key, TYPEFULLY_API_KEY env var, or run 'typefully config init'.".into(),
        ))
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max.saturating_sub(3)])
    }
}

fn json_str(v: &serde_json::Value, key: &str) -> String {
    v.get(key)
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string()
}

// ── Command handlers ────────────────────────────────────────────────

fn cmd_auth(api: &ApiClient) -> Result<(), AppError> {
    let me = api.get("/me")?;
    if api.json_output {
        println!("{}", serde_json::to_string_pretty(&me).unwrap());
    } else {
        println!("{}", "Authenticated successfully!".green());
        if let Some(name) = me.get("name").and_then(|v| v.as_str()) {
            println!("  Name: {}", name);
        }
        if let Some(email) = me.get("email").and_then(|v| v.as_str()) {
            println!("  Email: {}", email);
        }
        if let Some(handle) = me.get("handle").and_then(|v| v.as_str()) {
            println!("  Handle: @{}", handle);
        }
    }
    Ok(())
}

fn cmd_config_init(api: &ApiClient) -> Result<(), AppError> {
    let mut config = Config::load();

    let key: String = dialoguer::Input::new()
        .with_prompt("API Key")
        .default(config.api_key.clone().unwrap_or_default())
        .interact_text()
        .map_err(|e| AppError::Io(io::Error::new(io::ErrorKind::Other, e)))?;

    config.api_key = Some(key.clone());

    // Try to fetch social sets to let user pick a default
    let temp_api = ApiClient::new(key, false, false, api.verbose);
    match temp_api.get("/social-sets") {
        Ok(sets) => {
            if let Some(arr) = sets.as_array() {
                if !arr.is_empty() {
                    let items: Vec<String> = arr
                        .iter()
                        .map(|s| {
                            format!(
                                "{} ({})",
                                json_str(s, "name"),
                                json_str(s, "id")
                            )
                        })
                        .collect();
                    let selection = dialoguer::Select::new()
                        .with_prompt("Default social set")
                        .items(&items)
                        .default(0)
                        .interact()
                        .map_err(|e| AppError::Io(io::Error::new(io::ErrorKind::Other, e)))?;

                    config.default_social_set_id =
                        Some(json_str(&arr[selection], "id"));
                }
            }
        }
        Err(e) => {
            eprintln!(
                "{} Could not fetch social sets: {}",
                "Warning:".yellow(),
                e
            );
        }
    }

    config.save()?;
    if !api.quiet {
        println!("{}", "Configuration saved!".green());
        println!("  Path: {}", Config::path().display());
    }
    Ok(())
}

fn cmd_sets(api: &ApiClient) -> Result<(), AppError> {
    let sets = api.get("/social-sets")?;
    if api.json_output {
        println!("{}", serde_json::to_string_pretty(&sets).unwrap());
        return Ok(());
    }
    let empty = vec![]; let arr = sets.as_array().unwrap_or(&empty);
    let rows: Vec<SocialSetRow> = arr
        .iter()
        .map(|s| {
            let platforms = s
                .get("platforms")
                .and_then(|p| p.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|v| v.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                })
                .unwrap_or_default();
            SocialSetRow {
                id: json_str(s, "id"),
                name: json_str(s, "name"),
                platforms,
            }
        })
        .collect();
    if rows.is_empty() {
        println!("No social sets found.");
    } else {
        println!("{}", Table::new(&rows));
    }
    Ok(())
}

fn cmd_draft_create(
    api: &ApiClient,
    set: Option<String>,
    content: Option<String>,
    platforms: Vec<Platform>,
    publish_at: Option<String>,
    tags: Vec<String>,
    media_ids: Vec<String>,
) -> Result<(), AppError> {
    let set_id = resolve_set_id(set)?;

    let text = match content {
        Some(c) => c,
        None => {
            if std::io::stdin().is_terminal() {
                return Err(AppError::Usage(
                    "No content provided. Use --content or pipe text via stdin.".into(),
                ));
            }
            let mut buf = String::new();
            io::stdin().read_to_string(&mut buf)?;
            buf.trim().to_string()
        }
    };

    // Split threads on "---"
    let posts: Vec<&str> = text
        .split("\n---\n")
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    let platform_strs: Vec<String> = platforms.iter().map(|p| p.to_string()).collect();

    let mut body = serde_json::json!({
        "platforms": platform_strs,
    });

    if posts.len() > 1 {
        body["posts"] = serde_json::json!(
            posts.iter().map(|p| serde_json::json!({"content": p})).collect::<Vec<_>>()
        );
    } else {
        body["content"] = serde_json::json!(text);
    }

    if let Some(ref pa) = publish_at {
        body["publish_at"] = serde_json::json!(pa);
    }
    if !tags.is_empty() {
        body["tags"] = serde_json::json!(tags);
    }
    if !media_ids.is_empty() {
        body["media"] = serde_json::json!(media_ids);
    }

    let result = api.post(&format!("/social-sets/{}/drafts", set_id), &body)?;
    if api.json_output {
        println!("{}", serde_json::to_string_pretty(&result).unwrap());
    } else if !api.quiet {
        let id = json_str(&result, "id");
        println!("{} Draft created (ID: {})", "Success!".green(), id);
    }
    Ok(())
}

fn cmd_draft_list(
    api: &ApiClient,
    set: Option<String>,
    status: Option<DraftStatus>,
    tag: Option<String>,
    sort: String,
    limit: u32,
    offset: u32,
) -> Result<(), AppError> {
    let set_id = resolve_set_id(set)?;
    let mut params = vec![
        format!("sort={}", sort),
        format!("limit={}", limit),
        format!("offset={}", offset),
    ];
    if let Some(ref s) = status {
        params.push(format!("status={}", s));
    }
    if let Some(ref t) = tag {
        params.push(format!("tag={}", t));
    }
    let query = params.join("&");
    let result = api.get(&format!("/social-sets/{}/drafts?{}", set_id, query))?;

    if api.json_output {
        println!("{}", serde_json::to_string_pretty(&result).unwrap());
        return Ok(());
    }

    let empty = vec![]; let arr = result.as_array().unwrap_or(&empty);
    let rows: Vec<DraftRow> = arr
        .iter()
        .map(|d| DraftRow {
            id: json_str(d, "id"),
            status: json_str(d, "status"),
            content: truncate(&json_str(d, "content"), 60),
            scheduled: json_str(d, "scheduled_date"),
        })
        .collect();
    if rows.is_empty() {
        println!("No drafts found.");
    } else {
        println!("{}", Table::new(&rows));
    }
    Ok(())
}

fn cmd_draft_get(
    api: &ApiClient,
    draft_id: &str,
    set: Option<String>,
) -> Result<(), AppError> {
    let set_id = resolve_set_id(set)?;
    let result = api.get(&format!("/social-sets/{}/drafts/{}", set_id, draft_id))?;
    if api.json_output {
        println!("{}", serde_json::to_string_pretty(&result).unwrap());
    } else {
        println!("{}", serde_json::to_string_pretty(&result).unwrap());
    }
    Ok(())
}

fn cmd_draft_edit(
    api: &ApiClient,
    draft_id: &str,
    set: Option<String>,
    content: Option<String>,
    publish_at: Option<String>,
    tags: Vec<String>,
    share: Option<bool>,
) -> Result<(), AppError> {
    let set_id = resolve_set_id(set)?;
    let mut body = serde_json::json!({});
    if let Some(c) = content {
        body["content"] = serde_json::json!(c);
    }
    if let Some(pa) = publish_at {
        body["publish_at"] = serde_json::json!(pa);
    }
    if !tags.is_empty() {
        body["tags"] = serde_json::json!(tags);
    }
    if let Some(s) = share {
        body["share"] = serde_json::json!(s);
    }
    let result = api.patch(
        &format!("/social-sets/{}/drafts/{}", set_id, draft_id),
        &body,
    )?;
    if api.json_output {
        println!("{}", serde_json::to_string_pretty(&result).unwrap());
    } else if !api.quiet {
        println!("{} Draft updated.", "Success!".green());
    }
    Ok(())
}

fn cmd_draft_delete(
    api: &ApiClient,
    draft_id: &str,
    set: Option<String>,
    force: bool,
) -> Result<(), AppError> {
    let set_id = resolve_set_id(set)?;
    if !force {
        let confirm = dialoguer::Confirm::new()
            .with_prompt(format!("Delete draft {}?", draft_id))
            .default(false)
            .interact()
            .map_err(|e| AppError::Io(io::Error::new(io::ErrorKind::Other, e)))?;
        if !confirm {
            println!("Cancelled.");
            return Ok(());
        }
    }
    api.delete(&format!("/social-sets/{}/drafts/{}", set_id, draft_id))?;
    if !api.quiet {
        println!("{} Draft deleted.", "Success!".green());
    }
    Ok(())
}

fn cmd_draft_publish(
    api: &ApiClient,
    draft_id: &str,
    set: Option<String>,
) -> Result<(), AppError> {
    let set_id = resolve_set_id(set)?;
    let body = serde_json::json!({"publish_at": "now"});
    let result = api.patch(
        &format!("/social-sets/{}/drafts/{}", set_id, draft_id),
        &body,
    )?;
    if api.json_output {
        println!("{}", serde_json::to_string_pretty(&result).unwrap());
    } else if !api.quiet {
        println!("{} Draft published.", "Success!".green());
    }
    Ok(())
}

fn cmd_draft_schedule(
    api: &ApiClient,
    draft_id: &str,
    time: &str,
    set: Option<String>,
) -> Result<(), AppError> {
    let set_id = resolve_set_id(set)?;
    let body = serde_json::json!({"publish_at": time});
    let result = api.patch(
        &format!("/social-sets/{}/drafts/{}", set_id, draft_id),
        &body,
    )?;
    if api.json_output {
        println!("{}", serde_json::to_string_pretty(&result).unwrap());
    } else if !api.quiet {
        println!("{} Draft scheduled.", "Success!".green());
    }
    Ok(())
}

fn cmd_media_upload(
    api: &ApiClient,
    file: &PathBuf,
    set: Option<String>,
) -> Result<(), AppError> {
    let set_id = resolve_set_id(set)?;

    let file_name = file
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("file")
        .to_string();
    let data = std::fs::read(file)?;

    let content_type = match file.extension().and_then(|e| e.to_str()) {
        Some("png") => "image/png",
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        Some("mp4") => "video/mp4",
        _ => "application/octet-stream",
    };

    // Step 1: Get presigned URL
    let body = serde_json::json!({
        "file_name": file_name,
        "content_type": content_type,
    });
    let upload_info = api.post(
        &format!("/social-sets/{}/media/upload", set_id),
        &body,
    )?;
    let presigned_url = upload_info
        .get("presigned_url")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::Api("No presigned_url in response".into(), None))?;
    let media_id = json_str(&upload_info, "media_id");

    if api.verbose {
        eprintln!("{} Media ID: {}", "DEBUG:".dimmed(), media_id);
    }

    // Step 2: Upload to S3
    api.put_bytes(presigned_url, data, content_type)?;

    // Step 3: Poll until ready
    if !api.quiet {
        eprint!("Processing");
    }
    for _ in 0..60 {
        std::thread::sleep(std::time::Duration::from_secs(2));
        let status = api.get(&format!("/social-sets/{}/media/{}", set_id, media_id))?;
        let state = json_str(&status, "status");
        if state == "ready" {
            if !api.quiet {
                eprintln!();
            }
            if api.json_output {
                println!("{}", serde_json::to_string_pretty(&status).unwrap());
            } else {
                println!("{} Media uploaded (ID: {})", "Success!".green(), media_id);
            }
            return Ok(());
        }
        if state == "error" {
            return Err(AppError::Api("Media processing failed.".into(), None));
        }
        if !api.quiet {
            eprint!(".");
        }
    }
    Err(AppError::Api("Media processing timed out.".into(), None))
}

fn cmd_tags_list(api: &ApiClient, set: Option<String>) -> Result<(), AppError> {
    let set_id = resolve_set_id(set)?;
    let result = api.get(&format!("/social-sets/{}/tags", set_id))?;
    if api.json_output {
        println!("{}", serde_json::to_string_pretty(&result).unwrap());
        return Ok(());
    }
    let empty = vec![]; let arr = result.as_array().unwrap_or(&empty);
    let rows: Vec<TagRow> = arr
        .iter()
        .map(|t| TagRow {
            id: json_str(t, "id"),
            name: json_str(t, "name"),
        })
        .collect();
    if rows.is_empty() {
        println!("No tags found.");
    } else {
        println!("{}", Table::new(&rows));
    }
    Ok(())
}

fn cmd_tags_create(
    api: &ApiClient,
    name: &str,
    set: Option<String>,
) -> Result<(), AppError> {
    let set_id = resolve_set_id(set)?;
    let body = serde_json::json!({"name": name});
    let result = api.post(&format!("/social-sets/{}/tags", set_id), &body)?;
    if api.json_output {
        println!("{}", serde_json::to_string_pretty(&result).unwrap());
    } else if !api.quiet {
        println!("{} Tag '{}' created.", "Success!".green(), name);
    }
    Ok(())
}

// ── Main ────────────────────────────────────────────────────────────

fn main() {
    let cli = Cli::parse();

    if cli.no_color || std::env::var("NO_COLOR").is_ok() {
        colored::control::set_override(false);
    }

    let exit_code = match run(cli) {
        Ok(()) => 0,
        Err(e) => {
            match &e {
                AppError::Usage(_) => {
                    eprintln!("{} {}", "Error:".red().bold(), e);
                    2
                }
                _ => {
                    eprintln!("{} {}", "Error:".red().bold(), e);
                    1
                }
            }
        }
    };

    std::process::exit(exit_code);
}

fn run(cli: Cli) -> Result<(), AppError> {
    // Config init doesn't need auth
    if matches!(cli.command, Commands::Config(ConfigCmd::Init)) {
        let api = ApiClient::new(String::new(), cli.json, cli.quiet, cli.verbose);
        return cmd_config_init(&api);
    }

    let api_key = resolve_api_key(cli.api_key)?;
    let api = ApiClient::new(api_key, cli.json, cli.quiet, cli.verbose);

    match cli.command {
        Commands::Auth => cmd_auth(&api),
        Commands::Config(ConfigCmd::Init) => unreachable!(),
        Commands::Sets => cmd_sets(&api),
        Commands::Draft(cmd) => match cmd {
            DraftCmd::Create {
                set,
                content,
                platform,
                publish_at,
                tag,
                media,
            } => cmd_draft_create(&api, set, content, platform, publish_at, tag, media),
            DraftCmd::List {
                set,
                status,
                tag,
                sort,
                limit,
                offset,
            } => cmd_draft_list(&api, set, status, tag, sort, limit, offset),
            DraftCmd::Get { draft_id, set } => cmd_draft_get(&api, &draft_id, set),
            DraftCmd::Edit {
                draft_id,
                set,
                content,
                publish_at,
                tag,
                share,
            } => cmd_draft_edit(&api, &draft_id, set, content, publish_at, tag, share),
            DraftCmd::Delete {
                draft_id,
                set,
                force,
            } => cmd_draft_delete(&api, &draft_id, set, force),
            DraftCmd::Publish { draft_id, set } => cmd_draft_publish(&api, &draft_id, set),
            DraftCmd::Schedule {
                draft_id,
                time,
                set,
            } => cmd_draft_schedule(&api, &draft_id, &time, set),
        },
        Commands::Media(cmd) => match cmd {
            MediaCmd::Upload { file, set } => cmd_media_upload(&api, &file, set),
        },
        Commands::Tags(cmd) => match cmd {
            TagsCmd::List { set } => cmd_tags_list(&api, set),
            TagsCmd::Create { name, set } => cmd_tags_create(&api, &name, set),
        },
    }
}
