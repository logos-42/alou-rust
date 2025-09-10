use alou3_rust::cli::*;
use anyhow::Result;
use clap::Parser;


/// 主函数
#[tokio::main]
async fn main() -> Result<()> {
    // 创建CLI应用程序
    let mut app = CliApp::new()?;

    // 解析命令行参数
    let cli = Cli::parse();

    // 运行应用程序
    app.run(cli).await?;

    Ok(())
}

