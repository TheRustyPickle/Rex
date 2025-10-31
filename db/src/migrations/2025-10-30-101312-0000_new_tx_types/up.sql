PRAGMA foreign_keys = OFF;

ALTER TABLE txs RENAME TO txs_old;
ALTER TABLE activity_txs RENAME TO activity_txs_old;

CREATE TABLE txs (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    date DATETIME NOT NULL,
    details TEXT,
    from_method INTEGER NOT NULL REFERENCES tx_methods(id) ON DELETE CASCADE,
    to_method INTEGER REFERENCES tx_methods(id) ON DELETE CASCADE,
    amount BigInt NOT NULL,
    tx_type TEXT NOT NULL CHECK (
        tx_type IN (
            'Income',
            'Expense',
            'Transfer',
            'Borrow',
            'Lend',
            'Borrow Repay',
            'Lend Repay'
        )
    ),
    display_order INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE activity_txs (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    date TEXT,
    details TEXT,
    from_method INTEGER REFERENCES tx_methods(id) ON DELETE CASCADE,
    to_method INTEGER REFERENCES tx_methods(id) ON DELETE CASCADE,
    amount BigInt,
    amount_type TEXT CHECK (amount_type IN ('exact', 'more_than', 'more_than_equal', 'less_than', 'less_than_equal')),
    tx_type TEXT CHECK (
        tx_type IN (
            'Income',
            'Expense',
            'Transfer',
            'Borrow',
            'Lend',
            'Borrow Repay',
            'Lend Repay'
        )
    ),
    display_order INTEGER,
    activity_num INTEGER NOT NULL REFERENCES activities(id) ON DELETE CASCADE
);

INSERT INTO txs (id, date, details, from_method, to_method, amount, tx_type, display_order)
SELECT id, date, details, from_method, to_method, amount, tx_type, display_order FROM txs_old;

INSERT INTO activity_txs (id, date, details, from_method, to_method, amount, amount_type, tx_type, display_order, activity_num)
SELECT id, date, details, from_method, to_method, amount, amount_type, tx_type, display_order, activity_num FROM activity_txs_old;

ALTER TABLE tx_tags RENAME TO tx_tags_old;

CREATE TABLE tx_tags (
    tx_id INTEGER NOT NULL REFERENCES txs(id) ON DELETE CASCADE,
    tag_id INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (tx_id, tag_id)
);

INSERT INTO tx_tags (tx_id, tag_id)
SELECT tx_id, tag_id FROM tx_tags_old;

ALTER TABLE activity_tx_tags RENAME TO activity_tx_tags_old;

CREATE TABLE activity_tx_tags (
    tx_id INTEGER NOT NULL REFERENCES activity_txs(id) ON DELETE CASCADE,
    tag_id INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (tx_id, tag_id)
);

INSERT INTO activity_tx_tags (tx_id, tag_id)
SELECT tx_id, tag_id FROM activity_tx_tags_old;

DROP TABLE tx_tags_old;
DROP TABLE activity_tx_tags_old;
DROP TABLE txs_old;
DROP TABLE activity_txs_old;

CREATE UNIQUE INDEX IF NOT EXISTS idx_tx_method_name ON tx_methods(name);
CREATE UNIQUE INDEX IF NOT EXISTS idx_tags_name ON tags(name);
CREATE UNIQUE INDEX IF NOT EXISTS idx_balances_method_period ON balances(method_id, year, month);
CREATE UNIQUE INDEX IF NOT EXISTS idx_final_balance_unique ON balances(method_id) WHERE is_final_balance = 1;

CREATE INDEX IF NOT EXISTS idx_activities_date_type ON activities(date, activity_type);

CREATE INDEX IF NOT EXISTS idx_txs_date ON txs(date);
CREATE INDEX IF NOT EXISTS idx_txs_from_method ON txs(from_method);
CREATE INDEX IF NOT EXISTS idx_txs_to_method ON txs(to_method);
CREATE INDEX IF NOT EXISTS idx_txs_amount ON txs(amount);

CREATE INDEX IF NOT EXISTS idx_tx_tags_tx_id ON tx_tags(tx_id);
CREATE INDEX IF NOT EXISTS idx_tx_tags_tag_id ON tx_tags(tag_id);
CREATE INDEX IF NOT EXISTS idx_tx_tags_tag_tx ON tx_tags(tag_id, tx_id);

CREATE INDEX IF NOT EXISTS idx_balances_method_id ON balances(method_id);

CREATE INDEX IF NOT EXISTS idx_activity_txs_activity_num ON activity_txs(activity_num);

CREATE INDEX IF NOT EXISTS idx_activity_tx_tags_tx_id ON activity_tx_tags(tx_id);

-- Default tag
INSERT OR IGNORE INTO tags (id, name) VALUES (1, 'Unknown');

PRAGMA foreign_keys = ON;
