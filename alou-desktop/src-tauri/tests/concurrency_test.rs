//! 并发验证测试
//!
//! 验证多 Session 并发执行时的性能表现

use std::sync::Arc;
use std::time::Instant;
use tokio::time::Duration;
use tokio::task::JoinSet;

// 模拟 Session 执行
struct MockSession {
    session_id: String,
    tool_call_count: usize,
}

impl MockSession {
    fn new(session_id: &str, tool_call_count: usize) -> Self {
        Self {
            session_id: session_id.to_string(),
            tool_call_count,
        }
    }

    async fn simulate_ai_call(&self) -> Duration {
        let start = Instant::now();
        tokio::time::sleep(Duration::from_millis(500 + (rand::random::<u64>() % 1500))).await;
        start.elapsed()
    }

    async fn simulate_tool_call(&self, tool_name: &str) -> Duration {
        let start = Instant::now();
        tokio::time::sleep(Duration::from_millis(100 + (rand::random::<u64>() % 400))).await;
        start.elapsed()
    }

    async fn execute_ralph_loop_serial(&self) -> SessionResult {
        let start = Instant::now();
        let ai_duration = self.simulate_ai_call().await;
        let mut tool_durations = Vec::new();
        for i in 0..self.tool_call_count {
            let duration = self.simulate_tool_call(&format!("tool_{}", i)).await;
            tool_durations.push(duration);
        }
        let total_duration = start.elapsed();
        SessionResult {
            session_id: self.session_id.clone(),
            ai_duration,
            tool_durations,
            total_duration,
        }
    }

    async fn execute_ralph_loop_parallel(&self) -> SessionResult {
        let start = Instant::now();
        let ai_duration = self.simulate_ai_call().await;
        let mut join_set = JoinSet::new();
        for i in 0..self.tool_call_count {
            let tool_name = format!("tool_{}", i);
            join_set.spawn(async move {
                let tool_start = Instant::now();
                tokio::time::sleep(Duration::from_millis(100 + (rand::random::<u64>() % 400))).await;
                (tool_name, tool_start.elapsed())
            });
        }
        let mut tool_durations = Vec::new();
        while let Some(result) = join_set.join_next().await {
            if let Ok((name, duration)) = result {
                tool_durations.push(duration);
                println!("    {} completed in {} ms", name, duration.as_millis());
            }
        }
        let total_duration = start.elapsed();
        SessionResult {
            session_id: self.session_id.clone(),
            ai_duration,
            tool_durations,
            total_duration,
        }
    }
}

struct SessionResult {
    session_id: String,
    ai_duration: Duration,
    tool_durations: Vec<Duration>,
    total_duration: Duration,
}

struct ConcurrencyTestConfig {
    session_count: usize,
    tool_calls_per_session: usize,
    use_parallel: bool,
}

async fn run_concurrency_test(config: ConcurrencyTestConfig) -> TestReport {
    println!("\n{}", "=".repeat(60));
    println!("并发测试开始");
    println!("{}", "=".repeat(60));
    println!("Sessions: {}", config.session_count);
    println!("Tool calls per session: {}", config.tool_calls_per_session);
    println!("执行模式：{}", if config.use_parallel { "并行" } else { "串行" });
    
    let start = Instant::now();
    let mut join_set = JoinSet::new();
    
    for i in 0..config.session_count {
        let session = Arc::new(MockSession::new(
            &format!("session_{}", i),
            config.tool_calls_per_session,
        ));
        
        if config.use_parallel {
            join_set.spawn(async move {
                session.execute_ralph_loop_parallel().await
            });
        } else {
            join_set.spawn(async move {
                session.execute_ralph_loop_serial().await
            });
        }
    }
    
    let mut results = Vec::new();
    while let Some(result) = join_set.join_next().await {
        match result {
            Ok(session_result) => results.push(session_result),
            Err(e) => eprintln!("Session failed: {}", e),
        }
    }
    
    let total_test_duration = start.elapsed();
    generate_report(config, results, total_test_duration)
}

fn generate_report(
    config: ConcurrencyTestConfig,
    results: Vec<SessionResult>,
    total_test_duration: Duration,
) -> TestReport {
    let mut report = TestReport::new(config.session_count, config.use_parallel);
    
    let avg_ai_duration = results.iter()
        .map(|r| r.ai_duration.as_millis() as u64)
        .sum::<u64>() / results.len() as u64;
    
    let avg_tool_duration = results.iter()
        .flat_map(|r| r.tool_durations.iter())
        .map(|d| d.as_millis() as u64)
        .sum::<u64>() / (results.len() * config.tool_calls_per_session) as u64;
    
    let avg_total_duration = results.iter()
        .map(|r| r.total_duration.as_millis() as u64)
        .sum::<u64>() / results.len() as u64;
    
    let theoretical_serial_duration = results.iter()
        .map(|r| {
            r.ai_duration.as_millis() as u64 
            + r.tool_durations.iter().map(|d| d.as_millis() as u64).sum::<u64>()
        })
        .sum::<u64>();
    
    report.avg_ai_duration_ms = avg_ai_duration;
    report.avg_tool_duration_ms = avg_tool_duration;
    report.avg_total_duration_ms = avg_total_duration;
    report.theoretical_serial_duration_ms = theoretical_serial_duration;
    report.actual_parallel_duration_ms = total_test_duration.as_millis() as u64;
    report.speedup_ratio = if config.use_parallel {
        theoretical_serial_duration as f64 / total_test_duration.as_millis() as f64
    } else {
        1.0
    };
    
    report
}

struct TestReport {
    session_count: usize,
    use_parallel: bool,
    avg_ai_duration_ms: u64,
    avg_tool_duration_ms: u64,
    avg_total_duration_ms: u64,
    theoretical_serial_duration_ms: u64,
    actual_parallel_duration_ms: u64,
    speedup_ratio: f64,
}

impl TestReport {
    fn new(session_count: usize, use_parallel: bool) -> Self {
        Self {
            session_count,
            use_parallel,
            avg_ai_duration_ms: 0,
            avg_tool_duration_ms: 0,
            avg_total_duration_ms: 0,
            theoretical_serial_duration_ms: 0,
            actual_parallel_duration_ms: 0,
            speedup_ratio: 0.0,
        }
    }
    
    fn print(&self) {
        println!("\n{}", "=".repeat(60));
        println!("测试报告 - {} Sessions ({})", 
            self.session_count,
            if self.use_parallel { "并行" } else { "串行" });
        println!("{}", "=".repeat(60));
        println!("平均 AI 调用时间：     {} ms", self.avg_ai_duration_ms);
        println!("平均工具调用时间：    {} ms", self.avg_tool_duration_ms);
        println!("平均 Session 总时间：  {} ms", self.avg_total_duration_ms);
        println!("理论串行总时间：      {} ms ({:.2} s)", 
            self.theoretical_serial_duration_ms,
            self.theoretical_serial_duration_ms as f64 / 1000.0);
        println!("实际并行总时间：      {} ms ({:.2} s)", 
            self.actual_parallel_duration_ms,
            self.actual_parallel_duration_ms as f64 / 1000.0);
        println!("加速比：               {:.2}x", self.speedup_ratio);
        println!("{}", "=".repeat(60));
    }
}

#[tokio::test]
async fn test_5_sessions_serial() {
    let config = ConcurrencyTestConfig {
        session_count: 5,
        tool_calls_per_session: 3,
        use_parallel: false,
    };
    let report = run_concurrency_test(config).await;
    report.print();
}

#[tokio::test]
async fn test_10_sessions_serial() {
    let config = ConcurrencyTestConfig {
        session_count: 10,
        tool_calls_per_session: 3,
        use_parallel: false,
    };
    let report = run_concurrency_test(config).await;
    report.print();
}

#[tokio::test]
async fn test_20_sessions_serial() {
    let config = ConcurrencyTestConfig {
        session_count: 20,
        tool_calls_per_session: 3,
        use_parallel: false,
    };
    let report = run_concurrency_test(config).await;
    report.print();
}

#[tokio::test]
async fn test_5_sessions_parallel() {
    let config = ConcurrencyTestConfig {
        session_count: 5,
        tool_calls_per_session: 3,
        use_parallel: true,
    };
    let report = run_concurrency_test(config).await;
    report.print();
}

#[tokio::test]
async fn test_10_sessions_parallel() {
    let config = ConcurrencyTestConfig {
        session_count: 10,
        tool_calls_per_session: 3,
        use_parallel: true,
    };
    let report = run_concurrency_test(config).await;
    report.print();
}

#[tokio::test]
async fn test_20_sessions_parallel() {
    let config = ConcurrencyTestConfig {
        session_count: 20,
        tool_calls_per_session: 3,
        use_parallel: true,
    };
    let report = run_concurrency_test(config).await;
    report.print();
}

#[tokio::test]
async fn test_tool_call_serialization() {
    println!("\n{}", "=".repeat(60));
    println!("工具调用串行化验证");
    println!("{}", "=".repeat(60));
    
    let session = MockSession::new("test_session", 5);
    
    let start = Instant::now();
    let mut durations = Vec::new();
    for i in 0..5 {
        let d = session.simulate_tool_call(&format!("tool_{}", i)).await;
        durations.push(d);
        println!("Tool {} completed in {} ms", i, d.as_millis());
    }
    
    let total_serial = start.elapsed();
    let sum_individual: Duration = durations.iter().sum();
    
    println!("\n串行总时间：{} ms", total_serial.as_millis());
    println!("单个时间和：{} ms", sum_individual.as_millis());
    
    let diff_ratio = total_serial.as_millis() as f64 / sum_individual.as_millis() as f64;
    println!("\n串行化程度：{:.2} (1.0 = 完全串行)", diff_ratio);
    
    assert!(diff_ratio > 0.9, "当前工具调用是串行的");
}
