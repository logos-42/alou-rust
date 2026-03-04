//! TUI模式的主函数

use crate::tui::{Tui, TuiApp};

/// 运行TUI界面
pub fn run_tui() -> std::io::Result<()> {
    println!("🚀 启动Alou CLI TUI界面...");
    println!("🤖 Alou CLI v0.2.0 - AI智能体终端界面");
    println!("💬 支持PubSub群聊、任务管理、技能调用");
    
    // 创建TUI
    let mut tui = Tui::new()?;
    
    // 进入TUI模式
    tui.enter()?;
    
    // 创建应用程序
    let mut app = TuiApp::new();
    
    // 主事件循环
    while !app.should_quit {
        // 绘制界面
        tui.draw(&mut app)?;
        
        // 处理事件
        match tui.events.next() {
            Ok(event) => {
                app.handle_event(event);
            }
            Err(_) => {
                break;
            }
        }
    }
    
    // 退出TUI模式
    tui.exit()?;
    
    println!("👋 TUI界面已退出");
    Ok(())
}