-- UUID() is built into MySQL, so a fresh database can insert without an extension.
-- VARCHAR keeps the identifier indexable; MySQL refuses a TEXT primary key.
CREATE TABLE widgets (
    id VARCHAR(255) NOT NULL,
    name TEXT NOT NULL,
    PRIMARY KEY (id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
