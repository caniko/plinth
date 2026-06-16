use tracing::{info, instrument};

use crate::PlinthDb;

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

        super::create_tags_for_post(db, "welcome-to-my-blog", &["meta".into(), "welcome".into()])
            .await?;
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
