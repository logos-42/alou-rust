// 测试异步多步骤任务修复效果
const API_BASE = 'https://alou-edge.yuanjieliu65.workers.dev';

async function testAsyncTask() {
    console.log('🚀 开始测试异步多步骤任务修复...');

    try {
        // 1. 创建一个带有工具调用的任务（使用兼容性API）
        const taskData = {
            prompt: "请帮我分析一下今天的比特币价格，然后告诉我是否应该买入。",
            model: "deepseek-chat",
            tools: [
                {
                    name: "get_bitcoin_price",
                    description: "获取比特币当前价格",
                    parameters: {
                        type: "object",
                        properties: {},
                        required: []
                    }
                },
                {
                    name: "analyze_market_trend",
                    description: "分析市场趋势",
                    parameters: {
                        type: "object",
                        properties: {
                            current_price: {
                                type: "number",
                                description: "当前价格"
                            }
                        },
                        required: ["current_price"]
                    }
                }
            ],
            system_prompt: "你是一个专业的金融分析师，擅长分析加密货币市场。",
            history: []
        };

        console.log('📝 创建任务...');
        const createResponse = await fetch(`${API_BASE}/api/agent/chat`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify(taskData)
        });

        if (!createResponse.ok) {
            const errorText = await createResponse.text();
            throw new Error(`创建任务失败: ${createResponse.status} - ${errorText}`);
        }

        const taskResult = await createResponse.json();
        console.log('✅ 任务创建成功:', taskResult);

        // 检查是否是异步任务
        if (taskResult.task_id) {
            const taskId = taskResult.task_id;
            console.log('🔄 这是一个异步任务，任务ID:', taskId);

            // 2. 轮询检查任务状态
            console.log('🔄 开始轮询任务状态...');

            let attempts = 0;
            const maxAttempts = 30; // 最多等待30次

            while (attempts < maxAttempts) {
                attempts++;

                const statusResponse = await fetch(`${API_BASE}/api/tasks/${taskId}`);
                if (!statusResponse.ok) {
                    console.error(`获取状态失败 (${statusResponse.status}):`, await statusResponse.text());
                    break;
                }

                const status = await statusResponse.json();
                console.log(`📊 尝试 ${attempts}: 状态=${status.status}, 进度=${status.progress || 0}, 步骤=${status.current_step || '未知'}`);

                if (status.status === 'completed') {
                    console.log('🎉 任务完成！');
                    console.log('📋 最终结果:', status.result);
                    return status;
                } else if (status.status === 'failed') {
                    console.error('❌ 任务失败:', status.error);
                    return status;
                } else if (status.status === 'processing') {
                    console.log('⏳ 任务正在处理中，等待工具结果...');
                }

                // 等待2秒再检查
                await new Promise(resolve => setTimeout(resolve, 2000));
            }

            console.log('⏰ 达到最大等待次数');
            return null;
        } else {
            // 同步任务，直接返回结果
            console.log('⚡ 这是一个同步任务，直接返回结果');
            return taskResult;
        }

    } catch (error) {
        console.error('❌ 测试失败:', error);
        return null;
    }
}

// 运行测试
testAsyncTask().then(result => {
    if (result && (result.status === 'completed' || result.success)) {
        console.log('✅ 测试通过！异步多步骤任务修复成功！');
    } else {
        console.log('❌ 测试失败或超时');
        console.log('结果详情:', result);
    }
});