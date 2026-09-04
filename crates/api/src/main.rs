use std::sync::Arc;

use civico_api::db;
use civico_api::state::{AppState, Config};
use tower_http::trace::TraceLayer;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "civico_api=info,tower_http=info".into()),
        )
        .init();

    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL must be set for civico-api");
    let pool = db::connect(&database_url)
        .await
        .expect("failed to connect to PostgreSQL");

    db::migrate(&pool)
        .await
        .expect("failed to run database migrations");
    tracing::info!("database migrations applied");

    let config = Config::from_env();
    if let Err(err) = std::fs::create_dir_all(&config.media_dir) {
        tracing::warn!(
            "could not create media directory {}: {err}",
            config.media_dir.display()
        );
    }

    let app = civico_api::build_router(AppState {
        db: pool,
        config: Arc::new(config),
    })
    .layer(TraceLayer::new_for_http());

    let bind = std::env::var("API_BIND").unwrap_or_else(|_| "127.0.0.1:8080".to_string());
    let listener = tokio::net::TcpListener::bind(&bind).await.unwrap();
    tracing::info!("civico-api listening on {bind}");
    axum::serve(listener, app).await.unwrap();
}
