//! Entry point for the Typefully CLI.

use std::io::{IsTerminal, Read};

use clap::Parser;
use colored::Colorize;
use tracing::info;

use typefully::api::DraftListParams;
use typefully::api::{TypefullyApi, TypefullyClient};
use typefully::cli::{Cli, Commands, ConfigCmd, DraftCmd, MediaCmd, TagsCmd};
use typefully::config::AppConfig;
use typefully::error::{ApiError, AppError};
use typefully::output::{
    DraftRow, SocialSetRow, TagRow, json_str, print_error, print_json, print_success, print_table,
    truncate,
};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    if cli.no_color || std::env::var("NO_COLOR").is_ok() {
        colored::control::set_override(false);
    }

    let filter = if cli.verbose { "debug" } else { "warn" };
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .init();

    let exit_code = match run(&cli).await {
        Ok(()) => 0,
        Err(e) => {
            print_error(&e.to_string());
            e.exit_code()
        }
    };

    std::process::exit(exit_code);
}

async fn run(cli: &Cli) -> Result<(), AppError> {
    if matches!(cli.command, Commands::Config(ConfigCmd::Init)) {
        return cmd_config_init(cli).await;
    }

    if matches!(cli.command, Commands::Update) {
        return cmd_update_sync(cli);
    }

    let api_key = AppConfig::resolve_api_key(cli.api_key.as_deref())?;
    let client = TypefullyClient::new(api_key);

    match &cli.command {
        Commands::Auth => cmd_auth(cli, &client).await,
        Commands::Config(ConfigCmd::Init) | Commands::Update => unreachable!(),
        Commands::Sets => cmd_sets(cli, &client).await,
        Commands::Draft(cmd) => cmd_draft(cli, &client, cmd).await,
        Commands::Media(cmd) => cmd_media(cli, &client, cmd).await,
        Commands::Tags(cmd) => cmd_tags(cli, &client, cmd).await,
    }
}

async fn cmd_auth(cli: &Cli, api: &impl TypefullyApi) -> Result<(), AppError> {
    let me = api.get_me().await?;
    if cli.json {
        print_json(&me);
    } else {
        println!("{}", "Authenticated successfully!".green());
        if let Some(name) = me.get("name").and_then(|v| v.as_str()) {
            println!("  Name: {name}");
        }
        if let Some(email) = me.get("email").and_then(|v| v.as_str()) {
            println!("  Email: {email}");
        }
        if let Some(handle) = me.get("handle").and_then(|v| v.as_str()) {
            println!("  Handle: @{handle}");
        }
    }
    Ok(())
}

fn cmd_update_sync(cli: &Cli) -> Result<(), AppError> {
    use std::process::Command;

    // Detect install method and update accordingly
    let homebrew = Command::new("brew")
        .args(["list", "typefully"])
        .output()
        .ok()
        .is_some_and(|o| o.status.success());

    if homebrew {
        if !cli.quiet {
            println!("{}", "Updating via Homebrew...".cyan());
        }
        let status = Command::new("brew")
            .args(["upgrade", "typefully"])
            .status()
            .map_err(AppError::Io)?;
        if status.success() {
            print_success("Updated successfully via Homebrew.");
        } else {
            print_error("Homebrew upgrade failed. Try running: brew upgrade typefully");
        }
    } else {
        // Fall back to cargo install
        if !cli.quiet {
            println!(
                "{}",
                "Not installed via Homebrew. Updating via cargo...".cyan()
            );
        }
        let status = Command::new("cargo")
            .args(["install", "typefully-cli"])
            .status()
            .map_err(AppError::Io)?;
        if status.success() {
            print_success("Updated successfully via cargo.");
        } else {
            print_error("Cargo install failed. Try running: cargo install typefully-cli");
        }
    }
    Ok(())
}

async fn cmd_config_init(cli: &Cli) -> Result<(), AppError> {
    let mut config = AppConfig::load();

    let default_key = config
        .api_key
        .as_ref()
        .map(|s| secrecy::ExposeSecret::expose_secret(s).to_string())
        .unwrap_or_default();

    let key: String = dialoguer::Input::new()
        .with_prompt("API Key")
        .default(default_key)
        .interact_text()
        .map_err(|e| AppError::Io(std::io::Error::other(e)))?;

    config.api_key = Some(secrecy::SecretString::from(key.clone()));

    let temp_client = TypefullyClient::from_key(&key);
    match temp_client.get_social_sets().await {
        Ok(sets) => {
            if let Some(arr) = sets.as_array()
                && !arr.is_empty()
            {
                let items: Vec<String> = arr
                    .iter()
                    .map(|s| format!("{} ({})", json_str(s, "name"), json_str(s, "id")))
                    .collect();
                let selection = dialoguer::Select::new()
                    .with_prompt("Default social set")
                    .items(&items)
                    .default(0)
                    .interact()
                    .map_err(|e| AppError::Io(std::io::Error::other(e)))?;
                config.default_social_set_id = Some(json_str(&arr[selection], "id"));
            }
        }
        Err(e) => {
            eprintln!("{} Could not fetch social sets: {e}", "Warning:".yellow());
        }
    }

    config.save().map_err(AppError::Config)?;
    if !cli.quiet {
        print_success(&format!(
            "Configuration saved to {}",
            AppConfig::path().display()
        ));
    }
    Ok(())
}

async fn cmd_sets(cli: &Cli, api: &impl TypefullyApi) -> Result<(), AppError> {
    let sets = api.get_social_sets().await?;
    if cli.json {
        print_json(&sets);
        return Ok(());
    }
    let empty = vec![];
    let arr = sets.as_array().unwrap_or(&empty);
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
    print_table(&rows, "No social sets found.");
    Ok(())
}

#[allow(clippy::too_many_lines)]
async fn cmd_draft(cli: &Cli, api: &impl TypefullyApi, cmd: &DraftCmd) -> Result<(), AppError> {
    match cmd {
        DraftCmd::Create {
            set,
            content,
            platform,
            publish_at,
            tag,
            media,
        } => {
            cmd_draft_create(
                cli,
                api,
                set.as_deref(),
                content.as_deref(),
                platform,
                publish_at.as_deref(),
                tag,
                media,
            )
            .await
        }
        DraftCmd::List {
            set,
            status,
            tag,
            sort,
            limit,
            offset,
        } => {
            let set_id = AppConfig::resolve_set_id(set.as_deref())?;
            let params = DraftListParams {
                status: status.as_ref().map(ToString::to_string),
                tag: tag.clone(),
                sort: sort.clone(),
                limit: *limit,
                offset: *offset,
            };
            let result = api.list_drafts(&set_id, &params).await?;
            if cli.json {
                print_json(&result);
                return Ok(());
            }
            let empty = vec![];
            let arr = result.as_array().unwrap_or(&empty);
            let rows: Vec<DraftRow> = arr
                .iter()
                .map(|d| DraftRow {
                    id: json_str(d, "id"),
                    status: json_str(d, "status"),
                    content: truncate(&json_str(d, "content"), 60),
                    scheduled: json_str(d, "scheduled_date"),
                })
                .collect();
            print_table(&rows, "No drafts found.");
            Ok(())
        }
        DraftCmd::Get { draft_id, set } => {
            let set_id = AppConfig::resolve_set_id(set.as_deref())?;
            let result = api.get_draft(&set_id, draft_id).await?;
            print_json(&result);
            Ok(())
        }
        DraftCmd::Edit {
            draft_id,
            set,
            content,
            publish_at,
            tag,
            share,
        } => {
            let set_id = AppConfig::resolve_set_id(set.as_deref())?;
            let mut body = serde_json::json!({});
            if let Some(c) = content {
                body["content"] = serde_json::json!(c);
            }
            if let Some(pa) = publish_at {
                body["publish_at"] = serde_json::json!(pa);
            }
            if !tag.is_empty() {
                body["tags"] = serde_json::json!(tag);
            }
            if let Some(s) = share {
                body["share"] = serde_json::json!(s);
            }
            let result = api.update_draft(&set_id, draft_id, &body).await?;
            if cli.json {
                print_json(&result);
            } else if !cli.quiet {
                print_success("Draft updated.");
            }
            Ok(())
        }
        DraftCmd::Delete {
            draft_id,
            set,
            force,
        } => {
            let set_id = AppConfig::resolve_set_id(set.as_deref())?;
            if !force {
                let confirm = dialoguer::Confirm::new()
                    .with_prompt(format!("Delete draft {draft_id}?"))
                    .default(false)
                    .interact()
                    .map_err(|e| AppError::Io(std::io::Error::other(e)))?;
                if !confirm {
                    println!("Cancelled.");
                    return Ok(());
                }
            }
            api.delete_draft(&set_id, draft_id).await?;
            if !cli.quiet {
                print_success("Draft deleted.");
            }
            Ok(())
        }
        DraftCmd::Publish { draft_id, set } => {
            let set_id = AppConfig::resolve_set_id(set.as_deref())?;
            let body = serde_json::json!({"publish_at": "now"});
            let result = api.update_draft(&set_id, draft_id, &body).await?;
            if cli.json {
                print_json(&result);
            } else if !cli.quiet {
                print_success("Draft published.");
            }
            Ok(())
        }
        DraftCmd::Schedule {
            draft_id,
            time,
            set,
        } => {
            let set_id = AppConfig::resolve_set_id(set.as_deref())?;
            let body = serde_json::json!({"publish_at": time});
            let result = api.update_draft(&set_id, draft_id, &body).await?;
            if cli.json {
                print_json(&result);
            } else if !cli.quiet {
                print_success("Draft scheduled.");
            }
            Ok(())
        }
    }
}

#[allow(clippy::too_many_arguments)]
async fn cmd_draft_create(
    cli: &Cli,
    api: &impl TypefullyApi,
    set: Option<&str>,
    content: Option<&str>,
    platforms: &[typefully::api::Platform],
    publish_at: Option<&str>,
    tags: &[String],
    media_ids: &[String],
) -> Result<(), AppError> {
    let set_id = AppConfig::resolve_set_id(set)?;

    let text = if let Some(c) = content {
        c.to_string()
    } else {
        if std::io::stdin().is_terminal() {
            return Err(AppError::Usage(
                "No content provided. Use --content or pipe text via stdin.".into(),
            ));
        }
        let mut buf = String::new();
        std::io::stdin().read_to_string(&mut buf)?;
        buf.trim().to_string()
    };

    let posts: Vec<&str> = text
        .split("\n---\n")
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect();

    let platform_strs: Vec<String> = platforms.iter().map(ToString::to_string).collect();
    let mut body = serde_json::json!({"platforms": platform_strs});

    if posts.len() > 1 {
        body["posts"] = serde_json::json!(
            posts
                .iter()
                .map(|p| serde_json::json!({"content": p}))
                .collect::<Vec<_>>()
        );
    } else {
        body["content"] = serde_json::json!(text);
    }

    if let Some(pa) = publish_at {
        body["publish_at"] = serde_json::json!(pa);
    }
    if !tags.is_empty() {
        body["tags"] = serde_json::json!(tags);
    }
    if !media_ids.is_empty() {
        body["media"] = serde_json::json!(media_ids);
    }

    let result = api.create_draft(&set_id, &body).await?;
    if cli.json {
        print_json(&result);
    } else if !cli.quiet {
        let id = json_str(&result, "id");
        print_success(&format!("Draft created (ID: {id})"));
    }
    Ok(())
}

async fn cmd_media(cli: &Cli, api: &impl TypefullyApi, cmd: &MediaCmd) -> Result<(), AppError> {
    match cmd {
        MediaCmd::Upload { file, set } => {
            let set_id = AppConfig::resolve_set_id(set.as_deref())?;
            let file_name = file.file_name().and_then(|n| n.to_str()).unwrap_or("file");
            let data = tokio::fs::read(file).await?;
            let content_type = match file.extension().and_then(|e| e.to_str()) {
                Some("png") => "image/png",
                Some("jpg" | "jpeg") => "image/jpeg",
                Some("gif") => "image/gif",
                Some("webp") => "image/webp",
                Some("mp4") => "video/mp4",
                _ => "application/octet-stream",
            };

            let upload_info = api
                .create_media_upload(&set_id, file_name, content_type)
                .await?;
            let presigned_url = upload_info
                .get("presigned_url")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    AppError::Api(ApiError::Response {
                        status: 0,
                        message: "No presigned_url in response".into(),
                    })
                })?;
            let media_id = json_str(&upload_info, "media_id");

            info!(media_id = %media_id, "Uploading to S3");
            api.upload_to_presigned(presigned_url, data, content_type)
                .await?;

            if !cli.quiet {
                eprint!("Processing");
            }
            for _ in 0..60 {
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                let status = api.get_media_status(&set_id, &media_id).await?;
                let state = json_str(&status, "status");
                if state == "ready" {
                    if !cli.quiet {
                        eprintln!();
                    }
                    if cli.json {
                        print_json(&status);
                    } else {
                        print_success(&format!("Media uploaded (ID: {media_id})"));
                    }
                    return Ok(());
                }
                if state == "error" {
                    return Err(AppError::Api(ApiError::Response {
                        status: 0,
                        message: "Media processing failed.".into(),
                    }));
                }
                if !cli.quiet {
                    eprint!(".");
                }
            }
            Err(AppError::Api(ApiError::Response {
                status: 0,
                message: "Media processing timed out.".into(),
            }))
        }
    }
}

async fn cmd_tags(cli: &Cli, api: &impl TypefullyApi, cmd: &TagsCmd) -> Result<(), AppError> {
    match cmd {
        TagsCmd::List { set } => {
            let set_id = AppConfig::resolve_set_id(set.as_deref())?;
            let result = api.list_tags(&set_id).await?;
            if cli.json {
                print_json(&result);
                return Ok(());
            }
            let empty = vec![];
            let arr = result.as_array().unwrap_or(&empty);
            let rows: Vec<TagRow> = arr
                .iter()
                .map(|t| TagRow {
                    id: json_str(t, "id"),
                    name: json_str(t, "name"),
                })
                .collect();
            print_table(&rows, "No tags found.");
            Ok(())
        }
        TagsCmd::Create { name, set } => {
            let set_id = AppConfig::resolve_set_id(set.as_deref())?;
            let result = api.create_tag(&set_id, name).await?;
            if cli.json {
                print_json(&result);
            } else if !cli.quiet {
                print_success(&format!("Tag '{name}' created."));
            }
            Ok(())
        }
    }
}
