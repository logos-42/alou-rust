export interface Env {
	AI_PROVIDER?: string
	AI_MODEL?: string
	AI_API_KEY?: string
	DB?: any
	SESSIONS?: any
	CACHE?: any
	NONCES?: any
	AI_TASKS?: any
}

export default {
	async fetch(request: Request, env: Env, ctx: ExecutionContext): Promise<Response> {
		const url = new URL(request.url)
		
		// 健康检查
		if (url.pathname === "/api/health") {
			return new Response(JSON.stringify({ status: "ok", timestamp: new Date().toISOString() }), {
				headers: { "Content-Type": "application/json" },
			})
		}

		// 聊天 API - 直接调用后端 AI API
		if (url.pathname === "/api/chat" && request.method === "POST") {
			try {
				const body = await request.json() as any
				const apiKey = body.apiKey || env.AI_API_KEY
				const model = body.model || env.AI_MODEL || "gpt-4o-mini"
				const messages = body.messages || []
				const systemPrompt = body.systemPrompt || "You are a helpful AI assistant."

				// 构建请求体
				const requestBody = {
					model: model,
					messages: [
						{ role: "system", content: systemPrompt },
						...messages
					],
					stream: true,
				}

				// 调用后端 API（这里使用 OpenAI 兼容接口）
				const apiUrl = body.baseUrl || "https://api.openai.com/v1/chat/completions"
				
				const response = await fetch(apiUrl, {
					method: "POST",
					headers: {
						"Content-Type": "application/json",
						"Authorization": `Bearer ${apiKey}`,
					},
					body: JSON.stringify(requestBody),
				})

				if (!response.ok) {
					throw new Error(`API 调用失败: ${response.status} ${response.statusText}`)
				}

				// 将响应流直接返回给客户端
				return new Response(response.body, {
					headers: {
						"Content-Type": "text/plain; charset=utf-8",
						"Cache-Control": "no-cache",
					},
				})
			} catch (error) {
				const errorMessage = error instanceof Error ? error.message : String(error)
				return new Response(JSON.stringify({ error: errorMessage }), {
					status: 500,
					headers: { "Content-Type": "application/json" },
				})
			}
		}

		// 提供商列表
		if (url.pathname === "/api/providers" && request.method === "GET") {
			return new Response(JSON.stringify({
				success: true,
				providers: ["openai", "deepseek", "anthropic"],
			}), {
				headers: { "Content-Type": "application/json" },
			})
		}

		// 默认响应
		return new Response(JSON.stringify({
			message: "Alou Edge API",
			endpoints: [
				"POST /api/chat - Chat with AI",
				"GET /api/health - Health check",
				"GET /api/providers - List providers",
			],
		}), {
			headers: { "Content-Type": "application/json" },
		})
	},
}
