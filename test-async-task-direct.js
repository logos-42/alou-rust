// 直接测试异步任务创建功能
const API_BASE = 'https://alou-edge.yuanjieliu65.workers.dev';

async function testAsyncTaskCreation() {
    console.log('🧪 直接测试异步任务创建...');

    try {
        // 构建兼容性请求
        const compatRequest = {
            prompt: "请帮我创建一个文件夹，并查看当前目录内容。",
            system_prompt: "你是一个文件管理助手。",
            history: [],
            agent_info: {
                name: "文件助手",
                role_description: "帮助用户管理文件和目录"
            },
            tools: [
                {
                    name: "bash",
                    description: "执行bash命令",
                    parameters: {
                        type: "object",
                        properties: {
                            command: {
                                type: "string",
                                description: "要执行的bash命令"
                            }
                        },
                        required: ["command"]
                    }
                }
            ],
            model: "deepseek-chat",
            max_tokens: 8192,
            temperature: 0.7,
            task_type: "async"
        };

        console.log('📤 发送异步任务创建请求...');
        console.log('📍 请求URL:', `${API_BASE}/api/agent/chat`);
        console.log('📍 请求数据:', JSON.stringify({
            session_id: `test-session-${Date.now()}`,
            message: compatRequest.prompt,
            wallet_address: "0x123...",
            chain: "ethereum"
        }, null, 2));

        // 使用简单的fetch请求
        const controller = new AbortController();
        const timeoutId = setTimeout(() => controller.abort(), 30000); // 30秒超时

        const response = await fetch(`${API_BASE}/api/agent/chat`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify({
                session_id: `test-session-${Date.now()}`,
                message: compatRequest.prompt,
                wallet_address: "0x123...",
                chain: "ethereum"
            }),
            signal: controller.signal
        });

        clearTimeout(timeoutId);

        const responseText = await response.text();
        console.log('📋 响应状态:', response.status);
        console.log('📋 响应内容:', responseText);

        if (response.ok) {
            const result = JSON.parse(responseText);

            if (result.task_id) {
                console.log('✅ 异步任务创建成功！');
                console.log('📋 任务ID:', result.task_id);
                console.log('📋 状态:', result.status);
                console.log('📋 内容:', result.content);

                // 等待一会儿，然后查询任务状态
                console.log('\n⏳ 等待任务处理...');
                await new Promise(resolve => setTimeout(resolve, 5000));

                console.log('📊 查询任务状态...');
                const statusResponse = await fetch(`${API_BASE}/api/tasks/${result.task_id}`);
                if (statusResponse.ok) {
                    const status = await statusResponse.json();
                    console.log('📈 任务状态:', JSON.stringify(status, null, 2));
                } else {
                    console.log('❌ 查询任务状态失败:', statusResponse.status);
                }

                return true;
            } else {
                console.log('❌ 响应中没有task_id，说明没有创建异步任务');
                console.log('📋 完整响应:', result);
                return false;
            }
        } else {
            console.log('❌ 请求失败:', response.status, responseText);
            return false;
        }

    } catch (error) {
        console.log('❌ 测试异常:', error.message);
        return false;
    }
}

async function runTest() {
    console.log('🚀 开始异步任务创建测试...\n');

    const success = await testAsyncTaskCreation();

    console.log('\n🎯 测试结果:', success ? '✅ 成功' : '❌ 失败');

    if (success) {
        console.log('🎉 异步任务创建功能正常工作！');
    } else {
        console.log('💥 异步任务创建功能存在问题。');
    }
}

runTest();