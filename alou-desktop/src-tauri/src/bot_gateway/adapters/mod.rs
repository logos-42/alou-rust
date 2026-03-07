//! Bot Gateway 平台适配器模块

pub mod telegram;
pub mod feishu;

pub use telegram::TelegramAdapter;
pub use feishu::FeishuAdapter;
