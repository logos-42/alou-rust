//! TUI应用程序主逻辑

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs, Clear},
};
use crate::tui::state::AppState;

/// TUI应用程序
pub struct TuiApp {
    pub state: AppState,
    pub should_quit: bool,
}

impl TuiApp {
    /// 创建新的TUI应用程序
    pub fn new() -> Self {
        Self {
            state: AppState::new(),
            should_quit: false,
        }
    }
    
    /// 处理事件
    pub fn handle_event(&mut self, event: crate::tui::events::Event) {
        match event {
            crate::tui::events::Event::Tick => {
                self.state.tick_count += 1;
            }
            crate::tui::events::Event::Key(key_event) => {
                self.handle_key_event(key_event);
            }
            crate::tui::events::Event::Mouse(_) => {
                // 暂时不处理鼠标事件
            }
            crate::tui::events::Event::Resize(_, _) => {
                // 窗口大小改变
            }
        }
    }
    
    /// 处理键盘事件
    fn handle_key_event(&mut self, key_event: crossterm::event::KeyEvent) {
        use crossterm::event::{KeyCode, KeyModifiers};
        
        match (key_event.code, key_event.modifiers) {
            // 退出
            (KeyCode::Char('q'), _) | (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                self.should_quit = true;
            }
            
            // 切换标签页
            (KeyCode::Char('1'), _) => {
                self.state.current_tab = 0;
            }
            (KeyCode::Char('2'), _) => {
                self.state.current_tab = 1;
            }
            (KeyCode::Char('3'), _) => {
                self.state.current_tab = 2;
            }
            (KeyCode::Char('4'), _) => {
                self.state.current_tab = 3;
            }
            (KeyCode::Char('5'), _) => {
                self.state.current_tab = 4;
            }
            
            // 导航
            (KeyCode::Tab, _) => {
                self.state.current_tab = (self.state.current_tab + 1) % 5;
            }
            (KeyCode::BackTab, _) => {
                self.state.current_tab = if self.state.current_tab == 0 {
                    4
                } else {
                    self.state.current_tab - 1
                };
            }
            
            // 上下导航
            (KeyCode::Up, _) => {
                if self.state.selected_item > 0 {
                    self.state.selected_item -= 1;
                }
            }
            (KeyCode::Down, _) => {
                self.state.selected_item += 1;
            }
            
            // 回车选择
            (KeyCode::Enter, _) => {
                self.handle_enter();
            }
            
            _ => {}
        }
    }
    
    /// 处理回车键
    fn handle_enter(&mut self) {
        match self.state.current_tab {
            0 => { // 仪表板
                println!("进入仪表板详情");
            }
            1 => { // 任务管理
                println!("执行任务: {}", self.state.selected_item);
            }
            2 => { // 技能管理
                println!("使用技能: {}", self.state.selected_item);
            }
            3 => { // 群聊
                self.handle_group_chat_action();
            }
            4 => { // 设置
                println!("打开设置");
            }
            _ => {}
        }
    }
    
    /// 处理群聊操作
    fn handle_group_chat_action(&mut self) {
        match self.state.selected_item {
            0 => {
                println!("创建新的PubSub群聊...");
                // 这里应该调用实际的群聊创建功能
            }
            1 => {
                println!("加入现有群聊...");
            }
            2 => {
                println!("发送消息到群聊...");
            }
            _ => {}
        }
    }
    
    /// 绘制界面
    pub fn draw(&mut self, frame: &mut Frame) {
        let size = frame.size();
        
        // 创建布局
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // 标题栏
                Constraint::Length(3),  // 标签页
                Constraint::Min(10),    // 主内容区
                Constraint::Length(3),  // 状态栏
            ])
            .split(size);
        
        // 绘制标题
        self.draw_title(frame, chunks[0]);
        
        // 绘制标签页
        self.draw_tabs(frame, chunks[1]);
        
        // 绘制主内容
        self.draw_main_content(frame, chunks[2]);
        
        // 绘制状态栏
        self.draw_status_bar(frame, chunks[3]);
    }
    
    /// 绘制标题
    fn draw_title(&self, frame: &mut Frame, area: Rect) {
        let title = Paragraph::new("🤖 Alou CLI - AI智能体终端")
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .alignment(ratatui::layout::Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        
        frame.render_widget(title, area);
    }
    
    /// 绘制标签页
    fn draw_tabs(&self, frame: &mut Frame, area: Rect) {
        let titles = vec!["📊 仪表板", "📋 任务", "🛠️ 技能", "💬 群聊", "⚙️ 设置"];
        let tabs = Tabs::new(titles)
            .select(self.state.current_tab)
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            .divider("│");
        
        frame.render_widget(tabs, area);
    }
    
    /// 绘制主内容
    fn draw_main_content(&self, frame: &mut Frame, area: Rect) {
        match self.state.current_tab {
            0 => self.draw_dashboard(frame, area),
            1 => self.draw_tasks(frame, area),
            2 => self.draw_skills(frame, area),
            3 => self.draw_group_chat(frame, area),
            4 => self.draw_settings(frame, area),
            _ => {}
        }
    }
    
    /// 绘制仪表板
    fn draw_dashboard(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .title("📊 系统状态")
            .borders(Borders::ALL);
        
        let content = vec![
            Line::from(vec![
                Span::styled("状态: ", Style::default().fg(Color::Gray)),
                Span::styled("运行中", Style::default().fg(Color::Green)),
            ]),
            Line::from(vec![
                Span::styled("智能体: ", Style::default().fg(Color::Gray)),
                Span::styled("3个活跃", Style::default().fg(Color::Blue)),
            ]),
            Line::from(vec![
                Span::styled("任务: ", Style::default().fg(Color::Gray)),
                Span::styled("5个进行中", Style::default().fg(Color::Yellow)),
            ]),
            Line::from(vec![
                Span::styled("群聊: ", Style::default().fg(Color::Gray)),
                Span::styled("2个活跃", Style::default().fg(Color::Magenta)),
            ]),
            Line::from(""),
            Line::from("按 1-5 切换标签页，q 退出，↑↓ 导航，↲ 选择"),
        ];
        
        let paragraph = Paragraph::new(content)
            .block(block)
            .wrap(ratatui::widgets::Wrap { trim: true });
        
        frame.render_widget(paragraph, area);
    }
    
    /// 绘制任务管理
    fn draw_tasks(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .title("📋 任务管理")
            .borders(Borders::ALL);
        
        let tasks = vec![
            "1. 检查系统日志",
            "2. 备份数据库",
            "3. 更新依赖包",
            "4. 运行测试套件",
            "5. 部署到生产环境",
        ];
        
        let mut content = Vec::new();
        for (i, task) in tasks.iter().enumerate() {
            let style = if i == self.state.selected_item % tasks.len() {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            content.push(Line::from(Span::styled(*task, style)));
        }
        
        let paragraph = Paragraph::new(content)
            .block(block)
            .wrap(ratatui::widgets::Wrap { trim: true });
        
        frame.render_widget(paragraph, area);
    }
    
    /// 绘制技能管理
    fn draw_skills(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .title("🛠️ 可用技能")
            .borders(Borders::ALL);
        
        let skills = vec![
            "• 文件系统操作",
            "• 终端命令执行",
            "• 网络请求",
            "• 代码分析",
            "• 数据库查询",
            "• 系统监控",
            "• 群聊创建",
            "• 任务调度",
        ];
        
        let mut content = Vec::new();
        for (i, skill) in skills.iter().enumerate() {
            let style = if i == self.state.selected_item % skills.len() {
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            content.push(Line::from(Span::styled(*skill, style)));
        }
        
        let paragraph = Paragraph::new(content)
            .block(block)
            .wrap(ratatui::widgets::Wrap { trim: true });
        
        frame.render_widget(paragraph, area);
    }
    
    /// 绘制群聊管理
    fn draw_group_chat(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .title("💬 PubSub群聊 (基于IPFS)")
            .borders(Borders::ALL);
        
        let actions = vec![
            "1. 创建新群聊",
            "2. 加入现有群聊",
            "3. 发送消息",
            "4. 查看群聊列表",
            "5. AI自主创建群聊",
        ];
        
        let mut content = Vec::new();
        content.push(Line::from(Span::styled("PubSub群聊功能:", Style::default().fg(Color::Magenta))));
        content.push(Line::from(""));
        
        for (i, action) in actions.iter().enumerate() {
            let style = if i == self.state.selected_item % actions.len() {
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            content.push(Line::from(Span::styled(*action, style)));
        }
        
        content.push(Line::from(""));
        content.push(Line::from(Span::styled("💡 AI可以自主创建群聊用于多智能体协作", Style::default().fg(Color::Yellow))));
        
        let paragraph = Paragraph::new(content)
            .block(block)
            .wrap(ratatui::widgets::Wrap { trim: true });
        
        frame.render_widget(paragraph, area);
    }
    
    /// 绘制设置
    fn draw_settings(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .title("⚙️ 系统设置")
            .borders(Borders::ALL);
        
        let settings = vec![
            "• API配置",
            "• 网络设置",
            "• 日志级别",
            "• 主题设置",
            "• 快捷键配置",
            "• 自动更新",
        ];
        
        let mut content = Vec::new();
        for (i, setting) in settings.iter().enumerate() {
            let style = if i == self.state.selected_item % settings.len() {
                Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            content.push(Line::from(Span::styled(*setting, style)));
        }
        
        let paragraph = Paragraph::new(content)
            .block(block)
            .wrap(ratatui::widgets::Wrap { trim: true });
        
        frame.render_widget(paragraph, area);
    }
    
    /// 绘制状态栏
    fn draw_status_bar(&self, frame: &mut Frame, area: Rect) {
        let status = match self.state.current_tab {
            0 => "仪表板 | 按q退出",
            1 => "任务管理 | 选择任务后按回车执行",
            2 => "技能管理 | 选择技能后按回车使用",
            3 => "群聊管理 | AI可以自主创建群聊",
            4 => "系统设置 | 配置Alou CLI",
            _ => "就绪",
        };
        
        let status_bar = Paragraph::new(status)
            .style(Style::default().fg(Color::Gray))
            .alignment(ratatui::layout::Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        
        frame.render_widget(status_bar, area);
    }
}