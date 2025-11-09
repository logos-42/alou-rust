# D1 数据库落地方案

## 目标

- 为用户与钱包的转账记录提供持久化与可查询能力。
- 支撑后续的智能体社交功能（关注关系、互动事件）。
- 与现有 KV 存储解耦：KV 继续承担短期缓存、会话、一次性凭证。

## 数据建模

| 表名 | 说明 | 关键字段 |
| ---- | ---- | -------- |
| `users` | 平台用户档案 | `id` (UUID), `wallet_address`, `created_at`, `last_active_at` |
| `wallets` | 托管/绑定钱包 | `id`, `user_id`, `chain`, `address`, `label`, `created_at` |
| `transactions` | 转账流水 | `id`, `user_id`, `wallet_id`, `chain`, `tx_hash`, `status`, `direction`, `counterparty`, `asset_symbol`, `amount`, `fee`, `metadata`, `created_at` |
| `social_profiles` | 智能体或用户的社交资料 | `id`, `entity_type`, `display_name`, `avatar_url`, `bio`, `created_at`, `updated_at` |
| `social_relationships` | 关注/订阅关系 | `id`, `from_profile_id`, `to_profile_id`, `relation_type`, `created_at` |
| `interaction_events` | 聊天、点赞等事件 | `id`, `profile_id`, `event_type`, `payload`, `created_at` |

> 说明：
> - `metadata` 与 `payload` 字段使用 JSON 以便扩展。
> - 交易记录通过 `tx_hash` + `chain` 建唯一索引，避免重复写入。

## KV 与 D1 分工

| 场景 | 存储 | 说明 |
| ---- | ---- | ---- |
| 会话上下文、临时 nonce | KV | 需要快速读写，有 TTL。 |
| 转账记录、社交关系 | D1 | 需要查询、排序、分页。 |
| 链上查询缓存 | KV | 可继续使用 `cache_ttl::RPC_QUERY`。 |

## 迁移步骤

1. **准备 schema**：在 `alou-edge/migrations/` 新增 D1 SQL（见下文）。
2. **绑定检查**：`wrangler.toml` 已声明 D1；部署前用 `wrangler d1 migrations apply`。
3. **数据访问层**：新增 `storage::d1` 模块，封装查询与事务。
4. **写入逻辑**：在转账/社交相关工具中调用 D1，而不是 KV。
5. **回填历史数据**：若已有链上历史，可通过脚本拉取并写入 D1。
6. **监控与备份**：接入 Cloudflare D1 备份计划，或定期导出。

## 示例迁移脚本

```sql
-- migrations/0002_create_transactions.sql
CREATE TABLE IF NOT EXISTS transactions (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    wallet_id TEXT NOT NULL,
    chain TEXT NOT NULL,
    tx_hash TEXT NOT NULL,
    status TEXT NOT NULL,
    direction TEXT NOT NULL, -- inbound / outbound
    counterparty TEXT,
    asset_symbol TEXT NOT NULL,
    amount TEXT NOT NULL,
    fee TEXT,
    metadata TEXT,
    created_at INTEGER NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_transactions_chain_hash
    ON transactions(chain, tx_hash);
```

其他表可按上述建模扩展。

## 后续工作

- 定义 Rust 数据访问接口 (`StorageService`)，确保调用端无需感知底层差异。
- 为关键查询编写集成测试（使用 `miniflare` 或 `wrangler d1 execute`）。
- 结合智能体业务，规划触发器或队列，确保事件数据写入有序。

