use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use uuid::Uuid;

use crate::auth::AuthAdmin;
use crate::domain::{ContentType, HomeDestinationType};
use crate::dto::{HomeItemCreate, HomeItemDto, HomeItemUpdate};
use crate::error::{ApiError, ApiResult};
use crate::repo;
use crate::state::AppState;

/// Validate destination consistency at the API layer, mirroring the DB CHECKs.
/// Sections name a content_type; content destinations reference a content row.
fn validate_destination(
    destination_type: HomeDestinationType,
    destination_section_type: Option<ContentType>,
    destination_content_id: Option<Uuid>,
) -> ApiResult<()> {
    match destination_type {
        HomeDestinationType::Section => {
            if destination_section_type.is_none() {
                return Err(ApiError::validation(
                    "a section destination requires destination_section_type",
                    crate::dto::field_error("destination_section_type", "is required"),
                ));
            }
            if destination_content_id.is_some() {
                return Err(ApiError::validation(
                    "a section destination cannot reference specific content",
                    crate::dto::field_error("destination_content_id", "must be null for sections"),
                ));
            }
        }
        HomeDestinationType::Content => {
            if destination_content_id.is_none() {
                return Err(ApiError::validation(
                    "a content destination requires destination_content_id",
                    crate::dto::field_error("destination_content_id", "is required"),
                ));
            }
            if destination_section_type.is_some() {
                return Err(ApiError::validation(
                    "a content destination cannot name a section type",
                    crate::dto::field_error("destination_section_type", "must be null for content"),
                ));
            }
        }
    }
    Ok(())
}

pub async fn create_home_item(
    State(state): State<AppState>,
    _auth: AuthAdmin,
    Json(req): Json<HomeItemCreate>,
) -> ApiResult<(StatusCode, Json<HomeItemDto>)> {
    validate_destination(
        req.destination_type,
        req.destination_section_type,
        req.destination_content_id,
    )?;
    validate_content_reference(&state, req.destination_content_id).await?;

    let row = repo::home_item_create(
        &state.db,
        &req.title,
        req.subtitle.as_deref(),
        req.icon.as_deref(),
        req.image_id,
        req.position,
        req.enabled,
        req.destination_type,
        req.destination_section_type,
        req.destination_content_id,
    )
    .await
    .map_err(|_| ApiError::internal("creating home item"))?;

    Ok((StatusCode::CREATED, Json(row.into())))
}

pub async fn list_home_items(
    State(state): State<AppState>,
    _auth: AuthAdmin,
) -> ApiResult<Json<Vec<HomeItemDto>>> {
    let rows = repo::home_item_list(&state.db)
        .await
        .map_err(|_| ApiError::internal("listing home items"))?;
    Ok(Json(rows.into_iter().map(Into::into).collect()))
}

pub async fn get_home_item(
    State(state): State<AppState>,
    _auth: AuthAdmin,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<HomeItemDto>> {
    let row = repo::home_item_get(&state.db, id)
        .await
        .map_err(|_| ApiError::internal("fetching home item"))?
        .ok_or_else(|| ApiError::not_found("home item", &id.to_string()))?;
    Ok(Json(row.into()))
}

pub async fn update_home_item(
    State(state): State<AppState>,
    _auth: AuthAdmin,
    Path(id): Path<Uuid>,
    Json(req): Json<HomeItemUpdate>,
) -> ApiResult<Json<HomeItemDto>> {
    let current = repo::home_item_get(&state.db, id)
        .await
        .map_err(|_| ApiError::internal("fetching home item"))?
        .ok_or_else(|| ApiError::not_found("home item", &id.to_string()))?;

    // Resolve the final destination from current + requested fields so that a
    // destination change atomically sets the correct companion field.
    let destination_type = req.destination_type.unwrap_or(current.destination_type);
    let destination_section_type = req
        .destination_section_type
        .or(current.destination_section_type);
    let destination_content_id = req
        .destination_content_id
        .or(current.destination_content_id);

    validate_destination(
        destination_type,
        destination_section_type,
        destination_content_id,
    )?;
    validate_content_reference(&state, destination_content_id).await?;

    let row = repo::home_item_update(
        &state.db,
        id,
        req.title.as_deref(),
        req.subtitle.as_deref(),
        req.icon.as_deref(),
        req.image_id,
        req.position,
        req.enabled,
        destination_type,
        destination_section_type,
        destination_content_id,
    )
    .await
    .map_err(|_| ApiError::internal("updating home item"))?
    .ok_or_else(|| ApiError::not_found("home item", &id.to_string()))?;

    Ok(Json(row.into()))
}

pub async fn delete_home_item(
    State(state): State<AppState>,
    _auth: AuthAdmin,
    Path(id): Path<Uuid>,
) -> ApiResult<Response> {
    let deleted = repo::home_item_delete(&state.db, id)
        .await
        .map_err(|_| ApiError::internal("deleting home item"))?;
    if !deleted {
        return Err(ApiError::not_found("home item", &id.to_string()));
    }
    Ok(StatusCode::NO_CONTENT.into_response())
}

/// When a home item points at specific content, ensure that content exists.
async fn validate_content_reference(state: &AppState, id: Option<Uuid>) -> ApiResult<()> {
    if let Some(content_id) = id {
        let exists = repo::content_get(&state.db, content_id)
            .await
            .map_err(|_| ApiError::internal("checking content reference"))?
            .is_some();
        if !exists {
            return Err(ApiError::validation(
                "referenced content does not exist",
                crate::dto::field_error("destination_content_id", "content not found"),
            ));
        }
    }
    Ok(())
}

impl From<repo::HomeItemRow> for HomeItemDto {
    fn from(row: repo::HomeItemRow) -> Self {
        HomeItemDto {
            id: row.id,
            title: row.title,
            subtitle: row.subtitle,
            icon: row.icon,
            image_id: row.image_id,
            position: row.position,
            enabled: row.enabled,
            destination_type: row.destination_type,
            destination_section_type: row.destination_section_type,
            destination_content_id: row.destination_content_id,
        }
    }
}
