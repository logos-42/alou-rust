//! TUI (终端用户界面) 模块
//! 为Alou CLI提供图形化的终端界面

mod app;
mod components;
mod events;
mod state;
mod utils;

pub use app::TuiApp;
pub use events::{Event, EventHandler};
pub use state::AppState;

use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::io;

/// TUI应用程序
pub struct Tui {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    pub events: EventHandler,
}

impl Tui {
    /// 创建新的TUI实例
    pub fn new() -> io::Result<Self> {
        let backend = CrosstermBackend::new(io::stdout());
        let terminal = Terminal::new(backend)?;
        let events = EventHandler::new(250); // 250ms tick rate
        
        Ok(Self { terminal, events })
    }
    
    /// 进入TUI模式
    pub fn enter(&mut self) -> io::Result<()> {
        crossterm::terminal::enable_raw_mode()?;
        crossterm::execute!(
            io::stdout(),
            crossterm::terminal::EnterAlternateScreen,
            crossterm::event::EnableMouseCapture
        )?;
        
        self.terminal.hide_cursor()?;
        self.terminal.clear()?;
        
        Ok(())
    }
    
    /// 退出TUI模式
    pub fn exit(&mut self) -> io::Result<()> {
        crossterm::terminal::disable_raw_mode()?;
        crossterm::execute!(
            io::stdout(),
            crossterm::terminal::LeaveAlternateScreen,
            crossterm::event::DisableMouseCapture
        )?;
        
        self.terminal.show_cursor()?;
        
        Ok(())
    }
    
    /// 绘制界面
    pub fn draw(&mut self, app: &mut TuiApp) -> io::Result<()> {
        self.terminal.draw(|frame| app.draw(frame))?;
        Ok(())
    }
}