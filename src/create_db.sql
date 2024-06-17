CREATE TABLE IF NOT EXISTS cookies (
    display_name TEXT NOT NULL,
    creation_utc INTEGER NOT NULL,
    value TEXT NOT NULL,
    expires_utc INTEGER NOT NULL,
    last_access_utc INTEGER NOT NULL,
    UNIQUE (value)
)