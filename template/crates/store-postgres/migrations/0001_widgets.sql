-- gen_random_uuid() is provided by pgcrypto. PostgreSQL 13+ also has it in
-- core; the extension keeps a fresh database on either version able to insert.
CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE TABLE widgets (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL
);
