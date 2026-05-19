use std::collections::HashSet;

use sqlx::PgPool;

#[sqlx::test(migrations = "./migrations")]
async fn postgres_migrations_create_expected_schema_and_are_idempotent(pool: PgPool) {
    let second_run = plinth_server::services::migrations::run_migrations(&pool)
        .await
        .expect("run migrations a second time");
    assert_eq!(second_run, 0);

    let table_names: HashSet<String> = sqlx::query_scalar(
        r#"
        SELECT table_name
        FROM information_schema.tables
        WHERE table_schema = 'public'
          AND table_type = 'BASE TABLE'
        "#,
    )
    .fetch_all(&pool)
    .await
    .expect("list public tables")
    .into_iter()
    .collect();

    for table in [
        "site_content",
        "tags",
        "blog_posts",
        "blog_post_tags",
        "portfolio_items",
        "todos",
        "todo_tags",
        "schema_migrations",
        "_sqlx_migrations",
    ] {
        assert!(table_names.contains(table), "missing table {table}");
    }

    let vector_extension: bool =
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM pg_extension WHERE extname = 'vector')")
            .fetch_one(&pool)
            .await
            .expect("inspect vector extension");
    assert!(vector_extension);

    let embedding_type: String = sqlx::query_scalar(
        r#"
        SELECT format_type(a.atttypid, a.atttypmod)
        FROM pg_attribute a
        JOIN pg_class c ON c.oid = a.attrelid
        JOIN pg_namespace n ON n.oid = c.relnamespace
        WHERE n.nspname = 'public'
          AND c.relname = 'blog_posts'
          AND a.attname = 'embedding'
          AND NOT a.attisdropped
        "#,
    )
    .fetch_one(&pool)
    .await
    .expect("inspect blog_posts.embedding type");
    assert_eq!(embedding_type, "vector(384)");

    let index_names: HashSet<String> = sqlx::query_scalar(
        r#"
        SELECT indexname
        FROM pg_indexes
        WHERE schemaname = 'public'
        "#,
    )
    .fetch_all(&pool)
    .await
    .expect("list public indexes")
    .into_iter()
    .collect();

    for index in [
        "blog_post_tags_tag_id_idx",
        "todo_tags_tag_id_idx",
        "blog_posts_embedding_hnsw_idx",
    ] {
        assert!(index_names.contains(index), "missing index {index}");
    }

    let statuses = plinth_server::services::migrations::migration_status(&pool)
        .await
        .expect("read migration status");
    assert!(statuses.iter().all(|(_, _, _, applied)| *applied));
}
