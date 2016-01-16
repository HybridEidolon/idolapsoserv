pub static SCHEMA: &'static str = "
BEGIN;

CREATE TABLE IF NOT EXISTS version (
    version INTEGER PRIMARY KEY
);
INSERT OR REPLACE INTO version (version) VALUES (0);

CREATE TABLE IF NOT EXISTS accounts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    password_invalidated INTEGER NOT NULL DEFAULT 0,
    banned INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS bb_guildcard (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL DEFAULT 400000000,
    account_id INTEGER UNIQUE NOT NULL,
    team_id INTEGER NOT NULL DEFAULT 1,
    options INTEGER NOT NULL DEFAULT 0,
    key_config BLOB NOT NULL,
    joy_config BLOB NOT NULL,
    shortcuts BLOB NOT NULL,
    symbol_chats BLOB NOT NULL
);

CREATE TABLE IF NOT EXISTS bb_team (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    name TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS bb_character (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    account_id INTEGER NOT NULL DEFAULT 0,
    slot INTEGER NOT NULL DEFAULT 0,
    inventory BLOB,
    char_data BLOB,
    quest_data1 BLOB,
    bank BLOB,
    guildcard_desc TEXT,
    autoreply TEXT,
    infoboard TEXT,
    challenge_data BLOB,
    tech_menu BLOB,
    quest_data2 BLOB
);

CREATE TABLE IF NOT EXISTS bb_account_flags (
    account_id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    login_flags INTEGER NOT NULL DEFAULT 0
);

COMMIT;
";

//pub static MIGRATIONS: [&'static str] = [""];
