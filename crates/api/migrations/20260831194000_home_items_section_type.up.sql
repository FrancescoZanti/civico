-- HOME `section` destinations need to name the section (a content_type) so the
-- player knows which listing to open. This refines the Phase 2 destination
-- model: sections carry destination_section_type; content destinations keep
-- destination_content_id. The new CHECK constraints make the two mutually
-- exclusive, replacing the Phase 2 constraints.
ALTER TABLE home_items
    ADD COLUMN destination_section_type content_type;

ALTER TABLE home_items
    DROP CONSTRAINT home_item_section_destination_no_content;

ALTER TABLE home_items
    DROP CONSTRAINT home_item_content_destination_requires_content;

ALTER TABLE home_items
    ADD CONSTRAINT home_item_section_requires_type CHECK (
        destination_type <> 'section'
        OR (destination_section_type IS NOT NULL AND destination_content_id IS NULL)
    );

ALTER TABLE home_items
    ADD CONSTRAINT home_item_content_requires_content CHECK (
        destination_type <> 'content'
        OR (destination_section_type IS NULL AND destination_content_id IS NOT NULL)
    );

CREATE INDEX idx_home_items_section_type ON home_items (destination_section_type);
