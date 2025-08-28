CREATE TABLE tx_methods (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    name TEXT NOT NULL UNIQUE,
    position INTEGER NOT NULL
);

CREATE TABLE tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    name TEXT NOT NULL UNIQUE
);

CREATE TABLE activities (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    date DATE NOT NULL,
    activity_type TEXT NOT NULL CHECK (activity_type IN ('add_tx', 'edit_tx', 'delete_tx', 'search_tx', 'position_swap')),
    description TEXT NOT NULL
);

CREATE TABLE txs (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    date DATE NOT NULL,
    details TEXT,
    from_method INTEGER NOT NULL REFERENCES tx_methods(id),
    to_method INTEGER REFERENCES tx_methods(id),
    amount BigInt NOT NULL,
    tx_type TEXT NOT NULL CHECK (tx_type IN ('income', 'expense', 'transfer')),
    activity_id INTEGER REFERENCES activities(id) ON DELETE SET NULL
);

CREATE TABLE tx_tags (
    tx_id INTEGER NOT NULL REFERENCES txs(id) ON DELETE CASCADE,
    tag_id INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (tx_id, tag_id)
);

CREATE TABLE balances (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    method_id INTEGER NOT NULL REFERENCES tx_methods(id),
    year INTEGER NOT NULL,
    month INTEGER NOT NULL,
    balance BigInt NOT NULL,
    is_final_balance BOOLEAN NOT NULL DEFAULT false
);


CREATE UNIQUE INDEX idx_tx_method_name ON tx_methods(name);

CREATE UNIQUE INDEX idx_tags_name ON tags(name);

CREATE INDEX idx_activities_date ON activities(date);
CREATE INDEX idx_activities_type ON activities(activity_type);

CREATE INDEX idx_txs_date ON txs(date);
CREATE INDEX idx_txs_from_method ON txs(from_method);
CREATE INDEX idx_txs_to_method ON txs(to_method);
CREATE INDEX idx_txs_activity_id ON txs(activity_id);

CREATE INDEX idx_tx_tags_tx_id ON tx_tags(tx_id);
CREATE INDEX idx_tx_tags_tag_id ON tx_tags(tag_id);

CREATE UNIQUE INDEX idx_balances_method_period ON balances(method_id, year, month);
CREATE UNIQUE INDEX idx_final_balance_unique ON balances(method_id) WHERE is_final_balance = 1;


INSERT OR IGNORE INTO tags (id, name)
VALUES (1, 'Unknown');
