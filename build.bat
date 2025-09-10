@echo off
REM Alou3 Rust 构建脚本 (Windows)

echo 🚀 开始构建 Alou3 Rust...

REM 检查Rust是否安装
where cargo >nul 2>nul
if %errorlevel% neq 0 (
    echo ❌ Rust未安装，请先安装Rust: https://rustup.rs/
    exit /b 1
)

REM 检查Rust版本
for /f "tokens=2" %%i in ('rustc --version') do set RUST_VERSION=%%i
echo 📦 Rust版本: %RUST_VERSION%

REM 检查环境变量
if "%DEEPSEEK_API_KEY%"=="" (
    echo ⚠️  警告: 未设置 DEEPSEEK_API_KEY 环境变量
    echo    请设置您的DeepSeek API密钥:
    echo    set DEEPSEEK_API_KEY=your_api_key_here
    echo.
)

REM 构建项目
echo 🔨 构建项目...
cargo build --release

if %errorlevel% equ 0 (
    echo ✅ 构建成功！
    echo.
    echo 🎯 使用方法:
    echo   .\target\release\alou3-rust.exe --help
    echo   .\target\release\alou3-rust.exe chat
    echo   .\target\release\alou3-rust.exe exec "读取文件 C:\path\to\file.txt"
    echo.
    echo 📖 更多信息请查看 README.md
) else (
    echo ❌ 构建失败
    exit /b 1
)
