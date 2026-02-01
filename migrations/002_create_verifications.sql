CREATE TABLE verifications (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    discord_id TEXT NOT NULL,
    krunker_username TEXT NOT NULL,
    code TEXT NOT NULL UNIQUE,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    expires_at INTEGER NOT NULL,
    attempts INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX idx_verification_code ON verifications(code);
CREATE INDEX idx_verification_discord_id ON verifications(discord_id);
CREATE INDEX idx_verification_expires ON verifications(expires_at);
