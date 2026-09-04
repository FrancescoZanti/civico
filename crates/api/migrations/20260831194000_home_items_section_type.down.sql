DROP INDEX idx_home_items_section_type;

ALTER TABLE home_items
    DROP CONSTRAINT home_item_section_requires_type;

ALTER TABLE home_items
    DROP CONSTRAINT home_item_content_requires_content;

ALTER TABLE home_items
    ADD CONSTRAINT home_item_section_destination_no_content CHECK (
        destination_type <> 'section'
        OR destination_content_id IS NULL
    );

ALTER TABLE home_items
    ADD CONSTRAINT home_item_content_destination_requires_content CHECK (
        destination_type <> 'content'
        OR destination_content_id IS NOT NULL
    );

ALTER TABLE home_items DROP COLUMN destination_section_type;
