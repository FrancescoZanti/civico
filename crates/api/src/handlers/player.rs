use axum::Json;
use axum::extract::{Path, Query, State};
use serde::Deserialize;
use std::str::FromStr;
use uuid::Uuid;

use crate::domain::{ContentType, HomeDestinationType};
use crate::dto::{PlayerContent, PlayerDestination, PlayerHomeItem};
use crate::error::{ApiError, ApiResult};
use crate::repo;
use crate::state::AppState;

fn media_url(id: Uuid) -> String {
    format!("/api/v1/media/{id}/file")
}

// ------------------------------------------------------------------- player

/// Public HOME: only enabled items, ordered by position, with destinations
/// resolved so the player can navigate (section -> content type, content -> slug).
pub async fn player_home(State(state): State<AppState>) -> ApiResult<Json<Vec<PlayerHomeItem>>> {
    let items = repo::home_item_list_enabled(&state.db)
        .await
        .map_err(|_| ApiError::internal("loading home"))?;

    let mut out = Vec::with_capacity(items.len());
    for item in items {
        let image_url = item.image_id.map(media_url);

        let destination = match item.destination_type {
            HomeDestinationType::Section => {
                let Some(section_type) = item.destination_section_type else {
                    // Should not happen under the DB/API constraints; skip defensively.
                    continue;
                };
                PlayerDestination::Section {
                    content_type: section_type,
                }
            }
            HomeDestinationType::Content => {
                let Some(content_id) = item.destination_content_id else {
                    continue;
                };
                let Some(slug) = repo::content_slug(&state.db, content_id)
                    .await
                    .map_err(|_| ApiError::internal("resolving home destination"))?
                else {
                    continue;
                };
                PlayerDestination::Content { slug }
            }
        };

        out.push(PlayerHomeItem {
            id: item.id,
            title: item.title,
            subtitle: item.subtitle,
            icon: item.icon,
            image_url,
            destination,
        });
    }

    Ok(Json(out))
}

#[derive(Debug, Deserialize)]
pub struct ContentListQuery {
    #[serde(default)]
    pub content_type: Option<String>,
}

/// Public content listing: only effectively-published items, optionally by type.
pub async fn player_content_list(
    State(state): State<AppState>,
    Query(query): Query<ContentListQuery>,
) -> ApiResult<Json<Vec<PlayerContent>>> {
    let content_type = match query.content_type {
        Some(raw) => {
            if raw.is_empty() {
                None
            } else {
                Some(ContentType::from_str(&raw).map_err(|_| {
                    ApiError::validation(
                        "invalid content_type",
                        crate::dto::field_error("content_type", "unsupported content type"),
                    )
                })?)
            }
        }
        None => None,
    };

    let rows = repo::content_list_published(&state.db, content_type)
        .await
        .map_err(|_| ApiError::internal("listing published content"))?;

    Ok(Json(rows.into_iter().map(Into::into).collect()))
}

/// Public content detail: effectively-published only, resolved by slug.
pub async fn player_content_detail(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> ApiResult<Json<PlayerContent>> {
    let row = repo::content_get_public_by_slug(&state.db, &slug)
        .await
        .map_err(|_| ApiError::internal("fetching content"))?
        .ok_or_else(|| ApiError::not_found("content", &slug))?;
    Ok(Json(row.into()))
}

impl From<repo::ContentRow> for PlayerContent {
    fn from(row: repo::ContentRow) -> Self {
        PlayerContent {
            id: row.id,
            content_type: row.content_type,
            title: row.title,
            slug: row.slug,
            excerpt: row.excerpt,
            body: row.body,
            image_url: row.image_id.map(media_url),
        }
    }
}
