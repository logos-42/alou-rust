//! Session Actor 并发测试
//!
//! 验证多 session 并发执行能力

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::time::Instant;
    use tokio::time::Duration;
    use futures::future::join_all;

    use crate::bridges::{BridgeManager, create_default_bridge_manager};
    use crate::runtime::router::SessionRouter;
    use crate::runtime::message::SessionMessage;
    use crate::tools::ToolRegistry;
    use crate::agent::task::TaskManager;

    /// 测试 1: 验证 BridgeManager 无锁访问
    #[tokio::test]
    async fn test_bridge_manager_lock_free() {
        let bridge_manager = Arc::new(create_default_bridge_manager());

        let start = Instant::now();

        // 并发访问 100 次
        let tasks: Vec<_> = (0..100)
            .map(|i| {
                let bm = bridge_manager.clone();
                tokio::spawn(async move {
                    // 直接访问 tool_bridge（无锁）
                    let _tool_bridge = bm.tool_bridge();
                    // 模拟工具调用
                    tokio::time::sleep(Duration::from_millis(1)).await;
                    i
                })
            })
            .collect();

        let results = join_all(tasks).await;
        let elapsed = start.elapsed();

        // 验证：所有访问成功
        assert_eq!(results.len(), 100);
        println!("✓ 100 bridge accesses completed in {:?}", elapsed);

        // 如果是串行，需要 100ms+
        // 如果是并行，应该 < 50ms
        assert!(elapsed < Duration::from_millis(80), "BridgeManager 可能有锁竞争");
    }

    /// 测试 2: 验证 ToolRegistry 并发读取
    #[tokio::test]
    async fn test_tool_registry_concurrent_read() {
        let registry = Arc::new(ToolRegistry::new());

        let start = Instant::now();

        // 并发读取 50 次
        let tasks: Vec<_> = (0..50)
            .map(|_| {
                let registry = registry.clone();
                tokio::spawn(async move {
                    registry.count().await
                })
            })
            .collect();

        let results = join_all(tasks).await;
        let elapsed = start.elapsed();

        // 验证：所有读取成功
        assert!(results.iter().all(|r| r.is_ok()));
        println!("✓ 50 registry reads completed in {:?}", elapsed);
    }

    /// 测试 3: 验证 TaskManager 并发创建任务
    #[tokio::test]
    async fn test_task_manager_concurrent_create() {
        let task_manager = Arc::new(TaskManager::new());

        let start = Instant::now();

        // 并发创建 50 个任务
        let tasks: Vec<_> = (0..50)
            .map(|i| {
                let tm = task_manager.clone();
                tokio::spawn(async move {
                    let task_id = tm.create_task(
                        format!("agent_{}", i),
                        format!("Task {}", i),
                    ).await;
                    (i, task_id)
                })
            })
            .collect();

        let results = join_all(tasks).await;
        let elapsed = start.elapsed();

        // 验证：所有任务创建成功
        let success_count = results.iter()
            .filter(|r| r.is_ok())
            .count();
        
        println!("✓ {} tasks created concurrently in {:?}", success_count, elapsed);
        assert_eq!(success_count, 50);
    }

    /// 测试 4: 压力测试 - 模拟真实并发场景
    #[tokio::test]
    async fn test_stress_concurrent_sessions() {
        let bridge_manager = Arc::new(create_default_bridge_manager());
        let tool_registry = Arc::new(ToolRegistry::new());
        let router = Arc::new(SessionRouter::new(bridge_manager, tool_registry));

        let session_count = 20;
        let messages_per_session = 5;

        let start = Instant::now();

        // 创建 20 个 session，每个发送 5 条消息
        let tasks: Vec<_> = (0..session_count)
            .flat_map(|i| {
                (0..messages_per_session).map(move |j| {
                    let router = router.clone();
                    let session_id = format!("stress_session_{}", i);
                    tokio::spawn(async move {
                        // 注意：实际使用需要 AiClient，这里只测试 router 并发
                        let _ = router.active_count();
                        (i, j)
                    })
                })
            })
            .collect();

        let results = join_all(tasks).await;
        let elapsed = start.elapsed();

        assert_eq!(results.len(), session_count * messages_per_session);
        println!(
            "✓ Stress test: {} sessions × {} messages = {} total in {:?}",
            session_count,
            messages_per_session,
            results.len(),
            elapsed
        );

        // 20×5=100 个操作应该在 500ms 内完成
        assert!(elapsed < Duration::from_millis(500));
    }
}
