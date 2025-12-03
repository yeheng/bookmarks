-- Create users table
CREATE TABLE users (
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

-- Create indexes
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_username ON users(username);
CREATE INDEX idx_users_active ON users(is_active);
CREATE INDEX idx_users_email_verified ON users(email_verified);

-- Create trigger to update updated_at timestamp
CREATE TRIGGER update_users_updated_at BEFORE UPDATE ON users
    FOR EACH ROW BEGIN
        UPDATE users SET updated_at = CAST(strftime('%s', 'now') AS INTEGER) WHERE id = NEW.id;
    END;
