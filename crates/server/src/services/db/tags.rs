use crate::PlinthDb;

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
