-- media: library of uploaded image files.
-- Only JPEG, PNG and WebP are permitted in the MVP (FR-005). SVG stays disabled
-- until sanitization exists. The CHECK constraint enforces allowed MIME types.
CREATE TABLE media (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    filename     TEXT NOT NULL,
    mime_type    TEXT NOT NULL CHECK (mime_type IN ('image/jpeg', 'image/png', 'image/webp')),
    size_bytes   BIGINT NOT NULL CHECK (size_bytes >= 0),
    storage_path TEXT NOT NULL UNIQUE,
    width        INTEGER,
    height       INTEGER,
    created_by   UUID REFERENCES admin_users (id) ON DELETE SET NULL,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);
