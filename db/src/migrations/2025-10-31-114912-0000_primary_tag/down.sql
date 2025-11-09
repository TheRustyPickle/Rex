CREATE TABLE tx_tags_old (
    tx_id INTEGER NOT NULL REFERENCES txs(id) ON DELETE CASCADE,
    tag_id INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (tx_id, tag_id)
);

INSERT INTO tx_tags_old (tx_id, tag_id)
SELECT tx_id, tag_id
FROM tx_tags;

DROP TABLE tx_tags;

ALTER TABLE tx_tags_old RENAME TO tx_tags;
