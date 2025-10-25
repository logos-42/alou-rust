pub mod error;
pub mod crypto;
pub mod metrics;
pub mod batch;
pub mod async_lock;

// Re-exports for convenience (may be unused in lib.rs but used by other modules)
#[allow(unused_imports)]
pub use error::{AloudError, Result};
#[allow(unused_imports)]
pub use metrics::{MetricsCollector, MetricsSnapshot};
#[allow(unused_imports)]
pub use batch::RequestBatcher;
