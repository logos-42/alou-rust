/**
 * mod.rs - Soul Module
 *
 * Personality and identity management for Alou.
 */

pub mod types;
pub mod manager;

pub use types::{PersonalityTrait, Principle, Preference, GrowthLog, SoulProfile};
pub use manager::SoulManager;
