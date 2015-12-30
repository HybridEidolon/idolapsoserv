pub static SCHEMA: &'static str = "
BEGIN;

CREATE TABLE IF NOT EXISTS version (
    version INTEGER PRIMARY KEY
);
INSERT OR REPLACE INTO version (version) VALUES (0);

CREATE TABLE IF NOT EXISTS accounts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT UNIQUE,
    password_hash TEXT,
    password_invalidated INTEGER,
    banned INTEGER
);

COMMIT;
";

//pub static MIGRATIONS: [&'static str] = [""];
