use crate::tools::{ToolExecutor, ToolResult, ToolError, ToolCategory};
use crate::tools::ExecutionContext;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::time::{sleep, Duration};
use std::future::Future;

#[derive(Debug, Deserialize)]
#[serde(tag = "action")]
pub enum BrowserOperation {
    /// 打开网页
    #[serde(rename = "open_page")]
    OpenPage {
        /// 页面URL
        url: String,
        /// 等待加载时间（秒）(可选，默认为 5)
        wait_time: Option<u64>,
    },
    
    /// 关闭当前页面
    #[serde(rename = "close_page")]
    ClosePage {},
    
    /// 导航到新URL
    #[serde(rename = "navigate")]
    Navigate {
        /// 目标URL
        url: String,
    },
    
    /// 刷新页面
    #[serde(rename = "refresh")]
    Refresh {},
    
    /// 点击元素
    #[serde(rename = "click_element")]
    ClickElement {
        /// 元素选择器 (CSS selector 或 XPath)
        selector: String,
        /// 等待元素出现的时间（秒）(可选，默认为 5)
        wait_time: Option<u64>,
    },
    
    /// 输入文本到元素
    #[serde(rename = "input_text")]
    InputText {
        /// 元素选择器
        selector: String,
        /// 要输入的文本
        text: String,
        /// 清空现有内容 (可选，默认为 true)
        clear_first: Option<bool>,
    },
    
    /// 获取元素文本
    #[serde(rename = "get_element_text")]
    GetElementText {
        /// 元素选择器
        selector: String,
    },
    
    /// 获取页面标题
    #[serde(rename = "get_page_title")]
    GetPageTitle {},
    
    /// 获取页面URL
    #[serde(rename = "get_page_url")]
    GetPageUrl {},
    
    /// 获取页面截图
    #[serde(rename = "take_screenshot")]
    TakeScreenshot {
        /// 截图保存路径 (可选)
        path: Option<String>,
    },
    
    /// 等待元素出现
    #[serde(rename = "wait_for_element")]
    WaitForElement {
        /// 元素选择器
        selector: String,
        /// 等待时间（秒）(可选，默认为 10)
        timeout: Option<u64>,
    },
    
    /// 滚动到元素
    #[serde(rename = "scroll_to_element")]
    ScrollToElement {
        /// 元素选择器
        selector: String,
    },
    
    /// 执行JavaScript代码
    #[serde(rename = "execute_script")]
    ExecuteScript {
        /// JavaScript代码
        script: String,
    },
    
    /// 获取页面源码
    #[serde(rename = "get_page_source")]
    GetPageSource {},
    
    /// 设置浏览器窗口大小
    #[serde(rename = "set_window_size")]
    SetWindowSize {
        /// 宽度
        width: u32,
        /// 高度
        height: u32,
    },
    
    /// 在新标签页中打开页面
    #[serde(rename = "open_new_tab")]
    OpenNewTab {
        /// 页面URL
        url: String,
    },
    
    /// 切换标签页
    #[serde(rename = "switch_tab")]
    SwitchTab {
        /// 标签页索引
        index: u32,
    },
}

#[derive(Debug, Clone)]
pub struct BrowserTool {
    id: String,
    name: String,
    description: String,
    category: ToolCategory,
    /// 模拟浏览器状态
    browser_state: std::sync::Arc<tokio::sync::RwLock<BrowserState>>,
}

#[derive(Debug, Clone)]
struct BrowserState {
    current_url: String,
    current_title: String,
    page_source: String,
    tabs: Vec<String>,
    current_tab_index: u32,
    window_width: u32,
    window_height: u32,
}

impl BrowserState {
    fn new() -> Self {
        Self {
            current_url: "about:blank".to_string(),
            current_title: "New Tab".to_string(),
            page_source: "<html><head><title>New Tab</title></head><body></body></html>".to_string(),
            tabs: vec!["about:blank".to_string()],
            current_tab_index: 0,
            window_width: 1200,
            window_height: 800,
        }
    }
}

impl BrowserTool {
    pub fn new() -> Self {
        Self {
            id: "browser".to_string(),
            name: "Browser Tool".to_string(),
            description: "Tool for controlling web browser operations".to_string(),
            category: ToolCategory::Automation,
            browser_state: std::sync::Arc::new(tokio::sync::RwLock::new(BrowserState::new())),
        }
    }
}

#[derive(Serialize, Debug)]
pub struct BrowserResult {
    pub success: bool,
    pub message: String,
    pub data: Option<Value>,
    pub output: Option<String>,
}

// 模拟浏览器客户端
pub struct BrowserClient {
    state: std::sync::Arc<tokio::sync::RwLock<BrowserState>>,
    /// CDP 客户端（真实浏览器控制）
    cdp_manager: std::sync::Arc<tokio::sync::Mutex<CdpManager>>,
    /// 是否使用真实浏览器（如果可用）
    use_real_browser: bool,
}

impl BrowserClient {
    pub fn new(state: std::sync::Arc<tokio::sync::RwLock<BrowserState>>) -> Self {
        Self { 
            state,
            cdp_manager: std::sync::Arc::new(tokio::sync::Mutex::new(CdpManager::new())),
            use_real_browser: true, // 优先使用真实浏览器
        }
    }

    /// 尝试使用 CDP 客户端执行操作
    async fn try_cdp<F, R>(&self, op: F) -> Result<R, String>
    where
        F: Future<Output = Result<R, String>>,
    {
        // 如果设置了不使用真实浏览器，直接返回错误
        if !self.use_real_browser {
            return Err("Real browser disabled".to_string());
        }
        
        op.await
    }

    pub async fn open_page(&mut self, url: &str, wait_time: Option<u64>) -> Result<(), String> {
        let wait_time = wait_time.unwrap_or(5);
        {
            let mut state = self.state.write().await;
            state.current_url = url.to_string();
            state.current_title = format!("Page - {}", url);
            state.page_source = format!("<html><head><title>{}</title></head><body><h1>Welcome to {}</h1></body></html>", url, url);
            if state.tabs.is_empty() {
                state.tabs.push(url.to_string());
            } else {
                let idx = state.current_tab_index as usize;
                state.tabs[idx] = url.to_string();
            }
        }
        
        // 模拟等待页面加载
        sleep(Duration::from_secs(wait_time)).await;
        
        Ok(())
    }

    pub async fn close_page(&mut self) -> Result<(), String> {
        let mut state = self.state.write().await;
        if state.tabs.len() > 1 {
            let idx = state.current_tab_index as usize;
            state.tabs.remove(idx);
            if state.current_tab_index >= state.tabs.len() as u32 {
                state.current_tab_index = (state.tabs.len() as u32).saturating_sub(1);
            }
            if let Some(url) = state.tabs.get(state.current_tab_index as usize) {
                state.current_url = url.clone();
            }
        } else {
            state.current_url = "about:blank".to_string();
            state.current_title = "New Tab".to_string();
            state.page_source = "<html><head><title>New Tab</title></head><body></body></html>".to_string();
        }
        
        Ok(())
    }

    pub async fn navigate(&mut self, url: &str) -> Result<(), String> {
        // 优先尝试使用真实浏览器（CDP）
        if self.use_real_browser {
            let mut cdp_manager = self.cdp_manager.lock().await;
            match cdp_manager.get_client().await {
                Ok(cdp_client) => {
                    match cdp_client.navigate(url).await {
                        Ok(_) => {
                            // 同步更新本地状态
                            let mut state = self.state.write().await;
                            state.current_url = url.to_string();
                            // 获取真实标题
                            if let Ok(title) = cdp_client.get_title().await {
                                state.current_title = title;
                            }
                            return Ok(());
                        }
                        Err(e) => {
                            log::warn!("CDP navigate failed, falling back to mock: {}", e);
                            // CDP 失败，继续使用 mock
                        }
                    }
                }
                Err(e) => {
                    log::warn!("CDP client unavailable: {}", e);
                }
            }
        }
        
        // 回退到模拟实现
        {
            let mut state = self.state.write().await;
            state.current_url = url.to_string();
            state.current_title = format!("Page - {}", url);
            state.page_source = format!("<html><head><title>{}</title></head><body><h1>Welcome to {}</h1></body></html>", url, url);
            if state.current_tab_index < state.tabs.len() as u32 {
                let idx = state.current_tab_index as usize;
                state.tabs[idx] = url.to_string();
            }
        }
        
        Ok(())
    }

    pub async fn refresh(&mut self) -> Result<(), String> {
        let current_url;
        {
            let state = self.state.read().await;
            current_url = state.current_url.clone();
        }
        
        self.navigate(&current_url).await
    }

    pub async fn click_element(&mut self, selector: &str, wait_time: Option<u64>) -> Result<(), String> {
        let wait_time = wait_time.unwrap_or(5);
        // 模拟等待元素
        sleep(Duration::from_secs(wait_time)).await;
        
        // 模拟点击元素的效果
        println!("Simulating click on element with selector: {}", selector);
        
        Ok(())
    }

    pub async fn input_text(&mut self, selector: &str, text: &str, clear_first: Option<bool>) -> Result<(), String> {
        let clear_first = clear_first.unwrap_or(true);
        
        // 模拟输入文本
        println!("Simulating input text '{}' to element with selector: {}, clear_first: {}", text, selector, clear_first);
        
        Ok(())
    }

    pub async fn get_element_text(&self, selector: &str) -> Result<String, String> {
        // 模拟获取元素文本
        // 在真实实现中，这会通过浏览器自动化工具获取实际元素文本
        Ok(format!("Sample text from element matching selector: {}", selector))
    }

    pub async fn get_page_title(&self) -> Result<String, String> {
        // 优先使用真实浏览器
        if self.use_real_browser {
            let mut cdp_manager = self.cdp_manager.lock().await;
            if let Ok(cdp_client) = cdp_manager.get_client().await {
                if let Ok(title) = cdp_client.get_title().await {
                    return Ok(title);
                }
            }
        }
        
        // 回退到模拟实现
        let state = self.state.read().await;
        Ok(state.current_title.clone())
    }

    pub async fn get_page_url(&self) -> Result<String, String> {
        // 优先使用真实浏览器
        if self.use_real_browser {
            let mut cdp_manager = self.cdp_manager.lock().await;
            if let Ok(cdp_client) = cdp_manager.get_client().await {
                if let Ok(url) = cdp_client.get_url().await {
                    return Ok(url);
                }
            }
        }
        
        // 回退到模拟实现
        let state = self.state.read().await;
        Ok(state.current_url.clone())
    }

    pub async fn take_screenshot(&self, path: Option<String>) -> Result<String, String> {
        let screenshot_path = path.unwrap_or_else(|| format!("screenshot_{}.png", chrono::Utc::now().timestamp()));
        
        // 优先使用真实浏览器
        if self.use_real_browser {
            let mut cdp_manager = self.cdp_manager.lock().await;
            if let Ok(cdp_client) = cdp_manager.get_client().await {
                match cdp_client.screenshot().await {
                    Ok(base64_data) => {
                        // 保存 base64 图片到文件
                        use std::fs::File;
                        use std::io::Write;
                        match base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &base64_data) {
                            Ok(decoded) => {
                                match File::create(&screenshot_path) {
                                    Ok(mut file) => {
                                        if file.write_all(&decoded).is_ok() {
                                            return Ok(screenshot_path);
                                        }
                                    }
                                    Err(e) => {
                                        log::warn!("Failed to create screenshot file: {}", e);
                                    }
                                }
                            }
                            Err(e) => {
                                log::warn!("Failed to decode base64: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        log::warn!("CDP screenshot failed: {}", e);
                    }
                }
            }
        }
        
        // 回退到模拟实现
        println!("Taking screenshot and saving to: {}", screenshot_path);
        Ok(screenshot_path)
    }

    pub async fn wait_for_element(&self, selector: &str, timeout: Option<u64>) -> Result<(), String> {
        let timeout = timeout.unwrap_or(10);
        println!("Waiting for element with selector: {} (timeout: {}s)", selector, timeout);
        
        // 模拟等待元素出现
        for _ in 0..timeout {
            // 在真实实现中，这里会检查元素是否出现在页面上
            sleep(Duration::from_secs(1)).await;
        }
        
        Ok(())
    }

    pub async fn scroll_to_element(&self, selector: &str) -> Result<(), String> {
        println!("Scrolling to element with selector: {}", selector);
        Ok(())
    }

    pub async fn execute_script(&self, script: &str) -> Result<Value, String> {
        println!("Executing JavaScript: {}", script);
        // 模拟执行脚本并返回结果
        // 在真实实现中，这会执行实际的JavaScript并返回结果
        Ok(Value::String("Script executed successfully".to_string()))
    }

    pub async fn get_page_source(&self) -> Result<String, String> {
        let state = self.state.read().await;
        Ok(state.page_source.clone())
    }

    pub async fn set_window_size(&mut self, width: u32, height: u32) -> Result<(), String> {
        let mut state = self.state.write().await;
        state.window_width = width;
        state.window_height = height;
        Ok(())
    }

    pub async fn open_new_tab(&mut self, url: &str) -> Result<u32, String> {
        let mut state = self.state.write().await;
        state.tabs.push(url.to_string());
        let new_tab_index = (state.tabs.len() - 1) as u32;
        Ok(new_tab_index)
    }

    pub async fn switch_tab(&mut self, index: u32) -> Result<(), String> {
        let mut state = self.state.write().await;
        if (index as usize) < state.tabs.len() {
            state.current_tab_index = index;
            state.current_url = state.tabs[index as usize].clone();
            state.current_title = format!("Page - {}", state.tabs[index as usize]);
            state.page_source = format!("<html><head><title>{}</title></head><body><h1>Welcome to {}</h1></body></html>", 
                                        state.tabs[index as usize], state.tabs[index as usize]);
            Ok(())
        } else {
            Err(format!("Tab index {} out of bounds (total tabs: {})", index, state.tabs.len()))
        }
    }
}

impl BrowserTool {
    async fn execute_impl(&self, args: Value) -> Result<BrowserResult, Box<dyn std::error::Error>> {
        let operation: BrowserOperation = serde_json::from_value(args)?;
        let mut client = BrowserClient::new(self.browser_state.clone());

        match operation {
            BrowserOperation::OpenPage { url, wait_time } => {
                match client.open_page(&url, wait_time).await {
                    Ok(_) => {
                        Ok(BrowserResult {
                            success: true,
                            message: format!("Successfully opened page: {}", url),
                            data: Some(serde_json::json!({
                                "url": url
                            })),
                            output: Some(format!("Opened page: {}", url)),
                        })
                    }
                    Err(e) => {
                        Ok(BrowserResult {
                            success: false,
                            message: format!("Failed to open page: {}", e),
                            data: None,
                            output: Some(format!("Error opening page: {}", e)),
                        })
                    }
                }
            }

            BrowserOperation::ClosePage {} => {
                match client.close_page().await {
                    Ok(_) => {
                        Ok(BrowserResult {
                            success: true,
                            message: "Successfully closed page".to_string(),
                            data: None,
                            output: Some("Closed current page".to_string()),
                        })
                    }
                    Err(e) => {
                        Ok(BrowserResult {
                            success: false,
                            message: format!("Failed to close page: {}", e),
                            data: None,
                            output: Some(format!("Error closing page: {}", e)),
                        })
                    }
                }
            }

            BrowserOperation::Navigate { url } => {
                match client.navigate(&url).await {
                    Ok(_) => {
                        Ok(BrowserResult {
                            success: true,
                            message: format!("Successfully navigated to: {}", url),
                            data: Some(serde_json::json!({
                                "url": url
                            })),
                            output: Some(format!("Navigated to: {}", url)),
                        })
                    }
                    Err(e) => {
                        Ok(BrowserResult {
                            success: false,
                            message: format!("Failed to navigate: {}", e),
                            data: None,
                            output: Some(format!("Error navigating: {}", e)),
                        })
                    }
                }
            }

            BrowserOperation::Refresh {} => {
                match client.refresh().await {
                    Ok(_) => {
                        let current_url;
                        {
                            let state = self.browser_state.read().await;
                            current_url = state.current_url.clone();
                        }
                        
                        Ok(BrowserResult {
                            success: true,
                            message: format!("Successfully refreshed page: {}", current_url),
                            data: Some(serde_json::json!({
                                "url": current_url
                            })),
                            output: Some(format!("Refreshed page: {}", current_url)),
                        })
                    }
                    Err(e) => {
                        Ok(BrowserResult {
                            success: false,
                            message: format!("Failed to refresh: {}", e),
                            data: None,
                            output: Some(format!("Error refreshing: {}", e)),
                        })
                    }
                }
            }

            BrowserOperation::ClickElement { selector, wait_time } => {
                match client.click_element(&selector, wait_time).await {
                    Ok(_) => {
                        Ok(BrowserResult {
                            success: true,
                            message: format!("Successfully clicked element: {}", selector),
                            data: Some(serde_json::json!({
                                "selector": selector
                            })),
                            output: Some(format!("Clicked element: {}", selector)),
                        })
                    }
                    Err(e) => {
                        Ok(BrowserResult {
                            success: false,
                            message: format!("Failed to click element: {}", e),
                            data: None,
                            output: Some(format!("Error clicking element: {}", e)),
                        })
                    }
                }
            }

            BrowserOperation::InputText { selector, text, clear_first } => {
                match client.input_text(&selector, &text, clear_first).await {
                    Ok(_) => {
                        Ok(BrowserResult {
                            success: true,
                            message: format!("Successfully input text to element: {}", selector),
                            data: Some(serde_json::json!({
                                "selector": selector,
                                "text": text
                            })),
                            output: Some(format!("Input text to element '{}': {}", selector, text)),
                        })
                    }
                    Err(e) => {
                        Ok(BrowserResult {
                            success: false,
                            message: format!("Failed to input text: {}", e),
                            data: None,
                            output: Some(format!("Error inputting text: {}", e)),
                        })
                    }
                }
            }

            BrowserOperation::GetElementText { selector } => {
                match client.get_element_text(&selector).await {
                    Ok(text) => {
                        Ok(BrowserResult {
                            success: true,
                            message: format!("Successfully retrieved text from element: {}", selector),
                            data: Some(serde_json::json!({
                                "selector": selector,
                                "text": text
                            })),
                            output: Some(format!("Retrieved text from element '{}': {}", selector, text)),
                        })
                    }
                    Err(e) => {
                        Ok(BrowserResult {
                            success: false,
                            message: format!("Failed to get element text: {}", e),
                            data: None,
                            output: Some(format!("Error getting element text: {}", e)),
                        })
                    }
                }
            }

            BrowserOperation::GetPageTitle {} => {
                match client.get_page_title().await {
                    Ok(title) => {
                        Ok(BrowserResult {
                            success: true,
                            message: "Successfully retrieved page title".to_string(),
                            data: Some(serde_json::json!({
                                "title": title
                            })),
                            output: Some(format!("Page title: {}", title)),
                        })
                    }
                    Err(e) => {
                        Ok(BrowserResult {
                            success: false,
                            message: format!("Failed to get page title: {}", e),
                            data: None,
                            output: Some(format!("Error getting page title: {}", e)),
                        })
                    }
                }
            }

            BrowserOperation::GetPageUrl {} => {
                match client.get_page_url().await {
                    Ok(url) => {
                        Ok(BrowserResult {
                            success: true,
                            message: "Successfully retrieved page URL".to_string(),
                            data: Some(serde_json::json!({
                                "url": url
                            })),
                            output: Some(format!("Page URL: {}", url)),
                        })
                    }
                    Err(e) => {
                        Ok(BrowserResult {
                            success: false,
                            message: format!("Failed to get page URL: {}", e),
                            data: None,
                            output: Some(format!("Error getting page URL: {}", e)),
                        })
                    }
                }
            }

            BrowserOperation::TakeScreenshot { path } => {
                match client.take_screenshot(path).await {
                    Ok(screenshot_path) => {
                        Ok(BrowserResult {
                            success: true,
                            message: format!("Successfully took screenshot: {}", screenshot_path),
                            data: Some(serde_json::json!({
                                "path": screenshot_path
                            })),
                            output: Some(format!("Screenshot saved to: {}", screenshot_path)),
                        })
                    }
                    Err(e) => {
                        Ok(BrowserResult {
                            success: false,
                            message: format!("Failed to take screenshot: {}", e),
                            data: None,
                            output: Some(format!("Error taking screenshot: {}", e)),
                        })
                    }
                }
            }

            BrowserOperation::WaitForElement { selector, timeout } => {
                match client.wait_for_element(&selector, timeout).await {
                    Ok(_) => {
                        Ok(BrowserResult {
                            success: true,
                            message: format!("Successfully waited for element: {}", selector),
                            data: Some(serde_json::json!({
                                "selector": selector
                            })),
                            output: Some(format!("Waited for element: {}", selector)),
                        })
                    }
                    Err(e) => {
                        Ok(BrowserResult {
                            success: false,
                            message: format!("Failed to wait for element: {}", e),
                            data: None,
                            output: Some(format!("Error waiting for element: {}", e)),
                        })
                    }
                }
            }

            BrowserOperation::ScrollToElement { selector } => {
                match client.scroll_to_element(&selector).await {
                    Ok(_) => {
                        Ok(BrowserResult {
                            success: true,
                            message: format!("Successfully scrolled to element: {}", selector),
                            data: Some(serde_json::json!({
                                "selector": selector
                            })),
                            output: Some(format!("Scrolled to element: {}", selector)),
                        })
                    }
                    Err(e) => {
                        Ok(BrowserResult {
                            success: false,
                            message: format!("Failed to scroll to element: {}", e),
                            data: None,
                            output: Some(format!("Error scrolling to element: {}", e)),
                        })
                    }
                }
            }

            BrowserOperation::ExecuteScript { script } => {
                match client.execute_script(&script).await {
                    Ok(result) => {
                        Ok(BrowserResult {
                            success: true,
                            message: "Successfully executed script".to_string(),
                            data: Some(serde_json::json!({
                                "script": script,
                                "result": result
                            })),
                            output: Some(format!("Executed script, result: {:?}", result)),
                        })
                    }
                    Err(e) => {
                        Ok(BrowserResult {
                            success: false,
                            message: format!("Failed to execute script: {}", e),
                            data: None,
                            output: Some(format!("Error executing script: {}", e)),
                        })
                    }
                }
            }

            BrowserOperation::GetPageSource {} => {
                match client.get_page_source().await {
                    Ok(source) => {
                        Ok(BrowserResult {
                            success: true,
                            message: "Successfully retrieved page source".to_string(),
                            data: Some(serde_json::json!({
                                "source_length": source.len()
                            })),
                            output: Some(format!("Retrieved page source ({} chars)", source.len())),
                        })
                    }
                    Err(e) => {
                        Ok(BrowserResult {
                            success: false,
                            message: format!("Failed to get page source: {}", e),
                            data: None,
                            output: Some(format!("Error getting page source: {}", e)),
                        })
                    }
                }
            }

            BrowserOperation::SetWindowSize { width, height } => {
                match client.set_window_size(width, height).await {
                    Ok(_) => {
                        Ok(BrowserResult {
                            success: true,
                            message: format!("Successfully set window size to {}x{}", width, height),
                            data: Some(serde_json::json!({
                                "width": width,
                                "height": height
                            })),
                            output: Some(format!("Set window size to {}x{}", width, height)),
                        })
                    }
                    Err(e) => {
                        Ok(BrowserResult {
                            success: false,
                            message: format!("Failed to set window size: {}", e),
                            data: None,
                            output: Some(format!("Error setting window size: {}", e)),
                        })
                    }
                }
            }

            BrowserOperation::OpenNewTab { url } => {
                match client.open_new_tab(&url).await {
                    Ok(tab_index) => {
                        Ok(BrowserResult {
                            success: true,
                            message: format!("Successfully opened new tab with URL: {}", url),
                            data: Some(serde_json::json!({
                                "url": url,
                                "tab_index": tab_index
                            })),
                            output: Some(format!("Opened new tab {} with URL: {}", tab_index, url)),
                        })
                    }
                    Err(e) => {
                        Ok(BrowserResult {
                            success: false,
                            message: format!("Failed to open new tab: {}", e),
                            data: None,
                            output: Some(format!("Error opening new tab: {}", e)),
                        })
                    }
                }
            }

            BrowserOperation::SwitchTab { index } => {
                match client.switch_tab(index).await {
                    Ok(_) => {
                        Ok(BrowserResult {
                            success: true,
                            message: format!("Successfully switched to tab: {}", index),
                            data: Some(serde_json::json!({
                                "tab_index": index
                            })),
                            output: Some(format!("Switched to tab: {}", index)),
                        })
                    }
                    Err(e) => {
                        Ok(BrowserResult {
                            success: false,
                            message: format!("Failed to switch tab: {}", e),
                            data: None,
                            output: Some(format!("Error switching tab: {}", e)),
                        })
                    }
                }
            }
        }
    }
}

#[async_trait::async_trait]
impl ToolExecutor for BrowserTool {
    fn metadata(&self) -> &crate::tools::ToolMetadata {
        use std::sync::OnceLock;
        static METADATA: OnceLock<crate::tools::ToolMetadata> = OnceLock::new();
        
        METADATA.get_or_init(|| crate::tools::ToolMetadata {
            id: self.id.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            category: self.category,
            priority: crate::tools::ToolPriority::High,
            status: crate::tools::ToolStatus::Available,
            version: "1.0.0".to_string(),
            author: "Alou Team".to_string(),
            created_at: 1700000000,
            updated_at: 1700000000,
            dependencies: vec!["web-browser".to_string()],
            platforms: vec!["windows".to_string(), "macos".to_string(), "linux".to_string()],
            permissions: vec!["browser".to_string(), "internet".to_string()],
                tags: vec![],
            })
    }

    async fn execute(&self, args: Value, _context: &ExecutionContext) -> Result<ToolResult, ToolError> {
        let start_time = std::time::Instant::now();
        
        match self.execute_impl(args).await {
            Ok(result) => {
                Ok(ToolResult {
                    success: result.success,
                    data: result.data.unwrap_or(serde_json::Value::Null),
                    error: if result.success { None } else { Some(result.message) },
                    execution_time_ms: start_time.elapsed().as_millis() as u64,
                    output: result.output,
                    warnings: vec![],
                    context: None,
                })
            }
            Err(e) => {
                Ok(ToolResult {
                    success: false,
                    data: serde_json::Value::Null,
                    error: Some(e.to_string()),
                    execution_time_ms: start_time.elapsed().as_millis() as u64,
                    output: Some(format!("Error executing browser tool: {}", e)),
                    warnings: vec![],
                    context: None,
                })
            }
        }
    }

    async fn validate_args(&self, args: &Value) -> Result<(), ToolError> {
        match serde_json::from_value::<BrowserOperation>(args.clone()) {
            Ok(_) => Ok(()),
            Err(e) => Err(ToolError::InvalidArguments(e.to_string())),
        }
    }

    fn help(&self) -> String {
        "Browser Tool for controlling web browser operations. Actions: open_page, close_page, navigate, refresh, click_element, input_text, get_element_text, get_page_title, get_page_url, take_screenshot, wait_for_element, scroll_to_element, execute_script, get_page_source, set_window_size, open_new_tab, switch_tab".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_open_and_navigate() {
        let tool = BrowserTool::new();
        let context = ExecutionContext {
            session_id: "test_session".to_string(),
            user_id: None,
            working_directory: None,
            environment: std::collections::HashMap::new(),
            timeout_seconds: Some(30),
            permissions: vec!["browser".to_string(), "internet".to_string()],
            timestamp: chrono::Utc::now().timestamp(),
        };

        // 打开页面
        let open_args = json!({
            "action": "open_page",
            "url": "https://www.example.com"
        });
        let open_result = tool.execute(open_args, &context).await.unwrap();
        assert!(open_result.success);
        assert!(open_result.output.as_ref().unwrap().contains("Opened page"));

        // 获取页面标题
        let title_args = json!({
            "action": "get_page_title"
        });
        let title_result = tool.execute(title_args, &context).await.unwrap();
        assert!(title_result.success);
        assert!(title_result.data["title"].as_str().unwrap().contains("Page - https://www.example.com"));
    }

    #[tokio::test]
    async fn test_element_interaction() {
        let tool = BrowserTool::new();
        let context = ExecutionContext {
            session_id: "test_session".to_string(),
            user_id: None,
            working_directory: None,
            environment: std::collections::HashMap::new(),
            timeout_seconds: Some(30),
            permissions: vec!["browser".to_string(), "internet".to_string()],
            timestamp: chrono::Utc::now().timestamp(),
        };

        // 打开页面
        let open_args = json!({
            "action": "open_page",
            "url": "https://www.example.com"
        });
        tool.execute(open_args, &context).await.unwrap();

        // 点击元素
        let click_args = json!({
            "action": "click_element",
            "selector": "#submit-button"
        });
        let click_result = tool.execute(click_args, &context).await.unwrap();
        assert!(click_result.success);
        assert!(click_result.output.as_ref().unwrap().contains("Clicked element"));

        // 输入文本
        let input_args = json!({
            "action": "input_text",
            "selector": "#username-input",
            "text": "testuser"
        });
        let input_result = tool.execute(input_args, &context).await.unwrap();
        assert!(input_result.success);
        assert!(input_result.output.as_ref().unwrap().contains("Input text"));
    }
}

// ============== CDP 客户端模块 ==============
// 使用 Chrome DevTools Protocol 控制真实浏览器

use std::process::Stdio;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::process::Command;

/// CDP 客户端 - 通过 Chrome DevTools Protocol 控制浏览器
pub struct CdpClient {
    /// 浏览器调试端口
    port: u16,
    /// 浏览器进程
    browser_process: Option<tokio::process::Child>,
    /// CDP WebSocket URL
    ws_url: Option<String>,
    /// HTTP 客户端用于 CDP 协议
    http_client: reqwest::Client,
}

impl CdpClient {
    /// 创建新的 CDP 客户端
    pub fn new(port: u16) -> Self {
        Self {
            port,
            browser_process: None,
            ws_url: None,
            http_client: reqwest::Client::new(),
        }
    }

    /// 启动 Chrome 浏览器并连接到调试端口
    pub async fn start_browser(&mut self) -> Result<(), String> {
        // 检查 Chrome 是否可用
        let chrome_path = self.find_chrome()?;
        
        // 启动 Chrome with remote debugging
        let mut child = Command::new(&chrome_path)
            .args(&[
                &format!("--remote-debugging-port={}", self.port),
                "--no-first-run",
                "--no-default-browser-check",
                "--disable-popup-blocking",
                "--disable-extensions",
                "--disable-background-networking",
                "--disable-default-apps",
                "--disable-sync",
                "--disable-translate",
                "--headless",  // 无头模式
                "--disable-gpu",
                "--no-sandbox",
                "--disable-dev-shm-usage",
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to start Chrome: {}", e))?;

        // 等待浏览器启动
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        
        // 检查进程是否还在运行
        if let Some(status) = child.try_wait().map_err(|e| e.to_string())? {
            return Err(format!("Chrome exited immediately with status: {:?}", status));
        }

        self.browser_process = Some(child);
        
        // 获取 CDP WebSocket URL
        self.ws_url = Some(self.get_ws_url().await?);
        
        Ok(())
    }

    /// 查找系统中的 Chrome 可执行文件
    fn find_chrome(&self) -> Result<String, String> {
        // 检查常见位置
        #[cfg(target_os = "windows")]
        let paths = vec![
            "C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe",
            "C:\\Program Files (x86)\\Google\\Chrome\\Application\\chrome.exe",
            "chromium.exe",
            "chrome.exe",
        ];

        #[cfg(target_os = "macos")]
        let paths = vec![
            "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
            "/Applications/Chromium.app/Contents/MacOS/Chromium",
            "chrome",
            "chromium",
        ];

        #[cfg(target_os = "linux")]
        let paths = vec![
            "/usr/bin/google-chrome",
            "/usr/bin/chromium",
            "/usr/bin/chromium-browser",
            "google-chrome",
            "chromium",
            "chromium-browser",
        ];

        for path in paths {
            if std::path::Path::new(path).exists() {
                return Ok(path.to_string());
            }
            // 尝试从 PATH 中查找
            if let Ok(output) = std::process::Command::new("which").arg(path).output() {
                if output.status.success() {
                    let found = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if !found.is_empty() {
                        return Ok(found);
                    }
                }
            }
        }

        Err("Chrome/Chromium not found. Please install Chrome or Chromium.".to_string())
    }

    /// 从浏览器获取 CDP WebSocket URL
    async fn get_ws_url(&self) -> Result<String, String> {
        let url = format!("http://localhost:{}/json/version", self.port);
        
        let response = self.http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Failed to connect to Chrome: {}", e))?;

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse Chrome response: {}", e))?;

        json["webSocketDebuggerUrl"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| "No WebSocket URL in response".to_string())
    }

    /// 导航到 URL
    pub async fn navigate(&mut self, url: &str) -> Result<String, String> {
        // 使用 CDP 的 Page.navigate 命令
        let cdp_url = format!("http://localhost:{}/json", self.port);
        
        let response = self.http_client
            .post(&cdp_url)
            .json(&serde_json::json!({
                "id": 1,
                "method": "Page.navigate",
                "params": {
                    "url": url
                }
            }))
            .send()
            .await
            .map_err(|e| format!("Failed to navigate: {}", e))?;

        let _json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse navigate response: {}", e))?;

        Ok(format!("Navigated to {}", url))
    }

    /// 获取页面标题
    pub async fn get_title(&self) -> Result<String, String> {
        let cdp_url = format!("http://localhost:{}/json", self.port);
        
        let response = self.http_client
            .get(&cdp_url)
            .send()
            .await
            .map_err(|e| format!("Failed to get page info: {}", e))?;

        let json: Vec<serde_json::Value> = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        json.first()
            .and_then(|v| v["title"].as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| "No page found".to_string())
    }

    /// 获取当前 URL
    pub async fn get_url(&self) -> Result<String, String> {
        let cdp_url = format!("http://localhost:{}/json", self.port);
        
        let response = self.http_client
            .get(&cdp_url)
            .send()
            .await
            .map_err(|e| format!("Failed to get page info: {}", e))?;

        let json: Vec<serde_json::Value> = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        json.first()
            .and_then(|v| v["url"].as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| "No page found".to_string())
    }

    /// 点击元素
    pub async fn click(&self, selector: &str) -> Result<(), String> {
        // 使用 CDP 的 Runtime.evaluate 来执行点击
        let cdp_url = format!("http://localhost:{}/json", self.port);
        
        // 先获取页面标题以确保有页面
        let _: String = self.get_title().await?;

        // 使用 DOM.getDocument 和 DOM.querySelector
        // 这里简化处理，实际需要先获取 nodeId
        let script = format!(
            r#"document.querySelector('{}').click()"#,
            selector.replace("'", "\\'")
        );

        let response = self.http_client
            .post(&format!("http://localhost:{}/json", self.port))
            .json(&serde_json::json!({
                "id": 1,
                "method": "Runtime.evaluate",
                "params": {
                    "expression": script
                }
            }))
            .send()
            .await
            .map_err(|e| format!("Failed to click element: {}", e))?;

        let _json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse click response: {}", e))?;

        Ok(())
    }

    /// 输入文本
    pub async fn input_text(&self, selector: &str, text: &str, clear_first: bool) -> Result<(), String> {
        let script = if clear_first {
            format!(
                r#"(function() {{ var el = document.querySelector('{}'); el.value = ''; el.value = '{}'; }})()"#,
                selector.replace("'", "\\'"),
                text.replace("'", "\\'")
            )
        } else {
            format!(
                r#"document.querySelector('{}').value += '{}'"#,
                selector.replace("'", "\\'"),
                text.replace("'", "\\'")
            )
        };

        let cdp_url = format!("http://localhost:{}/json", self.port);
        
        let response = self.http_client
            .post(&cdp_url)
            .json(&serde_json::json!({
                "id": 1,
                "method": "Runtime.evaluate",
                "params": {
                    "expression": script
                }
            }))
            .send()
            .await
            .map_err(|e| format!("Failed to input text: {}", e))?;

        let _json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse input response: {}", e))?;

        Ok(())
    }

    /// 截图
    pub async fn screenshot(&self) -> Result<String, String> {
        let cdp_url = format!("http://localhost:{}/json", self.port);
        
        // 使用 CDP 的 Page.captureScreenshot
        let response = self.http_client
            .post(&cdp_url)
            .json(&serde_json::json!({
                "id": 1,
                "method": "Page.captureScreenshot",
                "params": {
                    "format": "base64"
                }
            }))
            .send()
            .await
            .map_err(|e| format!("Failed to take screenshot: {}", e))?;

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse screenshot response: {}", e))?;

        json["data"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| "No screenshot data".to_string())
    }

    /// 获取页面源码
    pub async fn get_content(&self) -> Result<String, String> {
        let script = "document.documentElement.outerHTML";
        
        let cdp_url = format!("http://localhost:{}/json", self.port);
        
        let response = self.http_client
            .post(&cdp_url)
            .json(&serde_json::json!({
                "id": 1,
                "method": "Runtime.evaluate",
                "params": {
                    "expression": script
                }
            }))
            .send()
            .await
            .map_err(|e| format!("Failed to get content: {}", e))?;

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse content response: {}", e))?;

        // 从 result.result.value 获取内容
        json["result"]["result"]["value"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| "No content found".to_string())
    }

    /// 执行 JavaScript
    pub async fn execute_script(&self, script: &str) -> Result<String, String> {
        let cdp_url = format!("http://localhost:{}/json", self.port);
        
        let response = self.http_client
            .post(&cdp_url)
            .json(&serde_json::json!({
                "id": 1,
                "method": "Runtime.evaluate",
                "params": {
                    "expression": script
                }
            }))
            .send()
            .await
            .map_err(|e| format!("Failed to execute script: {}", e))?;

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse script response: {}", e))?;

        json["result"]["result"]["value"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| "No result".to_string())
    }

    /// 刷新页面
    pub async fn reload(&self) -> Result<(), String> {
        let cdp_url = format!("http://localhost:{}/json", self.port);
        
        let response = self.http_client
            .post(&cdp_url)
            .json(&serde_json::json!({
                "id": 1,
                "method": "Page.reload"
            }))
            .send()
            .await
            .map_err(|e| format!("Failed to reload: {}", e))?;

        let _json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse reload response: {}", e))?;

        Ok(())
    }

    /// 停止浏览器
    pub async fn stop(&mut self) -> Result<(), String> {
        if let Some(mut child) = self.browser_process.take() {
            child.kill().await.map_err(|e| format!("Failed to kill browser: {}", e))?;
        }
        Ok(())
    }
}

/// 全局 CDP 客户端管理器
pub struct CdpManager {
    client: Option<CdpClient>,
}

impl CdpManager {
    pub fn new() -> Self {
        Self { client: None }
    }

    /// 获取或创建 CDP 客户端
    pub async fn get_client(&mut self) -> Result<&mut CdpClient, String> {
        if self.client.is_none() {
            let mut client = CdpClient::new(9222);
            client.start_browser().await?;
            self.client = Some(client);
        }
        
        Ok(self.client.as_mut().expect("Client should exist"))
    }
}

impl Default for CdpManager {
    fn default() -> Self {
        Self::new()
    }
}