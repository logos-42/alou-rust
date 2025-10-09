use worker::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct ChatRequest {
    message: String,
    session_id: Option<String>,
}

#[derive(Serialize)]
struct ChatResponse {
    response: String,
    status: String,
    timestamp: u64,
    session_id: String,
}

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    agent_ready: bool,
    timestamp: u64,
    version: String,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
    message: String,
    timestamp: u64,
}

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    console_error_panic_hook::set_once();
    
    // CORS 处理
    if req.method() == Method::Options {
        return Response::empty()
            .map(|resp| {
                resp.with_headers([
                    ("Access-Control-Allow-Origin", "*"),
                    ("Access-Control-Allow-Methods", "GET, POST, OPTIONS"),
                    ("Access-Control-Allow-Headers", "Content-Type, Authorization"),
                ])
            });
    }
    
    let router = Router::new();
    
    router
        .get("/api/health", |_req, _ctx| {
            let health = HealthResponse {
                status: "healthy".to_string(),
                agent_ready: true,
                timestamp: Date::now().as_millis(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            };
            
            Response::from_json(&health)
                .map(|resp| {
                    resp.with_headers([
                        ("Access-Control-Allow-Origin", "*"),
                        ("Content-Type", "application/json"),
                    ])
                })
        })
        .post_async("/api/chat", |mut req, ctx| async move {
            handle_chat(req, ctx).await
        })
        .get("/", |_req, _ctx| {
            Response::ok("🦀 Alou Rust Agent - Running on Cloudflare Workers! 🚀")
                .map(|resp| {
                    resp.with_headers([
                        ("Access-Control-Allow-Origin", "*"),
                        ("Content-Type", "text/plain"),
                    ])
                })
        })
        .run(req, env)
        .await
}

async fn handle_chat(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    // 解析请求
    let chat_req: ChatRequest = match req.json().await {
        Ok(req) => req,
        Err(e) => {
            let error = ErrorResponse {
                error: "Invalid JSON".to_string(),
                message: format!("Failed to parse request: {}", e),
                timestamp: Date::now().as_millis(),
            };
            return Response::from_json(&error)
                .map(|resp| {
                    resp.with_status(400)
                        .with_headers([
                            ("Access-Control-Allow-Origin", "*"),
                            ("Content-Type", "application/json"),
                        ])
                });
        }
    };
    
    // 获取 API Key
    let api_key = match ctx.env.secret("DEEPSEEK_API_KEY") {
        Ok(secret) => secret.to_string(),
        Err(_) => {
            let error = ErrorResponse {
                error: "Configuration Error".to_string(),
                message: "DEEPSEEK_API_KEY not configured".to_string(),
                timestamp: Date::now().as_millis(),
            };
            return Response::from_json(&error)
                .map(|resp| {
                    resp.with_status(500)
                        .with_headers([
                            ("Access-Control-Allow-Origin", "*"),
                            ("Content-Type", "application/json"),
                        ])
                });
        }
    };
    
    // 调用 DeepSeek API
    let response_text = match call_deepseek_api(&chat_req.message, &api_key).await {
        Ok(response) => response,
        Err(e) => {
            let error = ErrorResponse {
                error: "API Error".to_string(),
                message: format!("Failed to call DeepSeek API: {}", e),
                timestamp: Date::now().as_millis(),
            };
            return Response::from_json(&error)
                .map(|resp| {
                    resp.with_status(500)
                        .with_headers([
                            ("Access-Control-Allow-Origin", "*"),
                            ("Content-Type", "application/json"),
                        ])
                });
        }
    };
    
    let session_id = chat_req.session_id.unwrap_or_else(|| {
        format!("session_{}", (js_sys::Math::random() * 1000000.0) as u32)
    });
    
    let chat_response = ChatResponse {
        response: response_text,
        status: "success".to_string(),
        timestamp: Date::now().as_millis(),
        session_id,
    };
    
    Response::from_json(&chat_response)
        .map(|resp| {
            resp.with_headers([
                ("Access-Control-Allow-Origin", "*"),
                ("Content-Type", "application/json"),
            ])
        })
}

async fn call_deepseek_api(message: &str, api_key: &str) -> Result<String> {
    use worker::wasm_bindgen_futures::JsFuture;
    use worker::web_sys::{Request as WebRequest, RequestInit, RequestMode, Headers};
    
    // 构建请求体
    let request_body = serde_json::json!({
        "model": "deepseek-chat",
        "messages": [
            {
                "role": "user",
                "content": message
            }
        ],
        "max_tokens": 2000,
        "temperature": 0.7
    });
    
    // 创建 Headers
    let headers = Headers::new().map_err(|e| format!("Failed to create headers: {:?}", e))?;
    headers.set("Content-Type", "application/json").map_err(|e| format!("Failed to set content type: {:?}", e))?;
    headers.set("Authorization", &format!("Bearer {}", api_key)).map_err(|e| format!("Failed to set auth header: {:?}", e))?;
    
    // 创建请求配置
    let mut opts = RequestInit::new();
    opts.method("POST");
    opts.mode(RequestMode::Cors);
    opts.headers(&headers);
    opts.body(Some(&wasm_bindgen::JsValue::from_str(&request_body.to_string())));
    
    // 创建请求
    let request = WebRequest::new_with_str_and_init("https://api.deepseek.com/chat/completions", &opts)
        .map_err(|e| format!("Failed to create request: {:?}", e))?;
    
    // 发送请求
    let window = web_sys::window().ok_or("No window object")?;
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await
        .map_err(|e| format!("Fetch failed: {:?}", e))?;
    
    // 转换为 Response
    let resp: web_sys::Response = resp_value.dyn_into()
        .map_err(|_| "Failed to cast to Response")?;
    
    if !resp.ok() {
        return Err(format!("HTTP error: {}", resp.status()).into());
    }
    
    // 读取响应体
    let json_promise = resp.json().map_err(|e| format!("Failed to get JSON: {:?}", e))?;
    let json_value = JsFuture::from(json_promise).await
        .map_err(|e| format!("Failed to parse JSON: {:?}", e))?;
    
    // 解析响应
    let response_str = js_sys::JSON::stringify(&json_value)
        .map_err(|e| format!("Failed to stringify: {:?}", e))?;
    
    let response_json: serde_json::Value = serde_json::from_str(&response_str.as_string().unwrap_or_default())
        .map_err(|e| format!("Failed to parse response: {}", e))?;
    
    // 提取消息内容
    let content = response_json
        .get("choices")
        .and_then(|choices| choices.get(0))
        .and_then(|choice| choice.get("message"))
        .and_then(|message| message.get("content"))
        .and_then(|content| content.as_str())
        .unwrap_or("Sorry, I couldn't process your request.");
    
    Ok(content.to_string())
}
