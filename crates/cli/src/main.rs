use anyhow::Result;
use clap::{Parser, Subcommand};

mod api_client;
mod commands;
mod image_scanner;
mod immich_client;
mod typst_processor;
mod ui;

use api_client::ApiClient;
use commands::content;
use commands::init;
use commands::publish::publish_article;
use commands::tags;
use commands::todo;

#[derive(Parser)]
#[command(name = "plinth-cli")]
#[command(about = "Plinth CLI - publish and manage articles", long_about = None)]
#[command(version)]
struct Cli {
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
enum Commands {
    /// Publish an article from a markdown or typst file
    Publish {
        /// Path to the article file (.md or .typ)
        file: String,
    },
    /// List published articles (future)
    List,
    /// Delete an article by slug (future)
    Delete {
        /// Article slug
        slug: String,
    },
    /// Tag management
    #[command(subcommand)]
    Tag(TagCommands),
    /// Site content management
    #[command(subcommand)]
    Content(ContentCommands),
    /// Bucket list / TODO management
    #[command(subcommand)]
    Todo(TodoCommands),
    /// Create a new file from a built-in template
    Init {
        /// Template to use (post, bucket-list)
        template: String,
        /// Output file path (defaults to ./<template>.typ)
        #[arg(short, long)]
        output: Option<String>,
    },
}

#[derive(Subcommand)]
enum TodoCommands {
    /// Create a new TODO item
    Create {
        /// Item title
        title: String,
        /// Short description
        description: String,
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
    let api_client = ApiClient::new(cli.api_url, api_key);

    // Build optional Immich client
    let immich_client = match (cli.immich_url, cli.immich_api_key) {
        (Some(url), Some(key)) => Some(immich_client::ImmichClient::new(url, key)?),
        _ => None,
    };

    // Execute command
    match &cli.command {
        Commands::Init { .. } => unreachable!(),
        Commands::Publish { file } => {
            publish_article(file, &api_client, immich_client.as_ref()).await?;
        }
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
        Commands::Todo(todo_cmd) => match todo_cmd {
            TodoCommands::Create {
                title,
                description,
                content_file,
                tags,
                order,
            } => {
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
    }

    Ok(())
}
