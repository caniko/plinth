mod config;
mod comfyui;
mod immich;
mod format;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "pcomfy", about = "Semi-automated Plinth article image generator via ComfyUI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate images for articles that don't have them yet
    Generate {
        /// Comma-separated article slugs (default: all without images)
        #[arg(long)]
        articles: Option<String>,

        /// Images per article (1-4)
        #[arg(long, default_value = "3")]
        count: u32,

        /// Override workflow name
        #[arg(long)]
        workflow: Option<String>,

        /// Output directory for generated images
        #[arg(long, default_value = "/tmp/pcomfy")]
        output_dir: PathBuf,

        /// Dry run — only list articles needing images
        #[arg(long)]
        dry_run: bool,

        /// ComfyUI base URL
        #[arg(long, env = "COMFYUI_URL")]
        comfyui_url: Option<String>,

        /// Immich API base URL
        #[arg(long, env = "IMMICH_URL")]
        immich_url: Option<String>,

        /// Immich API key
        #[arg(long, env = "IMMICH_API_KEY")]
        immich_api_key: Option<String>,

        /// Plinth articles directory
        #[arg(long, env = "PLINTH_ARTICLES_DIR")]
        articles_dir: Option<PathBuf>,
    },

    /// Insert or update image references in an article
    Format {
        /// Article slug
        slug: String,

        /// Image URL (Immich proxy path, e.g. /api/images/uuid)
        #[arg(short, long)]
        image_url: String,

        /// Placement: hero, inline, gallery
        #[arg(short, long, default_value = "hero")]
        placement: String,
    },

    /// Show article → image status table
    Status {
        /// Plinth articles directory
        #[arg(long, env = "PLINTH_ARTICLES_DIR")]
        articles_dir: Option<PathBuf>,
    },

    /// Probe ComfyUI and list available workflows/models
    Probe {
        /// ComfyUI base URL
        #[arg(long, env = "COMFYUI_URL")]
        comfyui_url: Option<String>,

        /// Immich API base URL
        #[arg(long, env = "IMMICH_URL")]
        immich_url: Option<String>,
    },

    /// List available ComfyUI workflows from /object_info
    Workflows {
        /// ComfyUI base URL
        #[arg(long, env = "COMFYUI_URL")]
        comfyui_url: Option<String>,

        /// Filter workflows by name pattern
        #[arg(long)]
        filter: Option<String>,
    },

    /// Create default config file
    Init,
}

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        eprintln!("Error: {err:#}");
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Generate {
            articles,
            count,
            workflow,
            output_dir,
            dry_run,
            comfyui_url,
            immich_url,
            immich_api_key,
            articles_dir,
        } => {
            let cfg = config::Config::load(comfyui_url, immich_url, immich_api_key, articles_dir)?;
            generate::run(cfg, articles, count, workflow, output_dir, dry_run).await?;
        }

        Commands::Format { slug, image_url, placement } => {
            format::run(&slug, &image_url, &placement)?;
        }

        Commands::Status { articles_dir } => {
            let cfg = config::Config::load(None, None, None, articles_dir)?;
            status(cfg).await?;
        }

        Commands::Probe { comfyui_url, immich_url } => {
            let comfyui = comfyui_url.unwrap_or_else(|| "http://localhost:8188".into());
            let immich = immich_url.unwrap_or_else(|| "https://immich.candee.baby/api".into());
            probe(&comfyui, &immich).await?;
        }

        Commands::Workflows { comfyui_url, filter } => {
            let url = comfyui_url.unwrap_or_else(|| "http://localhost:8188".into());
            list_workflows(&url, filter.as_deref()).await?;
        }

        Commands::Init => {
            config::create_default()?;
            println!("Created ~/.config/pcomfy/config.toml");
        }
    }

    Ok(())
}

async fn status(cfg: config::Config) -> Result<()> {
    let articles = format::scan_articles(&cfg.articles_dir)?;

    println!("{:<30} {:<8} {:<40}", "Article", "Images", "Title");
    println!("{}", "-".repeat(80));
    for a in &articles {
        let img_count = a.image_count();
        let img_str = if img_count == 0 {
            console::style("none").red().to_string()
        } else {
            console::style(img_count.to_string()).green().to_string()
        };
        println!("{:<30} {:<8} {:<40}", a.slug, img_str, a.title);
    }
    println!();
    let total = articles.len();
    let with_images = articles.iter().filter(|a| a.image_count() > 0).count();
    println!(
        "{} / {} articles have images",
        console::style(with_images).green(),
        total
    );

    Ok(())
}

async fn probe(comfyui_url: &str, immich_url: &str) -> Result<()> {
    println!("🔌 Proving connectivity...\n");

    let comfyui_ok = comfyui::probe(comfyui_url).await;
    match comfyui_ok {
        Ok(info) => {
            println!("  ✓ ComfyUI at {}", console::style(comfyui_url).cyan());
            println!("     Version: {}", info.version);
            println!("     Device:  {}", info.device);
            println!("     Models:  {} available", info.model_count);
        }
        Err(e) => {
            println!("  ✗ ComfyUI at {}: {e}", console::style(comfyui_url).red());
        }
    }

    let immich_ok = immich::probe(immich_url).await;
    match immich_ok {
        Ok(version) => {
            println!("  ✓ Immich at {}", console::style(immich_url).cyan());
            println!("     Version: {version}");
        }
        Err(e) => {
            println!("  ✗ Immich at {}: {e}", console::style(immich_url).red());
        }
    }

    Ok(())
}

async fn list_workflows(url: &str, filter: Option<&str>) -> Result<()> {
    let workflows = comfyui::list_workflows(url).await?;

    println!("Available ComfyUI workflows:\n");
    for w in &workflows {
        if let Some(f) = filter {
            if !w.name.contains(f) {
                continue;
            }
        }
        println!("  {} — {} nodes", console::style(&w.name).cyan(), w.node_count);
    }
    println!("\nTotal: {} workflows", workflows.len());

    Ok(())
}
