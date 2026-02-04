use crate::tools::{ToolExecutor, ToolResult, ToolError, ToolCategory};
use crate::tools::ExecutionContext;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use tauri::AppHandle;

#[derive(Debug, Deserialize)]
#[serde(tag = "action")]
pub enum UIControlOperation {
    /// 触发按钮点击事件
    #[serde(rename = "click_button")]
    ClickButton {
        /// 按钮标识符
        button_id: String,
        /// 额外参数 (可选)
        params: Option<HashMap<String, Value>>,
    },
    
    /// 设置输入框文本
    #[serde(rename = "set_input_text")]
    SetInputText {
        /// 输入框标识符
        input_id: String,
        /// 要设置的文本
        text: String,
    },
    
    /// 获取输入框文本
    #[serde(rename = "get_input_text")]
    GetInputText {
        /// 输入框标识符
        input_id: String,
    },
    
    /// 选择下拉菜单选项
    #[serde(rename = "select_dropdown_option")]
    SelectDropdownOption {
        /// 下拉菜单标识符
        dropdown_id: String,
        /// 选项值
        option_value: String,
    },
    
    /// 触发复选框切换
    #[serde(rename = "toggle_checkbox")]
    ToggleCheckbox {
        /// 复选框标识符
        checkbox_id: String,
    },
    
    /// 触发单选按钮选择
    #[serde(rename = "select_radio_button")]
    SelectRadioButton {
        /// 单选按钮组标识符
        radio_group_id: String,
        /// 选项值
        option_value: String,
    },
    
    /// 显示通知
    #[serde(rename = "show_notification")]
    ShowNotification {
        /// 通知标题
        title: String,
        /// 通知内容
        message: String,
        /// 通知类型 (可选: info, warning, error, success)
        notification_type: Option<String>,
    },
    
    /// 显示模态对话框
    #[serde(rename = "show_modal")]
    ShowModal {
        /// 对话框标题
        title: String,
        /// 对话框内容
        content: String,
        /// 按钮配置 (可选)
        buttons: Option<Vec<String>>,
    },
    
    /// 获取窗口状态
    #[serde(rename = "get_window_state")]
    GetWindowState {},
    
    /// 设置窗口状态
    #[serde(rename = "set_window_state")]
    SetWindowState {
        /// 窗口状态 (minimized, maximized, fullscreen, normal)
        state: String,
    },
}

#[derive(Debug, Clone)]
pub struct UIControlTool {
    id: String,
    name: String,
    description: String,
    category: ToolCategory,
    app_handle: Option<AppHandle>,
}

impl UIControlTool {
    pub fn new(app_handle: Option<AppHandle>) -> Self {
        Self {
            id: "ui_control".to_string(),
            name: "UI Control Tool".to_string(),
            description: "Tool for interacting with desktop application UI elements".to_string(),
            category: ToolCategory::Automation,
            app_handle,
        }
    }
}

#[derive(Serialize, Debug)]
pub struct UIControlResult {
    pub success: bool,
    pub message: String,
    pub data: Option<Value>,
    pub output: Option<String>,
}

// 模拟 UI 控件交互客户端
pub struct UIControlClient {
    ui_elements: HashMap<String, UIElement>,
    window_state: WindowState,
}

#[derive(Clone, Debug)]
struct UIElement {
    element_type: String,
    current_value: String,
    enabled: bool,
    visible: bool,
}

#[derive(Clone, Debug, Serialize)]
struct WindowState {
    minimized: bool,
    maximized: bool,
    fullscreen: bool,
    width: u32,
    height: u32,
}

impl UIControlClient {
    pub fn new() -> Self {
        Self {
            ui_elements: [
                ("btn_send".to_string(), UIElement { element_type: "button".to_string(), current_value: "".to_string(), enabled: true, visible: true }),
                ("btn_receive".to_string(), UIElement { element_type: "button".to_string(), current_value: "".to_string(), enabled: true, visible: true }),
                ("btn_settings".to_string(), UIElement { element_type: "button".to_string(), current_value: "".to_string(), enabled: true, visible: true }),
                ("input_message".to_string(), UIElement { element_type: "input".to_string(), current_value: "".to_string(), enabled: true, visible: true }),
                ("input_amount".to_string(), UIElement { element_type: "input".to_string(), current_value: "".to_string(), enabled: true, visible: true }),
                ("dropdown_currency".to_string(), UIElement { element_type: "dropdown".to_string(), current_value: "USD".to_string(), enabled: true, visible: true }),
                ("checkbox_remember".to_string(), UIElement { element_type: "checkbox".to_string(), current_value: "false".to_string(), enabled: true, visible: true }),
                ("radio_theme_light".to_string(), UIElement { element_type: "radio".to_string(), current_value: "false".to_string(), enabled: true, visible: true }),
                ("radio_theme_dark".to_string(), UIElement { element_type: "radio".to_string(), current_value: "false".to_string(), enabled: true, visible: true }),
            ].iter().cloned().collect(),
            window_state: WindowState {
                minimized: false,
                maximized: false,
                fullscreen: false,
                width: 1200,
                height: 800,
            },
        }
    }

    pub async fn click_button(&mut self, button_id: &str, params: Option<HashMap<String, Value>>) -> Result<(), String> {
        if !self.ui_elements.contains_key(button_id) {
            return Err(format!("Button '{}' not found", button_id));
        }

        let element = self.ui_elements.get_mut(button_id).unwrap();
        if element.element_type != "button" {
            return Err(format!("Element '{}' is not a button", button_id));
        }

        if !element.enabled {
            return Err(format!("Button '{}' is disabled", button_id));
        }

        // 模拟按钮点击效果
        println!("Simulating click on button: {}", button_id);
        if let Some(params) = params {
            println!("With parameters: {:?}", params);
        }

        Ok(())
    }

    pub async fn set_input_text(&mut self, input_id: &str, text: String) -> Result<(), String> {
        if !self.ui_elements.contains_key(input_id) {
            return Err(format!("Input '{}' not found", input_id));
        }

        let element = self.ui_elements.get_mut(input_id).unwrap();
        if element.element_type != "input" {
            return Err(format!("Element '{}' is not an input", input_id));
        }

        if !element.enabled {
            return Err(format!("Input '{}' is disabled", input_id));
        }

        element.current_value = text;
        Ok(())
    }

    pub async fn get_input_text(&self, input_id: &str) -> Result<String, String> {
        if !self.ui_elements.contains_key(input_id) {
            return Err(format!("Input '{}' not found", input_id));
        }

        let element = self.ui_elements.get(input_id).unwrap();
        if element.element_type != "input" {
            return Err(format!("Element '{}' is not an input", input_id));
        }

        Ok(element.current_value.clone())
    }

    pub async fn select_dropdown_option(&mut self, dropdown_id: &str, option_value: String) -> Result<(), String> {
        if !self.ui_elements.contains_key(dropdown_id) {
            return Err(format!("Dropdown '{}' not found", dropdown_id));
        }

        let element = self.ui_elements.get_mut(dropdown_id).unwrap();
        if element.element_type != "dropdown" {
            return Err(format!("Element '{}' is not a dropdown", dropdown_id));
        }

        if !element.enabled {
            return Err(format!("Dropdown '{}' is disabled", dropdown_id));
        }

        element.current_value = option_value;
        Ok(())
    }

    pub async fn toggle_checkbox(&mut self, checkbox_id: &str) -> Result<String, String> {
        if !self.ui_elements.contains_key(checkbox_id) {
            return Err(format!("Checkbox '{}' not found", checkbox_id));
        }

        let element = self.ui_elements.get_mut(checkbox_id).unwrap();
        if element.element_type != "checkbox" {
            return Err(format!("Element '{}' is not a checkbox", checkbox_id));
        }

        if !element.enabled {
            return Err(format!("Checkbox '{}' is disabled", checkbox_id));
        }

        let current_state = element.current_value.parse::<bool>().unwrap_or(false);
        let new_state = !current_state;
        element.current_value = new_state.to_string();
        
        Ok(new_state.to_string())
    }

    pub async fn select_radio_button(&mut self, radio_group_id: &str, option_value: String) -> Result<(), String> {
        // 查找同组的单选按钮并取消选择其他选项
        let group_prefix = format!("{}_", radio_group_id.strip_prefix("radio_").unwrap_or(radio_group_id));
        
        for (id, element) in self.ui_elements.iter_mut() {
            if element.element_type == "radio" && id.starts_with(&group_prefix) {
                if id.ends_with(&option_value) {
                    element.current_value = "true".to_string();
                } else {
                    element.current_value = "false".to_string();
                }
            }
        }

        Ok(())
    }

    pub async fn show_notification(&self, title: String, message: String, notification_type: Option<String>) -> Result<(), String> {
        let note_type = notification_type.unwrap_or_else(|| "info".to_string());
        println!("Showing notification - Type: {}, Title: {}, Message: {}", note_type, title, message);
        Ok(())
    }

    pub async fn show_modal(&self, title: String, content: String, buttons: Option<Vec<String>>) -> Result<String, String> {
        let button_labels = buttons.unwrap_or_else(|| vec!["OK".to_string()]);
        println!("Showing modal - Title: {}, Content: {}, Buttons: {:?}", title, content, button_labels);
        
        // 模拟用户点击第一个按钮
        Ok(button_labels.first().cloned().unwrap_or_else(|| "OK".to_string()))
    }

    pub async fn get_window_state(&self) -> WindowState {
        self.window_state.clone()
    }

    pub async fn set_window_state(&mut self, state: String) -> Result<(), String> {
        match state.as_str() {
            "minimized" => {
                self.window_state.minimized = true;
                self.window_state.maximized = false;
                self.window_state.fullscreen = false;
            },
            "maximized" => {
                self.window_state.minimized = false;
                self.window_state.maximized = true;
                self.window_state.fullscreen = false;
            },
            "fullscreen" => {
                self.window_state.minimized = false;
                self.window_state.maximized = false;
                self.window_state.fullscreen = true;
            },
            "normal" => {
                self.window_state.minimized = false;
                self.window_state.maximized = false;
                self.window_state.fullscreen = false;
            },
            _ => return Err(format!("Invalid window state: {}", state)),
        }
        
        Ok(())
    }
}

impl UIControlTool {
    async fn execute_impl(&self, args: Value) -> Result<UIControlResult, Box<dyn std::error::Error>> {
        let operation: UIControlOperation = serde_json::from_value(args)?;
        let mut client = UIControlClient::new();

        match operation {
            UIControlOperation::ClickButton { button_id, params } => {
                match client.click_button(&button_id, params).await {
                    Ok(_) => {
                        Ok(UIControlResult {
                            success: true,
                            message: format!("Successfully clicked button: {}", button_id),
                            data: Some(serde_json::json!({
                                "button_id": button_id
                            })),
                            output: Some(format!("Clicked button: {}", button_id)),
                        })
                    }
                    Err(e) => {
                        Ok(UIControlResult {
                            success: false,
                            message: format!("Failed to click button: {}", e),
                            data: None,
                            output: Some(format!("Error clicking button: {}", e)),
                        })
                    }
                }
            }

            UIControlOperation::SetInputText { input_id, text } => {
                match client.set_input_text(&input_id, text).await {
                    Ok(_) => {
                        Ok(UIControlResult {
                            success: true,
                            message: format!("Successfully set text for input: {}", input_id),
                            data: Some(serde_json::json!({
                                "input_id": input_id
                            })),
                            output: Some(format!("Set text for input: {}", input_id)),
                        })
                    }
                    Err(e) => {
                        Ok(UIControlResult {
                            success: false,
                            message: format!("Failed to set input text: {}", e),
                            data: None,
                            output: Some(format!("Error setting input text: {}", e)),
                        })
                    }
                }
            }

            UIControlOperation::GetInputText { input_id } => {
                match client.get_input_text(&input_id).await {
                    Ok(text) => {
                        Ok(UIControlResult {
                            success: true,
                            message: format!("Successfully retrieved text from input: {}", input_id),
                            data: Some(serde_json::json!({
                                "input_id": input_id,
                                "text": text
                            })),
                            output: Some(format!("Retrieved text from input '{}': {}", input_id, text)),
                        })
                    }
                    Err(e) => {
                        Ok(UIControlResult {
                            success: false,
                            message: format!("Failed to get input text: {}", e),
                            data: None,
                            output: Some(format!("Error getting input text: {}", e)),
                        })
                    }
                }
            }

            UIControlOperation::SelectDropdownOption { dropdown_id, option_value } => {
                match client.select_dropdown_option(&dropdown_id, option_value).await {
                    Ok(_) => {
                        Ok(UIControlResult {
                            success: true,
                            message: format!("Successfully selected option in dropdown: {}", dropdown_id),
                            data: Some(serde_json::json!({
                                "dropdown_id": dropdown_id
                            })),
                            output: Some(format!("Selected option in dropdown: {}", dropdown_id)),
                        })
                    }
                    Err(e) => {
                        Ok(UIControlResult {
                            success: false,
                            message: format!("Failed to select dropdown option: {}", e),
                            data: None,
                            output: Some(format!("Error selecting dropdown option: {}", e)),
                        })
                    }
                }
            }

            UIControlOperation::ToggleCheckbox { checkbox_id } => {
                match client.toggle_checkbox(&checkbox_id).await {
                    Ok(state) => {
                        Ok(UIControlResult {
                            success: true,
                            message: format!("Successfully toggled checkbox: {}", checkbox_id),
                            data: Some(serde_json::json!({
                                "checkbox_id": checkbox_id,
                                "state": state
                            })),
                            output: Some(format!("Toggled checkbox '{}' to state: {}", checkbox_id, state)),
                        })
                    }
                    Err(e) => {
                        Ok(UIControlResult {
                            success: false,
                            message: format!("Failed to toggle checkbox: {}", e),
                            data: None,
                            output: Some(format!("Error toggling checkbox: {}", e)),
                        })
                    }
                }
            }

            UIControlOperation::SelectRadioButton { radio_group_id, option_value } => {
                match client.select_radio_button(&radio_group_id, option_value.clone()).await {
                    Ok(_) => {
                        Ok(UIControlResult {
                            success: true,
                            message: format!("Successfully selected radio button option"),
                            data: Some(serde_json::json!({
                                "radio_group_id": radio_group_id,
                                "option_value": option_value
                            })),
                            output: Some(format!("Selected option '{}' in radio group '{}'", option_value, radio_group_id)),
                        })
                    }
                    Err(e) => {
                        Ok(UIControlResult {
                            success: false,
                            message: format!("Failed to select radio button: {}", e),
                            data: None,
                            output: Some(format!("Error selecting radio button: {}", e)),
                        })
                    }
                }
            }

            UIControlOperation::ShowNotification { title, message, notification_type } => {
                match client.show_notification(title.clone(), message.clone(), notification_type).await {
                    Ok(_) => {
                        Ok(UIControlResult {
                            success: true,
                            message: format!("Successfully showed notification: {}", title),
                            data: Some(serde_json::json!({
                                "title": title,
                                "message": message
                            })),
                            output: Some(format!("Showed notification: {}", title)),
                        })
                    }
                    Err(e) => {
                        Ok(UIControlResult {
                            success: false,
                            message: format!("Failed to show notification: {}", e),
                            data: None,
                            output: Some(format!("Error showing notification: {}", e)),
                        })
                    }
                }
            }

            UIControlOperation::ShowModal { title, content, buttons } => {
                match client.show_modal(title.clone(), content.clone(), buttons).await {
                    Ok(result) => {
                        Ok(UIControlResult {
                            success: true,
                            message: format!("Successfully showed modal: {}", title),
                            data: Some(serde_json::json!({
                                "title": title,
                                "result": result
                            })),
                            output: Some(format!("Showed modal '{}', user result: {}", title, result)),
                        })
                    }
                    Err(e) => {
                        Ok(UIControlResult {
                            success: false,
                            message: format!("Failed to show modal: {}", e),
                            data: None,
                            output: Some(format!("Error showing modal: {}", e)),
                        })
                    }
                }
            }

            UIControlOperation::GetWindowState {} => {
                let state = client.get_window_state().await;
                Ok(UIControlResult {
                    success: true,
                    message: "Successfully retrieved window state".to_string(),
                    data: Some(serde_json::json!(state)),
                    output: Some(format!("Window state - Minimized: {}, Maximized: {}, Fullscreen: {}", 
                                         state.minimized, state.maximized, state.fullscreen)),
                })
            }

            UIControlOperation::SetWindowState { state } => {
                match client.set_window_state(state.clone()).await {
                    Ok(_) => {
                        Ok(UIControlResult {
                            success: true,
                            message: format!("Successfully set window state to: {}", state),
                            data: Some(serde_json::json!({
                                "state": state
                            })),
                            output: Some(format!("Set window state to: {}", state)),
                        })
                    }
                    Err(e) => {
                        Ok(UIControlResult {
                            success: false,
                            message: format!("Failed to set window state: {}", e),
                            data: None,
                            output: Some(format!("Error setting window state: {}", e)),
                        })
                    }
                }
            }
        }
    }
}

#[async_trait::async_trait]
impl ToolExecutor for UIControlTool {
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
            dependencies: vec!["tauri".to_string()],
            platforms: vec!["windows".to_string(), "macos".to_string(), "linux".to_string()],
            permissions: vec!["desktop".to_string(), "ui".to_string()],
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
                    output: Some(format!("Error executing UI control tool: {}", e)),
                    warnings: vec![],
                    context: None,
                })
            }
        }
    }

    async fn validate_args(&self, args: &Value) -> Result<(), ToolError> {
        match serde_json::from_value::<UIControlOperation>(args.clone()) {
            Ok(_) => Ok(()),
            Err(e) => Err(ToolError::InvalidArguments(e.to_string())),
        }
    }

    fn help(&self) -> String {
        "UI Control Tool for interacting with desktop application UI elements. Actions: click_button, set_input_text, get_input_text, select_dropdown_option, toggle_checkbox, select_radio_button, show_notification, show_modal, get_window_state, set_window_state".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_click_button() {
        let tool = UIControlTool::new(None);
        let context = ExecutionContext {
            session_id: "test_session".to_string(),
            user_id: None,
            working_directory: None,
            environment: std::collections::HashMap::new(),
            timeout_seconds: Some(30),
            permissions: vec!["desktop".to_string(), "ui".to_string()],
            timestamp: chrono::Utc::now().timestamp(),
        };

        let click_args = json!({
            "action": "click_button",
            "button_id": "btn_send"
        });
        let result = tool.execute(click_args, &context).await.unwrap();
        assert!(result.success);
        assert!(result.output.as_ref().unwrap().contains("Clicked button"));
    }

    #[tokio::test]
    async fn test_input_operations() {
        let tool = UIControlTool::new(None);
        let context = ExecutionContext {
            session_id: "test_session".to_string(),
            user_id: None,
            working_directory: None,
            environment: std::collections::HashMap::new(),
            timeout_seconds: Some(30),
            permissions: vec!["desktop".to_string(), "ui".to_string()],
            timestamp: chrono::Utc::now().timestamp(),
        };

        // 设置输入文本
        let set_args = json!({
            "action": "set_input_text",
            "input_id": "input_message",
            "text": "Hello, UI!"
        });
        let set_result = tool.execute(set_args, &context).await.unwrap();
        assert!(set_result.success);

        // 获取输入文本
        let get_args = json!({
            "action": "get_input_text",
            "input_id": "input_message"
        });
        let get_result = tool.execute(get_args, &context).await.unwrap();
        assert!(get_result.success);
        assert_eq!(get_result.data.as_ref().unwrap()["text"], "Hello, UI!");
    }
}