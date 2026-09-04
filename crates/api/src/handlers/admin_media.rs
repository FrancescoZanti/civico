use axum::Json;
use axum::body::Bytes;
use axum::extract::{Multipart, Path, State};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use uuid::Uuid;

use crate::auth::AuthAdmin;
use crate::dto::MediaDto;
use crate::error::{ApiError, ApiResult};
use crate::media_store;
use crate::repo;
use crate::state::AppState;

/// Public (no-auth) media file serving used by the player. Reads the stored
/// file from disk behind directory-traversal protection and streams it with
/// the correct content type.
pub async fn serve_media_file(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<Response> {
    let row = repo::media_get(&state.db, id)
        .await
        .map_err(|_| ApiError::internal("fetching media"))?
        .ok_or_else(|| ApiError::not_found("media", &id.to_string()))?;

    let path = media_store::resolve_stored_path(&state.config.media_dir, &row.storage_path)?;
    let bytes = tokio::fs::read(&path)
        .await
        .map_err(|_| ApiError::internal("reading media file"))?;

    Ok((
        [
            (header::CONTENT_TYPE, row.mime_type.clone()),
            (header::CACHE_CONTROL, "public, max-age=86400".to_string()),
        ],
        bytes,
    )
        .into_response())
}

// ------------------------------------------------------------------ admin

/// Multipart upload: expects a single file field named `file`. Validates size
/// and MIME (by content sniffing) before persisting.
pub async fn upload_media(
    State(state): State<AppState>,
    auth: AuthAdmin,
    mut multipart: Multipart,
) -> ApiResult<(StatusCode, Json<MediaDto>)> {
    let mut uploaded: Option<(String, Vec<u8>)> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| ApiError::bad_request("malformed multipart body"))?
    {
        let name = field.name().unwrap_or_default().to_string();
        if name != "file" {
            continue;
        }
        let filename = field.file_name().unwrap_or("upload").to_string();
        let declared_mime = field.content_type().map(|s| s.to_string());
        let bytes = read_bounded(field, state.config.max_upload_bytes as usize).await?;
        uploaded = Some((filename, bytes.to_vec()));

        // Only accept one file per request.
        let _ = declared_mime;
        break;
    }

    let (filename, bytes) = uploaded.ok_or_else(|| {
        ApiError::validation(
            "a multipart file field named `file` is required",
            serde_json::json!({"file": "required"}),
        )
    })?;

    let mime = media_store::validate_upload(&bytes, None, state.config.max_upload_bytes)?;
    let digest = media_store::sha256_hex(&bytes);
    let media_id = Uuid::new_v4();
    let storage_path =
        media_store::write_upload(&state.config.media_dir, media_id, &digest, mime, &bytes)?;

    let row = repo::media_create(
        &state.db,
        &sanitize_filename(&filename),
        mime,
        bytes.len() as i64,
        &storage_path,
        None,
        None,
        auth.id,
    )
    .await
    .map_err(|_| ApiError::internal("creating media record"))?;

    Ok((StatusCode::CREATED, Json(MediaDto::from_row(row, make_url))))
}

async fn read_bounded(field: axum::extract::multipart::Field<'_>, max: usize) -> ApiResult<Bytes> {
    let mut body = Vec::new();
    let mut chunks = field;
    while let Some(chunk) = chunks
        .chunk()
        .await
        .map_err(|_| ApiError::bad_request("failed to read upload body"))?
    {
        body.extend_from_slice(&chunk);
        if body.len() > max {
            return Err(ApiError::new(
                StatusCode::PAYLOAD_TOO_LARGE,
                "payload_too_large",
                format!("upload exceeds the maximum allowed size of {max} bytes"),
            ));
        }
    }
    Ok(Bytes::from(body))
}

pub async fn list_media(
    State(state): State<AppState>,
    _auth: AuthAdmin,
) -> ApiResult<Json<Vec<MediaDto>>> {
    let rows = repo::media_list(&state.db)
        .await
        .map_err(|_| ApiError::internal("listing media"))?;
    Ok(Json(
        rows.into_iter()
            .map(|r| MediaDto::from_row(r, make_url))
            .collect(),
    ))
}

pub async fn get_media(
    State(state): State<AppState>,
    _auth: AuthAdmin,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<MediaDto>> {
    let row = repo::media_get(&state.db, id)
        .await
        .map_err(|_| ApiError::internal("fetching media"))?
        .ok_or_else(|| ApiError::not_found("media", &id.to_string()))?;
    Ok(Json(MediaDto::from_row(row, make_url)))
}

pub async fn delete_media(
    State(state): State<AppState>,
    _auth: AuthAdmin,
    Path(id): Path<Uuid>,
) -> ApiResult<Response> {
    let row = repo::media_get(&state.db, id)
        .await
        .map_err(|_| ApiError::internal("fetching media"))?
        .ok_or_else(|| ApiError::not_found("media", &id.to_string()))?;

    let deleted = repo::media_delete(&state.db, id)
        .await
        .map_err(|_| ApiError::internal("deleting media"))?;
    if !deleted {
        return Err(ApiError::not_found("media", &id.to_string()));
    }

    media_store::delete_stored(&state.config.media_dir, &row.storage_path);
    Ok(StatusCode::NO_CONTENT.into_response())
}

fn make_url(id: Uuid) -> String {
    format!("/api/v1/media/{id}/file")
}

/// Keep user-supplied filenames safe for storage metadata: strip path
/// separators and other unsafe characters.
fn sanitize_filename(filename: &str) -> String {
    let name = filename.rsplit(['/', '\\']).next().unwrap_or("upload");
    let cleaned: String = name
        .chars()
        .map(|c| {
            if c.is_control() || c == '/' || c == '\\' {
                '_'
            } else {
                c
            }
        })
        .collect();
    if cleaned.is_empty() {
        "upload".to_string()
    } else {
        cleaned
    }
}
