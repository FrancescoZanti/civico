use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;
use uuid::Uuid;

use crate::auth::AuthAdmin;
use crate::dto::{ContentCreate, ContentDto, ContentUpdate};
use crate::error::{ApiError, ApiResult};
use crate::repo;
use crate::state::AppState;
use crate::validation::validate_publish_window;

pub async fn create_content(
    State(state): State<AppState>,
    auth: AuthAdmin,
    Json(req): Json<ContentCreate>,
) -> ApiResult<(StatusCode, Json<ContentDto>)> {
    validate_publish_window(req.publish_from, req.publish_until)?;
    crate::validation::validate_slug(&req.slug)?;

    let status = req.publication_status.unwrap_or_default();
    let data = req.data.unwrap_or_else(|| json!({}));

    let row = repo::content_create(
        &state.db,
        req.content_type,
        &req.title,
        &req.slug,
        req.excerpt.as_deref(),
        req.body.as_deref(),
        data,
        status,
        req.publish_from,
        req.publish_until,
        req.image_id,
        auth.id,
    )
    .await
    .map_err(map_content_error)?;

    Ok((StatusCode::CREATED, Json(row.into())))
}

pub async fn list_content(
    State(state): State<AppState>,
    _auth: AuthAdmin,
) -> ApiResult<Json<Vec<ContentDto>>> {
    let rows = repo::content_list(&state.db)
        .await
        .map_err(|_| ApiError::internal("listing content"))?;
    Ok(Json(rows.into_iter().map(Into::into).collect()))
}

pub async fn get_content(
    State(state): State<AppState>,
    _auth: AuthAdmin,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<ContentDto>> {
    let row = repo::content_get(&state.db, id)
        .await
        .map_err(|_| ApiError::internal("fetching content"))?
        .ok_or_else(|| ApiError::not_found("content", &id.to_string()))?;
    Ok(Json(row.into()))
}

pub async fn update_content(
    State(state): State<AppState>,
    _auth: AuthAdmin,
    Path(id): Path<Uuid>,
    Json(req): Json<ContentUpdate>,
) -> ApiResult<Json<ContentDto>> {
    // Validate before touching the DB.
    if let Some(slug) = &req.slug {
        crate::validation::validate_slug(slug)?;
    }
    validate_publish_window(req.publish_from, req.publish_until)?;

    let row = repo::content_update(
        &state.db,
        id,
        req.title.as_deref(),
        req.slug.as_deref(),
        req.excerpt.as_deref(),
        req.body.as_deref(),
        req.data,
        req.publication_status,
        req.publish_from,
        req.publish_until,
        req.image_id,
    )
    .await
    .map_err(map_content_error)?
    .ok_or_else(|| ApiError::not_found("content", &id.to_string()))?;

    Ok(Json(row.into()))
}

pub async fn delete_content(
    State(state): State<AppState>,
    _auth: AuthAdmin,
    Path(id): Path<Uuid>,
) -> ApiResult<Response> {
    let deleted = repo::content_delete(&state.db, id)
        .await
        .map_err(|_| ApiError::internal("deleting content"))?;
    if !deleted {
        return Err(ApiError::not_found("content", &id.to_string()));
    }
    Ok(StatusCode::NO_CONTENT.into_response())
}

fn map_content_error(err: sqlx::Error) -> ApiError {
    if let sqlx::Error::Database(db_err) = &err {
        if db_err.is_unique_violation() {
            return ApiError::conflict("a content item with that slug already exists");
        }
        if db_err.is_check_violation() {
            return ApiError::validation(
                "content violates a database constraint (check publish window/timestamps)",
                json!({}),
            );
        }
    }
    ApiError::internal("persisting content")
}

impl From<repo::ContentRow> for ContentDto {
    fn from(row: repo::ContentRow) -> Self {
        ContentDto {
            id: row.id,
            content_type: row.content_type,
            title: row.title,
            slug: row.slug,
            excerpt: row.excerpt,
            body: row.body,
            publication_status: row.publication_status,
            publish_from: row.publish_from,
            publish_until: row.publish_until,
            published_at: row.published_at,
            image_id: row.image_id,
        }
    }
}
