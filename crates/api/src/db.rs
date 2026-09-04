use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use std::time::Duration;

/// Connect to PostgreSQL using the provided `DATABASE_URL`.
pub async fn connect(database_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(5))
        .connect(database_url)
        .await
}

/// Run all pending SQLx migrations.
pub async fn migrate(pool: &PgPool) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!("./migrations").run(pool).await
}
