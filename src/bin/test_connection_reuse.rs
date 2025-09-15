use std::sync::Arc;
use std::time::Instant;
use alou::connection_pool::{ConnectionPool, McpServerConfig};
use anyhow::Result;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("🔍 测试MCP连接池复用机制");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // 创建连接池
    let pool = Arc::new(ConnectionPool::new());

    // 注册filesystem服务器配置
    let filesystem_config = McpServerConfig {
        command: "npx.cmd".to_string(),
        args: vec![
            "-y".to_string(),
            "@modelcontextprotocol/server-filesystem".to_string(),
            "C:\\".to_string(),
            "D:\\".to_string(),
            "E:\\".to_string(),
        ],
        directory: None,
        env: None,
    };

    pool.register_server("filesystem".to_string(), filesystem_config).await;

    println!("\n📊 测试连接复用性能:");
    
    // 第一次连接 - 应该创建新连接
    println!("\n1️⃣ 第一次连接 (创建新连接):");
    let start = Instant::now();
    let client1 = pool.get_connection("filesystem").await?;
    let first_time = start.elapsed();
    println!("   ⏱️  耗时: {:?}", first_time);
    
    // 测试工具列表
    {
        let client = client1.lock().await;
        let tools_result = client.list_tools().await?;
        println!("   🛠️  工具数量: {}", tools_result.tools.len());
    }

    // 第二次连接 - 应该复用现有连接
    println!("\n2️⃣ 第二次连接 (复用现有连接):");
    let start = Instant::now();
    let client2 = pool.get_connection("filesystem").await?;
    let second_time = start.elapsed();
    println!("   ⏱️  耗时: {:?}", second_time);
    
    // 测试工具列表
    {
        let client = client2.lock().await;
        let tools_result = client.list_tools().await?;
        println!("   🛠️  工具数量: {}", tools_result.tools.len());
    }

    // 第三次连接 - 应该复用现有连接
    println!("\n3️⃣ 第三次连接 (复用现有连接):");
    let start = Instant::now();
    let client3 = pool.get_connection("filesystem").await?;
    let third_time = start.elapsed();
    println!("   ⏱️  耗时: {:?}", third_time);

    // 分析结果
    println!("\n📈 性能分析:");
    println!("   🆕 首次连接耗时: {:?}", first_time);
    println!("   🔄 复用连接耗时: {:?}", second_time);
    println!("   🔄 复用连接耗时: {:?}", third_time);
    
    if second_time < first_time {
        println!("   ✅ 连接复用成功！复用连接比首次连接快 {:.2}x", 
                first_time.as_nanos() as f64 / second_time.as_nanos() as f64);
    } else {
        println!("   ⚠️  连接复用可能未生效");
    }

    // 验证是否为同一个连接对象
    println!("\n🔍 连接对象验证:");
    let ptr1 = Arc::as_ptr(&client1);
    let ptr2 = Arc::as_ptr(&client2);
    let ptr3 = Arc::as_ptr(&client3);
    
    println!("   Client1 指针: {:p}", ptr1);
    println!("   Client2 指针: {:p}", ptr2);
    println!("   Client3 指针: {:p}", ptr3);
    
    if ptr1 == ptr2 && ptr2 == ptr3 {
        println!("   ✅ 所有连接都指向同一个对象，连接复用成功！");
    } else {
        println!("   ❌ 连接对象不同，连接复用失败！");
    }

    // 显示连接池状态
    println!("\n📋 连接池状态:");
    pool.show_pool_status().await;

    // 清理连接
    println!("\n🧹 清理连接...");
    pool.close_all_connections().await?;
    println!("   ✅ 连接清理完成");

    Ok(())
}
