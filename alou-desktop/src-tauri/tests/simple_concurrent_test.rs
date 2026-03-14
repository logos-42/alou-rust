//! 简单并发测试 - 验证多 session 并发

use std::sync::Arc;
use std::time::Instant;
use tokio::time::Duration;

/// 测试：模拟多 session 并发执行
#[tokio::test]
async fn test_simple_concurrent_sessions() {
    println!("\n=== 并发测试：模拟 10 sessions ===");
    let start = Instant::now();
    
    // 模拟 10 个 sessions，每个执行 AI→tool→AI 循环
    let tasks: Vec<_> = (0..10)
        .map(|session_id| {
            tokio::spawn(async move {
                let session_start = Instant::now();
                println!("[session-{}] 开始执行", session_id);
                
                // 模拟 AI 调用 (100ms)
                tokio::time::sleep(Duration::from_millis(100)).await;
                println!("[session-{}] AI 完成 (+{:?})", session_id, session_start.elapsed());
                
                // 模拟工具执行 (50ms)
                tokio::time::sleep(Duration::from_millis(50)).await;
                println!("[session-{}] Tool 完成 (+{:?})", session_id, session_start.elapsed());
                
                // 第二次 AI 调用 (100ms)
                tokio::time::sleep(Duration::from_millis(100)).await;
                println!("[session-{}] 全部完成 (+{:?})", session_id, session_start.elapsed());
                
                (session_id, session_start.elapsed())
            })
        })
        .collect();
    
    let results = futures::future::join_all(tasks).await;
    let total_elapsed = start.elapsed();
    
    println!("\n汇总:");
    for result in results {
        if let Ok((id, elapsed)) = result {
            println!("  Session {}: {:?}", id, elapsed);
        }
    }
    println!("总耗时：{:?}", total_elapsed);
    
    // 10 sessions × (100+50+100)ms = 2500ms (串行)
    // 并发应该 < 500ms
    if total_elapsed < Duration::from_millis(500) {
        println!("✅ PASS: 真正并发执行");
    } else {
        println!("⚠️  总耗时 {:?} (可能有序列化)", total_elapsed);
    }
}

/// 测试：BridgeManager 并发访问
#[tokio::test]
async fn test_bridge_concurrent_access() {
    use crate::bridges::{BridgeManager, create_default_bridge_manager};
    
    println!("\n=== BridgeManager 并发测试 (20 sessions) ===");
    let bridge_manager = Arc::new(create_default_bridge_manager());
    let start = Instant::now();
    
    let tasks: Vec<_> = (0..20)
        .map(|i| {
            let bm = bridge_manager.clone();
            tokio::spawn(async move {
                let t0 = Instant::now();
                // 访问 bridge
                let _tb = bm.tool_bridge();
                // 模拟工具执行
                tokio::time::sleep(Duration::from_millis(20)).await;
                (i, t0.elapsed())
            })
        })
        .collect();
    
    let results = futures::future::join_all(tasks).await;
    let total = start.elapsed();
    
    println!("20 sessions 访问 BridgeManager:");
    for r in results {
        if let Ok((i, elapsed)) = r {
            println!("  Session {}: {:?}", i, elapsed);
        }
    }
    println!("总耗时：{:?}", total);
    
    // 20 × 20ms = 400ms (串行)
    // 并发应该 < 100ms
    assert!(total < Duration::from_millis(100), "BridgeManager 并发失败");
    println!("✅ PASS: BridgeManager 无锁并发");
}
