-- Initial database schema for alou-edge
-- Creates users table with wallet authentication support

CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY,
    wallet_address TEXT NOT NULL UNIQUE,
    chain TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    last_login INTEGER NOT NULL
);

-- Index for fast wallet address lookups
CREATE INDEX IF NOT EXISTS idx_users_wallet_address ON users(wallet_address);

-- Index for chain-specific queries
CREATE INDEX IF NOT EXISTS idx_users_chain ON users(chain);
