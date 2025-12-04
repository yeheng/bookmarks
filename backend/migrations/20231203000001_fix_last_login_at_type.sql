-- Fix last_login_at column type mismatch
-- Convert TEXT datetime values to INTEGER Unix timestamps

-- SQLite approach: recreate the table with proper schema

-- Create new users table with correct schema
CREATE TABLE users_new (
    id INTEGER PRIMARY KEY,
    username TEXT UNIQUE NOT NULL,
    email TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    avatar_url TEXT,
    is_active INTEGER DEFAULT 1,
    email_verified INTEGER DEFAULT 0,
    email_verification_token TEXT,
    password_reset_token TEXT,
    password_reset_expires_at INTEGER,
    last_login_at INTEGER,
    created_at INTEGER DEFAULT (CAST(strftime('%s', 'now') AS INTEGER)),
    updated_at INTEGER DEFAULT (CAST(strftime('%s', 'now') AS INTEGER))
);

-- Copy data from old table, converting TEXT dates to INTEGER timestamps
INSERT INTO users_new (
    id, username, email, password_hash, avatar_url, is_active, email_verified,
    email_verification_token, password_reset_token, password_reset_expires_at,
    last_login_at, created_at, updated_at
)
SELECT 
    id, username, email, password_hash, avatar_url, is_active, email_verified,
    email_verification_token, password_reset_token, password_reset_expires_at,
    CASE 
        WHEN last_login_at IS NULL THEN NULL
        WHEN typeof(last_login_at) = 'text' THEN CAST(strftime('%s', last_login_at) AS INTEGER)
        ELSE last_login_at
    END as last_login_at,
    created_at, updated_at
FROM users;

-- Drop the old table
DROP TABLE users;

-- Rename the new table
ALTER TABLE users_new RENAME TO users;

-- Recreate all indexes
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_username ON users(username);
CREATE INDEX idx_users_active ON users(is_active);
CREATE INDEX idx_users_email_verified ON users(email_verified);
CREATE INDEX idx_users_active_email_verified ON users(is_active, email_verified);
CREATE INDEX idx_users_last_login_desc ON users(last_login_at DESC);

-- Recreate the trigger
CREATE TRIGGER update_users_updated_at BEFORE UPDATE ON users
    FOR EACH ROW BEGIN
        UPDATE users SET updated_at = CAST(strftime('%s', 'now') AS INTEGER) WHERE id = NEW.id;
    END;
