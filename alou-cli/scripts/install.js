#!/usr/bin/env node

/**
 * Alou CLI 安装脚本
 * 负责编译 Rust 二进制文件
 */

const { execSync } = require('child_process');
const path = require('path');
const fs = require('fs');

const CLI_DIR = __dirname;
const BINARY_NAME = process.platform === 'win32' ? 'alou.exe' : 'alou';
const BINARY_PATH = path.join(CLI_DIR, 'bin', BINARY_NAME);

console.log('🔨 正在编译 Alou CLI...');

try {
  // 检查 Rust 是否已安装
  try {
    execSync('rustc --version', { stdio: 'pipe' });
  } catch (e) {
    console.error('❌ Rust 未安装。请先安装 Rust: https://rustup.rs/');
    process.exit(1);
  }

  // 编译 Release 版本
  console.log('📦 编译 Release 版本...');
  execSync('cargo build --release', {
    cwd: CLI_DIR,
    stdio: 'inherit',
    env: { ...process.env, RUST_BACKTRACE: '1' }
  });

  // 移动二进制文件
  const srcBinary = path.join(CLI_DIR, 'target', 'release', BINARY_NAME);
  const destBinary = BINARY_PATH;

  if (fs.existsSync(srcBinary)) {
    fs.copyFileSync(srcBinary, destBinary);
    
    // 设置可执行权限 (Unix)
    if (process.platform !== 'win32') {
      fs.chmodSync(destBinary, '755');
    }
    
    console.log(`✅ Alou CLI 编译成功: ${destBinary}`);
  } else {
    console.error('❌ 编译失败：找不到二进制文件');
    process.exit(1);
  }

} catch (error) {
  console.error('❌ 安装失败:', error.message);
  process.exit(1);
}
