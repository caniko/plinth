use surrealdb::Surreal;
use surrealdb::engine::local::{Db, RocksDb};
use surrealdb::types::RecordId;
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

/// Initialize database schema via the migration system.
///
/// This is a convenience wrapper around `migrations::run_migrations`.
/// Prefer calling `migrations::run_migrations` directly for more control.
#[instrument(skip(db))]
pub async fn init_schema(db: &Surreal<Db>) -> Result<(), surrealdb::Error> {
    crate::services::migrations::run_migrations(db).await?;
    Ok(())
}

/// Seed the database with sample data for development.
///
/// Only seeds data for enabled bricks. Skips if data already exists.
#[instrument(skip(db))]
pub async fn seed_sample_data(db: &Surreal<Db>) -> Result<(), surrealdb::Error> {
    info!("Seeding sample data...");

    // Check if we already have any tags (core table, always present)
    let existing_tags: Vec<String> = db
        .query("SELECT VALUE slug FROM tags LIMIT 1")
        .await?
        .take(0)?;

    if !existing_tags.is_empty() {
        info!("   Database already has data, skipping seed");
        return Ok(());
    }

    // Sample blog post + tags
    #[cfg(feature = "brick-blog")]
    {
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
    }

    // Sample portfolio item
    #[cfg(feature = "brick-portfolio")]
    {
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
    }

    info!("Sample data seeded successfully");

    Ok(())
}

/// Create tags and graph relations (blog_posts->tagged->tags) for a post.
///
/// For each tag: creates the tag record if it doesn't exist, then creates a
/// `tagged` relation from the post to the tag. Runs as a single batched query.
pub async fn create_tags_for_post(
    db: &Surreal<Db>,
    post_slug: &str,
    tags: &[String],
) -> Result<(), surrealdb::Error> {
    if tags.is_empty() {
        return Ok(());
    }

    let mut tag_sql = String::from(
        "LET $post = (SELECT VALUE id FROM blog_posts WHERE slug = $post_slug LIMIT 1)[0];\n",
    );
    let mut binds: Vec<(String, String)> = vec![("post_slug".into(), post_slug.to_string())];

    for (i, tag_name) in tags.iter().enumerate() {
        let tag_slug = crate::services::markdown_processor::generate_slug(tag_name);
        let name_key = format!("tag_name_{i}");
        let slug_key = format!("tag_slug_{i}");
        tag_sql.push_str(&format!(
            r#"
            IF (SELECT count() FROM tags WHERE slug = ${slug_key}) = 0 THEN
                CREATE tags CONTENT {{
                    name: ${name_key},
                    slug: ${slug_key},
                    created_at: time::now()
                }}
            END;
            LET $tag_{i} = (SELECT VALUE id FROM tags WHERE slug = ${slug_key} LIMIT 1)[0];
            RELATE $post->tagged->$tag_{i} CONTENT {{ created_at: time::now() }};
            "#
        ));
        binds.push((name_key, tag_name.to_string()));
        binds.push((slug_key, tag_slug));
    }

    let mut q = db.query(&tag_sql);
    for (key, value) in binds {
        q = q.bind((key, value));
    }
    q.await?;

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
    format!("{}:{:?}", record.table.as_str(), record.key)
}

/// Helper to parse string ID to RecordId
#[allow(dead_code)]
pub fn string_to_record_id(id: &str) -> Result<RecordId, String> {
    RecordId::parse_simple(id).map_err(|e| format!("Invalid ID format: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_id_conversion() {
        let record = RecordId::new("blog_posts", "my-post");
        let id_string = record_id_to_string(&record);
        assert!(id_string.contains("blog_posts"));
        assert!(id_string.contains("my-post"));

        let parsed = string_to_record_id(&id_string).unwrap();
        assert_eq!(parsed.table.as_str(), "blog_posts");
    }
}
