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
    date DATETIME NOT NULL,
    activity_type TEXT NOT NULL CHECK (activity_type IN ('add_tx', 'edit_tx', 'delete_tx', 'search_tx', 'position_swap'))
);

CREATE TABLE txs (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    date DATE NOT NULL,
    details TEXT,
    from_method INTEGER NOT NULL REFERENCES tx_methods(id) ON DELETE CASCADE,
    to_method INTEGER REFERENCES tx_methods(id) ON DELETE CASCADE,
    amount BigInt NOT NULL,
    tx_type TEXT NOT NULL CHECK (tx_type IN ('Income', 'Expense', 'Transfer')),
    display_order INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE tx_tags (
    tx_id INTEGER NOT NULL REFERENCES txs(id) ON DELETE CASCADE,
    tag_id INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (tx_id, tag_id)
);

CREATE TABLE balances (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    method_id INTEGER NOT NULL REFERENCES tx_methods(id) ON DELETE CASCADE,
    year INTEGER NOT NULL,
    month INTEGER NOT NULL,
    balance BigInt NOT NULL,
    is_final_balance BOOLEAN NOT NULL DEFAULT false
);

CREATE TABLE activity_txs (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    date TEXT,
    details TEXT,
    from_method INTEGER REFERENCES tx_methods(id) ON DELETE CASCADE,
    to_method INTEGER REFERENCES tx_methods(id) ON DELETE CASCADE,
    amount BigInt,
    amount_type TEXT CHECK (amount_type IN ('exact', 'more_than', 'more_than_equal', 'less_than', 'less_than_equal')),
    tx_type TEXT CHECK (tx_type IN ('Income', 'Expense', 'Transfer')),
    display_order INTEGER,
    activity_num INTEGER NOT NULL REFERENCES activities(id) ON DELETE CASCADE
);


CREATE TABLE activity_tx_tags (
    tx_id INTEGER NOT NULL REFERENCES activity_txs(id) ON DELETE CASCADE,
    tag_id INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (tx_id, tag_id)
);


CREATE UNIQUE INDEX idx_tx_method_name ON tx_methods(name);

CREATE UNIQUE INDEX idx_tags_name ON tags(name);

CREATE INDEX idx_activities_date_type ON activities(date, activity_type);

CREATE INDEX idx_txs_date ON txs(date);
CREATE INDEX idx_txs_from_method ON txs(from_method);
CREATE INDEX idx_txs_to_method ON txs(to_method);
CREATE INDEX idx_txs_amount ON txs(amount);

CREATE INDEX idx_tx_tags_tx_id ON tx_tags(tx_id);
CREATE INDEX idx_tx_tags_tag_id ON tx_tags(tag_id);
CREATE INDEX idx_tx_tags_tag_tx ON tx_tags(tag_id, tx_id);

CREATE UNIQUE INDEX idx_balances_method_period ON balances(method_id, year, month);
CREATE UNIQUE INDEX idx_final_balance_unique ON balances(method_id) WHERE is_final_balance = 1;
CREATE INDEX idx_balances_method_id ON balances(method_id);

CREATE INDEX idx_activity_txs_activity_num ON activity_txs(activity_num);

CREATE INDEX idx_activity_tx_tags_tx_id ON activity_tx_tags(tx_id);

-- Default tag
INSERT OR IGNORE INTO tags (id, name)
VALUES (1, 'Unknown');
