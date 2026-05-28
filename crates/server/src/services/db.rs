use std::time::Duration;

use pgvector::Vector;
#[cfg(feature = "brick-portfolio")]
use plinth_shared::PortfolioItem;
use sqlx::postgres::PgPoolOptions;
use tracing::{info, instrument};

pub type Db = sqlx::PgPool;

use crate::PlinthDb;

/// Initialize a Postgres connection pool from config.
#[instrument(skip(config))]
pub async fn init_db(config: &crate::config::DatabaseConfig) -> Result<PlinthDb, sqlx::Error> {
    info!(database_url = %config.database_url, "Connecting to Postgres");

    let pool = PgPoolOptions::new()
        .max_connections(16)
        .acquire_timeout(Duration::from_secs(10))
        .after_connect(|conn, _meta| {
            Box::pin(async move {
                sqlx::query("SET search_path TO public")
                    .execute(&mut *conn)
                    .await?;
                sqlx::query("SELECT 1").execute(&mut *conn).await?;
                let _ = pgvector::Vector::from(vec![0.0_f32]);
                Ok(())
            })
        })
        .connect(&config.database_url)
        .await?;

    info!("Postgres connected");
    Ok(pool)
}

/// Initialize database schema via the migration system.
#[instrument(skip(db))]
pub async fn init_schema(db: &PlinthDb) -> Result<(), sqlx::Error> {
    crate::services::migrations::run_migrations(db).await?;
    Ok(())
}

/// Seed sample data for development.
#[instrument(skip(db))]
pub async fn seed_sample_data(db: &PlinthDb) -> Result<(), sqlx::Error> {
    info!("Seeding sample data...");

    let existing_tags: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM tags")
        .fetch_one(db)
        .await?;

    if existing_tags > 0 {
        info!("Database already has data, skipping seed");
        return Ok(());
    }

    #[cfg(feature = "brick-blog")]
    {
        sqlx::query(
            r##"
            INSERT INTO blog_posts (
                slug, title, description, content, html_content, published_at,
                author, tags, featured, published, reading_time_minutes, embedding
            )
            VALUES ($1, $2, $3, $4, $5, now(), $6, $7, true, true, 1, NULL)
            ON CONFLICT (slug) DO NOTHING
            "##,
        )
        .bind("welcome-to-my-blog")
        .bind("Welcome to My Blog")
        .bind("A first blog post built with Rust, Leptos, and Postgres.")
        .bind("# Welcome!\n\nThis is my first blog post built with Rust, Leptos, and Postgres!")
        .bind("<h1>Welcome!</h1><p>This is my first blog post built with Rust, Leptos, and Postgres!</p>")
        .bind("Author Name")
        .bind(vec!["meta".to_string(), "welcome".to_string()])
        .execute(db)
        .await?;

        create_tags_for_post(db, "welcome-to-my-blog", &["meta".into(), "welcome".into()]).await?;
    }

    #[cfg(feature = "brick-portfolio")]
    {
        sqlx::query(
            r##"
            INSERT INTO portfolio_items (
                slug, title, description, content, html_content, tech_stack,
                link, demo, image_url, date, featured, "order"
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, NULL, NULL, now(), true, 0)
            ON CONFLICT (slug) DO NOTHING
            "##,
        )
        .bind("sample-project")
        .bind("Sample Project")
        .bind("A sample portfolio project to demonstrate the system")
        .bind("# Sample Project\n\nThis is a sample project description.")
        .bind("<h1>Sample Project</h1><p>This is a sample project description.</p>")
        .bind(vec![
            "Rust".to_string(),
            "Leptos".to_string(),
            "Postgres".to_string(),
        ])
        .bind("https://github.com/user/project")
        .execute(db)
        .await?;
    }

    info!("Sample data seeded successfully");
    Ok(())
}

/// Create tags and relations for a post.
pub async fn create_tags_for_post(
    db: &PlinthDb,
    post_slug: &str,
    tags: &[String],
) -> Result<(), sqlx::Error> {
    let mut tx = db.begin().await?;
    create_tags_for_post_tx(&mut tx, post_slug, tags).await?;
    sync_post_tags_cache_tx(&mut tx, post_slug).await?;
    tx.commit().await
}

/// Sync the denormalized `tags` array on a blog post from tag relations.
///
/// The junction table remains the normalized source for tag associations. The
/// array is kept as a read-side cache because list pages and tag filters fetch
/// lightweight rows frequently and can use the GIN index without joining for
/// every request.
pub async fn sync_post_tags_cache(db: &PlinthDb, post_slug: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE blog_posts bp
        SET tags = COALESCE((
            SELECT array_agg(t.name ORDER BY t.name)
            FROM blog_post_tags bpt
            JOIN tags t ON t.id = bpt.tag_id
            WHERE bpt.post_id = bp.id
        ), '{}'::text[])
        WHERE bp.slug = $1
        "#,
    )
    .bind(post_slug)
    .execute(db)
    .await?;
    Ok(())
}

/// Sync the denormalized `tags` array on a todo item from tag relations.
///
/// The junction table remains the normalized source for tag associations. The
/// array is kept as a read-side cache because list pages and tag filters fetch
/// lightweight rows frequently and can use the GIN index without joining for
/// every request.
pub async fn sync_todo_tags_cache(db: &PlinthDb, todo_slug: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE todos td
        SET tags = COALESCE((
            SELECT array_agg(t.name ORDER BY t.name)
            FROM todo_tags tt
            JOIN tags t ON t.id = tt.tag_id
            WHERE tt.todo_id = td.id
        ), '{}'::text[])
        WHERE td.slug = $1
        "#,
    )
    .bind(todo_slug)
    .execute(db)
    .await?;
    Ok(())
}

pub(crate) async fn create_tags_for_post_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    post_slug: &str,
    tags: &[String],
) -> Result<(), sqlx::Error> {
    let post_id: i64 = sqlx::query_scalar("SELECT id FROM blog_posts WHERE slug = $1")
        .bind(post_slug)
        .fetch_one(&mut **tx)
        .await?;

    for tag_name in tags {
        let tag_slug = crate::services::markdown_processor::generate_slug(tag_name);
        let tag_id: i64 = sqlx::query_scalar(
            r#"
            INSERT INTO tags (name, slug)
            VALUES ($1, $2)
            ON CONFLICT (slug) DO UPDATE SET name = EXCLUDED.name
            RETURNING id
            "#,
        )
        .bind(tag_name)
        .bind(tag_slug)
        .fetch_one(&mut **tx)
        .await?;

        sqlx::query(
            r#"
            INSERT INTO blog_post_tags (post_id, tag_id)
            VALUES ($1, $2)
            ON CONFLICT DO NOTHING
            "#,
        )
        .bind(post_id)
        .bind(tag_id)
        .execute(&mut **tx)
        .await?;
    }

    Ok(())
}

pub(crate) async fn sync_post_tags_cache_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    post_slug: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE blog_posts bp
        SET tags = COALESCE((
            SELECT array_agg(t.name ORDER BY t.name)
            FROM blog_post_tags bpt
            JOIN tags t ON t.id = bpt.tag_id
            WHERE bpt.post_id = bp.id
        ), '{}'::text[])
        WHERE bp.slug = $1
        "#,
    )
    .bind(post_slug)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

pub(crate) async fn create_tags_for_todo_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    todo_slug: &str,
    tags: &[String],
) -> Result<(), sqlx::Error> {
    let todo_id: i64 = sqlx::query_scalar("SELECT id FROM todos WHERE slug = $1")
        .bind(todo_slug)
        .fetch_one(&mut **tx)
        .await?;

    for tag_name in tags {
        let tag_slug = crate::services::markdown_processor::generate_slug(tag_name);
        let tag_id: i64 = sqlx::query_scalar(
            r#"
            INSERT INTO tags (name, slug)
            VALUES ($1, $2)
            ON CONFLICT (slug) DO UPDATE SET name = EXCLUDED.name
            RETURNING id
            "#,
        )
        .bind(tag_name)
        .bind(tag_slug)
        .fetch_one(&mut **tx)
        .await?;

        sqlx::query(
            r#"
            INSERT INTO todo_tags (todo_id, tag_id)
            VALUES ($1, $2)
            ON CONFLICT DO NOTHING
            "#,
        )
        .bind(todo_id)
        .bind(tag_id)
        .execute(&mut **tx)
        .await?;
    }

    Ok(())
}

pub(crate) async fn sync_todo_tags_cache_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    todo_slug: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE todos td
        SET tags = COALESCE((
            SELECT array_agg(t.name ORDER BY t.name)
            FROM todo_tags tt
            JOIN tags t ON t.id = tt.tag_id
            WHERE tt.todo_id = td.id
        ), '{}'::text[])
        WHERE td.slug = $1
        "#,
    )
    .bind(todo_slug)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

pub(crate) fn vector_or_none(embedding: Option<Vec<f32>>) -> Option<Vector> {
    embedding.map(Vector::from)
}

/// Insert or update a portfolio item keyed by slug.
#[cfg(feature = "brick-portfolio")]
pub async fn upsert_portfolio_item(
    db: &PlinthDb,
    item: &PortfolioItem,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        r##"
        INSERT INTO portfolio_items (
            slug, title, description, content, html_content, tech_stack,
            link, demo, image_url, date, featured, "order"
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
        ON CONFLICT (slug) DO UPDATE SET
            title = EXCLUDED.title,
            description = EXCLUDED.description,
            content = EXCLUDED.content,
            html_content = EXCLUDED.html_content,
            tech_stack = EXCLUDED.tech_stack,
            link = EXCLUDED.link,
            demo = EXCLUDED.demo,
            image_url = EXCLUDED.image_url,
            date = EXCLUDED.date,
            featured = EXCLUDED.featured,
            "order" = EXCLUDED."order"
        RETURNING id
        "##,
    )
    .bind(&item.slug)
    .bind(&item.title)
    .bind(&item.description)
    .bind(&item.content)
    .bind(&item.html_content)
    .bind(&item.tech_stack)
    .bind(&item.link)
    .bind(&item.demo)
    .bind(&item.image_url)
    .bind(item.date)
    .bind(item.featured)
    .bind(item.order)
    .fetch_one(db)
    .await
}
