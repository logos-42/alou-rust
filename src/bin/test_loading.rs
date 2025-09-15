use std::time::Duration;
use tokio;

/// 简单的加载动画测试
async fn show_loading_animation(message: &str) {
    let spinner_chars = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    let mut index = 0;
    
    print!("\r{} {}", spinner_chars[index], message);
    std::io::Write::flush(&mut std::io::stdout()).unwrap();
    
    loop {
        tokio::time::sleep(Duration::from_millis(100)).await;
        index = (index + 1) % spinner_chars.len();
        print!("\r{} {}", spinner_chars[index], message);
        std::io::Write::flush(&mut std::io::stdout()).unwrap();
    }
}

#[tokio::main]
async fn main() {
    println!("测试加载动画功能...");
    println!("按 Ctrl+C 停止测试");
    
    // 启动加载动画
    let loading_handle = tokio::spawn(async {
        show_loading_animation("🤔 正在思考，请稍候...").await;
    });
    
    // 模拟API调用延迟
    tokio::time::sleep(Duration::from_secs(5)).await;
    
    // 停止加载动画
    loading_handle.abort();
    
    // 清除加载动画行
    print!("\r{}", " ".repeat(50));
    print!("\r");
    std::io::Write::flush(&mut std::io::stdout()).unwrap();
    
    println!("思考完成！");
}
