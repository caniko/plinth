mod cli;

use anyhow::Result;
use clap::Parser;

mod api_client;
mod commands;
mod image_scanner;
mod immich_client;
mod prompts;
mod typst_processor;
mod ui;

use api_client::ApiClient;
#[cfg(feature = "brick-activity")]
use cli::ActivityCommands;
#[cfg(feature = "brick-portfolio")]
use cli::PortfolioCommands;
#[cfg(feature = "brick-blog")]
use cli::TagCommands;
#[cfg(feature = "brick-todo")]
use cli::TodoCommands;
use cli::{Cli, Commands, ContentCommands};
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
    if let Commands::CheckSites { config, json } = &cli.command {
        return commands::check_sites::check_sites(config.as_deref(), *json).await;
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
        | Commands::Status
        | Commands::CheckSites { .. } => unreachable!(),

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
