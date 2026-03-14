//! Session Actor 真实并发压力测试

use std::sync::Arc;
use std::time::Instant;
use tokio::time::Duration;
use futures::future::join_all;

use crate::bridges::{BridgeManager, create_default_bridge_manager};
use crate::tools::ToolRegistry;

/// 测试 1: 验证 BridgeManager 并发访问（真实计时）
#[tokio::test]
async fn test_bridge_manager_real_concurrent() {
    let bridge_manager = Arc::new(create_default_bridge_manager());
    
    println!("\n=== BridgeManager 并发测试 (100 任务) ===");
    let start = Instant::now();
    
    let tasks: Vec<_> = (0..100)
        .map(|i| {
            let bm = bridge_manager.clone();
            tokio::spawn(async move {
                let t0 = Instant::now();
                let _tb = bm.tool_bridge();
                tokio::time::sleep(Duration::from_millis(10)).await;
                let elapsed = t0.elapsed();
                (i, elapsed)
            })
        })
        .collect();
    
    let results = join_all(tasks).await;
    let total_elapsed = start.elapsed();
    
    let mut min_elapsed = Duration::MAX;
    let mut max_elapsed = Duration::ZERO;
    for result in results {
        if let Ok((i, elapsed)) = result {
            if elapsed < min_elapsed { min_elapsed = elapsed; }
            if elapsed > max_elapsed { max_elapsed = elapsed; }
            println!("  Task {}: {:?}", i, elapsed);
        }
    }
    
    println!("总耗时：{:?}", total_elapsed);
    println!("最快任务：{:?}", min_elapsed);
    println!("最慢任务：{:?}", max_elapsed);
    
    if total_elapsed < Duration::from_millis(200) {
        println!("✅ PASS: 真正并发执行 (总耗时 < 200ms)");
    } else {
        println!("❌ FAIL: 可能有序列化瓶颈 (总耗时 {:?})", total_elapsed);
    }
    
    assert!(total_elapsed < Duration::from_millis(200), "BridgeManager 并发失败");
}

/// 测试 2: 验证 ToolRegistry 并发读取
#[tokio::test]
async fn test_tool_registry_concurrent_read() {
    let registry = Arc::new(ToolRegistry::new());
    
    println!("\n=== ToolRegistry 并发测试 (50 次读取) ===");
    let start = Instant::now();
    
    let tasks: Vec<_> = (0..50)
        .map(|i| {
            let reg = registry.clone();
            tokio::spawn(async move {
                let t0 = Instant::now();
                let count = reg.count().await;
                let elapsed = t0.elapsed();
                (i, count, elapsed)
            })
        })
        .collect();
    
    let results = join_all(tasks).await;
    let total_elapsed = start.elapsed();
    
    println!("总耗时：{:?}", total_elapsed);
    
    assert!(total_elapsed < Duration::from_millis(50), "Registry 读取有阻塞");
    println!("✅ PASS: Registry 读取无阻塞");
}

/// 测试 3: 验证 TaskManager 并发创建
#[tokio::test]
async fn test_task_manager_concurrent_create() {
    use crate::agent::task::TaskManager;
    
    let task_manager = Arc::new(TaskManager::new());
    
    println!("\n=== TaskManager 并发测试 (20 个任务创建) ===");
    let start = Instant::now();
    
    let tasks: Vec<_> = (0..20)
        .map(|i| {
            let tm = task_manager.clone();
            tokio::spawn(async move {
                let t0 = Instant::now();
                let task_id = tm.create_task(
                    format!("agent_{}", i),
                    format!("Task {}", i),
                ).await;
                let elapsed = t0.elapsed();
                (i, task_id, elapsed)
            })
        })
        .collect();
    
    let results = join_all(tasks).await;
    let total_elapsed = start.elapsed();
    
    println!("总耗时：{:?}", total_elapsed);
    
    assert!(total_elapsed < Duration::from_millis(100), "Task 创建有阻塞");
    println!("✅ PASS: Task 创建无阻塞");
}
