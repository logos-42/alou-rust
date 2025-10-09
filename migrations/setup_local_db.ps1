# ============================================
# 本地数据库设置脚本（Windows PostgreSQL）
# ============================================

$DB_NAME = "alou_pay"
$DB_USER = "alou_admin"
$DB_PASSWORD = "local_dev_password"

Write-Host ""
Write-Host "🗄️  设置本地PostgreSQL数据库..." -ForegroundColor Green
Write-Host ""

try {
    # 检查PostgreSQL服务是否运行
    $pgService = Get-Service -Name "postgresql*" -ErrorAction SilentlyContinue
    if ($null -eq $pgService) {
        Write-Host "❌ PostgreSQL服务未找到，请确保PostgreSQL已安装" -ForegroundColor Red
        exit 1
    }
    
    if ($pgService.Status -ne "Running") {
        Write-Host "⚠️  PostgreSQL服务未运行，正在启动..." -ForegroundColor Yellow
        Start-Service $pgService.Name
        Start-Sleep -Seconds 2
    }
    
    Write-Host "✅ PostgreSQL服务运行正常" -ForegroundColor Green
    Write-Host ""

    # 创建用户
    Write-Host "📝 创建数据库用户: $DB_USER" -ForegroundColor Cyan
    $createUserCmd = "CREATE USER $DB_USER WITH ENCRYPTED PASSWORD '$DB_PASSWORD';"
    $result = psql -U postgres -c $createUserCmd 2>&1
    
    if ($LASTEXITCODE -eq 0) {
        Write-Host "   ✅ 用户创建成功" -ForegroundColor Green
    } else {
        if ($result -match "already exists") {
            Write-Host "   ⚠️  用户已存在，跳过..." -ForegroundColor Yellow
        } else {
            Write-Host "   ❌ 用户创建失败: $result" -ForegroundColor Red
        }
    }

    # 创建数据库
    Write-Host "📝 创建数据库: $DB_NAME" -ForegroundColor Cyan
    $createDbCmd = "CREATE DATABASE $DB_NAME OWNER $DB_USER;"
    $result = psql -U postgres -c $createDbCmd 2>&1
    
    if ($LASTEXITCODE -eq 0) {
        Write-Host "   ✅ 数据库创建成功" -ForegroundColor Green
    } else {
        if ($result -match "already exists") {
            Write-Host "   ⚠️  数据库已存在，跳过..." -ForegroundColor Yellow
        } else {
            Write-Host "   ❌ 数据库创建失败: $result" -ForegroundColor Red
        }
    }

    # 授予权限
    Write-Host "📝 授予权限..." -ForegroundColor Cyan
    psql -U postgres -c "GRANT ALL PRIVILEGES ON DATABASE $DB_NAME TO $DB_USER;" | Out-Null
    psql -U postgres -d $DB_NAME -c "GRANT ALL ON SCHEMA public TO $DB_USER;" | Out-Null
    Write-Host "   ✅ 权限配置完成" -ForegroundColor Green

    # 检查迁移文件是否存在
    $sqlFile = "migrations\001_init.sql"
    if (-not (Test-Path $sqlFile)) {
        Write-Host ""
        Write-Host "❌ 找不到 $sqlFile" -ForegroundColor Red
        Write-Host "   请确保在项目根目录运行此脚本" -ForegroundColor Yellow
        Write-Host "   当前目录: $PWD" -ForegroundColor Yellow
        exit 1
    }

    # 执行初始化SQL
    Write-Host "📝 执行数据库迁移: $sqlFile" -ForegroundColor Cyan
    $migrationResult = psql -U $DB_USER -d $DB_NAME -f $sqlFile 2>&1
    
    if ($LASTEXITCODE -eq 0) {
        Write-Host "   ✅ 数据库迁移完成" -ForegroundColor Green
    } else {
        Write-Host "   ❌ 数据库迁移失败" -ForegroundColor Red
        Write-Host "   错误信息: $migrationResult" -ForegroundColor Red
        exit 1
    }

    Write-Host ""
    Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Green
    Write-Host "✅ 数据库设置完成！" -ForegroundColor Green
    Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Green
    Write-Host ""
    Write-Host "📝 连接信息：" -ForegroundColor Cyan
    Write-Host "   数据库名: $DB_NAME"
    Write-Host "   用户名: $DB_USER"
    Write-Host "   密码: $DB_PASSWORD"
    Write-Host "   连接字符串: postgresql://${DB_USER}:${DB_PASSWORD}@localhost:5432/${DB_NAME}"
    Write-Host ""
    Write-Host "🔧 .env 文件已包含正确的配置" -ForegroundColor Yellow
    Write-Host "   DATABASE_URL=postgresql://${DB_USER}:${DB_PASSWORD}@localhost:5432/${DB_NAME}"
    Write-Host ""
    Write-Host "🧪 验证表结构：" -ForegroundColor Cyan
    Write-Host ""
    psql -U $DB_USER -d $DB_NAME -c "\dt"
    
    Write-Host ""
    Write-Host "📊 users 表结构：" -ForegroundColor Cyan
    Write-Host ""
    psql -U $DB_USER -d $DB_NAME -c "\d users"
    
    Write-Host ""
    Write-Host "🎉 下一步：" -ForegroundColor Green
    Write-Host "   1. 编辑 .env 文件，填入真实的 GOOGLE_CLIENT_SECRET" -ForegroundColor Yellow
    Write-Host "   2. 运行 cargo build 安装后端依赖" -ForegroundColor Yellow
    Write-Host "   3. 进入 frontend 目录运行 npm install" -ForegroundColor Yellow
    Write-Host ""
    
} catch {
    Write-Host ""
    Write-Host "❌ 设置失败: $_" -ForegroundColor Red
    Write-Host ""
    Write-Host "💡 故障排除：" -ForegroundColor Yellow
    Write-Host "   1. 确保PostgreSQL服务正在运行" -ForegroundColor Yellow
    Write-Host "   2. 检查postgres用户是否需要密码" -ForegroundColor Yellow
    Write-Host "   3. 尝试手动执行: psql -U postgres -c 'SELECT version();'" -ForegroundColor Yellow
    Write-Host ""
    exit 1
}

