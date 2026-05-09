//! AI Provider 实现
//!
//! 文本 LLM Provider: DeepSeek, OpenAI, Claude, Kimi, OpenRouter, GLM, Gemini, MiniMax
//! 媒体 Provider: MiniMax, Google, Jimeng, 海绵音乐 (Haimian), Seedance, Seedream, Suno, MiniMax Music

pub mod deepseek;
pub mod openai;
pub mod claude;
pub mod kimi;
pub mod openrouter;
pub mod glm;
pub mod gemini;
pub mod minimax_text;
pub mod media_provider;
pub mod media_factory;
pub mod registry;
pub mod alou_code;

// 媒体 Provider
pub mod minimax;
pub mod google;
pub mod jimeng;
pub mod haimian;
pub mod seedance;
pub mod seedream;
pub mod suno;

pub use deepseek::DeepSeekProvider;
pub use openai::OpenAiProvider;
pub use claude::ClaudeProvider;
pub use kimi::KimiProvider;
pub use openrouter::OpenRouterProvider;
pub use glm::GlmProvider;
pub use gemini::GeminiProvider;
pub use minimax_text::MiniMaxTextProvider;
pub use alou_code::AlouCodeProvider;
pub use media_provider::{MediaProvider, MediaType, MediaOutput, MediaTask, TaskStatus};
pub use media_factory::{MediaProviderFactory, ProviderInfo};
pub use registry::ProviderRegistry;

// 即梦系列 Provider
pub use seedance::SeedanceProvider;
pub use seedream::SeedreamProvider;

// 音乐 Provider
pub use suno::SunoProvider;
// pub use minimax::MiniMaxMusicProvider;
