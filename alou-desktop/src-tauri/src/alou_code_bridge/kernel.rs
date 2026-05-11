//! Alou Code Kernel Core
//!
//! Provides the main entry point to the embedded alou_code runtime.

use std::sync::Arc;
use tokio::sync::RwLock;
use once_cell::sync::OnceCell;
use std::path::PathBuf;

use alou_code_runtime::{
    ConfigLoader,
    SessionStore,
    SessionIdentity,
};

static KERNEL_INSTANCE: OnceCell<Arc<RwLock<AlouCodeKernel>>> = OnceCell::new();

pub struct AlouCodeKernel {
    pub config_loader: ConfigLoader,
    pub session_store: SessionStore,
}

impl AlouCodeKernel {
    pub fn new() -> Result<Self, String> {
        let config_loader = ConfigLoader::new();

        let cwd = std::env::current_dir()
            .map_err(|e| format!("Failed to get current directory: {}", e))?;
        let session_store = SessionStore::from_cwd(&cwd)
            .map_err(|e| format!("Failed to create session store: {}", e))?;

        Ok(Self {
            config_loader,
            session_store,
        })
    }

    pub fn get_or_init() -> Result<Arc<RwLock<AlouCodeKernel>>, String> {
        KERNEL_INSTANCE
            .get_or_try_init(|| {
                let kernel = AlouCodeKernel::new()?;
                Ok(Arc::new(RwLock::new(kernel)))
            })
            .cloned()
    }

    pub fn list_tools(&self) -> Vec<alou_code_tools::ToolDefinition> {
        let registry = alou_code_tools::GlobalToolRegistry::builtin();
        registry.definitions(None)
    }

    pub fn execute_tool(&self, name: &str, input: &serde_json::Value) -> Result<String, String> {
        let registry = alou_code_tools::GlobalToolRegistry::builtin();
        registry.execute(name, input)
    }
}

impl Default for AlouCodeKernel {
    fn default() -> Self {
        Self::new().expect("Failed to initialize AlouCodeKernel")
    }
}
