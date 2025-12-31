use std::time::Duration;

/// Error type for retry decisions
#[derive(Debug, Clone, PartialEq)]
pub enum ErrorType {
    Transient,
    Permanent,
    RateLimited,
    NetworkError,
    RateLimitError,
    TransactionError,
    ArgumentError,
    AuthenticationError,
    PermissionError,
}

/// Retry decision
#[derive(Debug, Clone, PartialEq)]
pub enum RetryDecision {
    /// Do not retry
    NoRetry {
        reason: String,
    },
    /// Retry with specific delay
    Retry {
        delay_ms: u64,
        attempt: u32,
        max_attempts: u32,
        reason: String,
    },
    /// Retry with backoff
    RetryWithBackoff {
        base_delay_ms: u64,
        backoff_factor: f64,
        attempt: u32,
        max_attempts: u32,
    },
}

/// Backoff strategy
#[derive(Debug, Clone, Copy)]
pub enum BackoffStrategy {
    /// Fixed delay
    Fixed,
    /// Exponential backoff (delay = base * factor^attempt)
    Exponential,
    /// Linear backoff (delay = base * (1 + attempt * factor))
    Linear,
}

/// Retry policy configuration
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub base_delay_ms: u64,
    pub backoff_strategy: BackoffStrategy,
    pub backoff_factor: f64,
    pub max_delay_ms: u64,
    pub retryable_errors: Vec<ErrorType>,
    pub non_retryable_errors: Vec<ErrorType>,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay_ms: 1000,
            backoff_strategy: BackoffStrategy::Exponential,
            backoff_factor: 2.0,
            max_delay_ms: 30000,
            retryable_errors: vec![
                ErrorType::NetworkError,
                ErrorType::RateLimitError,
                ErrorType::TransactionError,
            ],
            non_retryable_errors: vec![
                ErrorType::ArgumentError,
                ErrorType::AuthenticationError,
                ErrorType::PermissionError,
            ],
        }
    }
}

impl RetryPolicy {
    /// Create a new retry policy with custom settings
    pub fn new(
        max_attempts: u32,
        base_delay_ms: u64,
        backoff_strategy: BackoffStrategy,
    ) -> Self {
        Self {
            max_attempts,
            base_delay_ms,
            backoff_strategy,
            ..Default::default()
        }
    }
    
    /// Set backoff factor
    pub fn with_backoff_factor(mut self, factor: f64) -> Self {
        self.backoff_factor = factor;
        self
    }
    
    /// Set max delay
    pub fn with_max_delay(mut self, delay_ms: u64) -> Self {
        self.max_delay_ms = delay_ms;
        self
    }
    
    /// Add retryable error type
    pub fn with_retryable_error(mut self, error_type: ErrorType) -> Self {
        if !self.retryable_errors.contains(&error_type) {
            self.retryable_errors.push(error_type);
        }
        self
    }
    
    /// Add non-retryable error type
    pub fn with_non_retryable_error(mut self, error_type: ErrorType) -> Self {
        if !self.non_retryable_errors.contains(&error_type) {
            self.non_retryable_errors.push(error_type);
        }
        self
    }
    
    /// Determine if an error should be retried
    pub fn should_retry(
        &self,
        error: &ErrorType,
        attempt: u32,
    ) -> RetryDecision {
        // Check if max attempts exceeded
        if attempt >= self.max_attempts {
            return RetryDecision::NoRetry {
                reason: format!("超过最大重试次数 {}", self.max_attempts),
            };
        }
        
        // Check if error is non-retryable
        if self.non_retryable_errors.contains(error) {
            return RetryDecision::NoRetry {
                reason: format!("错误类型 {} 不可重试", format!("{:?}", error)),
            };
        }
        
        // Check if error is explicitly retryable
        if self.retryable_errors.contains(error) {
            let delay = self.calculate_delay(attempt);
            
            return RetryDecision::Retry {
                delay_ms: delay,
                attempt: attempt + 1,
                max_attempts: self.max_attempts,
                reason: format!("错误 {} 可重试", format!("{:?}", error)),
            };
        }
        
        // Default: don't retry unknown errors
        RetryDecision::NoRetry {
            reason: format!("未知错误类型 {}", format!("{:?}", error)),
        }
    }
    
    /// Calculate delay based on strategy and attempt
    fn calculate_delay(&self, attempt: u32) -> u64 {
        match self.backoff_strategy {
            BackoffStrategy::Fixed => self.base_delay_ms,
            BackoffStrategy::Exponential => {
                let delay = (self.base_delay_ms as f64) * self.backoff_factor.powi(attempt as i32);
                delay.min(self.max_delay_ms as f64) as u64
            },
            BackoffStrategy::Linear => {
                let delay = self.base_delay_ms * (1 + attempt * self.backoff_factor as u32) as u64;
                delay.min(self.max_delay_ms)
            },
        }
    }
    
    /// Get a Duration for the retry delay
    pub fn retry_duration(&self, decision: &RetryDecision) -> Option<Duration> {
        match decision {
            RetryDecision::Retry { delay_ms, .. } => {
                Some(Duration::from_millis(*delay_ms))
            },
            RetryDecision::RetryWithBackoff { base_delay_ms, backoff_factor, attempt, .. } => {
                let delay = (*base_delay_ms as f64) * backoff_factor.powi(*attempt as i32);
                Some(Duration::from_millis(delay.min(self.max_delay_ms as f64) as u64))
            },
            RetryDecision::NoRetry { .. } => None,
        }
    }
}

/// Retry state tracking
#[derive(Debug, Clone)]
pub struct RetryState {
    pub current_attempt: u32,
    pub last_error: Option<String>,
    pub total_delay_ms: u64,
}

impl Default for RetryState {
    fn default() -> Self {
        Self {
            current_attempt: 0,
            last_error: None,
            total_delay_ms: 0,
        }
    }
}

impl RetryState {
    /// Create a new retry state
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Increment attempt count
    pub fn increment(&mut self) {
        self.current_attempt += 1;
    }
    
    /// Record error
    pub fn record_error(&mut self, error: String) {
        self.last_error = Some(error);
    }
    
    /// Add delay to total
    pub fn add_delay(&mut self, delay_ms: u64) {
        self.total_delay_ms += delay_ms;
    }
    
    /// Reset the retry state
    pub fn reset(&mut self) {
        *self = Self::default();
    }
    
    /// Check if should give up
    pub fn should_give_up(&self, max_attempts: u32) -> bool {
        self.current_attempt >= max_attempts
    }
    
    /// Get retry statistics
    pub fn stats(&self) -> RetryStats {
        RetryStats {
            total_attempts: self.current_attempt,
            total_delay_ms: self.total_delay_ms,
            last_error: self.last_error.clone(),
        }
    }
}

/// Retry statistics
#[derive(Debug, Clone)]
pub struct RetryStats {
    pub total_attempts: u32,
    pub total_delay_ms: u64,
    pub last_error: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_retry_policy() {
        let policy = RetryPolicy::default();
        
        assert_eq!(policy.max_attempts, 3);
        assert_eq!(policy.base_delay_ms, 1000);
        assert_eq!(policy.backoff_strategy, BackoffStrategy::Exponential);
    }
    
    #[test]
    fn test_should_retry_retryable_error() {
        let policy = RetryPolicy::default();
        
        let decision = policy.should_retry(&ErrorType::NetworkError, 0);
        
        assert!(matches!(decision, RetryDecision::Retry { .. }));
    }
    
    #[test]
    fn test_should_not_retry_non_retryable_error() {
        let policy = RetryPolicy::default();
        
        let decision = policy.should_retry(&ErrorType::ArgumentError, 0);
        
        assert!(matches!(decision, RetryDecision::NoRetry { .. }));
    }
    
    #[test]
    fn test_should_not_retry_max_attempts() {
        let policy = RetryPolicy::default();
        
        let decision = policy.should_retry(&ErrorType::NetworkError, 5);
        
        assert!(matches!(decision, RetryDecision::NoRetry { reason }));
        assert!(decision.to_string().contains("超过最大重试次数"));
    }
    
    #[test]
    fn test_calculate_delay_exponential() {
        let policy = RetryPolicy::default();
        
        // base_delay * 2^0 = 1000
        let delay1 = policy.calculate_delay(0);
        assert_eq!(delay1, 1000);
        
        // base_delay * 2^1 = 2000
        let delay2 = policy.calculate_delay(1);
        assert_eq!(delay2, 2000);
        
        // base_delay * 2^2 = 4000
        let delay3 = policy.calculate_delay(2);
        assert_eq!(delay3, 4000);
    }
    
    #[test]
    fn test_calculate_delay_linear() {
        let policy = RetryPolicy::new(3, 1000, BackoffStrategy::Linear)
            .with_backoff_factor(1.5);
        
        // base_delay * (1 + 0 * 1.5) = 1000
        let delay1 = policy.calculate_delay(0);
        assert_eq!(delay1, 1000);
        
        // base_delay * (1 + 1 * 1.5) = 2500
        let delay2 = policy.calculate_delay(1);
        assert_eq!(delay2, 2500);
        
        // base_delay * (1 + 2 * 1.5) = 4000
        let delay3 = policy.calculate_delay(2);
        assert_eq!(delay3, 4000);
    }
    
    #[test]
    fn test_retry_state() {
        let mut state = RetryState::new();
        
        assert_eq!(state.current_attempt, 0);
        
        state.increment();
        assert_eq!(state.current_attempt, 1);
        
        state.record_error("test error".to_string());
        assert_eq!(state.last_error, Some("test error".to_string()));
        
        state.add_delay(1000);
        assert_eq!(state.total_delay_ms, 1000);
        
        let stats = state.stats();
        assert_eq!(stats.total_attempts, 1);
        assert_eq!(stats.total_delay_ms, 1000);
    }
    
    #[test]
    fn test_should_give_up() {
        let mut state = RetryState::new();
        
        assert!(!state.should_give_up(3));
        
        state.current_attempt = 3;
        assert!(state.should_give_up(3));
        
        assert!(!state.should_give_up(5));
    }
}

// Helper for display
impl RetryDecision {
    pub fn to_string(&self) -> String {
        match self {
            RetryDecision::NoRetry { reason } => {
                format!("不重试: {}", reason)
            },
            RetryDecision::Retry { delay_ms, attempt, max_attempts, reason } => {
                format!("在 {}ms 后重试 ({}/{}) - {}", delay_ms, attempt, max_attempts, reason)
            },
            RetryDecision::RetryWithBackoff { base_delay_ms, attempt, max_attempts, .. } => {
                format!("使用退避重试 ({}/{}) - 基础延迟: {}ms", attempt, max_attempts, base_delay_ms)
            },
        }
    }
}
