use surrealdb::engine::local::{Db, RocksDb};
use surrealdb::RecordId;
use surrealdb::Surreal;
use tracing::{info, instrument};

/// Initialize SurrealDB connection
/// For development, uses file-based storage: database.db
/// Can be switched to memory or remote server for production
#[instrument]
pub async fn init_db() -> Result<Surreal<Db>, surrealdb::Error> {
    // Read configuration from environment variables
    let db_path = std::env::var("SURREALDB_PATH").unwrap_or_else(|_| "database.db".to_string());
    let namespace =
        std::env::var("SURREALDB_NAMESPACE").unwrap_or_else(|_| "personal_website".to_string());
    let database = std::env::var("SURREALDB_DATABASE").unwrap_or_else(|_| "main".to_string());

    info!(db_path = %db_path, namespace = %namespace, database = %database, "Connecting to SurrealDB");

    // Connect to SurrealDB with file-based storage
    let db = Surreal::new::<RocksDb>(&db_path).await?;

    // Select namespace and database
    db.use_ns(&namespace).use_db(&database).await?;

    info!(
        "SurrealDB connected: file://{}, namespace: {}, database: {}",
        db_path, namespace, database
    );

    Ok(db)
}

/// Initialize database schema
/// Creates tables and indexes if they don't exist
#[instrument(skip(db))]
pub async fn init_schema(db: &Surreal<Db>) -> Result<(), surrealdb::Error> {
    info!("Initializing database schema...");

    // Create blog_posts table with schema
    db.query(r#"
        DEFINE TABLE blog_posts SCHEMAFULL;

        DEFINE FIELD slug ON TABLE blog_posts TYPE string;
        DEFINE FIELD title ON TABLE blog_posts TYPE string;
        DEFINE FIELD content ON TABLE blog_posts TYPE string;
        DEFINE FIELD html_content ON TABLE blog_posts TYPE string;
        DEFINE FIELD published_at ON TABLE blog_posts TYPE datetime;
        DEFINE FIELD updated_at ON TABLE blog_posts TYPE option<datetime>;
        DEFINE FIELD author ON TABLE blog_posts TYPE string;
        DEFINE FIELD tags ON TABLE blog_posts TYPE array<string>;
        DEFINE FIELD featured ON TABLE blog_posts TYPE bool DEFAULT false;
        DEFINE FIELD published ON TABLE blog_posts TYPE bool DEFAULT true;
        DEFINE FIELD reading_time_minutes ON TABLE blog_posts TYPE int;
        DEFINE FIELD embedding ON TABLE blog_posts TYPE option<array<float>>;

        -- Indexes
        DEFINE INDEX blog_posts_slug_idx ON TABLE blog_posts COLUMNS slug UNIQUE;
        DEFINE INDEX blog_posts_published_at_idx ON TABLE blog_posts COLUMNS published_at;
        DEFINE INDEX blog_posts_tags_idx ON TABLE blog_posts COLUMNS tags;

        -- Vector index for semantic search (SurrealDB 2.0+ syntax)
        -- DEFINE INDEX blog_posts_embedding_idx ON TABLE blog_posts COLUMNS embedding MTREE DIMENSION 384;
    "#).await?;

    // Create portfolio_items table with schema
    db.query(
        r#"
        DEFINE TABLE portfolio_items SCHEMAFULL;

        DEFINE FIELD slug ON TABLE portfolio_items TYPE string;
        DEFINE FIELD title ON TABLE portfolio_items TYPE string;
        DEFINE FIELD description ON TABLE portfolio_items TYPE string;
        DEFINE FIELD content ON TABLE portfolio_items TYPE option<string>;
        DEFINE FIELD html_content ON TABLE portfolio_items TYPE option<string>;
        DEFINE FIELD tech_stack ON TABLE portfolio_items TYPE array<string>;
        DEFINE FIELD link ON TABLE portfolio_items TYPE option<string>;
        DEFINE FIELD demo ON TABLE portfolio_items TYPE option<string>;
        DEFINE FIELD image_url ON TABLE portfolio_items TYPE option<string>;
        DEFINE FIELD date ON TABLE portfolio_items TYPE datetime;
        DEFINE FIELD featured ON TABLE portfolio_items TYPE bool DEFAULT false;
        DEFINE FIELD order ON TABLE portfolio_items TYPE int DEFAULT 0;

        -- Indexes
        DEFINE INDEX portfolio_items_slug_idx ON TABLE portfolio_items COLUMNS slug UNIQUE;
        DEFINE INDEX portfolio_items_date_idx ON TABLE portfolio_items COLUMNS date;
        DEFINE INDEX portfolio_items_order_idx ON TABLE portfolio_items COLUMNS order;
    "#,
    )
    .await?;

    info!("Database schema initialized");

    Ok(())
}

/// Seed the database with sample data for development
#[instrument(skip(db))]
pub async fn seed_sample_data(db: &Surreal<Db>) -> Result<(), surrealdb::Error> {
    info!("Seeding sample data...");

    // Check if we already have data
    let existing_posts: Vec<RecordId> = db
        .query("SELECT id FROM blog_posts LIMIT 1")
        .await?
        .take(0)?;

    if !existing_posts.is_empty() {
        info!("   Database already has data, skipping seed");
        return Ok(());
    }

    // Sample blog post
    db.query(r##"
        CREATE blog_posts CONTENT {
            slug: "welcome-to-my-blog",
            title: "Welcome to My Blog",
            content: "# Welcome!\n\nThis is my first blog post built with Rust, Leptos, and SurrealDB!",
            html_content: "<h1>Welcome!</h1><p>This is my first blog post built with Rust, Leptos, and SurrealDB!</p>",
            published_at: time::now(),
            author: "Author Name",
            tags: ["meta", "welcome"],
            featured: true,
            published: true,
            reading_time_minutes: 1,
            embedding: NONE
        };
    "##).await?;

    // Sample portfolio item
    db.query(
        r##"
        CREATE portfolio_items CONTENT {
            slug: "sample-project",
            title: "Sample Project",
            description: "A sample portfolio project to demonstrate the system",
            content: "# Sample Project\n\nThis is a sample project description.",
            html_content: "<h1>Sample Project</h1><p>This is a sample project description.</p>",
            tech_stack: ["Rust", "Leptos", "SurrealDB"],
            link: "https://github.com/user/project",
            demo: NONE,
            image_url: NONE,
            date: time::now(),
            featured: true,
            order: 0
        };
    "##,
    )
    .await?;

    info!("Sample data seeded successfully");

    Ok(())
}

/// Helper to convert SurrealDB RecordId to string ID
#[allow(dead_code)]
pub fn record_id_to_string(record: &RecordId) -> String {
    record.to_string()
}

/// Helper to parse string ID to RecordId
#[allow(dead_code)]
pub fn string_to_record_id(id: &str) -> Result<RecordId, String> {
    let parts: Vec<&str> = id.splitn(2, ':').collect();
    if parts.len() != 2 {
        return Err(format!("Invalid ID format: {}", id));
    }

    Ok(RecordId::from_table_key(parts[0], parts[1]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_id_conversion() {
        let record = RecordId::from_table_key("blog_posts", "my-post");
        let id_string = record_id_to_string(&record);
        assert!(id_string.contains("blog_posts"));
        assert!(id_string.contains("my-post"));

        let parsed = string_to_record_id(&id_string).unwrap();
        assert_eq!(parsed.table(), "blog_posts");
    }
}
