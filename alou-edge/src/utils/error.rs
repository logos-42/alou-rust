use thiserror::Error;

#[derive(Debug, Error, Clone)]
pub enum AloudError {
    // Agent errors
    #[error("Agent error: {0}")]
    AgentError(String),
    
    #[error("Claude API error: {0}")]
    ClaudeApiError(String),
    
    // MCP errors
    #[error("Tool not found: {0}")]
    ToolNotFound(String),
    
    #[error("Tool execution error: {0}")]
    ToolExecutionError(String),
    
    #[error("Invalid tool arguments: {0}")]
    InvalidToolArgs(String),
    
    #[error("MCP error: {0}")]
    McpError(String),
    
    #[error("MCP connection error: {0}")]
    McpConnectionError(String),
    
    #[error("MCP timeout")]
    McpTimeout,
    
    // Authentication errors
    #[error("Authentication error: {0}")]
    AuthError(String),
    
    #[error("Invalid signature")]
    InvalidSignature,
    
    #[error("Nonce expired")]
    NonceExpired,
    
    // Web3 errors
    #[error("RPC error: {0}")]
    RpcError(String),
    
    #[error("Transaction failed: {0}")]
    TransactionFailed(String),
    
    #[error("Insufficient balance")]
    InsufficientBalance,
    
    // Storage errors
    #[error("Database error: {0}")]
    DatabaseError(String),
    
    #[error("Cache error: {0}")]
    CacheError(String),
    
    // Other errors
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    
    #[error("Internal error: {0}")]
    InternalError(String),
    
    #[error("Worker error: {0}")]
    WorkerError(String),
}

impl From<worker::Error> for AloudError {
    fn from(err: worker::Error) -> Self {
        AloudError::WorkerError(err.to_string())
    }
}

impl From<serde_json::Error> for AloudError {
    fn from(err: serde_json::Error) -> Self {
        AloudError::InvalidInput(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, AloudError>;
