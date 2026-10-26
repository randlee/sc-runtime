-- UUID() is built into MySQL, so a fresh database can insert without an extension.
-- VARCHAR keeps the identifier indexable; MySQL refuses a TEXT primary key
-- (error 1170). utf8mb4_bin keeps primary-key equality case-sensitive, matching
-- TEXT keys on SQLite and Postgres. MEDIUMTEXT holds a legal 1 MiB pairing
-- payload; MySQL TEXT is only 64 KiB.
CREATE TABLE widgets (
    id VARCHAR(255) CHARACTER SET utf8mb4 COLLATE utf8mb4_bin NOT NULL,
    name MEDIUMTEXT NOT NULL,
    PRIMARY KEY (id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
