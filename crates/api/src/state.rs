use std::path::PathBuf;
use std::sync::Arc;

use sqlx::PgPool;

/// Configuration shared across handlers.
pub struct Config {
    /// Secret used to sign admin session tokens. Never expose via API.
    pub jwt_secret: String,
    /// Directory where uploaded media files are stored on disk.
    pub media_dir: PathBuf,
    /// Maximum accepted upload size in bytes.
    pub max_upload_bytes: u64,
}

impl Config {
    /// Build configuration from the environment, applying sensible defaults.
    pub fn from_env() -> Config {
        let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| {
            tracing::warn!("JWT_SECRET not set; using an insecure development default");
            "dev-only-insecure-secret-do-not-use-in-production".to_string()
        });

        let media_dir = std::env::var("MEDIA_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("media"));

        let max_upload_bytes = std::env::var("MAX_UPLOAD_BYTES")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(10 * 1024 * 1024);

        Config {
            jwt_secret,
            media_dir,
            max_upload_bytes,
        }
    }
}

/// Application state passed to every handler via an Axum extractor.
#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub config: Arc<Config>,
}
