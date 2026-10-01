CREATE TABLE recurring_txs (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    created_at DATETIME NOT NULL,
    details TEXT,
    from_method INTEGER NOT NULL REFERENCES tx_methods (id) ON DELETE CASCADE,
    to_method INTEGER REFERENCES tx_methods (id) ON DELETE CASCADE,
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
    frequency TEXT NOT NULL CHECK (
        frequency IN ('Daily', 'Weekly', 'Monthly', 'Yearly')
    ),
    recur_interval INTEGER NOT NULL DEFAULT 1 CHECK (recur_interval >= 1),
    recur_value INTEGER CHECK (
        (
            frequency = 'Daily'
            AND recur_value IS NULL
        )
        OR (
            frequency = 'Weekly'
            AND recur_value BETWEEN 0 AND 6
        )
        OR (
            frequency IN ('Monthly', 'Yearly')
            AND recur_value BETWEEN 1 AND 31
        )
    ),
    recur_month INTEGER CHECK (
        (
            frequency != 'Yearly'
            AND recur_month IS NULL
        )
        OR (
            frequency = 'Yearly'
            AND recur_month BETWEEN 1 AND 12
        )
    ),
    last_recurred_date DATE,
    next_recurring_date DATE NOT NULL,
    end_date DATE,
    is_paused BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE TABLE recurring_tx_tags (
    recurring_tx_id INTEGER NOT NULL REFERENCES recurring_txs (id) ON DELETE CASCADE,
    tag_id INTEGER NOT NULL REFERENCES tags (id) ON DELETE CASCADE,
    is_primary BOOLEAN NOT NULL DEFAULT FALSE,
    PRIMARY KEY (recurring_tx_id, tag_id)
);

CREATE INDEX idx_recurring_txs_next_date ON recurring_txs (next_recurring_date);

CREATE INDEX idx_recurring_txs_from_method ON recurring_txs (from_method);

CREATE INDEX idx_recurring_txs_to_method ON recurring_txs (to_method);

CREATE INDEX idx_recurring_tx_tags_recurring_tx_id ON recurring_tx_tags (recurring_tx_id);

CREATE INDEX idx_recurring_tx_tags_tag_id ON recurring_tx_tags (tag_id);
