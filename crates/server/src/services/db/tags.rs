use crate::PlinthDb;

/// Which content table a tag operation targets.
///
/// Table names are fixed per variant — no caller-supplied SQL identifiers are
/// ever interpolated.
#[derive(Clone, Copy)]
enum TagTarget {
    Post,
    Todo,
}

impl TagTarget {
    /// `SELECT id ... WHERE slug = $1` for the content row.
    fn id_query(self) -> &'static str {
        match self {
            TagTarget::Post => "SELECT id FROM blog_posts WHERE slug = $1",
            TagTarget::Todo => "SELECT id FROM todos WHERE slug = $1",
        }
    }

    /// `INSERT INTO <join> (<content_id>, tag_id) ...` linking a tag.
    fn link_query(self) -> &'static str {
        match self {
            TagTarget::Post => {
                r#"
            INSERT INTO blog_post_tags (post_id, tag_id)
            VALUES ($1, $2)
            ON CONFLICT DO NOTHING
            "#
            }
            TagTarget::Todo => {
                r#"
            INSERT INTO todo_tags (todo_id, tag_id)
            VALUES ($1, $2)
            ON CONFLICT DO NOTHING
            "#
            }
        }
    }

    /// `UPDATE <table> SET tags = (denormalized array from the join table) ...`.
    fn sync_query(self) -> &'static str {
        match self {
            TagTarget::Post => {
                r#"
            UPDATE blog_posts bp
            SET tags = COALESCE((
                SELECT array_agg(t.name ORDER BY t.name)
                FROM blog_post_tags bpt
                JOIN tags t ON t.id = bpt.tag_id
                WHERE bpt.post_id = bp.id
            ), '{}'::text[])
            WHERE bp.slug = $1
            "#
            }
            TagTarget::Todo => {
                r#"
            UPDATE todos td
            SET tags = COALESCE((
                SELECT array_agg(t.name ORDER BY t.name)
                FROM todo_tags tt
                JOIN tags t ON t.id = tt.tag_id
                WHERE tt.todo_id = td.id
            ), '{}'::text[])
            WHERE td.slug = $1
            "#
            }
        }
    }
}

/// Create tags for a content row and update the denormalized tags cache.
async fn create_tags_for_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    target: TagTarget,
    slug: &str,
    tags: &[String],
) -> Result<(), sqlx::Error> {
    let content_id: i64 = sqlx::query_scalar(target.id_query())
        .bind(slug)
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

        sqlx::query(target.link_query())
            .bind(content_id)
            .bind(tag_id)
            .execute(&mut **tx)
            .await?;
    }

    Ok(())
}

/// Sync the denormalized tags array on a content row from the join table.
async fn sync_tags_cache_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    target: TagTarget,
    slug: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(target.sync_query())
        .bind(slug)
        .execute(&mut **tx)
        .await?;
    Ok(())
}

/// Create tags for a blog post and update the denormalized tags cache.
pub async fn create_tags_for_post(
    db: &PlinthDb,
    post_slug: &str,
    tags: &[String],
) -> Result<(), sqlx::Error> {
    let mut tx = db.begin().await?;
    create_tags_for_tx(&mut tx, TagTarget::Post, post_slug, tags).await?;
    sync_tags_cache_tx(&mut tx, TagTarget::Post, post_slug).await?;
    tx.commit().await
}

/// Sync the denormalized tags array on a blog post from the join table.
pub async fn sync_post_tags_cache(db: &PlinthDb, post_slug: &str) -> Result<(), sqlx::Error> {
    sqlx::query(TagTarget::Post.sync_query())
        .bind(post_slug)
        .execute(db)
        .await?;
    Ok(())
}

/// Sync the denormalized tags array on a todo item from the join table.
pub async fn sync_todo_tags_cache(db: &PlinthDb, todo_slug: &str) -> Result<(), sqlx::Error> {
    sqlx::query(TagTarget::Todo.sync_query())
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
    create_tags_for_tx(tx, TagTarget::Post, post_slug, tags).await
}

pub(crate) async fn sync_post_tags_cache_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    post_slug: &str,
) -> Result<(), sqlx::Error> {
    sync_tags_cache_tx(tx, TagTarget::Post, post_slug).await
}

pub(crate) async fn create_tags_for_todo_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    todo_slug: &str,
    tags: &[String],
) -> Result<(), sqlx::Error> {
    create_tags_for_tx(tx, TagTarget::Todo, todo_slug, tags).await
}

pub(crate) async fn sync_todo_tags_cache_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    todo_slug: &str,
) -> Result<(), sqlx::Error> {
    sync_tags_cache_tx(tx, TagTarget::Todo, todo_slug).await
}
