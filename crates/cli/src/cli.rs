use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "plinth")]
#[command(about = "Plinth CLI - publish and manage articles", long_about = None)]
#[command(version)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: Commands,

    /// API base URL (can also be set via PLINTH_API_URL env var)
    #[arg(long, env = "PLINTH_API_URL", default_value = "http://localhost:3000")]
    pub(crate) api_url: String,

    /// API key for authentication (can also be set via PLINTH_API_KEY env var)
    #[arg(long, env = "PLINTH_API_KEY")]
    pub(crate) api_key: Option<String>,

    /// Immich server URL for image uploads (can also be set via IMMICH_API_URL env var)
    #[arg(long, env = "IMMICH_API_URL")]
    pub(crate) immich_url: Option<String>,

    /// Immich API key for image uploads (can also be set via IMMICH_API_KEY env var)
    #[arg(long, env = "IMMICH_API_KEY")]
    pub(crate) immich_api_key: Option<String>,
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
    /// Check registered Plinth and project-site deployments
    CheckSites {
        /// Path to the site-checks TOML file
        #[arg(long, env = "PLINTH_SITE_CHECK_CONFIG")]
        config: Option<String>,
        /// Emit machine-readable JSON
        #[arg(long)]
        json: bool,
    },
    /// Generate shell completions
    Completions {
        /// Shell to generate completions for (bash, zsh, fish, elvish, nushell)
        shell: String,
    },
}

#[cfg(feature = "brick-activity")]
#[derive(Subcommand)]
pub(crate) enum ActivityCommands {
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
pub(crate) enum ForgeArg {
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
pub(crate) enum PortfolioCommands {
    /// Publish or update one portfolio item from a portfolio.toml manifest
    Publish {
        /// Path to the portfolio.toml file
        path: String,
    },
}

#[cfg(feature = "brick-todo")]
#[derive(Subcommand)]
pub(crate) enum TodoCommands {
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
pub(crate) enum TagCommands {
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
pub(crate) enum ContentCommands {
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
