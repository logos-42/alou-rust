use js_sys::Date;
use serde_json::{json, Map, Value};

use crate::agent::context::AgentContext;
use crate::agent::session::SessionManager;
use crate::mcp::registry::McpTool;
use crate::mcp::tools::AgentWalletTool;
use crate::utils::error::{AloudError, Result};

/// Builder for MCP UI resources exposed to the frontend mcp-ui renderer
pub struct UiResourceBuilder<'a> {
    session_manager: &'a SessionManager,
    agent_wallet_tool: &'a AgentWalletTool,
}

impl<'a> UiResourceBuilder<'a> {
    pub fn new(
        session_manager: &'a SessionManager,
        agent_wallet_tool: &'a AgentWalletTool,
    ) -> Self {
        Self {
            session_manager,
            agent_wallet_tool,
        }
    }

    /// Build an embedded MCP UI resource for the given target and parameters
    pub async fn build_resource(&self, target: &str, params: Value) -> Result<Value> {
        match target {
            "channel_list" => self.build_channel_list(&params).await,
            "wallet_overview" => self.build_wallet_overview(&params).await,
            "transaction_detail" => self.build_transaction_detail(&params).await,
            "agent_profile" => self.build_agent_profile(&params).await,
            "conversation_detail" => self.build_conversation_detail(&params).await,
            "channel_detail" => self.build_channel_detail(&params).await,
            "channel_create" => self.build_channel_create(&params).await,
            _ => Err(AloudError::InvalidInput(format!(
                "Unrecognized MCP UI target: {}",
                target
            ))),
        }
    }

    async fn build_wallet_overview(&self, params: &Value) -> Result<Value> {
        let session_id = params
            .get("session_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                AloudError::InvalidInput(
                    "wallet_overview requires session_id parameter".to_string(),
                )
            })?;

        let session = self.session_manager.get_session(session_id).await.ok();
        let chain = params
            .get("chain")
            .and_then(|v| v.as_str())
            .or_else(|| session.as_ref().and_then(|s| s.chain.as_deref()))
            .unwrap_or("ethereum");

        let mut context = AgentContext::new(session_id.to_string());
        context.chain = Some(chain.to_string());
        if let Some(session) = &session {
            context.wallet_address = session.wallet_address.clone();
        }

        let wallet_response = self
            .agent_wallet_tool
            .execute(
                json!({
                    "action": "get_wallet",
                    "chain": chain,
                }),
                &context,
            )
            .await?;

        let wallet_exists = wallet_response
            .get("success")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let wallet_value = wallet_response.get("wallet").cloned();

        let wallets_response = self
            .agent_wallet_tool
            .execute(json!({ "action": "list_wallets" }), &context)
            .await?;

        let wallets = wallets_response
            .get("wallets")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        let mut sections = String::new();

        if wallet_exists {
            if let Some(wallet) = wallet_value.clone() {
                sections.push_str(&self.render_wallet_card(&wallet));
            }
        } else {
            sections.push_str(&format!(
                r#"<div class="empty-card">
                        <h2>尚未创建智能体钱包</h2>
                        <p>可以让智能体执行 <code>agent_wallet.create_wallet</code> 来创建一个多链钱包。</p>
                        <p>当前目标链：<strong>{}</strong></p>
                    </div>"#,
                escape_html(chain)
            ));
        }

        if !wallets.is_empty() {
            sections.push_str(r#"<div class="section"><h2>全部链钱包</h2>"#);
            sections.push_str(r#"<div class="grid">"#);
            for wallet in wallets.iter() {
                sections.push_str(&self.render_wallet_summary(wallet));
            }
            sections.push_str(r#"</div></div>"#);
        }

        let session_wallet_address = session
            .as_ref()
            .and_then(|session| session.wallet_address.clone());

        if let Some(ref address) = session_wallet_address {
            sections.push_str(&format!(
                r#"<div class="section">
                        <h2>用户会话钱包</h2>
                        <div class="meta">
                            <div><span class="label">绑定地址</span><span class="value">{}</span></div>
                            <div><span class="label">提示</span><span class="value">用于代理用户签名与交易，需在钱包中确认操作。</span></div>
                        </div>
                    </div>"#,
                escape_html(&address)
            ));
        }

        let mut meta = Map::new();
        if let Some(wallet) = wallet_value.clone() {
            if let Some(transactions_value) = wallet.get("transactions") {
                meta.insert("transactions".to_string(), transactions_value.clone());
            }
            meta.insert("wallet".to_string(), wallet);
        }
        meta.insert("wallets".to_string(), Value::Array(wallets.clone()));
        if let Some(address) = session_wallet_address.clone() {
            meta.insert("sessionWalletAddress".to_string(), json!(address));
        }
        meta.insert("walletCount".to_string(), json!(wallets.len()));
        meta.insert("chain".to_string(), json!(chain));

        Ok(embed_html(
            "ui://wallet/overview",
            render_html("智能体钱包概览", &sections),
            Some(Value::Object(meta)),
        ))
    }

    async fn build_transaction_detail(&self, params: &Value) -> Result<Value> {
        let session_id = params
            .get("session_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                AloudError::InvalidInput(
                    "transaction_detail requires session_id parameter".to_string(),
                )
            })?;

        let session = self.session_manager.get_session(session_id).await.ok();
        let chain = params
            .get("chain")
            .and_then(|v| v.as_str())
            .or_else(|| session.as_ref().and_then(|s| s.chain.as_deref()))
            .unwrap_or("ethereum");

        let tx_id = params
            .get("transaction_id")
            .or_else(|| params.get("hash"))
            .and_then(|v| v.as_str())
            .unwrap_or_default();

        if tx_id.is_empty() {
            return Err(AloudError::InvalidInput(
                "transaction_detail requires transaction_id or hash".to_string(),
            ));
        }

        let mut context = AgentContext::new(session_id.to_string());
        context.chain = Some(chain.to_string());
        if let Some(session) = &session {
            context.wallet_address = session.wallet_address.clone();
        }

        let wallet_response = self
            .agent_wallet_tool
            .execute(
                json!({
                    "action": "get_wallet",
                    "chain": chain,
                }),
                &context,
            )
            .await?;

        let wallet = wallet_response
            .get("wallet")
            .and_then(|v| v.as_object())
            .ok_or_else(|| {
                AloudError::InvalidInput(format!(
                    "wallet not found on chain {} for transaction lookup",
                    chain
                ))
            })?;

        let transactions = wallet
            .get("transactions")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        let mut matched: Option<Value> = None;

        for tx in transactions.iter() {
            if let Some(hash) = tx.get("hash").and_then(|v| v.as_str()) {
                if hash.eq_ignore_ascii_case(tx_id) {
                    matched = Some(tx.clone());
                    break;
                }
            }
            if let Some(id) = tx.get("id").and_then(|v| v.as_str()) {
                if id == tx_id {
                    matched = Some(tx.clone());
                    break;
                }
            }
        }

        let html_body = if let Some(tx) = matched.clone() {
            self.render_transaction_detail(&tx)
        } else {
            format!(
                r#"<div class="empty-card">
                        <h2>未找到交易记录</h2>
                        <p>事务 ID / Hash: <code>{}</code></p>
                        <p>请确认已调用 <code>agent_wallet.record_transaction</code> 记录该交易。</p>
                    </div>"#,
                escape_html(tx_id)
            )
        };

        let mut meta = Map::new();
        meta.insert("transactionId".to_string(), json!(tx_id));
        if let Some(tx) = matched {
            meta.insert("transaction".to_string(), tx);
        }

        Ok(embed_html(
            "ui://wallet/transaction",
            render_html("交易详情", &html_body),
            Some(Value::Object(meta)),
        ))
    }

    async fn build_agent_profile(&self, params: &Value) -> Result<Value> {
        let agent_id = params
            .get("agent_id")
            .and_then(|v| v.as_str())
            .unwrap_or("alou");

        let session_id = params
            .get("session_id")
            .and_then(|v| v.as_str())
            .unwrap_or("agent-profile");

        let context_session = self.session_manager.get_session(session_id).await.ok();
        let mut context = AgentContext::new(session_id.to_string());
        if let Some(session) = &context_session {
            context.chain = session.chain.clone();
            context.wallet_address = session.wallet_address.clone();
        }

        let mut wallet_cards = String::new();
        let mut wallet_list: Vec<Value> = Vec::new();

        match self
            .agent_wallet_tool
            .execute(json!({ "action": "list_wallets" }), &context)
            .await
        {
            Ok(wallet_response) => {
                if let Some(list) = wallet_response.get("wallets").and_then(|v| v.as_array()) {
                    wallet_list = list.clone();
                }

                if wallet_list.is_empty() {
                    wallet_cards.push_str(
                        r#"<p class="empty">暂无智能体钱包记录，可通过 agent_wallet 工具创建。</p>"#,
                    );
                } else {
                    wallet_cards.push_str(r#"<div class="wallet-grid">"#);
                    for wallet in wallet_list.iter().take(3) {
                        wallet_cards.push_str(&self.render_wallet_summary(wallet));
                    }
                    wallet_cards.push_str("</div>");
                }
            }
            Err(error) => {
                wallet_cards.push_str(&format!(
                    r#"<p class="empty">无法加载钱包状态：{}</p>"#,
                    escape_html(&error.to_string())
                ));
            }
        }

        let mut conversation_block = String::new();
        let mut message_count = 0usize;
        let mut last_interaction: Option<i64> = None;

        if let Some(session) = &context_session {
            message_count = session.messages.len();
            if let Some(last) = session.messages.last() {
                last_interaction = Some(last.timestamp);
            }

            if !session.messages.is_empty() {
                let mut recent = session.messages.iter().rev().take(5).collect::<Vec<_>>();
                recent.reverse();

                conversation_block
                    .push_str(r#"<div class="section"><h3>最近对话</h3><div class="timeline">"#);
                for message in recent {
                    let role_class = match message.role.as_str() {
                        "user" => "badge user",
                        "assistant" => "badge assistant",
                        "tool" => "badge tool",
                        _ => "badge other",
                    };
                    let role_label = match message.role.as_str() {
                        "user" => "用户",
                        "assistant" => "智能体",
                        "tool" => "工具",
                        _ => "其他",
                    };
                    let preview = message.content.lines().next().unwrap_or(&message.content);
                    conversation_block.push_str(&format!(
                        r#"<div class="timeline-row">
                                <span class="{}">{}</span>
                                <span class="text">{}</span>
                                <span class="time">{}</span>
                            </div>"#,
                        role_class,
                        escape_html(role_label),
                        escape_html(preview),
                        format_timestamp(message.timestamp)
                    ));
                }
                conversation_block.push_str("</div></div>");
            }
        }

        if conversation_block.is_empty() {
            conversation_block.push_str(
                r#"<div class="section"><h3>最近对话</h3><p class="empty">还没有对话记录。</p></div>"#,
            );
        }

        let html = format!(
            r#"<div class="profile">
                    <div class="avatar">
                        <img src="https://avatars.githubusercontent.com/u/16309930?v=4" alt="Agent Avatar" />
                    </div>
                    <h2>{}</h2>
                    <p class="subtitle">Web3 链上智能执行体</p>
                    <div class="meta">
                        <div><span class="label">Agent ID</span><span class="value">{}</span></div>
                        <div><span class="label">关联会话</span><span class="value">{}</span></div>
                        <div><span class="label">能力域</span><span class="value">钱包管理、跨链查询、交易自动化</span></div>
                        <div><span class="label">消息量</span><span class="value">{}</span></div>
                    </div>
                    <div class="section">
                        <h3>钱包概览</h3>
                        {}
                    </div>
                    {}
                </div>"#,
            escape_html(agent_id),
            escape_html(agent_id),
            escape_html(session_id),
            message_count,
            wallet_cards,
            conversation_block
        );

        let metadata = json!({
            "agentId": agent_id,
            "sessionId": session_id,
            "wallets": wallet_list,
            "messageCount": message_count,
            "lastInteraction": last_interaction,
            "chain": context.chain,
        });

        Ok(embed_html(
            "ui://agent/profile",
            render_html("Agent Profile", &html),
            Some(metadata),
        ))
    }

    async fn build_conversation_detail(&self, params: &Value) -> Result<Value> {
        let session_id = params
            .get("conversation_id")
            .or_else(|| params.get("session_id"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                AloudError::InvalidInput(
                    "conversation_detail requires conversation_id or session_id".to_string(),
                )
            })?;

        let session = self.session_manager.get_session(session_id).await?;

        let mut timeline = String::new();
        timeline.push_str(r#"<div class="timeline">"#);
        for message in session.messages.iter() {
            let role_class = match message.role.as_str() {
                "user" => "bubble user",
                "assistant" => "bubble assistant",
                "tool" => "bubble tool",
                _ => "bubble other",
            };

            let title = match message.role.as_str() {
                "user" => "用户",
                "assistant" => "智能体",
                "tool" => "工具返回",
                _ => "消息",
            };

            timeline.push_str(&format!(
                r#"<div class="{}">
                        <div class="bubble-header">
                            <span class="role">{}</span>
                            <span class="time">{}</span>
                        </div>
                        <pre>{}</pre>
                    </div>"#,
                role_class,
                escape_html(title),
                format_timestamp(message.timestamp),
                escape_html(&message.content),
            ));
        }
        timeline.push_str("</div>");

        Ok(embed_html(
            "ui://conversation/detail",
            render_html("会话详情", &timeline),
            Some(json!({
                "sessionId": session.session_id,
                "messageCount": session.messages.len(),
            })),
        ))
    }

    async fn build_channel_list(&self, _params: &Value) -> Result<Value> {
        let now = crate::utils::time::now_timestamp();
        let channels = vec![
            json!({
                "id": "dev-relay",
                "name": "TRX Smart Contract Staking",
                "status": "online",
                "statusLabel": "在线",
                "icon": "⚡",
                "color": "linear-gradient(135deg,#6366f1,#8b5cf6)",
                "updatedAt": now - 2 * 60 * 60,
                "description": "TRX 智能合约质押与运营"
            }),
            json!({
                "id": "eth-announce",
                "name": "ETH Contract Announcement",
                "status": "busy",
                "statusLabel": "执行任务",
                "icon": "⬡",
                "color": "linear-gradient(135deg,#0ea5e9,#2563eb)",
                "updatedAt": now - 6 * 60 * 60,
                "description": "以太坊主网合约监控与公告"
            }),
            json!({
                "id": "firefly",
                "name": "Firefly Research",
                "status": "offline",
                "statusLabel": "离线",
                "icon": "🛰️",
                "color": "linear-gradient(135deg,#ec4899,#f97316)",
                "updatedAt": now - 24 * 60 * 60,
                "description": "跨链智能体协同研究"
            }),
            json!({
                "id": "wallet-ops",
                "name": "Wallet Operations",
                "status": "online",
                "statusLabel": "在线",
                "icon": "💼",
                "color": "linear-gradient(135deg,#14b8a6,#0ea5e9)",
                "updatedAt": now - 30 * 60,
                "description": "主钱包与子钱包的资产管理"
            }),
        ];

        let mut list_html = String::new();
        list_html.push_str(r#"<div class="section"><h2>可用智能体频道</h2><div class="list">"#);
        for channel in channels.iter() {
            let name = channel
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("未知频道");
            let status = channel
                .get("statusLabel")
                .and_then(|v| v.as_str())
                .unwrap_or("在线");
            let icon = channel.get("icon").and_then(|v| v.as_str()).unwrap_or("🛰️");
            let description = channel
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let updated_at = channel
                .get("updatedAt")
                .and_then(|v| v.as_i64())
                .unwrap_or_default();

            list_html.push_str(&format!(
                r#"<div class="channel-row">
                        <div class="channel-icon">{}</div>
                        <div class="channel-info">
                            <div class="channel-name">{}</div>
                            <div class="channel-desc">{}</div>
                        </div>
                        <div class="channel-status">{}</div>
                        <div class="channel-time">{}</div>
                    </div>"#,
                escape_html(icon),
                escape_html(name),
                escape_html(description),
                escape_html(status),
                format_timestamp(updated_at),
            ));
        }
        list_html.push_str("</div></div>");

        Ok(embed_html(
            "ui://channel/list",
            render_html("智能体频道列表", &list_html),
            Some(json!({ "channels": channels })),
        ))
    }

    async fn build_channel_detail(&self, params: &Value) -> Result<Value> {
        let channel_id = params
            .get("channel_id")
            .and_then(|v| v.as_str())
            .unwrap_or("wallet-ops");

        let html = format!(
            r#"<div class="section">
                    <h2>频道详情</h2>
                    <div class="meta">
                        <div><span class="label">频道 ID</span><span class="value">{}</span></div>
                        <div><span class="label">角色定位</span><span class="value">链上资产调度 · 支付执行</span></div>
                        <div><span class="label">关联合约</span><span class="value">USDC 代币 · 智能钱包</span></div>
                    </div>
                    <div class="section">
                        <h3>推荐操作</h3>
                        <ul>
                            <li>同步最新钱包余额，确认资产状态</li>
                            <li>发起批量支付前，准备收款地址列表</li>
                            <li>使用 <code>workflow</code> 定义自动化付款流程</li>
                        </ul>
                    </div>
                </div>"#,
            escape_html(channel_id)
        );

        Ok(embed_html(
            "ui://channel/detail",
            render_html("频道详情", &html),
            Some(json!({
                "channelId": channel_id,
            })),
        ))
    }

    async fn build_channel_create(&self, _params: &Value) -> Result<Value> {
        let html = r#"
            <div class="section">
                <h2>创建新的任务频道</h2>
                <ol class="list">
                    <li>为频道定义唯一 ID 与中文名称，描述业务场景。</li>
                    <li>指定默认工具集（如 wallet_manager、agent_wallet、workflow）。</li>
                    <li>配置权限：谁可以触发自动化，哪些钱包授权给智能体。</li>
                    <li>保存后可在左侧列表中看到新频道，并通过 MCP 指令激活。</li>
                </ol>
                <p class="note">提示：频道用于约束上下文与资源，帮助智能体在特定团队/业务下协作。</p>
            </div>
        "#;

        Ok(embed_html(
            "ui://channel/create",
            render_html("创建频道", html),
            None,
        ))
    }

    fn render_wallet_card(&self, wallet: &Value) -> String {
        let address = wallet
            .get("address")
            .and_then(|v| v.as_str())
            .unwrap_or("0x0000");
        let balance = wallet
            .get("balance")
            .and_then(|v| v.as_str())
            .unwrap_or("0");
        let chain = wallet
            .get("chain")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        let created_at = wallet
            .get("created_at")
            .and_then(|v| v.as_i64())
            .unwrap_or_default();
        let transactions = wallet
            .get("transactions")
            .and_then(|v| v.as_array())
            .map(|arr| arr.len())
            .unwrap_or(0);

        format!(
            r#"<div class="card">
                    <h2>当前链钱包</h2>
                    <div class="meta">
                        <div><span class="label">地址</span><span class="value">{}</span></div>
                        <div><span class="label">链</span><span class="value">{}</span></div>
                        <div><span class="label">余额</span><span class="value">{}</span></div>
                        <div><span class="label">交易记录</span><span class="value">{}</span></div>
                        <div><span class="label">创建时间</span><span class="value">{}</span></div>
                    </div>
                </div>"#,
            escape_html(address),
            escape_html(chain),
            escape_html(balance),
            transactions,
            format_timestamp(created_at),
        )
    }

    fn render_wallet_summary(&self, wallet: &Value) -> String {
        let chain = wallet
            .get("chain")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        let balance = wallet
            .get("balance")
            .and_then(|v| v.as_str())
            .unwrap_or("0");
        let updated_at = wallet
            .get("last_updated")
            .and_then(|v| v.as_i64())
            .unwrap_or_default();

        format!(
            r#"<div class="mini-card">
                    <div class="mini-title">{}</div>
                    <div class="mini-value">{}</div>
                    <div class="mini-foot">更新于 {}</div>
                </div>"#,
            escape_html(chain),
            escape_html(balance),
            format_timestamp(updated_at),
        )
    }

    fn render_transaction_detail(&self, tx: &Value) -> String {
        let hash = tx.get("hash").and_then(|v| v.as_str()).unwrap_or("未指定");
        let from = tx.get("from").and_then(|v| v.as_str()).unwrap_or("未知");
        let to = tx.get("to").and_then(|v| v.as_str()).unwrap_or("未知");
        let value = tx.get("value").and_then(|v| v.as_str()).unwrap_or("0");
        let token = tx.get("token").and_then(|v| v.as_str()).unwrap_or("ETH");
        let status = tx
            .get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("pending");
        let tx_type = tx.get("type").and_then(|v| v.as_str()).unwrap_or("send");
        let timestamp = tx
            .get("timestamp")
            .and_then(|v| v.as_i64())
            .unwrap_or_default();

        let mut detail_rows = String::new();
        if let Some(gas) = tx.get("gas").and_then(|v| v.as_str()) {
            detail_rows.push_str(&format!(
                r#"<div><span class="label">Gas</span><span class="value">{}</span></div>"#,
                escape_html(gas)
            ));
        }
        if let Some(fee) = tx.get("fee").and_then(|v| v.as_str()) {
            detail_rows.push_str(&format!(
                r#"<div><span class="label">手续费</span><span class="value">{}</span></div>"#,
                escape_html(fee)
            ));
        }
        if let Some(chain) = tx.get("chain").and_then(|v| v.as_str()) {
            detail_rows.push_str(&format!(
                r#"<div><span class="label">链</span><span class="value">{}</span></div>"#,
                escape_html(chain)
            ));
        }

        format!(
            r#"<div class="card">
                    <h2>交易信息</h2>
                    <div class="meta">
                        <div><span class="label">交易 Hash</span><span class="value mono">{}</span></div>
                        <div><span class="label">类型</span><span class="value">{}</span></div>
                        <div><span class="label">状态</span><span class="value status {}">{}</span></div>
                        <div><span class="label">时间</span><span class="value">{}</span></div>
                    </div>
                    <div class="section">
                        <h3>转账信息</h3>
                        <div class="meta">
                            <div><span class="label">From</span><span class="value mono">{}</span></div>
                            <div><span class="label">To</span><span class="value mono">{}</span></div>
                            <div><span class="label">金额</span><span class="value">{}</span></div>
                            <div><span class="label">代币</span><span class="value">{}</span></div>
                            {}
                        </div>
                    </div>
                </div>"#,
            escape_html(hash),
            escape_html(tx_type),
            escape_html(status),
            escape_html(status),
            format_timestamp(timestamp),
            escape_html(from),
            escape_html(to),
            escape_html(value),
            escape_html(token),
            detail_rows
        )
    }
}

fn embed_html(uri: &str, html: String, meta: Option<Value>) -> Value {
    let mut resource = Map::new();
    resource.insert("uri".to_string(), json!(uri));
    resource.insert("mimeType".to_string(), json!("text/html"));
    resource.insert("contentType".to_string(), json!("rawHtml"));
    resource.insert("text".to_string(), Value::String(html));
    if let Some(meta_value) = meta {
        resource.insert("_meta".to_string(), meta_value);
    }

    let mut embedded = Map::new();
    embedded.insert("type".to_string(), json!("embedded"));
    embedded.insert("resource".to_string(), Value::Object(resource));

    Value::Object(embedded)
}

fn render_html(title: &str, body: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="zh">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <title>{title}</title>
  <style>
    :root {{
      color-scheme: light dark;
      font-family: 'Inter', 'Microsoft YaHei', system-ui, -apple-system, sans-serif;
    }}
    body {{
      margin: 0;
      padding: 24px;
      background: rgba(248, 250, 252, 0.9);
      color: #111827;
    }}
    .container {{
      max-width: 720px;
      margin: 0 auto;
      display: flex;
      flex-direction: column;
      gap: 24px;
    }}
    h1 {{
      margin: 0;
      font-size: 24px;
      font-weight: 600;
    }}
    h2 {{
      margin: 0 0 12px;
      font-size: 18px;
      font-weight: 600;
    }}
    h3 {{
      margin: 16px 0 8px;
      font-size: 16px;
    }}
    .card {{
      background: rgba(255, 255, 255, 0.9);
      border-radius: 16px;
      padding: 20px;
      box-shadow: 0 12px 30px rgba(15, 23, 42, 0.08);
    }}
    .empty-card {{
      background: rgba(255, 255, 255, 0.8);
      border-radius: 16px;
      padding: 24px;
      text-align: center;
      border: 1px dashed rgba(148, 163, 184, 0.6);
    }}
    .grid {{
      display: grid;
      grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
      gap: 12px;
    }}
    .section {{
      display: flex;
      flex-direction: column;
      gap: 12px;
    }}
    .meta {{
      display: flex;
      flex-direction: column;
      gap: 8px;
    }}
    .profile {{
      display: flex;
      flex-direction: column;
      gap: 16px;
      align-items: center;
      text-align: center;
    }}
    .profile .subtitle {{
      color: #5b7290;
      margin: -8px 0 12px;
    }}
    .avatar {{
      width: 96px;
      height: 96px;
      border-radius: 50%;
      overflow: hidden;
      border: 4px solid rgba(99, 102, 241, 0.2);
    }}
    .avatar img {{
      width: 100%;
      height: 100%;
      object-fit: cover;
    }}
    .label {{
      color: #64748b;
      font-size: 0.85rem;
    }}
    .value {{
      color: #0f172a;
      font-weight: 500;
    }}
    .value.mono {{
      font-family: 'Fira Code', 'JetBrains Mono', monospace;
      word-break: break-all;
    }}
    .status.success {{
      color: #16a34a;
    }}
    .status.failed {{
      color: #dc2626;
    }}
    .mini-card {{
      background: rgba(59, 130, 246, 0.08);
      border-radius: 14px;
      padding: 16px;
      display: flex;
      flex-direction: column;
      gap: 6px;
    }}
    .mini-title {{
      font-weight: 600;
      color: #1d4ed8;
    }}
    .mini-value {{
      font-size: 20px;
      font-weight: 700;
      color: #0f172a;
    }}
    .mini-foot {{
      font-size: 12px;
      color: #64748b;
    }}
    .timeline {{
      display: flex;
      flex-direction: column;
      gap: 12px;
    }}
    .bubble {{
      border-radius: 16px;
      padding: 16px;
      background: rgba(255, 255, 255, 0.9);
      box-shadow: 0 10px 25px rgba(15, 23, 42, 0.08);
    }}
    .bubble.user {{
      border-left: 4px solid #6366f1;
    }}
    .bubble.assistant {{
      border-left: 4px solid #22d3ee;
    }}
    .bubble.tool {{
      border-left: 4px solid #f59e0b;
    }}
    .bubble-header {{
      display: flex;
      justify-content: space-between;
      margin-bottom: 8px;
      color: #475569;
      font-size: 0.875rem;
      font-weight: 500;
    }}
    pre {{
      margin: 0;
      white-space: pre-wrap;
      font-family: 'Fira Code', 'JetBrains Mono', monospace;
      background: rgba(148, 163, 184, 0.12);
      border-radius: 12px;
      padding: 12px;
      color: #0f172a;
    }}
    ul {{
      margin: 0;
      padding-left: 20px;
      text-align: left;
      color: #334155;
    }}
    .list {{
      margin: 0;
      padding-left: 20px;
      color: #1f2937;
    }}
    .note {{
      font-size: 0.9rem;
      color: #475569;
      background: rgba(226, 232, 240, 0.5);
      border-radius: 12px;
      padding: 12px;
    }}
    code {{
      font-family: 'Fira Code', 'JetBrains Mono', monospace;
      background: rgba(148, 163, 184, 0.25);
      padding: 2px 6px;
      border-radius: 6px;
    }}
    @media (prefers-color-scheme: dark) {{
      body {{
        background: rgba(15, 23, 42, 0.85);
        color: #e2e8f0;
      }}
      .card, .bubble {{
        background: rgba(30, 41, 59, 0.78);
        color: #e2e8f0;
      }}
      .empty-card {{
        background: rgba(30, 41, 59, 0.6);
        border-color: rgba(148, 163, 184, 0.4);
      }}
      .value {{
        color: #f8fafc;
      }}
      pre {{
        background: rgba(15, 23, 42, 0.8);
        color: #f1f5f9;
      }}
    }}
  </style>
</head>
<body>
  <div class="container">
    <h1>{title}</h1>
    {body}
  </div>
</body>
</html>"#,
        title = escape_html(title),
        body = body
    )
}

fn escape_html(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn format_timestamp(ts: i64) -> String {
    if ts <= 0 {
        return "—".to_string();
    }
    let millis = (ts as f64) * 1000.0;
    let date = Date::new(&millis.into());
    if let Some(iso) = date.to_iso_string().as_string() {
        if iso.len() >= 19 {
            format!("{} {}", &iso[0..10], &iso[11..19])
        } else {
            iso
        }
    } else {
        ts.to_string()
    }
}
