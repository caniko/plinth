use anyhow::Result;
use clap::{Parser, Subcommand};

mod api_client;
mod commands;
mod image_scanner;
mod immich_client;
mod prompts;
mod typst_processor;
mod ui;

use api_client::ApiClient;
#[cfg(feature = "brick-activity")]
use commands::activity;
use commands::content;
use commands::init;
#[cfg(feature = "brick-portfolio")]
use commands::portfolio;
#[cfg(feature = "brick-blog")]
use commands::publish::publish_article;
#[cfg(feature = "brick-blog")]
use commands::tags;
#[cfg(feature = "brick-todo")]
use commands::todo;

#[derive(Parser)]
#[command(name = "plinth")]
#[command(about = "Plinth CLI - publish and manage articles", long_about = None)]
#[command(version)]
pub(crate) struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// API base URL (can also be set via PLINTH_API_URL env var)
    #[arg(long, env = "PLINTH_API_URL", default_value = "http://localhost:3000")]
    api_url: String,

    /// API key for authentication (can also be set via PLINTH_API_KEY env var)
    #[arg(long, env = "PLINTH_API_KEY")]
    api_key: Option<String>,

    /// Immich server URL for image uploads (can also be set via IMMICH_API_URL env var)
    #[arg(long, env = "IMMICH_API_URL")]
    immich_url: Option<String>,

    /// Immich API key for image uploads (can also be set via IMMICH_API_KEY env var)
    #[arg(long, env = "IMMICH_API_KEY")]
    immich_api_key: Option<String>,
}

#[derive(Subcommand)]
pub(crate) enum Commands {
    /// Publish an article from a markdown or typst file
    #[cfg(feature = "brick-blog")]
    Publish {
        /// Path to the article file (.md or .typ)
        file: Option<String>,
        /// Interactive mode — prompts for all fields
        #[arg(short, long)]
        interactive: bool,
        /// Skip image uploads (publish without uploading local images)
        #[arg(long)]
        skip_images: bool,
    },
    /// List published articles (future)
    #[cfg(feature = "brick-blog")]
    List,
    /// Delete an article by slug (future)
    #[cfg(feature = "brick-blog")]
    Delete {
        /// Article slug
        slug: String,
    },
    /// Tag management
    #[cfg(feature = "brick-blog")]
    #[command(subcommand)]
    Tag(TagCommands),
    /// Site content management
    #[command(subcommand)]
    Content(ContentCommands),
    /// Bucket list / TODO management
    #[cfg(feature = "brick-todo")]
    #[command(subcommand)]
    Todo(TodoCommands),
    /// Portfolio item publishing
    #[cfg(feature = "brick-portfolio")]
    #[command(subcommand)]
    Portfolio(PortfolioCommands),
    /// External activity (PRs/issues) management
    #[cfg(feature = "brick-activity")]
    #[command(subcommand)]
    Activity(ActivityCommands),
    /// Create a new file from a built-in template
    Init {
        /// Template to use (post, bucket-list)
        template: String,
        /// Output file path (defaults to ./<template>.typ)
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Validate a plinth.toml configuration file
    CheckConfig {
        /// Path to the TOML file (default: plinth.toml or PLINTH_CONFIG env)
        path: Option<String>,
    },
    /// Check instance health
    Status,
    /// Generate shell completions
    Completions {
        /// Shell to generate completions for (bash, zsh, fish, elvish, nushell)
        shell: String,
    },
}

#[cfg(feature = "brick-activity")]
#[derive(Subcommand)]
enum ActivityCommands {
    /// Fetch a PR/issue from a forge, embed it, and publish it
    #[command(group(
        clap::ArgGroup::new("ref_kind")
            .required(true)
            .args(["pr", "issue"]),
    ))]
    Add {
        /// Forge to fetch from
        #[arg(long, value_enum)]
        forge: ForgeArg,
        /// Repository in owner/name form (e.g. cli/cli)
        #[arg(long)]
        repo: String,
        /// Pull-request number (mutually exclusive with --issue)
        #[arg(long, group = "ref_kind")]
        pr: Option<u32>,
        /// Issue number (mutually exclusive with --pr)
        #[arg(long, group = "ref_kind")]
        issue: Option<u32>,
        /// Curated impact score, 1..=10
        #[arg(long, value_parser = clap::value_parser!(u8).range(1..=10))]
        impact: u8,
        /// Mark as a featured (home-strip) entry
        #[arg(long)]
        featured: bool,
    },
    /// Remove an activity by numeric id
    Remove {
        /// Numeric id (e.g. 42)
        id: i64,
    },
    /// Update an existing activity's impact and/or featured flag
    Update {
        /// Numeric id (e.g. 42)
        id: i64,
        /// New impact score, 1..=10
        #[arg(long, value_parser = clap::value_parser!(u8).range(1..=10))]
        impact: Option<u8>,
        /// New featured flag (true/false)
        #[arg(long)]
        featured: Option<bool>,
    },
    /// List all activities
    List,
}

/// CLI-facing forge selector. Maps to `plinth_shared::Forge`.
#[cfg(feature = "brick-activity")]
#[derive(Clone, Copy, clap::ValueEnum)]
enum ForgeArg {
    Github,
    Codeberg,
}

#[cfg(feature = "brick-activity")]
impl From<ForgeArg> for plinth_shared::Forge {
    fn from(a: ForgeArg) -> Self {
        match a {
            ForgeArg::Github => plinth_shared::Forge::GitHub,
            ForgeArg::Codeberg => plinth_shared::Forge::Codeberg,
        }
    }
}

#[cfg(feature = "brick-portfolio")]
#[derive(Subcommand)]
enum PortfolioCommands {
    /// Publish or update one portfolio item from a portfolio.toml manifest
    Publish {
        /// Path to the portfolio.toml file
        path: String,
    },
}

#[cfg(feature = "brick-todo")]
#[derive(Subcommand)]
enum TodoCommands {
    /// Create a new TODO item
    Create {
        /// Item title
        title: Option<String>,
        /// Short description
        description: Option<String>,
        /// Interactive mode — prompts for all fields
        #[arg(short, long)]
        interactive: bool,
        /// Optional Typst content file for long-form description
        #[arg(long)]
        content_file: Option<String>,
        /// Tags (comma-separated)
        #[arg(long)]
        tags: Option<String>,
        /// Display order
        #[arg(long, default_value = "0")]
        order: i32,
    },
    /// Update an existing TODO item
    Update {
        /// Slug of the TODO to update
        slug: String,
        /// Mark as completed
        #[arg(long)]
        complete: bool,
        /// Mark as not completed
        #[arg(long)]
        uncomplete: bool,
        /// Update title
        #[arg(long)]
        title: Option<String>,
        /// Update description
        #[arg(long)]
        description: Option<String>,
        /// Update content from Typst file
        #[arg(long)]
        content_file: Option<String>,
        /// Update tags (comma-separated)
        #[arg(long)]
        tags: Option<String>,
        /// Update display order
        #[arg(long)]
        order: Option<i32>,
    },
    /// Delete a TODO item
    Delete {
        /// Item slug
        slug: String,
    },
    /// List all TODO items
    List,
}

#[cfg(feature = "brick-blog")]
#[derive(Subcommand)]
enum TagCommands {
    /// List all tags with post counts
    List,
    /// Add a tag to a post
    Add {
        /// Post slug
        post: String,
        /// Tag name
        tag: String,
    },
    /// Remove a tag from a post
    Remove {
        /// Post slug
        post: String,
        /// Tag slug
        tag: String,
    },
}

#[derive(Subcommand)]
enum ContentCommands {
    /// Set site content from a Typst file
    Set {
        /// Content key (e.g., "home-intro", "about")
        key: String,
        /// Path to the Typst file (.typ)
        file: String,
    },
    /// Get current site content
    Get {
        /// Content key
        key: String,
    },
}

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        ui::print_error(&err);
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    let cli = Cli::parse();

    // Commands that don't need API credentials
    if let Commands::Init { template, output } = &cli.command {
        return init::create_from_template(template, output.as_deref());
    }
    if let Commands::CheckConfig { path } = &cli.command {
        return commands::check_config::validate(path.as_deref());
    }
    if let Commands::Completions { shell } = &cli.command {
        return commands::completions::generate_completions(shell);
    }
    if let Commands::Status = &cli.command {
        return commands::status::check_status(&cli.api_url).await;
    }
    #[cfg(feature = "brick-activity")]
    if let Commands::Activity(ActivityCommands::List) = &cli.command {
        let api_client = ApiClient::new(cli.api_url.clone(), String::new())?;
        return activity::list(&api_client).await;
    }

    // Get API key from CLI args or environment
    let api_key = cli
        .api_key
        .or_else(|| std::env::var("PLINTH_API_KEY").ok())
        .ok_or_else(|| {
            anyhow::anyhow!(
                "API key required. Set PLINTH_API_KEY environment variable or use --api-key flag"
            )
        })?;

    // Create API client
    let api_client = ApiClient::new(cli.api_url, api_key)?;

    // Build optional Immich client
    let immich_client = match (cli.immich_url, cli.immich_api_key) {
        (Some(url), Some(key)) => Some(immich_client::ImmichClient::new(url, key)?),
        _ => None,
    };

    // Execute command
    match &cli.command {
        Commands::Init { .. }
        | Commands::CheckConfig { .. }
        | Commands::Completions { .. }
        | Commands::Status => unreachable!(),

        #[cfg(feature = "brick-blog")]
        Commands::Publish {
            file,
            interactive,
            skip_images,
        } => {
            if *interactive {
                commands::publish::interactive_publish(&api_client, immich_client.as_ref()).await?;
            } else {
                let file = file.as_deref().ok_or_else(|| {
                    anyhow::anyhow!("File path required. Use -i for interactive mode.")
                })?;
                publish_article(file, &api_client, immich_client.as_ref(), *skip_images).await?;
            }
        }

        #[cfg(feature = "brick-blog")]
        Commands::List => {
            let sp = ui::spinner("Fetching articles...");
            let articles = api_client.list_articles().await?;
            sp.finish_and_clear();

            if articles.is_empty() {
                ui::status("Info", "No articles found.");
            } else {
                ui::status("Found", &format!("{} article(s)", articles.len()));
                for article in articles {
                    println!("  {}", serde_json::to_string_pretty(&article)?);
                }
            }
        }

        #[cfg(feature = "brick-blog")]
        Commands::Delete { slug } => {
            let sp = ui::spinner(&format!("Deleting article '{slug}'..."));
            api_client.delete_article(slug).await?;
            sp.finish_and_clear();
            ui::success(&format!("Article '{slug}' deleted"));
        }

        Commands::Content(content_cmd) => match content_cmd {
            ContentCommands::Set { key, file } => {
                content::set_content(key, file, &api_client).await?;
            }
            ContentCommands::Get { key } => {
                content::get_content(key, &api_client).await?;
            }
        },

        #[cfg(feature = "brick-blog")]
        Commands::Tag(tag_cmd) => match tag_cmd {
            TagCommands::List => {
                tags::list_tags(&api_client).await?;
            }
            TagCommands::Add { post, tag } => {
                tags::add_tag(post, tag, &api_client).await?;
            }
            TagCommands::Remove { post, tag } => {
                tags::remove_tag(post, tag, &api_client).await?;
            }
        },

        #[cfg(feature = "brick-todo")]
        Commands::Todo(todo_cmd) => match todo_cmd {
            TodoCommands::Create {
                title,
                description,
                interactive,
                content_file,
                tags,
                order,
            } => {
                if *interactive {
                    todo::interactive_create_todo(&api_client).await?;
                } else {
                    let title = title.as_deref().ok_or_else(|| {
                        anyhow::anyhow!("Title required. Use -i for interactive mode.")
                    })?;
                    let description = description.as_deref().ok_or_else(|| {
                        anyhow::anyhow!("Description required. Use -i for interactive mode.")
                    })?;
                    todo::create_todo(
                        title,
                        description,
                        content_file.as_deref(),
                        tags.as_deref(),
                        *order,
                        &api_client,
                    )
                    .await?;
                }
            }
            TodoCommands::Update {
                slug,
                complete,
                uncomplete,
                title,
                description,
                content_file,
                tags,
                order,
            } => {
                todo::update_todo(
                    slug,
                    *complete,
                    *uncomplete,
                    title.as_deref(),
                    description.as_deref(),
                    content_file.as_deref(),
                    tags.as_deref(),
                    *order,
                    &api_client,
                )
                .await?;
            }
            TodoCommands::Delete { slug } => {
                todo::delete_todo(slug, &api_client).await?;
            }
            TodoCommands::List => {
                todo::list_todos(&api_client).await?;
            }
        },

        #[cfg(feature = "brick-portfolio")]
        Commands::Portfolio(portfolio_cmd) => match portfolio_cmd {
            PortfolioCommands::Publish { path } => {
                portfolio::publish(std::path::Path::new(path), &api_client).await?;
            }
        },

        #[cfg(feature = "brick-activity")]
        Commands::Activity(activity_cmd) => match activity_cmd {
            ActivityCommands::Add {
                forge,
                repo,
                pr,
                issue,
                impact,
                featured,
            } => {
                activity::add(
                    (*forge).into(),
                    repo,
                    *pr,
                    *issue,
                    *impact,
                    *featured,
                    &api_client,
                )
                .await?;
            }
            ActivityCommands::Remove { id } => {
                activity::remove(*id, &api_client).await?;
            }
            ActivityCommands::Update {
                id,
                impact,
                featured,
            } => {
                activity::update(*id, *impact, *featured, &api_client).await?;
            }
            ActivityCommands::List => {
                activity::list(&api_client).await?;
            }
        },
    }

    Ok(())
}
