CREATE TABLE journal_entries (
    id          UUID PRIMARY KEY,
    date        DATE NOT NULL,
    description TEXT NOT NULL CHECK (description <> '')
);

CREATE TABLE journal_line_items (
    id               UUID PRIMARY KEY,
    journal_entry_id UUID NOT NULL REFERENCES journal_entries (id) ON DELETE CASCADE,
    account_code     TEXT NOT NULL REFERENCES accounts (code),
    amount           BIGINT NOT NULL CHECK (amount > 0),
    entry_type       TEXT NOT NULL CHECK (entry_type IN ('credit', 'debit')),
    description      TEXT CHECK (description <> '')
);

CREATE INDEX journal_line_items_journal_entry_id_idx ON journal_line_items (journal_entry_id);
