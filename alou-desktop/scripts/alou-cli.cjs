#!/usr/bin/env node
/**
 * Alou CLI - 终端命令行工具
 * 
 * 用法: alou [命令] [参数]
 * 
 * 示例:
 *   alou start          # 启动自主循环
 *   alou stop           # 停止
 *   alou status         # 查看状态
 *   alou task add "测试任务" # 添加任务
 *   alou agent chat "你好"  # 和 Agent 对话
 */

const https = require('https');
const http = require('http');
const fs = require('fs');
const path = require('path');

// 配置
const CONFIG = {
  backendUrl: 'http://localhost:1420',
  frontendUrl: 'http://localhost:1420',
};

// 颜色输出
const colors = {
  reset: '\x1b[0m',
  bright: '\x1b[1m',
  dim: '\x1b[2m',
  green: '\x1b[32m',
  yellow: '\x1b[33m',
  red: '\x1b[31m',
  blue: '\x1b[34m',
  cyan: '\x1b[36m',
};

function log(color, msg) {
  console.log(`${color}${msg}${colors.reset}`);
}

function logSuccess(msg) { log(colors.green, `✅ ${msg}`); }
function logError(msg) { log(colors.red, `❌ ${msg}`); }
function logInfo(msg) { log(colors.blue, `ℹ️  ${msg}`); }
function logSection(msg) { console.log(`\n${colors.cyan}${colors.bright}=== ${msg} ===${colors.reset}\n`); }

// HTTP 请求
async function request(method, path, data = null) {
  return new Promise((resolve, reject) => {
    const url = new URL(path, CONFIG.backendUrl);
    const options = {
      hostname: url.hostname,
      port: url.port,
      path: url.pathname,
      method,
      headers: {
        'Content-Type': 'application/json',
      },
    };

    const req = http.request(options, (res) => {
      let body = '';
      res.on('data', chunk => body += chunk);
      res.on('end', () => {
        try {
          resolve(JSON.parse(body));
        } catch {
          resolve(body);
        }
      });
    });

    req.on('error', reject);
    if (data) {
      req.write(JSON.stringify(data));
    }
    req.end();
  });
}

// 命令实现
const commands = {
  // 自主循环控制
  async start() {
    logInfo('启动自主循环...');
    const result = await request('POST', '/start_autonomous_loop');
    if (result.success) {
      logSuccess(result.message);
    } else {
      logError(result.error || '启动失败');
    }
  },

  async stop() {
    logInfo('停止自主循环...');
    const result = await request('POST', '/stop_autonomous_loop');
    if (result.success) {
      logSuccess(result.message);
    } else {
      logError(result.error || '停止失败');
    }
  },

  async pause() {
    logInfo('暂停自主循环...');
    const result = await request('POST', '/pause_autonomous_loop');
    if (result.success) {
      logSuccess(result.message);
    } else {
      logError(result.error || '暂停失败');
    }
  },

  async resume() {
    logInfo('恢复自主循环...');
    const result = await request('POST', '/resume_autonomous_loop');
    if (result.success) {
      logSuccess(result.message);
    } else {
      logError(result.error || '恢复失败');
    }
  },

  async status() {
    logSection('自主循环状态');
    const result = await request('GET', '/get_autonomous_loop_state');
    
    if (!result) {
      logError('无法获取状态，后端可能未运行');
      return;
    }

    const statusText = result.is_running 
      ? (result.is_paused ? '已暂停' : '运行中')
      : '已停止';
    
    console.log(`状态: ${result.is_running ? colors.green : colors.red}${statusText}${colors.reset}`);
    console.log(`完成任务: ${result.tasks_completed}`);
    console.log(`失败任务: ${result.tasks_failed}`);
    console.log(`循环次数: ${result.total_iterations}`);
    console.log(`当前任务: ${result.current_task_id || '无'}`);
    
    if (result.config) {
      console.log(`\n配置:`);
      console.log(`  心跳间隔: ${result.config.heartbeat_interval_seconds}秒`);
      console.log(`  任务检查: ${result.config.task_check_interval_seconds}秒`);
      console.log(`  记忆保存: ${result.config.memory_save_interval_seconds}秒`);
    }
  },

  // 任务管理
  async task(args) {
    const action = args[0];
    
    if (action === 'add') {
      const title = args[1];
      const description = args[2] || '';
      const priority = args[3] || 'medium';
      
      if (!title) {
        logError('请提供任务标题');
        console.log('用法: alou task add "标题" "描述" [优先级]');
        return;
      }
      
      logInfo(`添加任务: ${title}`);
      const result = await request('POST', '/add_autonomous_task', {
        title,
        description,
        priority,
      });
      
      if (result.success) {
        logSuccess('任务已添加');
      } else {
        logError(result.error || '添加失败');
      }
    } else if (action === 'list') {
      logSection('任务队列');
      const result = await request('GET', '/list_tasks');
      console.log(JSON.stringify(result, null, 2));
    } else if (action === 'stats') {
      logSection('任务统计');
      const result = await request('GET', '/get_task_stats');
      console.log(JSON.stringify(result, null, 2));
    } else {
      logError(`未知任务操作: ${action}`);
      console.log('用法: alou task [add|list|stats]');
    }
  },

  // Agent 对话
  async agent(args) {
    const action = args[0];
    
    if (action === 'chat') {
      const message = args.slice(1).join(' ');
      
      if (!message) {
        logError('请提供对话内容');
        console.log('用法: alou agent chat "你好"');
        return;
      }
      
      logSection('Agent 对话');
      logInfo(`发送: ${message}`);
      
      const result = await request('POST', '/execute_ai_conversation', {
        message,
        stream: false,
      });
      
      if (result.success) {
        console.log(`\n回复: ${result.response}`);
      } else {
        logError(result.error || '对话失败');
      }
    } else if (action === 'config') {
      const configJson = args.slice(1).join(' ');
      if (!configJson) {
        logSection('Agent 配置');
        const result = await request('GET', '/get_agent_config');
        console.log(JSON.stringify(result, null, 2));
      } else {
        try {
          const config = JSON.parse(configJson);
          logInfo('更新配置...');
          const result = await request('POST', '/update_agent_config', config);
          if (result.success) {
            logSuccess('配置已更新');
          }
        } catch (e) {
          logError('无效的 JSON 配置');
        }
      }
    } else if (action === 'health') {
      logSection('Agent 健康检查');
      const result = await request('GET', '/health_check');
      console.log(JSON.stringify(result, null, 2));
    } else {
      logError(`未知 Agent 操作: ${action}`);
      console.log('用法: alou agent [chat|config|health]');
    }
  },

  // 开发工具
  async dev(args) {
    const action = args[0];
    
    if (action === 'build') {
      logSection('构建项目');
      logInfo('构建前端...');
      execSync('npm run build', { cwd: '/Users/apple/Downloads/alou/alou-desktop', stdio: 'inherit' });
      logSuccess('前端构建完成');
      
      logInfo('检查 Rust 编译...');
      try {
        execSync('cargo check', { cwd: '/Users/apple/Downloads/alou/alou-desktop/src-tauri', stdio: 'inherit' });
        logSuccess('Rust 检查通过');
      } catch {
        logInfo('Rust 有警告，但可以运行');
      }
    } else if (action === 'start') {
      logSection('启动开发服务器');
      logInfo('启动前端开发服务器...');
      execSync('npm run dev', { cwd: '/Users/apple/Downloads/alou/alou-desktop', stdio: 'inherit' });
    } else if (action === 'test') {
      logSection('运行测试');
      execSync('node scripts/test-autonomy-complete.cjs', { cwd: '/Users/apple/Downloads/alou/alou-desktop', stdio: 'inherit' });
    } else {
      logError(`未知开发操作: ${action}`);
      console.log('用法: alou dev [build|start|test]');
    }
  },

  // 进化 Alou
  async evolve(args) {
    const target = args[0];
    
    if (target === 'backend') {
      logSection('进化后端 (Rust)');
      logInfo('重新编译 Rust...');
      try {
        execSync('cargo build', { 
          cwd: '/Users/apple/Downloads/alou/alou-desktop/src-tauri', 
          stdio: 'inherit' 
        });
        logSuccess('后端编译完成');
      } catch (e) {
        logError('编译失败');
      }
    } else if (target === 'frontend') {
      logSection('进化前端');
      logInfo('重新构建前端...');
      execSync('npm run build', { 
        cwd: '/Users/apple/Downloads/alou/alou-desktop', 
        stdio: 'inherit' 
      });
      logSuccess('前端构建完成');
    } else if (target === 'full') {
      logSection('完全进化');
      this.dev(['build']);
    } else {
      logError(`未知进化目标: ${target}`);
      console.log('用法: alou evolve [backend|frontend|full]');
    }
  },

  // 帮助
  help() {
    console.log(`
${colors.cyan}${colors.bright}Alou CLI - AI Agent 命令行工具${colors.reset}

${colors.yellow}使用: alou [命令] [参数]${colors.reset}

${colors.cyan}自主循环控制:${colors.reset}
  start           启动自主循环
  stop            停止自主循环
  pause           暂停自主循环
  resume          恢复自主循环
  status          查看运行状态

${colors.cyan}任务管理:${colors.reset}
  task add <标题> [描述] [优先级]  添加任务
  task list                        查看任务队列
  task stats                       查看统计

${colors.cyan}Agent 对话:${colors.reset}
  agent chat <消息>               与 Agent 对话
  agent config [JSON]             查看/更新配置
  agent health                    健康检查

${colors.cyan}开发工具:${colors.reset}
  dev build        构建项目
  dev start       启动开发服务器
  dev test        运行测试

${colors.cyan}进化 Alou:${colors.reset}
  evolve backend  重新编译后端
  evolve frontend 重新构建前端
  evolve full     完全重新构建

${colors.cyan}其他:${colors.reset}
  help            显示此帮助
  open            在浏览器中打开

${colors.green}示例:${colors.reset}
  alou start
  alou task add "检查邮件" "检查未读邮件" high
  alou agent chat "你好，小星"
  alou status
  alou evolve full
`);
  },

  // 在浏览器打开
  async open() {
    logInfo('在浏览器中打开 Alou...');
    const { execSync } = require('child_process');
    execSync(`open "${CONFIG.frontendUrl}"`);
    logSuccess('已打开浏览器');
  },
};

// 主入口
async function main() {
  const command = process.argv[2];
  const args = process.argv.slice(3);

  // 显示横幅
  if (!['help', '-h', '--help'].includes(command)) {
    console.log(`${colors.cyan}${colors.bright}`);
    console.log('╔═══════════════════════════════════════╗');
    console.log('║     🤖 Alou CLI - AI Agent 终端      ║');
    console.log('╚═══════════════════════════════════════╝');
    console.log(colors.reset);
  }

  if (!command || command === 'help' || command === '-h' || command === '--help') {
    commands.help();
    return;
  }

  const cmd = commands[command];
  if (cmd) {
    try {
      await cmd(args);
    } catch (error) {
      logError(error.message);
      process.exit(1);
    }
  } else {
    logError(`未知命令: ${command}`);
    console.log(`运行 ${colors.green}alou help${colors.reset} 查看可用命令`);
    process.exit(1);
  }
}

main();
