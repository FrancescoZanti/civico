pub mod auth;
pub mod db;
pub mod domain;
pub mod dto;
pub mod error;
pub mod handlers;
pub mod media_store;
pub mod repo;
pub mod state;
pub mod validation;

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Router, routing::post};
use serde::Serialize;

use crate::error::ApiError;
use crate::state::AppState;

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    version: &'static str,
    database: &'static str,
}

async fn health(State(state): State<AppState>) -> Result<Json<HealthResponse>, ApiError> {
    let database_ok = sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(&state.db)
        .await
        .is_ok();
    if !database_ok {
        return Err(ApiError::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "database_unavailable",
            "the database is unavailable",
        ));
    }
    Ok(Json(HealthResponse {
        status: "ok",
        version: env!("CARGO_PKG_VERSION"),
        database: "up",
    }))
}

/// Build the full API router. All admin routes are authenticated via the
/// `AuthAdmin` extractor (returns 401 on missing/invalid token); only
/// `/admin/auth/login` and `/admin/auth/bootstrap` are public.
pub fn build_router(state: AppState) -> Router {
    let admin = Router::new()
        // home items
        .route(
            "/home-items",
            get(handlers::admin_home::list_home_items).post(handlers::admin_home::create_home_item),
        )
        .route(
            "/home-items/{id}",
            get(handlers::admin_home::get_home_item)
                .put(handlers::admin_home::update_home_item)
                .delete(handlers::admin_home::delete_home_item),
        )
        // content
        .route(
            "/content",
            get(handlers::admin_content::list_content)
                .post(handlers::admin_content::create_content),
        )
        .route(
            "/content/{id}",
            get(handlers::admin_content::get_content)
                .put(handlers::admin_content::update_content)
                .delete(handlers::admin_content::delete_content),
        )
        // media
        .route(
            "/media",
            get(handlers::admin_media::list_media).post(handlers::admin_media::upload_media),
        )
        .route(
            "/media/{id}",
            get(handlers::admin_media::get_media).delete(handlers::admin_media::delete_media),
        )
        .with_state(state.clone());

    Router::new()
        .route("/api/v1/health", get(health))
        .route("/api/v1/admin/auth/login", post(auth::login))
        .route("/api/v1/admin/auth/bootstrap", post(auth::bootstrap))
        .nest("/api/v1/admin", admin)
        .route("/api/v1/player/home", get(handlers::player::player_home))
        .route(
            "/api/v1/player/content",
            get(handlers::player::player_content_list),
        )
        .route(
            "/api/v1/player/content/{slug}",
            get(handlers::player::player_content_detail),
        )
        .route(
            "/api/v1/media/{id}/file",
            get(handlers::admin_media::serve_media_file),
        )
        .with_state(state)
}
