//! AI Provider 实现
//!
//! 支持多种 AI Provider：DeepSeek, OpenAI, Claude, Kimi
//! 媒体 Provider: MiniMax, Google, Jimeng, 海绵音乐 (Haimian), Seedance, Seedream

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
pub mod haimian;
pub mod seedance;
pub mod seedream;

pub use deepseek::DeepSeekProvider;
pub use openai::OpenAiProvider;
pub use claude::ClaudeProvider;
pub use kimi::KimiProvider;
pub use media_provider::{MediaProvider, MediaType, MediaOutput, MediaTask, TaskStatus};
pub use media_factory::{MediaProviderFactory, ProviderInfo};
pub use registry::ProviderRegistry;

// 即梦系列 Provider
pub use seedance::SeedanceProvider;
pub use seedream::SeedreamProvider;
