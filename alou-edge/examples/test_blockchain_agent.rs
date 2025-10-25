// 这是一个示例测试文件，展示如何使用 BlockchainAgent
// 注意：这不是实际的测试文件，仅用于演示

use alou_edge::agent::{BlockchainAgent, tools::{QueryTool, TransactionTool, BroadcastTool}};

// 示例 1: 创建和使用 BlockchainAgent
async fn example_blockchain_agent() -> Result<(), Box<dyn std::error::Error>> {
    let agent = BlockchainAgent::new(
        "deepseek",
        "your-api-key".to_string(),
        Some("deepseek-chat".to_string()),
        "https://eth-mainnet.g.alchemy.com/v2/YOUR_KEY".to_string(),
        "https://api.mainnet-beta.solana.com".to_string(),
    )?;

    // 处理用户查询
    let response = agent.process_message(
        "查询地址 0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb 的 ETH 余额"
    ).await?;
    
    println!("AI Response: {}", response);
    
    Ok(())
}

// 示例 2: 直接使用 QueryTool
async fn example_query_tool() -> Result<(), Box<dyn std::error::Error>> {
    let query_tool = QueryTool::new(
        "https://eth-mainnet.g.alchemy.com/v2/YOUR_KEY".to_string(),
        "https://api.mainnet-beta.solana.com".to_string(),
    );

    // 查询 ETH 余额
    let eth_balance = query_tool.get_eth_balance(
        "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb"
    ).await?;
    println!("ETH Balance: {}", eth_balance);

    // 查询 SOL 余额
    let sol_balance = query_tool.get_sol_balance(
        "DYw8jCTfwHNRJhhmFcbXvVDTqWMEVFBX6ZKUmG5CNSKK"
    ).await?;
    println!("SOL Balance: {}", sol_balance);

    // 查询 ERC20 余额 (USDT)
    let usdt_balance = query_tool.get_erc20_balance(
        "0xdAC17F958D2ee523a2206206994597C13D831ec7", // USDT contract
        "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb"
    ).await?;
    println!("USDT Balance: {}", usdt_balance);

    Ok(())
}

// 示例 3: 构建交易
async fn example_transaction_tool() -> Result<(), Box<dyn std::error::Error>> {
    let tx_tool = TransactionTool::new(
        "https://eth-mainnet.g.alchemy.com/v2/YOUR_KEY".to_string(),
        "https://api.mainnet-beta.solana.com".to_string(),
    );

    // 构建 ETH 转账交易
    let tx_data = tx_tool.build_eth_transaction(
        "0xSenderAddress",
        "0xRecipientAddress",
        0.1  // 0.1 ETH
    ).await?;
    
    println!("Transaction Data: {:?}", tx_data);
    println!("From: {}", tx_data.from);
    println!("To: {}", tx_data.to);
    println!("Value: {}", tx_data.value);
    println!("Gas: {:?}", tx_data.gas);
    println!("Gas Price: {:?}", tx_data.gas_price);
    println!("Nonce: {:?}", tx_data.nonce);

    // 估算 Gas
    let estimated_gas = tx_tool.estimate_gas(&tx_data).await?;
    println!("Estimated Gas: {}", estimated_gas);

    Ok(())
}

// 示例 4: 广播交易（需要已签名的交易）
async fn example_broadcast_tool() -> Result<(), Box<dyn std::error::Error>> {
    let broadcast_tool = BroadcastTool::new(
        "https://eth-mainnet.g.alchemy.com/v2/YOUR_KEY".to_string(),
        "https://api.mainnet-beta.solana.com".to_string(),
    );

    // 广播已签名的交易
    let signed_tx = "0xYourSignedTransactionData";
    let tx_hash = broadcast_tool.broadcast_eth_transaction(signed_tx).await?;
    println!("Transaction Hash: {}", tx_hash);

    // 查询交易状态
    let receipt = broadcast_tool.get_eth_transaction_receipt(&tx_hash).await?;
    println!("Transaction Status: {}", receipt.status);
    println!("Block Number: {:?}", receipt.block_number);

    // 检查是否确认
    let confirmed = broadcast_tool.is_transaction_confirmed(&tx_hash, "eth").await?;
    println!("Confirmed: {}", confirmed);

    Ok(())
}

// 示例 5: 使用不同的 AI Provider
async fn example_different_providers() -> Result<(), Box<dyn std::error::Error>> {
    // DeepSeek
    let deepseek_agent = BlockchainAgent::new(
        "deepseek",
        "sk-xxx".to_string(),
        Some("deepseek-chat".to_string()),
        "eth_rpc".to_string(),
        "sol_rpc".to_string(),
    )?;

    // Qwen
    let qwen_agent = BlockchainAgent::new(
        "qwen",
        "sk-xxx".to_string(),
        Some("qwen-max".to_string()),
        "eth_rpc".to_string(),
        "sol_rpc".to_string(),
    )?;

    // OpenAI
    let openai_agent = BlockchainAgent::new(
        "openai",
        "sk-xxx".to_string(),
        Some("gpt-4".to_string()),
        "eth_rpc".to_string(),
        "sol_rpc".to_string(),
    )?;

    // Claude
    let claude_agent = BlockchainAgent::new(
        "claude",
        "sk-xxx".to_string(),
        Some("claude-3-5-sonnet-20241022".to_string()),
        "eth_rpc".to_string(),
        "sol_rpc".to_string(),
    )?;

    Ok(())
}

// 示例 6: 完整的工作流程
async fn example_complete_workflow() -> Result<(), Box<dyn std::error::Error>> {
    let agent = BlockchainAgent::new(
        "deepseek",
        "your-api-key".to_string(),
        Some("deepseek-chat".to_string()),
        "https://eth-mainnet.g.alchemy.com/v2/YOUR_KEY".to_string(),
        "https://api.mainnet-beta.solana.com".to_string(),
    )?;

    // 1. 查询余额
    let balance_response = agent.process_message(
        "查询我的 ETH 余额，地址是 0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb"
    ).await?;
    println!("Step 1 - Balance: {}", balance_response);

    // 2. 构建交易
    let tx_response = agent.process_message(
        "帮我构建一笔转账交易，从 0xAAA 转 0.1 ETH 到 0xBBB"
    ).await?;
    println!("Step 2 - Transaction: {}", tx_response);

    // 3. 查询交易状态（假设已广播）
    let status_response = agent.process_message(
        "查询交易 0x123abc 的状态"
    ).await?;
    println!("Step 3 - Status: {}", status_response);

    Ok(())
}

// 主函数（仅用于演示，实际不会运行）
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Blockchain Agent Examples ===\n");

    // 运行示例
    // example_blockchain_agent().await?;
    // example_query_tool().await?;
    // example_transaction_tool().await?;
    // example_broadcast_tool().await?;
    // example_different_providers().await?;
    // example_complete_workflow().await?;

    println!("\n所有示例完成！");
    Ok(())
}
