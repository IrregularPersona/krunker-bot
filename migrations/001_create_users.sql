CREATE TABLE users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT NOT NULL,
    discord_id TEXT NOT NULL UNIQUE,
    country TEXT,
    day_created INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

CREATE INDEX idx_discord_id ON users(discord_id);
