CREATE TABLE tx_tags_new (
    tx_id INTEGER NOT NULL REFERENCES txs(id) ON DELETE CASCADE,
    tag_id INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    is_primary BOOLEAN NOT NULL DEFAULT FALSE,
    PRIMARY KEY (tx_id, tag_id)
);

INSERT INTO tx_tags_new (tx_id, tag_id, is_primary)
SELECT
    tx_id,
    tag_id,
    CASE
        WHEN ROW_NUMBER() OVER (PARTITION BY tx_id ORDER BY tag_id) = 1 THEN 1
        ELSE 0
    END AS is_primary
FROM tx_tags;

DROP TABLE tx_tags;

ALTER TABLE tx_tags_new RENAME TO tx_tags;

CREATE INDEX IF NOT EXISTS idx_tx_tags_tx_id ON tx_tags(tx_id);
CREATE INDEX IF NOT EXISTS idx_tx_tags_tag_id ON tx_tags(tag_id);
CREATE INDEX IF NOT EXISTS idx_tx_tags_tag_tx ON tx_tags(tag_id, tx_id);
