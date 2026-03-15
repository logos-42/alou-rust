//! AI Provider 实现
//!
//! 支持多种 AI Provider：DeepSeek, OpenAI, Claude, Kimi
//! 媒体 Provider: MiniMax, Google, Jimeng

pub mod deepseek;
pub mod openai;
pub mod claude;
pub mod kimi;
pub mod media_provider;
pub mod media_factory;
pub mod registry;

// 媒体 Provider
pub mod minimax;
pub mod google;
pub mod jimeng;

pub use deepseek::DeepSeekProvider;
pub use openai::OpenAiProvider;
pub use claude::ClaudeProvider;
pub use kimi::KimiProvider;
pub use media_provider::{MediaProvider, MediaType, MediaOutput, MediaTask, TaskStatus};
pub use media_factory::{MediaProviderFactory, ProviderInfo};
pub use registry::ProviderRegistry;
