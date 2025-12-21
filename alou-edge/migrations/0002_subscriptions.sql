-- Subscription system schema
-- Creates tables for subscription management, trial periods, and payments

-- Subscription plans configuration
CREATE TABLE IF NOT EXISTS subscription_plans (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,                    -- e.g., "monthly", "yearly"
    display_name TEXT NOT NULL,             -- e.g., "Monthly Plan", "Yearly Plan"
    price_usd REAL NOT NULL,                -- Price in USD (20.0 for monthly, 199.0 for yearly)
    duration_days INTEGER NOT NULL,         -- 30 for monthly, 365 for yearly
    chain_type TEXT NOT NULL,               -- "eth", "solana", "bitcoin"
    supported_tokens TEXT NOT NULL,         -- JSON array of supported token symbols
    is_active INTEGER DEFAULT 1,            -- 1 for active, 0 for inactive
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

-- Trial periods tracking
CREATE TABLE IF NOT EXISTS trial_periods (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    wallet_address TEXT NOT NULL,
    started_at INTEGER NOT NULL,
    expires_at INTEGER NOT NULL,            -- started_at + 12 days
    is_used INTEGER DEFAULT 1,              -- 1 if trial is active/used, 0 if not started
    created_at INTEGER NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id),
    FOREIGN KEY (wallet_address) REFERENCES users(wallet_address)
);

-- User subscriptions
CREATE TABLE IF NOT EXISTS subscriptions (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    wallet_address TEXT NOT NULL,
    plan_id TEXT NOT NULL,
    chain_type TEXT NOT NULL,               -- "eth", "solana", "bitcoin"
    token_symbol TEXT NOT NULL,             -- "USDC", "USDT", "BTC", etc.
    amount_paid TEXT NOT NULL,              -- Amount in token units (as string for precision)
    amount_usd REAL NOT NULL,               -- Amount in USD
    started_at INTEGER NOT NULL,
    expires_at INTEGER NOT NULL,
    status TEXT NOT NULL,                   -- "active", "expired", "cancelled"
    tx_hash TEXT,                           -- Transaction hash for payment
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id),
    FOREIGN KEY (wallet_address) REFERENCES users(wallet_address),
    FOREIGN KEY (plan_id) REFERENCES subscription_plans(id)
);

-- Subscription payment records
CREATE TABLE IF NOT EXISTS subscription_payments (
    id TEXT PRIMARY KEY,
    subscription_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    wallet_address TEXT NOT NULL,
    chain_type TEXT NOT NULL,
    token_symbol TEXT NOT NULL,
    amount TEXT NOT NULL,                   -- Amount in token units
    amount_usd REAL NOT NULL,
    tx_hash TEXT NOT NULL,
    from_address TEXT,                      -- Payer address
    to_address TEXT NOT NULL,               -- Recipient address (platform wallet)
    status TEXT NOT NULL,                   -- "pending", "confirmed", "failed"
    block_number INTEGER,                   -- Block number (for ETH/Solana)
    confirmation_count INTEGER DEFAULT 0,    -- Number of confirmations
    created_at INTEGER NOT NULL,
    confirmed_at INTEGER,
    FOREIGN KEY (subscription_id) REFERENCES subscriptions(id),
    FOREIGN KEY (user_id) REFERENCES users(id),
    FOREIGN KEY (wallet_address) REFERENCES users(wallet_address)
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_trial_periods_user_id ON trial_periods(user_id);
CREATE INDEX IF NOT EXISTS idx_trial_periods_wallet ON trial_periods(wallet_address);
CREATE INDEX IF NOT EXISTS idx_trial_periods_expires ON trial_periods(expires_at);

CREATE INDEX IF NOT EXISTS idx_subscriptions_user_id ON subscriptions(user_id);
CREATE INDEX IF NOT EXISTS idx_subscriptions_wallet ON subscriptions(wallet_address);
CREATE INDEX IF NOT EXISTS idx_subscriptions_status ON subscriptions(status);
CREATE INDEX IF NOT EXISTS idx_subscriptions_expires ON subscriptions(expires_at);
CREATE INDEX IF NOT EXISTS idx_subscriptions_plan ON subscriptions(plan_id);

CREATE INDEX IF NOT EXISTS idx_payments_subscription ON subscription_payments(subscription_id);
CREATE INDEX IF NOT EXISTS idx_payments_user_id ON subscription_payments(user_id);
CREATE INDEX IF NOT EXISTS idx_payments_tx_hash ON subscription_payments(tx_hash);
CREATE INDEX IF NOT EXISTS idx_payments_status ON subscription_payments(status);

-- Insert default subscription plans
INSERT OR IGNORE INTO subscription_plans (id, name, display_name, price_usd, duration_days, chain_type, supported_tokens, is_active, created_at, updated_at)
VALUES 
    ('plan_monthly_eth', 'monthly', 'Monthly Plan', 20.0, 30, 'eth', '["USDC","USDT","ETH","DAI"]', 1, CAST(strftime('%s', 'now') AS INTEGER), CAST(strftime('%s', 'now') AS INTEGER)),
    ('plan_yearly_solana', 'yearly', 'Yearly Plan', 199.0, 365, 'solana', '["USDC","USDT","SOL"]', 1, CAST(strftime('%s', 'now') AS INTEGER), CAST(strftime('%s', 'now') AS INTEGER));

