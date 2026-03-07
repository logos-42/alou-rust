/**
 * mod.rs - Tasks Module
 *
 * Task tracking and breakpoint resumption for Alou.
 */

pub mod types;
pub mod manager;

pub use types::{Task, TaskStatus, TaskStats};
pub use manager::TasksManager;
