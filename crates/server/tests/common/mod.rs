#![allow(dead_code)]

use chrono::Utc;
use sqlx::{PgPool, Row};

pub async fn insert_blog_post(
    pool: &PgPool,
    slug: &str,
    title: &str,
    tags: &[&str],
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        INSERT INTO blog_posts (
            slug, title, description, content, html_content, published_at,
            author, tags, featured, published, reading_time_minutes
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, false, true, 1)
        RETURNING id
        "#,
    )
    .bind(slug)
    .bind(title)
    .bind(format!("Description for {title}"))
    .bind(format!("# {title}\n\nBody"))
    .bind(format!("<h1>{title}</h1><p>Body</p>"))
    .bind(Utc::now())
    .bind("Test Author")
    .bind(tags.iter().map(|tag| tag.to_string()).collect::<Vec<_>>())
    .fetch_one(pool)
    .await
}

pub async fn insert_todo(
    pool: &PgPool,
    slug: &str,
    title: &str,
    order: i32,
    completed: bool,
    tags: &[&str],
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        INSERT INTO todos (
            slug, title, description, content, html_content, tags,
            completed, completed_at, created_at, "order"
        )
        VALUES ($1, $2, $3, NULL, NULL, $4, $5, CASE WHEN $5 THEN now() ELSE NULL END, now(), $6)
        RETURNING id
        "#,
    )
    .bind(slug)
    .bind(title)
    .bind(format!("Description for {title}"))
    .bind(tags.iter().map(|tag| tag.to_string()).collect::<Vec<_>>())
    .bind(completed)
    .bind(order)
    .fetch_one(pool)
    .await
}

pub async fn ensure_tag(pool: &PgPool, name: &str) -> Result<i64, sqlx::Error> {
    let slug = plinth_server::services::markdown_processor::generate_slug(name);
    sqlx::query_scalar(
        r#"
        INSERT INTO tags (name, slug)
        VALUES ($1, $2)
        ON CONFLICT (slug) DO UPDATE SET name = EXCLUDED.name
        RETURNING id
        "#,
    )
    .bind(name)
    .bind(slug)
    .fetch_one(pool)
    .await
}

pub async fn attach_tag_to_todo(
    pool: &PgPool,
    todo_id: i64,
    tag_name: &str,
) -> Result<i64, sqlx::Error> {
    let tag_id = ensure_tag(pool, tag_name).await?;
    sqlx::query(
        r#"
        INSERT INTO todo_tags (todo_id, tag_id)
        VALUES ($1, $2)
        ON CONFLICT DO NOTHING
        "#,
    )
    .bind(todo_id)
    .bind(tag_id)
    .execute(pool)
    .await?;

    let todo_slug: String = sqlx::query_scalar("SELECT slug FROM todos WHERE id = $1")
        .bind(todo_id)
        .fetch_one(pool)
        .await?;
    plinth_server::services::db::sync_todo_tags_cache(pool, &todo_slug).await?;

    Ok(tag_id)
}

pub async fn blog_tag_names(pool: &PgPool, post_slug: &str) -> Result<Vec<String>, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        SELECT t.name
        FROM tags t
        JOIN blog_post_tags bpt ON bpt.tag_id = t.id
        JOIN blog_posts bp ON bp.id = bpt.post_id
        WHERE bp.slug = $1
        ORDER BY t.name
        "#,
    )
    .bind(post_slug)
    .fetch_all(pool)
    .await
}

pub async fn todo_tag_names(pool: &PgPool, todo_slug: &str) -> Result<Vec<String>, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        SELECT t.name
        FROM tags t
        JOIN todo_tags tt ON tt.tag_id = t.id
        JOIN todos td ON td.id = tt.todo_id
        WHERE td.slug = $1
        ORDER BY t.name
        "#,
    )
    .bind(todo_slug)
    .fetch_all(pool)
    .await
}

pub async fn column_text_array(
    pool: &PgPool,
    table: &str,
    slug: &str,
) -> Result<Vec<String>, sqlx::Error> {
    let query = match table {
        "blog_posts" => "SELECT tags FROM blog_posts WHERE slug = $1",
        "todos" => "SELECT tags FROM todos WHERE slug = $1",
        other => panic!("unsupported table {other}"),
    };

    let row = sqlx::query(query).bind(slug).fetch_one(pool).await?;
    row.try_get("tags")
}
