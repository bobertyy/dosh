CREATE TABLE accounts (
    code        TEXT PRIMARY KEY,
    class       TEXT NOT NULL CHECK (class IN ('asset', 'equity', 'expense', 'liability', 'revenue')),
    description TEXT CHECK (description <> '')
);
