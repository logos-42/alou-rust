//! TUI应用程序状态管理

/// 应用程序状态
pub struct AppState {
    /// 当前选中的标签页
    pub current_tab: usize,
    /// 当前选中的项目
    pub selected_item: usize,
    /// 计时器计数
    pub tick_count: u64,
    /// 是否显示帮助
    pub show_help: bool,
    /// 是否显示确认对话框
    pub show_confirm: bool,
    /// 确认对话框消息
    pub confirm_message: String,
    /// 确认回调函数
    pub confirm_callback: Option<Box<dyn FnOnce(bool)>>,
}

impl AppState {
    /// 创建新的应用程序状态
    pub fn new() -> Self {
        Self {
            current_tab: 0,
            selected_item: 0,
            tick_count: 0,
            show_help: false,
            show_confirm: false,
            confirm_message: String::new(),
            confirm_callback: None,
        }
    }
    
    /// 显示确认对话框
    pub fn show_confirm(&mut self, message: String, callback: Box<dyn FnOnce(bool)>) {
        self.show_confirm = true;
        self.confirm_message = message;
        self.confirm_callback = Some(callback);
    }
    
    /// 隐藏确认对话框
    pub fn hide_confirm(&mut self) {
        self.show_confirm = false;
        self.confirm_message.clear();
        self.confirm_callback = None;
    }
    
    /// 切换帮助显示
    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }
    
    /// 获取当前标签页名称
    pub fn current_tab_name(&self) -> &str {
        match self.current_tab {
            0 => "仪表板",
            1 => "任务管理",
            2 => "技能管理",
            3 => "群聊管理",
            4 => "系统设置",
            _ => "未知",
        }
    }
}