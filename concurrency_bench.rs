#!/usr/bin/env cargo
//! 并发验证 Benchmark
//!
//! 运行：cargo script concurrency_bench.rs

/*
[dependencies]
tokio = { version = "1", features = ["full"] }
rand = "0.8"
*/

use std::sync::Arc;
use std::time::Instant;
use tokio::time::Duration;
use tokio::task::JoinSet;

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

    async fn simulate_tool_call(&self, _tool_name: &str) -> Duration {
        let start = Instant::now();
        tokio::time::sleep(Duration::from_millis(100 + (rand::random::<u64>() % 400))).await;
        start.elapsed()
    }

    async fn execute_serial(&self) -> SessionResult {
        let start = Instant::now();
        let ai_duration = self.simulate_ai_call().await;
        let mut tool_durations = Vec::new();
        for i in 0..self.tool_call_count {
            let duration = self.simulate_tool_call(&format!("tool_{}", i)).await;
            tool_durations.push(duration);
        }
        SessionResult {
            session_id: self.session_id.clone(),
            ai_duration,
            tool_durations,
            total_duration: start.elapsed(),
        }
    }

    async fn execute_parallel(&self) -> SessionResult {
        let start = Instant::now();
        let ai_duration = self.simulate_ai_call().await;
        
        let mut join_set = JoinSet::new();
        for i in 0..self.tool_call_count {
            join_set.spawn(async move {
                let tool_start = Instant::now();
                tokio::time::sleep(Duration::from_millis(100 + (rand::random::<u64>() % 400))).await;
                tool_start.elapsed()
            });
        }
        
        let mut tool_durations = Vec::new();
        while let Some(result) = join_set.join_next().await {
            if let Ok(duration) = result {
                tool_durations.push(duration);
            }
        }
        
        SessionResult {
            session_id: self.session_id.clone(),
            ai_duration,
            tool_durations,
            total_duration: start.elapsed(),
        }
    }
}

struct SessionResult {
    session_id: String,
    ai_duration: Duration,
    tool_durations: Vec<Duration>,
    total_duration: Duration,
}

#[tokio::main]
async fn main() {
    println!("\n{}", "=".repeat(70));
    println!("Agent Runtime 并发验证 Benchmark");
    println!("{}", "=".repeat(70));
    
    for session_count in [5, 10, 20] {
        println!("\n{}", "-".repeat(70));
        println!("测试：{} Sessions (串行模式)", session_count);
        println!("{}", "-".repeat(70));
        
        let start = Instant::now();
        let mut join_set = JoinSet::new();
        
        for i in 0..session_count {
            let session = Arc::new(MockSession::new(&format!("session_{}", i), 3));
            join_set.spawn(async move {
                session.execute_serial().await
            });
        }
        
        let mut results = Vec::new();
        while let Some(result) = join_set.join_next().await {
            if let Ok(r) = result {
                results.push(r);
            }
        }
        
        let total_duration = start.elapsed();
        print_report(session_count, &results, total_duration, "串行");
        
        println!("\n{}", "-".repeat(70));
        println!("测试：{} Sessions (并行模式)", session_count);
        println!("{}", "-".repeat(70));
        
        let start = Instant::now();
        let mut join_set = JoinSet::new();
        
        for i in 0..session_count {
            let session = Arc::new(MockSession::new(&format!("session_{}", i), 3));
            join_set.spawn(async move {
                session.execute_parallel().await
            });
        }
        
        let mut results = Vec::new();
        while let Some(result) = join_set.join_next().await {
            if let Ok(r) = result {
                results.push(r);
            }
        }
        
        let total_duration = start.elapsed();
        print_report(session_count, &results, total_duration, "并行");
    }
    
    println!("\n{}", "=".repeat(70));
    println!("工具调用串行化验证");
    println!("{}", "=".repeat(70));
    
    let session = MockSession::new("test_session", 5);
    let start = Instant::now();
    let mut durations = Vec::new();
    for i in 0..5 {
        let d = session.simulate_tool_call(&format!("tool_{}", i)).await;
        durations.push(d);
        println!("  Tool {} completed in {} ms", i, d.as_millis());
    }
    let total_serial = start.elapsed();
    let sum_individual: Duration = durations.iter().sum();
    
    println!("\n  串行总时间：{} ms", total_serial.as_millis());
    println!("  单个时间和：{} ms", sum_individual.as_millis());
    println!("  串行化程度：{:.2} (1.0 = 完全串行)", 
        total_serial.as_millis() as f64 / sum_individual.as_millis() as f64);
    
    println!("\n{}", "=".repeat(70));
    println!("Benchmark 完成");
    println!("{}", "=".repeat(70));
}

fn print_report(session_count: usize, results: &[SessionResult], total_duration: Duration, mode: &str) {
    let avg_ai = results.iter()
        .map(|r| r.ai_duration.as_millis() as u64)
        .sum::<u64>() / results.len() as u64;
    
    let avg_tool = results.iter()
        .flat_map(|r| r.tool_durations.iter())
        .map(|d| d.as_millis() as u64)
        .sum::<u64>() / (results.len() * 3) as u64;
    
    let avg_total = results.iter()
        .map(|r| r.total_duration.as_millis() as u64)
        .sum::<u64>() / results.len() as u64;
    
    let theoretical_serial = results.iter()
        .map(|r| {
            r.ai_duration.as_millis() as u64 
            + r.tool_durations.iter().map(|d| d.as_millis() as u64).sum::<u64>()
        })
        .sum::<u64>();
    
    println!("  Sessions 数量：        {}", session_count);
    println!("  执行模式：            {}", mode);
    println!("  平均 AI 调用时间：     {} ms", avg_ai);
    println!("  平均工具调用时间：    {} ms", avg_tool);
    println!("  平均 Session 总时间：  {} ms", avg_total);
    println!("  理论串行总时间：      {} ms ({:.2} s)", theoretical_serial, theoretical_serial as f64 / 1000.0);
    println!("  实际总时间：          {} ms ({:.2} s)", total_duration.as_millis(), total_duration.as_millis() as f64 / 1000.0);
}
