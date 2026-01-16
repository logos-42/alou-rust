// 验证异步多步骤任务修复效果的测试脚本
const API_BASE = 'https://alou-edge.yuanjieliu65.workers.dev';

async function testBasicConnectivity() {
    console.log('🔍 测试基本连接...');

    try {
        const response = await fetch(`${API_BASE}/api/health`, {
            timeout: 5000
        });

        if (response.ok) {
            const data = await response.json();
            console.log('✅ API连接正常:', data);
            return true;
        } else {
            console.log('❌ API响应错误:', response.status, response.statusText);
            return false;
        }
    } catch (error) {
        console.log('❌ 连接失败:', error.message);
        return false;
    }
}

async function testAsyncTaskCreation() {
    console.log('\n🔄 测试异步任务创建...');

    try {
        // 创建一个简单的多步骤任务
        const taskData = {
            prompt: "请帮我分析一下当前的时间，并告诉我现在是上午还是下午。",
            model: "deepseek-chat",
            tools: [
                {
                    name: "get_current_time",
                    description: "获取当前时间",
                    parameters: {
                        type: "object",
                        properties: {},
                        required: []
                    }
                },
                {
                    name: "analyze_time_period",
                    description: "分析时间段",
                    parameters: {
                        type: "object",
                        properties: {
                            hour: {
                                type: "number",
                                description: "小时数"
                            }
                        },
                        required: ["hour"]
                    }
                }
            ],
            system_prompt: "你是一个时间分析助手。",
            history: []
        };

        console.log('📤 发送任务请求...');
        const response = await fetch(`${API_BASE}/api/agent/chat`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify(taskData),
            timeout: 10000
        });

        if (!response.ok) {
            const errorText = await response.text();
            console.log('❌ 任务创建失败:', response.status, errorText);
            return null;
        }

        const result = await response.json();
        console.log('📋 任务创建响应:', JSON.stringify(result, null, 2));

        if (result.task_id) {
            console.log('✅ 异步任务已创建，任务ID:', result.task_id);
            return result.task_id;
        } else if (result.success) {
            console.log('⚡ 同步任务已完成');
            return 'sync_completed';
        } else {
            console.log('❌ 意外的响应格式');
            return null;
        }

    } catch (error) {
        console.log('❌ 任务创建异常:', error.message);
        return null;
    }
}

async function testTaskStatus(taskId) {
    console.log(`\n📊 测试任务状态查询 (ID: ${taskId})...`);

    try {
        const response = await fetch(`${API_BASE}/api/tasks/${taskId}`, {
            timeout: 5000
        });

        if (!response.ok) {
            console.log('❌ 状态查询失败:', response.status, await response.text());
            return false;
        }

        const status = await response.json();
        console.log('📈 任务状态:', JSON.stringify(status, null, 2));
        return true;

    } catch (error) {
        console.log('❌ 状态查询异常:', error.message);
        return false;
    }
}

async function testMultiStepFlow() {
    console.log('\n🔄 测试完整多步骤流程...');

    // 1. 创建任务
    const taskId = await testAsyncTaskCreation();
    if (!taskId || taskId === 'sync_completed') {
        console.log('⚠️ 无法进行多步骤测试（任务未创建或已同步完成）');
        return false;
    }

    // 2. 检查初始状态
    await testTaskStatus(taskId);

    // 3. 等待一会儿，让alarm有机会触发
    console.log('⏳ 等待任务处理...');
    await new Promise(resolve => setTimeout(resolve, 3000));

    // 4. 再次检查状态
    await testTaskStatus(taskId);

    // 5. 如果任务还在处理中，尝试提交模拟的工具结果
    console.log('\n🔧 尝试提交工具结果...');
    try {
        const mockToolResult = {
            success: true,
            time: "14:25:00",
            period: "afternoon"
        };

        const toolResponse = await fetch(`${API_BASE}/api/tasks/${taskId}/tool-result`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify(mockToolResult),
            timeout: 5000
        });

        if (toolResponse.ok) {
            console.log('✅ 工具结果提交成功');
        } else {
            console.log('⚠️ 工具结果提交失败:', toolResponse.status, await toolResponse.text());
        }
    } catch (error) {
        console.log('⚠️ 工具结果提交异常:', error.message);
    }

    // 6. 最终状态检查
    await new Promise(resolve => setTimeout(resolve, 2000));
    await testTaskStatus(taskId);

    console.log('✅ 多步骤流程测试完成');
    return true;
}

async function runVerificationTests() {
    console.log('🚀 开始验证异步多步骤任务修复效果...\n');

    // 测试1: 基本连接
    const connectivityOk = await testBasicConnectivity();

    if (!connectivityOk) {
        console.log('\n❌ 基本连接测试失败，无法继续其他测试');
        return;
    }

    // 测试2: 多步骤流程
    await testMultiStepFlow();

    console.log('\n🎯 验证测试完成！');
    console.log('💡 如果看到异步任务能正确创建和处理，说明修复成功');
}

// 运行测试
runVerificationTests().catch(error => {
    console.error('💥 测试过程中发生未捕获的错误:', error);
});