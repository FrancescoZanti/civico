use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::error::{ApiError, ApiResult};

/// Allowed upload MIME types (REQUIREMENTS FR-005). SVG stays disabled until
/// sanitization is implemented.
pub const ALLOWED_MIME_TYPES: &[&str] = &["image/jpeg", "image/png", "image/webp"];

/// Sniff the media type from the raw bytes (magic numbers), never trusting the
/// file extension alone.
pub fn sniff_mime(bytes: &[u8]) -> Option<&'static str> {
    if bytes.starts_with(&[0xff, 0xd8, 0xff]) {
        Some("image/jpeg")
    } else if bytes.len() >= 8 && bytes[..8] == [0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a] {
        Some("image/png")
    } else if bytes.len() >= 12 && bytes[..4] == *b"RIFF" && &bytes[8..12] == b"WEBP" {
        Some("image/webp")
    } else {
        None
    }
}

/// Validate size and MIME of an upload and, on success, return the normalized
/// MIME type. Rejects empty bodies and unsupported content.
pub fn validate_upload(
    bytes: &[u8],
    declared_mime: Option<&str>,
    max_bytes: u64,
) -> ApiResult<&'static str> {
    if bytes.is_empty() {
        return Err(ApiError::bad_request("uploaded file is empty"));
    }
    if bytes.len() as u64 > max_bytes {
        return Err(ApiError::new(
            axum::http::StatusCode::PAYLOAD_TOO_LARGE,
            "payload_too_large",
            format!("upload exceeds the maximum allowed size of {max_bytes} bytes"),
        ));
    }

    let sniffed = sniff_mime(bytes).ok_or_else(|| {
        ApiError::new(
            axum::http::StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "unsupported_media_type",
            "file content is not a supported image type",
        )
    })?;

    // If a declared MIME is present it must be consistent with the sniffed type.
    if let Some(declared) = declared_mime {
        let declared_lower = declared.split(';').next().unwrap_or_default().trim();
        if !ALLOWED_MIME_TYPES.contains(&declared_lower) {
            return Err(ApiError::new(
                axum::http::StatusCode::UNSUPPORTED_MEDIA_TYPE,
                "unsupported_media_type",
                format!("media type `{declared_lower}` is not allowed"),
            ));
        }
    }

    Ok(sniffed)
}

/// Compute the SHA-256 hex digest of the given bytes.
pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex(&hasher.finalize())
}

fn hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

/// Resolve a stored media path under the media directory, guaranteeing the
/// result stays within it (directory-traversal protection).
pub fn resolve_stored_path(media_dir: &Path, storage_path: &str) -> ApiResult<PathBuf> {
    let raw = media_dir.join(storage_path);
    let base = media_dir
        .canonicalize()
        .map_err(|_| ApiError::internal("resolving media directory"))?;
    let candidate = raw.canonicalize().map_err(|_| {
        ApiError::new(
            axum::http::StatusCode::NOT_FOUND,
            "media_not_found",
            "media file not found",
        )
    })?;
    if !candidate.starts_with(&base) {
        return Err(ApiError::bad_request("invalid media path"));
    }
    Ok(candidate)
}

/// Generate a storage path (relative to the media root) derived from content
/// hash, independent of the user-supplied filename. `ok` is the file id so the
/// path is unique even if two identical files are uploaded twice.
pub fn storage_path_for(media_id: Uuid, digest: &str, mime: &str) -> String {
    let ext = extension_for(mime).unwrap_or("bin");
    format!("{media_id}/{digest}.{ext}")
}

/// Persist an upload to disk under the media root and return its relative
/// `storage_path`. The parent directory is created as needed.
pub fn write_upload(
    media_dir: &Path,
    media_id: Uuid,
    digest: &str,
    mime: &str,
    bytes: &[u8],
) -> ApiResult<String> {
    let relative = storage_path_for(media_id, digest, mime);
    let absolute = media_dir.join(&relative);
    let parent = absolute
        .parent()
        .ok_or_else(|| ApiError::internal("building storage path"))?;
    std::fs::create_dir_all(parent)
        .map_err(|_| ApiError::internal("creating storage directory"))?;
    std::fs::write(&absolute, bytes).map_err(|_| ApiError::internal("writing media file"))?;
    Ok(relative)
}

/// Remove a stored file by its relative storage path, ignoring missing files.
pub fn delete_stored(media_dir: &Path, storage_path: &str) {
    if let Ok(abs) = resolve_stored_path(media_dir, storage_path) {
        let _ = std::fs::remove_file(&abs);
    }
}

fn extension_for(mime: &str) -> Option<&'static str> {
    match mime {
        "image/jpeg" => Some("jpg"),
        "image/png" => Some("png"),
        "image/webp" => Some("webp"),
        _ => None,
    }
}
