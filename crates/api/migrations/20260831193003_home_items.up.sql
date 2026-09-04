-- home_items: the configurable HOME screen buttons (FR-001).
-- Buttons are data rows, not hardcoded in the player. Each button's destination
-- is discriminated: it may target a whole section (a content_type listing) or a
-- specific content row.
CREATE TYPE home_destination_type AS ENUM ('section', 'content');

CREATE TABLE home_items (
    id                     UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title                  TEXT NOT NULL,
    subtitle               TEXT,
    icon                   TEXT,
    image_id               UUID REFERENCES media (id) ON DELETE SET NULL,
    position               INTEGER NOT NULL DEFAULT 0,
    enabled                BOOLEAN NOT NULL DEFAULT TRUE,
    destination_type       home_destination_type NOT NULL,
    destination_content_id UUID REFERENCES content (id) ON DELETE SET NULL,
    created_at             TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at             TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT home_item_section_destination_no_content CHECK (
        destination_type <> 'section'
        OR destination_content_id IS NULL
    ),
    CONSTRAINT home_item_content_destination_requires_content CHECK (
        destination_type <> 'content'
        OR destination_content_id IS NOT NULL
    )
);

CREATE INDEX idx_home_items_position ON home_items (position);
CREATE INDEX idx_home_items_enabled ON home_items (enabled);
