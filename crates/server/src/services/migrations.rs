use surrealdb::Surreal;
use surrealdb::engine::local::Db;
use tracing::{info, instrument, warn};

use crate::bricks::BrickMigration;
use crate::db_helpers::take_as;

// ── Core migrations (always present) ────────────────────────────────────────

/// Core migrations for tables shared across bricks (tags, site_content).
pub fn core_migrations() -> Vec<BrickMigration> {
    vec![BrickMigration {
        brick: "core",
        version: 1,
        name: "core_schema",
        up: r#"
                DEFINE TABLE site_content SCHEMAFULL;

                DEFINE FIELD key ON TABLE site_content TYPE string;
                DEFINE FIELD title ON TABLE site_content TYPE option<string>;
                DEFINE FIELD content ON TABLE site_content TYPE string;
                DEFINE FIELD html_content ON TABLE site_content TYPE string;
                DEFINE FIELD updated_at ON TABLE site_content TYPE datetime;

                DEFINE INDEX site_content_key_idx ON TABLE site_content COLUMNS key UNIQUE;

                DEFINE TABLE tags SCHEMAFULL;

                DEFINE FIELD name ON TABLE tags TYPE string;
                DEFINE FIELD slug ON TABLE tags TYPE string;
                DEFINE FIELD created_at ON TABLE tags TYPE datetime;

                DEFINE INDEX tags_slug_idx ON TABLE tags COLUMNS slug UNIQUE;
                DEFINE INDEX tags_name_idx ON TABLE tags COLUMNS name UNIQUE;
            "#,
    }]
}

// ── Legacy migration support ────────────────────────────────────────────────

/// The old monolithic migrations that existed before the brick system.
/// Used only to detect existing databases that need to be "split" into
/// per-brick migration records.
struct LegacyMigration {
    version: u32,
    /// Which brick migrations this legacy version covers.
    /// Format: (brick, version)
    covers: Vec<(&'static str, u32)>,
}

/// Map old monolithic migration versions to the brick migrations they contain.
fn legacy_migration_map() -> Vec<LegacyMigration> {
    vec![
        LegacyMigration {
            version: 1,
            covers: vec![("core", 1), ("blog", 1), ("portfolio", 1), ("todo", 1)],
        },
        LegacyMigration {
            version: 2,
            covers: vec![("blog", 2)],
        },
    ]
}

// ── Schema migrations table (v2 with brick column) ─────────────────────────

/// Bootstrap or upgrade the schema_migrations table.
///
/// The new table schema has `(brick, version)` as composite key.
/// If the old table exists (version-only), we migrate its records.
async fn ensure_migrations_table(db: &Surreal<Db>) -> Result<(), surrealdb::Error> {
    // Define the new schema. DEFINE TABLE/FIELD is idempotent in SurrealDB,
    // so running this on an already-upgraded table is safe.
    db.query(
        r#"
        DEFINE TABLE schema_migrations SCHEMAFULL;
        DEFINE FIELD brick ON TABLE schema_migrations TYPE string DEFAULT "";
        DEFINE FIELD version ON TABLE schema_migrations TYPE int;
        DEFINE FIELD name ON TABLE schema_migrations TYPE string;
        DEFINE FIELD applied_at ON TABLE schema_migrations TYPE datetime;

        -- Remove old version-only unique index if it exists (from pre-brick era).
        -- This is safe because the composite index below replaces it.
        REMOVE INDEX IF EXISTS schema_migrations_version_idx ON TABLE schema_migrations;

        DEFINE INDEX schema_migrations_brick_version_idx
            ON TABLE schema_migrations COLUMNS brick, version UNIQUE;
    "#,
    )
    .await?;

    Ok(())
}

/// Detect and migrate from the old version-only schema_migrations to the new
/// brick-namespaced format.
///
/// If old-style records exist (brick field is empty/missing), we map them to
/// per-brick records using `legacy_migration_map()` and delete the old ones.
async fn migrate_legacy_records(db: &Surreal<Db>) -> Result<(), surrealdb::Error> {
    // Check if we have any old-style records (brick field empty, missing, or NONE).
    // Records created before the `brick` field was added will have `brick` as
    // NONE/null. We detect these by checking for records with a non-empty brick
    // field vs total records.
    // SurrealDB's three-valued logic makes WHERE clauses unreliable for
    // detecting NONE/null fields. Query all records and filter in Rust.
    #[derive(serde::Deserialize)]
    struct MigrationRecord {
        brick: Option<String>,
        version: u32,
    }

    let mut response = db
        .query("SELECT brick, version FROM schema_migrations ORDER BY version ASC")
        .await?;
    let records: Vec<MigrationRecord> =
        take_as(&mut response, 0).map_err(surrealdb::Error::thrown)?;

    let old_versions: Vec<u32> = records
        .iter()
        .filter(|r| r.brick.as_ref().is_none_or(|b| b.is_empty()))
        .map(|r| r.version)
        .collect();

    if old_versions.is_empty() {
        return Ok(());
    }

    info!(
        count = old_versions.len(),
        "Found legacy migration records, converting to brick-namespaced format"
    );

    let legacy_map = legacy_migration_map();

    for old_version in &old_versions {
        if let Some(legacy) = legacy_map.iter().find(|l| l.version == *old_version) {
            for (brick, brick_version) in &legacy.covers {
                // Insert brick-namespaced record. Skip if it already exists
                // (check in Rust to avoid SurrealDB's three-valued logic issues).
                if !is_migration_applied(db, brick, *brick_version).await? {
                    db.query(
                        r#"
                        CREATE schema_migrations CONTENT {
                            brick: $brick,
                            version: $version,
                            name: $name,
                            applied_at: time::now()
                        };
                        "#,
                    )
                    .bind(("brick", brick.to_string()))
                    .bind(("version", *brick_version as i64))
                    .bind(("name", format!("migrated_from_legacy_v{}", old_version)))
                    .await?;
                }
            }
        } else {
            warn!(
                version = old_version,
                "Unknown legacy migration version, skipping"
            );
        }
    }

    // Delete old-style records by version. We already know which versions
    // are legacy (from the Rust-side filter above), so delete by version
    // where brick is not set.
    for old_version in &old_versions {
        // Delete all records with this version that don't have a proper brick name.
        // We delete by re-selecting with exact version match and checking the record.
        db.query(
            "DELETE FROM schema_migrations WHERE version = $version AND (brick = NONE OR brick = '')",
        )
        .bind(("version", *old_version as i64))
        .await?;
    }

    info!("Legacy migration records converted successfully");
    Ok(())
}

// ── Brick-aware migration runner ────────────────────────────────────────────

/// Check if a specific (brick, version) migration has been applied.
async fn is_migration_applied(
    db: &Surreal<Db>,
    brick: &str,
    version: u32,
) -> Result<bool, surrealdb::Error> {
    let mut response = db
        .query(
            "SELECT count() AS c FROM schema_migrations WHERE brick = $brick AND version = $version",
        )
        .bind(("brick", brick.to_string()))
        .bind(("version", version as i64))
        .await?;

    let count: Option<i64> = response.take("c").unwrap_or(None);
    Ok(count.unwrap_or(0) > 0)
}

/// Record that a brick migration was successfully applied.
async fn record_brick_migration(
    db: &Surreal<Db>,
    brick: &str,
    version: u32,
    name: &str,
) -> Result<(), surrealdb::Error> {
    db.query(
        r#"
        CREATE schema_migrations CONTENT {
            brick: $brick,
            version: $version,
            name: $name,
            applied_at: time::now()
        };
    "#,
    )
    .bind(("brick", brick.to_string()))
    .bind(("version", version as i64))
    .bind(("name", name.to_string()))
    .await?;
    Ok(())
}

/// Run all pending migrations for a set of brick migrations.
///
/// This is called once for each brick + the core migrations.
/// Returns the number of migrations applied.
pub async fn run_brick_migrations(
    db: &Surreal<Db>,
    migrations: &[BrickMigration],
) -> Result<u32, surrealdb::Error> {
    let mut applied = 0u32;

    for migration in migrations {
        if is_migration_applied(db, migration.brick, migration.version).await? {
            continue;
        }

        info!(
            brick = migration.brick,
            version = migration.version,
            name = migration.name,
            "Applying brick migration"
        );

        db.query(migration.up).await?;
        record_brick_migration(db, migration.brick, migration.version, migration.name).await?;
        applied += 1;

        info!(
            brick = migration.brick,
            version = migration.version,
            name = migration.name,
            "Brick migration applied successfully"
        );
    }

    Ok(applied)
}

/// Run all migrations: core + all enabled bricks.
///
/// This is the main entry point called from `main.rs`. It:
/// 1. Ensures the schema_migrations table exists (with brick column)
/// 2. Migrates any legacy (pre-brick) migration records
/// 3. Runs core migrations
/// 4. Runs migrations for each enabled brick
#[instrument(skip(db, brick_migrations))]
pub async fn run_all_migrations(
    db: &Surreal<Db>,
    brick_migrations: &[Vec<BrickMigration>],
) -> Result<u32, surrealdb::Error> {
    ensure_migrations_table(db).await?;
    migrate_legacy_records(db).await?;

    let mut total_applied = 0u32;

    // Run core migrations first
    let core = core_migrations();
    total_applied += run_brick_migrations(db, &core).await?;

    // Run each brick's migrations
    for migrations in brick_migrations {
        total_applied += run_brick_migrations(db, migrations).await?;
    }

    if total_applied == 0 {
        info!("Database schema is up to date");
    } else {
        info!(total_applied, "All migrations complete");
    }

    Ok(total_applied)
}

// ── Legacy compatibility wrappers ───────────────────────────────────────────
//
// These keep existing code working during the transition. They will be removed
// once all callers are updated to use `run_all_migrations` directly.

/// Run all pending migrations (legacy wrapper).
///
/// This collects migrations from all enabled bricks and runs them.
/// Prefer `run_all_migrations` for new code.
#[instrument(skip(db))]
pub async fn run_migrations(db: &Surreal<Db>) -> Result<u32, surrealdb::Error> {
    let bricks = crate::bricks::enabled_bricks();
    let brick_migrations: Vec<Vec<BrickMigration>> =
        bricks.iter().map(|b| b.migrations()).collect();
    run_all_migrations(db, &brick_migrations).await
}

/// Get the status of all migrations (applied and pending).
///
/// Returns a list of (brick, version, name, applied) tuples.
pub async fn migration_status(
    db: &Surreal<Db>,
) -> Result<Vec<(&'static str, u32, &'static str, bool)>, surrealdb::Error> {
    ensure_migrations_table(db).await?;
    migrate_legacy_records(db).await?;

    let mut statuses = Vec::new();

    // Core migrations
    for m in core_migrations() {
        let applied = is_migration_applied(db, m.brick, m.version).await?;
        statuses.push((m.brick, m.version, m.name, applied));
    }

    // Brick migrations
    let bricks = crate::bricks::enabled_bricks();
    for brick in &bricks {
        for m in brick.migrations() {
            let applied = is_migration_applied(db, m.brick, m.version).await?;
            statuses.push((m.brick, m.version, m.name, applied));
        }
    }

    Ok(statuses)
}

/// Get the latest available migration version across all bricks.
/// Returns the total count of all migrations (core + bricks).
pub fn latest_available_version() -> u32 {
    let mut count = core_migrations().len() as u32;
    let bricks = crate::bricks::enabled_bricks();
    for brick in &bricks {
        count += brick.migrations().len() as u32;
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;
    use surrealdb::engine::local::Mem;

    async fn test_db() -> Surreal<Db> {
        let db = Surreal::new::<Mem>(())
            .await
            .expect("Failed to create in-memory DB");
        db.use_ns("test")
            .use_db("test")
            .await
            .expect("Failed to select ns/db");
        db
    }

    #[tokio::test]
    async fn test_migrations_on_empty_db() {
        let db = test_db().await;
        let applied = run_migrations(&db).await.unwrap();
        assert!(
            applied > 0,
            "Should apply at least one migration on empty DB"
        );
    }

    #[tokio::test]
    async fn test_migrations_idempotent() {
        let db = test_db().await;

        let first_run = run_migrations(&db).await.unwrap();
        assert!(first_run > 0);

        let second_run = run_migrations(&db).await.unwrap();
        assert_eq!(
            second_run, 0,
            "No migrations should be applied on second run"
        );
    }

    #[tokio::test]
    async fn test_migration_tracking() {
        let db = test_db().await;
        run_migrations(&db).await.unwrap();

        let statuses = migration_status(&db).await.unwrap();
        assert!(!statuses.is_empty());
        for (_, _, _, applied) in &statuses {
            assert!(applied, "All migrations should be marked as applied");
        }
    }

    #[tokio::test]
    async fn test_schema_works_after_migration() {
        let db = test_db().await;
        run_migrations(&db).await.unwrap();

        // Verify we can insert into tables created by brick migrations
        db.query(
            r#"
            CREATE blog_posts CONTENT {
                slug: "test",
                title: "Test",
                content: "content",
                html_content: "<p>content</p>",
                published_at: time::now(),
                author: "test",
                tags: [],
                featured: false,
                published: true,
                reading_time_minutes: 1,
                embedding: NONE
            };
            "#,
        )
        .await
        .expect("Should be able to insert after migration");

        let mut response = db
            .query("SELECT VALUE slug FROM blog_posts WHERE slug = 'test'")
            .await
            .unwrap();
        let slugs: Vec<String> = response.take(0).unwrap();
        assert_eq!(slugs.len(), 1);
        assert_eq!(slugs[0], "test");
    }

    #[tokio::test]
    async fn test_core_migrations_create_tags_and_site_content() {
        let db = test_db().await;
        ensure_migrations_table(&db).await.unwrap();
        let core = core_migrations();
        run_brick_migrations(&db, &core).await.unwrap();

        // Verify tags table exists
        db.query(r#"CREATE tags CONTENT { name: "test", slug: "test", created_at: time::now() }"#)
            .await
            .expect("tags table should exist");

        // Verify site_content table exists
        db.query(
            r#"CREATE site_content CONTENT {
                key: "test", title: "Test", content: "c",
                html_content: "<p>c</p>", updated_at: time::now()
            }"#,
        )
        .await
        .expect("site_content table should exist");
    }

    #[tokio::test]
    async fn test_brick_migration_applied_check() {
        let db = test_db().await;
        ensure_migrations_table(&db).await.unwrap();

        assert!(!is_migration_applied(&db, "test_brick", 1).await.unwrap());

        record_brick_migration(&db, "test_brick", 1, "test_migration")
            .await
            .unwrap();

        assert!(is_migration_applied(&db, "test_brick", 1).await.unwrap());
        // Different brick, same version — not applied
        assert!(!is_migration_applied(&db, "other_brick", 1).await.unwrap());
    }

    #[tokio::test]
    async fn test_legacy_migration_conversion() {
        let db = test_db().await;

        // Simulate old-style schema_migrations table
        db.query(
            r#"
            DEFINE TABLE schema_migrations SCHEMAFULL;
            DEFINE FIELD version ON TABLE schema_migrations TYPE int;
            DEFINE FIELD name ON TABLE schema_migrations TYPE string;
            DEFINE FIELD applied_at ON TABLE schema_migrations TYPE datetime;
            DEFINE INDEX schema_migrations_version_idx ON TABLE schema_migrations COLUMNS version UNIQUE;
        "#,
        )
        .await
        .unwrap();

        // Insert old-style records (no brick field)
        db.query(
            r#"
            CREATE schema_migrations CONTENT { version: 1, name: "initial_schema", applied_at: time::now() };
            CREATE schema_migrations CONTENT { version: 2, name: "add_series_fields", applied_at: time::now() };
        "#,
        )
        .await
        .unwrap();

        // Now run the new migration system
        ensure_migrations_table(&db).await.unwrap();

        migrate_legacy_records(&db).await.unwrap();

        // Verify brick-namespaced records were created
        assert!(is_migration_applied(&db, "core", 1).await.unwrap());
        assert!(is_migration_applied(&db, "blog", 1).await.unwrap());
        assert!(is_migration_applied(&db, "blog", 2).await.unwrap());
        assert!(is_migration_applied(&db, "portfolio", 1).await.unwrap());
        assert!(is_migration_applied(&db, "todo", 1).await.unwrap());

        // Verify old records were cleaned up (use same Rust-side approach)
        #[derive(serde::Deserialize)]
        struct BrickField {
            brick: Option<String>,
        }
        let mut response = db
            .query("SELECT brick FROM schema_migrations")
            .await
            .unwrap();
        let records: Vec<BrickField> = take_as(&mut response, 0).unwrap();
        let old_count = records
            .iter()
            .filter(|r| r.brick.as_ref().is_none_or(|b| b.is_empty()))
            .count();
        assert_eq!(old_count, 0, "All old records should be cleaned up");
    }
}
