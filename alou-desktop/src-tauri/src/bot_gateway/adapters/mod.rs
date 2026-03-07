//! Bot Gateway 平台适配器模块

pub mod telegram;
pub mod feishu;
pub mod discord;
pub mod qq;

pub use telegram::TelegramAdapter;
pub use feishu::FeishuAdapter;
pub use discord::DiscordAdapter;
pub use qq::QQAdapter;
