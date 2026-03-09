//! 群聊适配器模块
//!
//! 提供统一的群聊操作接口，支持 Memory、PubSub、Iroh 三种模式
//!
//! # 架构
//!
//! ```text
//! adapters/
//! ├── mod.rs           # 模块入口
//! ├── types.rs          # 类型定义（权威源）
//! └── group_adapter.rs # 主适配器实现
//! ```

pub mod types;
pub mod group_adapter;

pub use group_adapter::GroupAdapter;
