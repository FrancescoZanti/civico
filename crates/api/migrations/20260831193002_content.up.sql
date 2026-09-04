-- content: single table for all content types (FR-004).
-- A concrete content_type discriminator keeps the model extensible for
-- pages, news, events, places, contacts and galleries without a generic
-- website-builder abstraction. Type-specific fields live in the `data` JSONB.
CREATE TYPE content_type AS ENUM ('page', 'news', 'event', 'place', 'contact', 'gallery');

CREATE TYPE publication_status AS ENUM ('draft', 'published', 'archived');

CREATE TABLE content (
    id                 UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    content_type       content_type NOT NULL,
    title              TEXT NOT NULL,
    slug               TEXT NOT NULL UNIQUE,
    excerpt            TEXT,
    body               TEXT,
    data               JSONB NOT NULL DEFAULT '{}'::jsonb,
    publication_status publication_status NOT NULL DEFAULT 'draft',
    publish_from       TIMESTAMPTZ,
    publish_until      TIMESTAMPTZ,
    published_at       TIMESTAMPTZ,
    image_id           UUID REFERENCES media (id) ON DELETE SET NULL,
    created_by         UUID REFERENCES admin_users (id) ON DELETE SET NULL,
    created_at         TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at         TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT content_publish_window CHECK (
        publish_from IS NULL
        OR publish_until IS NULL
        OR publish_until > publish_from
    )
);

CREATE INDEX idx_content_type ON content (content_type);
CREATE INDEX idx_content_status ON content (publication_status);
