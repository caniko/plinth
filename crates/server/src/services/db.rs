use surrealdb::RecordId;
use surrealdb::Surreal;
use surrealdb::engine::local::{Db, RocksDb};
use tracing::{info, instrument};

/// Initialize SurrealDB connection from config
#[instrument(skip(config))]
pub async fn init_db(
    config: &crate::config::DatabaseConfig,
) -> Result<Surreal<Db>, surrealdb::Error> {
    info!(db_path = %config.path, namespace = %config.namespace, database = %config.database, "Connecting to SurrealDB");

    // Connect to SurrealDB with file-based storage
    let db = Surreal::new::<RocksDb>(&config.path).await?;

    // Select namespace and database
    db.use_ns(&config.namespace)
        .use_db(&config.database)
        .await?;

    info!(
        "SurrealDB connected: file://{}, namespace: {}, database: {}",
        config.path, config.namespace, config.database
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
        DEFINE FIELD description ON TABLE blog_posts TYPE string DEFAULT "";
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
        DEFINE FIELD content_format ON TABLE blog_posts TYPE string DEFAULT "markdown";

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

    // Create site_content table for customizable page content
    db.query(
        r#"
        DEFINE TABLE site_content SCHEMAFULL;

        DEFINE FIELD key ON TABLE site_content TYPE string;
        DEFINE FIELD title ON TABLE site_content TYPE option<string>;
        DEFINE FIELD content ON TABLE site_content TYPE string;
        DEFINE FIELD html_content ON TABLE site_content TYPE string;
        DEFINE FIELD updated_at ON TABLE site_content TYPE datetime;

        DEFINE INDEX site_content_key_idx ON TABLE site_content COLUMNS key UNIQUE;
    "#,
    )
    .await?;

    // Create tags table and tagged graph relation
    db.query(
        r#"
        DEFINE TABLE tags SCHEMAFULL;

        DEFINE FIELD name ON TABLE tags TYPE string;
        DEFINE FIELD slug ON TABLE tags TYPE string;
        DEFINE FIELD created_at ON TABLE tags TYPE datetime;

        DEFINE INDEX tags_slug_idx ON TABLE tags COLUMNS slug UNIQUE;
        DEFINE INDEX tags_name_idx ON TABLE tags COLUMNS name UNIQUE;

        -- Graph relation: blog_posts -> tagged -> tags
        DEFINE TABLE tagged SCHEMAFULL TYPE RELATION FROM blog_posts TO tags;
        DEFINE FIELD created_at ON TABLE tagged TYPE datetime;
    "#,
    )
    .await?;

    // Create todos table for bucket list items
    db.query(
        r#"
        DEFINE TABLE todos SCHEMAFULL;

        DEFINE FIELD slug ON TABLE todos TYPE string;
        DEFINE FIELD title ON TABLE todos TYPE string;
        DEFINE FIELD description ON TABLE todos TYPE string;
        DEFINE FIELD content ON TABLE todos TYPE option<string>;
        DEFINE FIELD html_content ON TABLE todos TYPE option<string>;
        DEFINE FIELD tags ON TABLE todos TYPE array<string>;
        DEFINE FIELD completed ON TABLE todos TYPE bool DEFAULT false;
        DEFINE FIELD completed_at ON TABLE todos TYPE option<datetime>;
        DEFINE FIELD created_at ON TABLE todos TYPE datetime;
        DEFINE FIELD order ON TABLE todos TYPE int DEFAULT 0;

        DEFINE INDEX todos_slug_idx ON TABLE todos COLUMNS slug UNIQUE;
        DEFINE INDEX todos_completed_idx ON TABLE todos COLUMNS completed;
        DEFINE INDEX todos_order_idx ON TABLE todos COLUMNS order;
        DEFINE INDEX todos_tags_idx ON TABLE todos COLUMNS tags;

        -- Graph relation: todos -> todo_tagged -> tags
        DEFINE TABLE todo_tagged SCHEMAFULL TYPE RELATION FROM todos TO tags;
        DEFINE FIELD created_at ON TABLE todo_tagged TYPE datetime;
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
    let existing_slugs: Vec<String> = db
        .query("SELECT VALUE slug FROM blog_posts LIMIT 1")
        .await?
        .take(0)?;

    if !existing_slugs.is_empty() {
        info!("   Database already has data, skipping seed");
        return Ok(());
    }

    // Sample blog post
    db.query(r##"
        CREATE blog_posts CONTENT {
            slug: "welcome-to-my-blog",
            title: "Welcome to My Blog",
            description: "A first blog post built with Rust, Leptos, and SurrealDB.",
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

        -- Create tags and graph relations
        CREATE tags CONTENT { name: "meta", slug: "meta", created_at: time::now() };
        CREATE tags CONTENT { name: "welcome", slug: "welcome", created_at: time::now() };

        LET $post = (SELECT VALUE id FROM blog_posts WHERE slug = "welcome-to-my-blog" LIMIT 1)[0];
        LET $tag_meta = (SELECT VALUE id FROM tags WHERE slug = "meta" LIMIT 1)[0];
        LET $tag_welcome = (SELECT VALUE id FROM tags WHERE slug = "welcome" LIMIT 1)[0];
        RELATE $post->tagged->$tag_meta CONTENT { created_at: time::now() };
        RELATE $post->tagged->$tag_welcome CONTENT { created_at: time::now() };
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

/// Sync the denormalized `tags` array on a blog post from graph relations.
/// Reads all tags linked via `->tagged->tags` and updates the post's `tags` field.
pub async fn sync_post_tags_cache(
    db: &Surreal<Db>,
    post_slug: &str,
) -> Result<(), surrealdb::Error> {
    db.query(
        r#"
        LET $post = (SELECT VALUE id FROM blog_posts WHERE slug = $slug LIMIT 1)[0];
        LET $tag_names = (SELECT VALUE name FROM $post->tagged->tags);
        UPDATE blog_posts SET tags = $tag_names WHERE slug = $slug;
    "#,
    )
    .bind(("slug", post_slug.to_string()))
    .await?;
    Ok(())
}

/// Sync the denormalized `tags` array on a todo item from graph relations.
/// Reads all tags linked via `->todo_tagged->tags` and updates the todo's `tags` field.
pub async fn sync_todo_tags_cache(
    db: &Surreal<Db>,
    todo_slug: &str,
) -> Result<(), surrealdb::Error> {
    db.query(
        r#"
        LET $todo = (SELECT VALUE id FROM todos WHERE slug = $slug LIMIT 1)[0];
        LET $tag_names = (SELECT VALUE name FROM $todo->todo_tagged->tags);
        UPDATE todos SET tags = $tag_names WHERE slug = $slug;
    "#,
    )
    .bind(("slug", todo_slug.to_string()))
    .await?;
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
