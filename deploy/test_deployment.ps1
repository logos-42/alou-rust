# Windows部署测试脚本
# PowerShell版本

Write-Host "🧪 测试部署配置..." -ForegroundColor Green

# 检查项目结构
Write-Host "📁 检查项目结构..."
if (-not (Test-Path "Cargo.toml")) {
    Write-Host "❌ 未找到Cargo.toml，请在项目根目录运行" -ForegroundColor Red
    exit 1
}

if (-not (Test-Path "frontend")) {
    Write-Host "❌ 未找到frontend目录" -ForegroundColor Red
    exit 1
}

Write-Host "✅ 项目结构正确" -ForegroundColor Green

# 测试后端构建
Write-Host "🦀 测试后端构建..."
try {
    $buildResult = cargo build --release --bin agent-http-server 2>&1
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✅ 后端构建成功" -ForegroundColor Green
    } else {
        Write-Host "❌ 后端构建失败: $buildResult" -ForegroundColor Red
        exit 1
    }
} catch {
    Write-Host "❌ 后端构建异常: $_" -ForegroundColor Red
    exit 1
}

# 测试前端构建
Write-Host "🎨 测试前端构建..."
try {
    Set-Location frontend
    npm run build
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✅ 前端构建成功" -ForegroundColor Green
    } else {
        Write-Host "❌ 前端构建失败" -ForegroundColor Red
        Set-Location ..
        exit 1
    }
    Set-Location ..
} catch {
    Write-Host "❌ 前端构建异常: $_" -ForegroundColor Red
    Set-Location ..
    exit 1
}

# 检查构建产物
Write-Host "📦 检查构建产物..."

$backendBinary = "target/release/agent-http-server.exe"
if (Test-Path $backendBinary) {
    $size = (Get-Item $backendBinary).Length
    Write-Host "✅ 后端二进制文件: $backendBinary (${size} bytes)" -ForegroundColor Green
} else {
    Write-Host "❌ 未找到后端二进制文件: $backendBinary" -ForegroundColor Red
}

$frontendDist = "frontend/dist"
if (Test-Path $frontendDist) {
    $files = Get-ChildItem $frontendDist -Recurse | Measure-Object
    Write-Host "✅ 前端构建文件: $frontendDist ($($files.Count) 个文件)" -ForegroundColor Green
} else {
    Write-Host "❌ 未找到前端构建目录: $frontendDist" -ForegroundColor Red
}

# 检查配置文件
Write-Host "⚙️ 检查配置文件..."

$configs = @(
    "agent_config.json",
    "mcp.json"
)

foreach ($config in $configs) {
    if (Test-Path $config) {
        Write-Host "✅ 找到配置文件: $config" -ForegroundColor Green
    } else {
        Write-Host "⚠️ 配置文件不存在: $config (将使用默认配置)" -ForegroundColor Yellow
    }
}

# 检查部署脚本
Write-Host "📜 检查部署脚本..."
$scripts = @(
    "deploy/cloud_deploy.sh",
    "deploy/manage.sh"
)

foreach ($script in $scripts) {
    if (Test-Path $script) {
        Write-Host "✅ 找到部署脚本: $script" -ForegroundColor Green
    } else {
        Write-Host "❌ 缺少部署脚本: $script" -ForegroundColor Red
    }
}

# 生成部署摘要
Write-Host ""
Write-Host "📋 部署摘要" -ForegroundColor Cyan
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

Write-Host "🎯 目标服务器: 140.179.186.11"
Write-Host "🔌 后端端口: 3001"
Write-Host "🌐 前端端口: 80 (通过Nginx)"

Write-Host ""
Write-Host "📁 部署文件列表:"
Write-Host "  后端: target/release/agent-http-server.exe → /opt/alou-pay/backend/"
Write-Host "  前端: frontend/dist/* → /opt/alou-pay/frontend/"
Write-Host "  配置: agent_config.json, mcp.json → /opt/alou-pay/backend/"

Write-Host ""
Write-Host "🚀 云服务器部署步骤:"
Write-Host "1. 上传项目文件到服务器"
Write-Host "2. 在服务器上运行: chmod +x deploy/cloud_deploy.sh"
Write-Host "3. 设置环境变量: export DEEPSEEK_API_KEY='your-key'"
Write-Host "4. 执行部署: ./deploy/cloud_deploy.sh"

Write-Host ""
Write-Host "🔧 服务管理命令:"
Write-Host "  ./deploy/manage.sh status   # 查看服务状态"
Write-Host "  ./deploy/manage.sh health   # 健康检查"
Write-Host "  ./deploy/manage.sh logs     # 查看日志"
Write-Host "  ./deploy/manage.sh restart  # 重启服务"

Write-Host ""
Write-Host "🌐 部署完成后访问:"
Write-Host "  前端: http://140.179.186.11"
Write-Host "  API:  http://140.179.186.11/api/health"

Write-Host ""
Write-Host "✅ 部署测试完成！项目已准备好部署到云服务器。" -ForegroundColor Green
