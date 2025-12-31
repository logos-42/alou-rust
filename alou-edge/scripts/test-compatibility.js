// 兼容性测试脚本
// 用于验证Durable Objects架构的核心功能

import { execSync } from 'child_process';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

console.log('=== Durable Objects兼容性测试 ===');

// 检查关键文件是否存在
const requiredFiles = [
    'wrangler.toml',
    'src/durable_objects/ai_task.rs',
    'src/compatibility/models.rs',
    'src/compatibility/router.rs',
    'src/compatibility/tools/mod.rs',
    'src/compatibility/metrics.rs'
];

console.log('检查关键文件...');
for (const file of requiredFiles) {
    if (fs.existsSync(path.join(__dirname, '..', file))) {
        console.log(`✓ ${file}`);
    } else {
        console.log(`✗ ${file} - 文件不存在`);
        process.exit(1);
    }
}

// 检查wrangler.toml中的Durable Objects配置
console.log('\n检查wrangler.toml配置...');
const wranglerConfig = fs.readFileSync(path.join(__dirname, '..', 'wrangler.toml'), 'utf8');
if (wranglerConfig.includes('[[durable_objects.bindings]]')) {
    console.log('✓ Durable Objects绑定配置存在');
} else {
    console.log('✗ Durable Objects绑定配置不存在');
    process.exit(1);
}

if (wranglerConfig.includes('class_name = "AITaskDO"')) {
    console.log('✓ AITaskDO类配置存在');
} else {
    console.log('✗ AITaskDO类配置不存在');
    process.exit(1);
}

// 检查数据结构
console.log('\n检查数据结构...');
const modelsContent = fs.readFileSync(path.join(__dirname, '..', 'src/compatibility/models.rs'), 'utf8');
if (modelsContent.includes('struct CompatibleRequest')) {
    console.log('✓ CompatibleRequest结构体存在');
} else {
    console.log('✗ CompatibleRequest结构体不存在');
    process.exit(1);
}

if (modelsContent.includes('struct CompatibleResponse')) {
    console.log('✓ CompatibleResponse结构体存在');
} else {
    console.log('✗ CompatibleResponse结构体不存在');
    process.exit(1);
}

if (modelsContent.includes('enum TaskStatus')) {
    console.log('✓ TaskStatus枚举存在');
} else {
    console.log('✗ TaskStatus枚举不存在');
    process.exit(1);
}

// 检查路由
console.log('\n检查路由配置...');
const routerContent = fs.readFileSync(path.join(__dirname, '..', 'src/router/mod.rs'), 'utf8');
if (routerContent.includes('/api/agent/chat')) {
    console.log('✓ /api/agent/chat路由存在');
} else {
    console.log('✗ /api/agent/chat路由不存在');
    process.exit(1);
}

if (routerContent.includes('handle_compatible_chat')) {
    console.log('✓ 兼容性聊天处理器存在');
} else {
    console.log('✗ 兼容性聊天处理器不存在');
    process.exit(1);
}

// 检查工具框架
console.log('\n检查工具框架...');
const toolsContent = fs.readFileSync(path.join(__dirname, '..', 'src/compatibility/tools/mod.rs'), 'utf8');
if (toolsContent.includes('struct ToolExecutor')) {
    console.log('✓ ToolExecutor存在');
} else {
    console.log('✗ ToolExecutor不存在');
    process.exit(1);
}

if (toolsContent.includes('trait CompatibleTool')) {
    console.log('✓ CompatibleTool特征存在');
} else {
    console.log('✗ CompatibleTool特征不存在');
    process.exit(1);
}

// 检查监控指标
console.log('\n检查监控指标...');
const metricsContent = fs.readFileSync(path.join(__dirname, '..', 'src/compatibility/metrics.rs'), 'utf8');
if (metricsContent.includes('struct CompatibilityMetrics')) {
    console.log('✓ CompatibilityMetrics存在');
} else {
    console.log('✗ CompatibilityMetrics不存在');
    process.exit(1);
}

// 测试编译
console.log('\n测试编译...');
try {
    // 尝试编译Rust代码
    console.log('编译Rust代码...');
    execSync('cargo check --lib', { cwd: path.join(__dirname, '..'), stdio: 'inherit' });
    console.log('✓ Rust代码编译成功');
} catch (error) {
    console.log('✗ Rust代码编译失败');
    process.exit(1);
}

// 生成架构摘要
console.log('\n=== 架构摘要 ===');
console.log('1. Durable Objects配置: 已配置');
console.log('2. 兼容性数据结构: 已实现');
console.log('3. 路由层: 已实现');
console.log('4. 工具框架: 已实现');
console.log('5. 监控指标: 已实现');
console.log('6. 编译检查: 通过');

console.log('\n=== 部署建议 ===');
console.log('1. 运行部署脚本: scripts/deploy-durable-objects.ps1');
console.log('2. 测试环境: 使用 --Environment development 参数');
console.log('3. 生产环境: 使用 --Environment production 参数');

console.log('\n=== 测试完成 ===');
console.log('所有核心组件检查通过！Durable Objects架构已准备就绪。');