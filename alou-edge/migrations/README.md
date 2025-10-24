# Database Migrations

## Setup

To apply migrations to your D1 database:

```bash
# Development
wrangler d1 execute alou-edge-dev --file=./migrations/0001_init.sql

# Staging
wrangler d1 execute alou-edge-staging --file=./migrations/0001_init.sql --env=staging

# Production
wrangler d1 execute alou-edge-prod --file=./migrations/0001_init.sql --env=production
```

## Creating D1 Databases

```bash
# Development
wrangler d1 create alou-edge-dev

# Staging
wrangler d1 create alou-edge-staging

# Production
wrangler d1 create alou-edge-prod
```

After creating databases, update the `database_id` values in `wrangler.toml` with the IDs returned by the create commands.

## Creating KV Namespaces

```bash
# Development
wrangler kv:namespace create "SESSIONS"
wrangler kv:namespace create "CACHE"
wrangler kv:namespace create "NONCES"

# Production
wrangler kv:namespace create "SESSIONS" --env=production
wrangler kv:namespace create "CACHE" --env=production
wrangler kv:namespace create "NONCES" --env=production

# Staging
wrangler kv:namespace create "SESSIONS" --env=staging
wrangler kv:namespace create "CACHE" --env=staging
wrangler kv:namespace create "NONCES" --env=staging
```

Update the `id` values in `wrangler.toml` with the IDs returned by the create commands.

## Setting Secrets

```bash
# Set secrets (same for all environments, use --env flag for staging/production)
wrangler secret put CLAUDE_API_KEY
wrangler secret put JWT_SECRET
wrangler secret put ETH_RPC_URL
wrangler secret put SOLANA_RPC_URL
```
