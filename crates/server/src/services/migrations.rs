use tracing::instrument;

use crate::PlinthDb;
use crate::bricks::BrickMigration;

static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!();

/// Core migrations for tables shared across bricks.
pub fn core_migrations() -> Vec<BrickMigration> {
    vec![
        BrickMigration {
            brick: "core",
            version: 1,
            name: "init",
            up: "",
        },
        BrickMigration {
            brick: "core",
            version: 2,
            name: "core_schema",
            up: "",
        },
    ]
}

/// Run all pending migrations for a set of brick migrations.
pub async fn run_brick_migrations(
    db: &PlinthDb,
    _migrations: &[BrickMigration],
) -> Result<u32, sqlx::Error> {
    run_embedded_migrations(db).await
}

/// Run all migrations: core + all enabled bricks.
#[instrument(skip(db, _brick_migrations))]
pub async fn run_all_migrations(
    db: &PlinthDb,
    _brick_migrations: &[Vec<BrickMigration>],
) -> Result<u32, sqlx::Error> {
    run_embedded_migrations(db).await
}

/// Run all pending migrations.
#[instrument(skip(db))]
pub async fn run_migrations(db: &PlinthDb) -> Result<u32, sqlx::Error> {
    let bricks = crate::bricks::enabled_bricks();
    let brick_migrations: Vec<Vec<BrickMigration>> =
        bricks.iter().map(|b| b.migrations()).collect();
    run_all_migrations(db, &brick_migrations).await
}

/// Get the status of all migrations.
pub async fn migration_status(
    db: &PlinthDb,
) -> Result<Vec<(&'static str, u32, &'static str, bool)>, sqlx::Error> {
    let mut expected = core_migrations();
    for brick in crate::bricks::enabled_bricks() {
        expected.extend(brick.migrations());
    }

    if !relation_exists(db, "schema_migrations").await? {
        return Ok(expected
            .into_iter()
            .map(|m| (m.brick, m.version, m.name, false))
            .collect());
    }

    let applied: std::collections::HashSet<(String, i32)> =
        sqlx::query_as::<_, (String, i32)>("SELECT brick, version FROM schema_migrations")
            .fetch_all(db)
            .await?
            .into_iter()
            .collect();

    Ok(expected
        .into_iter()
        .map(|m| {
            let is_applied = applied.contains(&(m.brick.to_string(), m.version as i32));
            (m.brick, m.version, m.name, is_applied)
        })
        .collect())
}

/// Get the latest available migration version across all bricks.
pub fn latest_available_version() -> u32 {
    let mut count = core_migrations().len() as u32;
    let bricks = crate::bricks::enabled_bricks();
    for brick in &bricks {
        count += brick.migrations().len() as u32;
    }
    count
}

async fn run_embedded_migrations(db: &PlinthDb) -> Result<u32, sqlx::Error> {
    let before = applied_sqlx_migration_count(db).await?;
    MIGRATOR.run(db).await.map_err(migrate_error_to_sqlx)?;
    let after = applied_sqlx_migration_count(db).await?;
    Ok(after.saturating_sub(before))
}

async fn applied_sqlx_migration_count(db: &PlinthDb) -> Result<u32, sqlx::Error> {
    if !relation_exists(db, "_sqlx_migrations").await? {
        return Ok(0);
    }

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM _sqlx_migrations")
        .fetch_one(db)
        .await?;

    Ok(count.try_into().unwrap_or(u32::MAX))
}

async fn relation_exists(db: &PlinthDb, relation: &str) -> Result<bool, sqlx::Error> {
    let exists: Option<String> = sqlx::query_scalar("SELECT to_regclass($1)::text")
        .bind(relation)
        .fetch_one(db)
        .await?;

    Ok(exists.is_some())
}

fn migrate_error_to_sqlx(error: sqlx::migrate::MigrateError) -> sqlx::Error {
    sqlx::Error::Protocol(format!("migration failed: {error}"))
}
