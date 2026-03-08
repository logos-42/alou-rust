//! 智能重试策略模块
//!
//! 实现基于AI反馈的智能重试策略

use super::super::AsyncWorkflowExecutor;
use super::super::*;

impl AsyncWorkflowExecutor {
    /// 智能重试策略判断（增强版，集成AI反馈）
    pub async fn should_retry_with_smart_strategy(
        &self,
        execution_id: &str,
        ralph_config: &RalphLoopConfig,
        current_iteration: u32,
        error: &str,
    ) -> bool {
        if !ralph_config.smart_retry.enabled {
            return false;
        }

        println!("🧠 [SMART-RETRY] Analyzing error for retry strategy: {}", error);

        let histories = self.ralph_loop_histories.read().await;
        let history = match histories.get(execution_id) {
            Some(h) => h,
            None => return false,
        };

        // 检查连续失败次数
        let recent_failures = history.iterations.iter()
            .rev()
            .take(ralph_config.smart_retry.max_consecutive_failures as usize)
            .filter(|iter| iter.error.is_some())
            .count();

        if recent_failures >= ralph_config.smart_retry.max_consecutive_failures as usize {
            println!("🚫 [SMART-RETRY] Too many consecutive failures ({}), not retrying", recent_failures);
            return false;
        }

        // AI分析错误并提供修复建议
        if let Ok(ai_analysis) = self.ai_analyze_error(error, current_iteration).await {
            println!("🤖 [AI-ANALYSIS] AI error analysis: {}", ai_analysis);
            
            // 根据AI分析决定是否重试
            if ai_analysis.contains("CRITICAL") || ai_analysis.contains("FATAL") {
                println!("🚫 [SMART-RETRY] AI identified critical error, not retrying");
                return false;
            } else if ai_analysis.contains("RETRYABLE") || ai_analysis.contains("TEMPORARY") {
                println!("✅ [SMART-RETRY] AI identified retryable error, proceeding with retry");
                return true;
            }
        }

        // 基于错误类型的重试策略
        if let Some(retry_config) = ralph_config.smart_retry.error_based_retry.get(error) {
            println!("🔧 [SMART-RETRY] Applying error-specific retry strategy for: {}", error);
            return current_iteration < retry_config.max_retries;
        }

        // 自适应重试：根据历史表现调整
        if ralph_config.smart_retry.adaptive_retry && current_iteration >= ralph_config.smart_retry.learning_period {
            let success_rate = history.iterations.iter()
                .filter(|iter| iter.result.is_some())
                .count() as f64 / history.iterations.len() as f64;

            println!("📊 [SMART-RETRY] Current success rate: {:.2}%", success_rate * 100.0);

            if success_rate > 0.6 {
                println!("📈 [SMART-RETRY] Good success rate ({:.2}%), continuing retry", success_rate);
                return true;
            } else if success_rate < 0.2 {
                println!("📉 [SMART-RETRY] Low success rate ({:.2}%), reducing retry frequency", success_rate);
                return false;
            } else {
                println!("⚖️ [SMART-RETRY] Moderate success rate ({:.2}%), using default retry logic", success_rate);
                return current_iteration < 5;
            }
        }

        // 默认重试逻辑（基于错误类型）
        self.is_retryable_error(error) && current_iteration < 3
    }

    /// 判断错误是否可重试
    fn is_retryable_error(&self, error: &str) -> bool {
        let error_lower = error.to_lowercase();
        
        // 网络相关错误（通常可重试）
        if error_lower.contains("timeout") || 
           error_lower.contains("connection") || 
           error_lower.contains("network") ||
           error_lower.contains("temporary") {
            return true;
        }

        // 资源相关错误（通常可重试）
        if error_lower.contains("resource") && 
           (error_lower.contains("busy") || error_lower.contains("unavailable")) {
            return true;
        }

        // 速率限制错误（通常可重试）
        if error_lower.contains("rate limit") || 
           error_lower.contains("too many requests") {
            return true;
        }

        // 临时性错误关键词
        if error_lower.contains("temporary") || 
           error_lower.contains("transient") ||
           error_lower.contains("retry") {
            return true;
        }

        // 权限、配置、致命错误（通常不可重试）
        if error_lower.contains("permission") || 
           error_lower.contains("access denied") ||
           error_lower.contains("authentication") ||
           error_lower.contains("authorization") ||
           error_lower.contains("fatal") ||
           error_lower.contains("critical") {
            return false;
        }

        // 默认情况下，未知错误可重试一次
        true
    }

    /// 计算重试延迟（指数退避）
    pub fn calculate_retry_delay(&self, iteration: u32, base_delay_ms: u64) -> u64 {
        let delay = base_delay_ms * (2_u64.pow(iteration.min(6)));
        std::cmp::min(delay, 30000) // 最大30秒
    }

    /// 获取错误重试建议
    async fn get_retry_recommendation(&self, _error: &str, iteration: u32) -> String {
        let retry_delay = self.calculate_retry_delay(iteration, 1000);
        
        format!(
            "建议{}后重试 (第{}次尝试)",
            if retry_delay < 1000 {
                format!("{}ms", retry_delay)
            } else {
                format!("{}秒", retry_delay / 1000)
            },
            iteration + 1
        )
    }

    /// 优化重试策略
    async fn optimize_retry_strategy(&self, execution_id: &str) -> serde_json::Value {
        let histories = self.ralph_loop_histories.read().await;
        if let Some(history) = histories.get(execution_id) {
            let error_frequency = self.analyze_error_frequency(&history.iterations);
            let success_by_iteration = self.analyze_success_by_iteration(&history.iterations);
            
            serde_json::json!({
                "error_frequency": error_frequency,
                "success_by_iteration": success_by_iteration,
                "recommendations": self.generate_retry_recommendations(&history.iterations),
                "optimal_retry_count": self.calculate_optimal_retry_count(&history.iterations)
            })
        } else {
            serde_json::json!({
                "error_frequency": {},
                "success_by_iteration": {},
                "recommendations": [],
                "optimal_retry_count": 3
            })
        }
    }

    /// 分析错误频率
    fn analyze_error_frequency(&self, iterations: &[RalphLoopIterationHistory]) -> std::collections::HashMap<String, usize> {
        let mut frequency = std::collections::HashMap::new();
        
        for iteration in iterations {
            if let Some(error) = &iteration.error {
                let error_type = self.categorize_error(error);
                *frequency.entry(error_type).or_insert(0) += 1;
            }
        }
        
        frequency
    }

    /// 分析按迭代的成功率
    fn analyze_success_by_iteration(&self, iterations: &[RalphLoopIterationHistory]) -> Vec<f64> {
        let mut success_rates = Vec::new();
        
        for i in 1..=10 { // 分析前10次迭代
            let iteration_success = iterations.iter()
                .filter(|iter| iter.iteration == i as u32 && iter.result.is_some())
                .count();
            
            let iteration_total = iterations.iter()
                .filter(|iter| iter.iteration == i as u32)
                .count();
            
            let success_rate = if iteration_total > 0 {
                iteration_success as f64 / iteration_total as f64
            } else {
                0.0
            };
            
            success_rates.push(success_rate);
        }
        
        success_rates
    }

    /// 生成重试建议
    fn generate_retry_recommendations(&self, iterations: &[RalphLoopIterationHistory]) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        let error_frequency = self.analyze_error_frequency(iterations);
        
        // 基于错误频率生成建议
        for (error_type, count) in error_frequency {
            if count > 3 {
                recommendations.push(format!("{}错误频繁出现，建议检查相关配置", error_type));
            }
        }
        
        // 基于成功率生成建议
        let success_rates = self.analyze_success_by_iteration(iterations);
        if success_rates.len() > 3 {
            let early_success_rate = success_rates[0..3].iter().sum::<f64>() / 3.0;
            let late_success_rate = success_rates[3..].iter().sum::<f64>() / (success_rates.len() - 3) as f64;
            
            if late_success_rate < early_success_rate {
                recommendations.push("后期成功率下降，建议增加重试次数或调整策略".to_string());
            }
        }
        
        recommendations
    }

    /// 计算最优重试次数
    fn calculate_optimal_retry_count(&self, iterations: &[RalphLoopIterationHistory]) -> u32 {
        let success_rates = self.analyze_success_by_iteration(iterations);
        
        // 找到成功率开始显著下降的点
        for (i, &rate) in success_rates.iter().enumerate() {
            if i > 0 && rate < success_rates[i-1] * 0.8 {
                return (i + 1) as u32;
            }
        }
        
        3 // 默认重试3次
    }
}
