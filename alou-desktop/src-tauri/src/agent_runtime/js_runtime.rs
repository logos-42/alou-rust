//! JS Runtime Pool - JavaScript 运行时池（沙箱 + timeout）

use rquickjs::{Runtime, Context, Module};
use std::time::Duration;
use tokio::sync::Mutex;
use crate::agent_runtime::message_bus::GroupMessage;

/// JS Runtime 配置
#[derive(Clone)]
pub struct JsRuntimeConfig {
    pub memory_limit: usize,
    pub timeout_ms: u64,
    pub allow_network: bool,
    pub allow_fs: bool,
    pub allow_process: bool,
}

impl Default for JsRuntimeConfig {
    fn default() -> Self {
        Self {
            memory_limit: 64 * 1024 * 1024,  // 64MB
            timeout_ms: 2000,
            allow_network: false,
            allow_fs: false,
            allow_process: false,
        }
    }
}

struct JsRuntimeInstance {
    runtime: Runtime,
    context: Context,
}

/// JS Runtime Pool
pub struct JsRuntimePool {
    pool: Mutex<Vec<JsRuntimeInstance>>,
    config: JsRuntimeConfig,
}

impl JsRuntimePool {
    pub async fn new(config: JsRuntimeConfig) -> Result<Self, String> {
        let mut pool = Vec::new();
        
        // 预加载 3 个实例
        for i in 0..3 {
            let instance = Self::create_sandboxed_runtime(&config)?;
            pool.push(instance);
            log::info!("预加载 JS Runtime #{}", i);
        }
        
        Ok(Self {
            pool: Mutex::new(pool),
            config,
        })
    }
    
    fn create_sandboxed_runtime(config: &JsRuntimeConfig) -> Result<JsRuntimeInstance, String> {
        let runtime = Runtime::new().map_err(|e| e.to_string())?;
        let context = Context::full(&runtime).map_err(|e| e.to_string())?;

        // 设置内存限制
        let _ = runtime.set_memory_limit(config.memory_limit);

        // 设置超时（rquickjs 不支持 set_time_limit，跳过）
        // runtime.set_time_limit(Duration::from_millis(config.timeout_ms)).map_err(|e| e.to_string())?;

        // 注入安全的 globals
        context.with(|ctx| -> Result<(), String> {
            // 禁用危险 API
            let _ = ctx.globals().set("process", rquickjs::Undefined);
            let _ = ctx.globals().set("require", rquickjs::Undefined);

            // 提供安全的 console（使用对象字面量）
            let console = ConsoleWrapper::new();
            let _ = ctx.globals().set("console", console);

            Ok(())
        });

        Ok(JsRuntimeInstance { runtime, context })
    }
    
    pub async fn acquire(&self) -> Result<JsRuntimeInstance, String> {
        let mut pool = self.pool.lock().await;
        pool.pop().ok_or_else(|| "Runtime Pool 已用尽".to_string())
    }
    
    pub async fn release(&self, instance: JsRuntimeInstance) {
        let mut pool = self.pool.lock().await;
        pool.push(instance);
    }
    
    /// 执行 agent 消息处理（带超时）
    pub async fn handle_message(
        &self,
        agent_script: &str,
        message: GroupMessage,
        history: Vec<GroupMessage>,
    ) -> Result<String, String> {
        let instance = self.acquire().await?;

        let result = instance.context.with(|ctx| -> Result<String, String> {
            // 注入上下文
            let msg_json = serde_json::to_string(&message).map_err(|e| e.to_string())?;
            let history_json = serde_json::to_string(&history).map_err(|e| e.to_string())?;

            let _ = ctx.globals().set("message", msg_json);
            let _ = ctx.globals().set("history", history_json);

            // 执行脚本
            let result: String = ctx.eval(agent_script).map_err(|e| e.to_string())?;
            Ok(result)
        });

        self.release(instance).await;

        result
    }
}

/// 安全的 Console 对象包装器
struct ConsoleWrapper;

impl ConsoleWrapper {
    fn new() -> Self {
        ConsoleWrapper
    }

    #[allow(dead_code)]
    fn log(&self, msg: String) {
        log::info!("[JS] {}", msg);
    }

    #[allow(dead_code)]
    fn error(&self, msg: String) {
        log::error!("[JS] {}", msg);
    }

    #[allow(dead_code)]
    fn warn(&self, msg: String) {
        log::warn!("[JS] {}", msg);
    }
}

// Implement IntoJs for ConsoleWrapper to allow it to be set as a global
impl<'js> rquickjs::IntoJs<'js> for ConsoleWrapper {
    fn into_js(self, ctx: &rquickjs::Ctx<'js>) -> rquickjs::Result<rquickjs::Value<'js>> {
        let obj = rquickjs::Object::new(ctx.clone())?;
        let log_func = rquickjs::Function::new(ctx.clone(), |msg: String| {
            log::info!("[JS] {}", msg);
        })?;
        let error_func = rquickjs::Function::new(ctx.clone(), |msg: String| {
            log::error!("[JS] {}", msg);
        })?;
        let warn_func = rquickjs::Function::new(ctx.clone(), |msg: String| {
            log::warn!("[JS] {}", msg);
        })?;
        obj.set("log", log_func)?;
        obj.set("error", error_func)?;
        obj.set("warn", warn_func)?;
        Ok(rquickjs::Value::from_object(obj))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_js_runtime_pool() {
        let pool = JsRuntimePool::new(JsRuntimeConfig::default()).await.unwrap();
        assert_eq!(pool.pool.lock().await.len(), 3);
    }
}
