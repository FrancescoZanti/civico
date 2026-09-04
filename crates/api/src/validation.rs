use chrono::{DateTime, Utc};

use crate::error::{ApiError, ApiResult};

/// Validate that a slug uses only lowercase letters, digits and single hyphens.
pub fn validate_slug(slug: &str) -> ApiResult<()> {
    let valid = !slug.is_empty()
        && slug.len() <= 120
        && slug
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        && !slug.starts_with('-')
        && !slug.ends_with('-')
        && !slug.contains("--");

    if !valid {
        return Err(ApiError::validation(
            "invalid slug",
            crate::dto::field_error(
                "slug",
                "must be 1-120 characters: lowercase letters, digits, single hyphens",
            ),
        ));
    }
    Ok(())
}

/// Validate the publish_from / publish_until window, if both are set.
pub fn validate_publish_window(
    publish_from: Option<DateTime<Utc>>,
    publish_until: Option<DateTime<Utc>>,
) -> ApiResult<()> {
    if let (Some(from), Some(until)) = (publish_from, publish_until)
        && until <= from
    {
        return Err(ApiError::validation(
            "publish_until must be later than publish_from",
            crate::dto::field_error("publish_until", "must be after publish_from"),
        ));
    }
    Ok(())
}
