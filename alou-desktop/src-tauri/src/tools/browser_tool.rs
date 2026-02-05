use crate::tools::{ToolExecutor, ToolResult, ToolError, ToolCategory};
use crate::tools::ExecutionContext;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use tokio::time::{sleep, Duration};

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
}

impl BrowserClient {
    pub fn new(state: std::sync::Arc<tokio::sync::RwLock<BrowserState>>) -> Self {
        Self { state }
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
        let state = self.state.read().await;
        Ok(state.current_title.clone())
    }

    pub async fn get_page_url(&self) -> Result<String, String> {
        let state = self.state.read().await;
        Ok(state.current_url.clone())
    }

    pub async fn take_screenshot(&self, path: Option<String>) -> Result<String, String> {
        let screenshot_path = path.unwrap_or_else(|| format!("screenshot_{}.png", chrono::Utc::now().timestamp()));
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