use anyhow::Result;
use clap::{Parser, Subcommand};

mod api_client;
mod commands;

use api_client::ApiClient;
use commands::publish::publish_article;

#[derive(Parser)]
#[command(name = "blog-cli")]
#[command(about = "CLI tool for publishing articles to the blog", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// API base URL (can also be set via BLOG_API_URL env var)
    #[arg(long, env = "BLOG_API_URL", default_value = "http://localhost:3000")]
    api_url: String,

    /// API key for authentication (can also be set via BLOG_API_KEY env var)
    #[arg(long, env = "BLOG_API_KEY")]
    api_key: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Publish an article from a markdown file
    Publish {
        /// Path to the markdown file
        file: String,
    },
    /// List published articles (future)
    List,
    /// Delete an article by slug (future)
    Delete {
        /// Article slug
        slug: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Get API key from CLI args or environment
    let api_key = cli.api_key
        .or_else(|| std::env::var("BLOG_API_KEY").ok())
        .ok_or_else(|| anyhow::anyhow!(
            "API key required. Set BLOG_API_KEY environment variable or use --api-key flag"
        ))?;

    // Create API client
    let api_client = ApiClient::new(cli.api_url, api_key);

    // Execute command
    match &cli.command {
        Commands::Publish { file } => {
            publish_article(file, &api_client).await?;
        }
        Commands::List => {
            println!("📋 Listing articles...");
            println!();

            match api_client.list_articles().await {
                Ok(articles) => {
                    println!("Found {} article(s):", articles.len());
                    for article in articles {
                        println!("  - {}", serde_json::to_string_pretty(&article)?);
                    }
                }
                Err(e) => {
                    eprintln!("❌ Failed to list articles: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::Delete { slug } => {
            println!("🗑️  Deleting article: {}", slug);
            println!();

            match api_client.delete_article(slug).await {
                Ok(_) => {
                    println!("✅ Article deleted successfully!");
                }
                Err(e) => {
                    eprintln!("❌ Failed to delete article: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }

    Ok(())
}
