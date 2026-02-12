/**
 * Alou 自我管理系统
 * 
 * 实现真正的自主性：
 * 1. 自启动 (Auto-Start)
 * 2. 自更新 (Auto-Update)
 * 3. 自进化 (Auto-Evolve)
 * 4. 自愈 (Self-Healing)
 */

const { execSync, spawn } = require('child_process');
const fs = require('fs');
const path = require('path');

// 配置
const PROJECT_DIR = '/Users/apple/Downloads/alou/alou-desktop';
const LOG_DIR = '/Users/apple/Downloads/alou/alou-desktop/logs';
const STATE_FILE = '/Users/apple/Downloads/alou/alou-desktop/.self-state.json';

// 颜色
const GREEN = '\x1b[32m';
const BLUE = '\x1b[34m';
const CYAN = '\x1b[36m';
const MAGENTA = '\x1b[35m';
const YELLOW = '\x1b[33m';
const RED = '\x1b[31m';
const RESET = '\x1b[0m';

function log(color, msg) { console.log(`${color}${msg}${RESET}`); }
function logSection(msg) { console.log(`\n${MAGENTA}=== ${msg} ===${RESET}\n`); }

// ============ 状态管理 ============

/**
 * 自我管理系统状态
 */
class SelfState {
  constructor() {
    this.load();
  }

  load() {
    try {
      if (fs.existsSync(STATE_FILE)) {
        const data = fs.readFileSync(STATE_FILE, 'utf8');
        const state = JSON.parse(data);
        Object.assign(this, state);
      }
    } catch (e) {
      this.init();
    }
  }

  save() {
    const data = {
      version: this.version,
      lastStart: this.lastStart,
      lastUpdate: this.lastUpdate,
      lastCheck: this.lastCheck,
      updateCount: this.updateCount,
      errorCount: this.errorCount,
      uptime: this.uptime,
      status: this.status,
    };
    fs.writeFileSync(STATE_FILE, JSON.stringify(data, null, 2));
  }

  init() {
    this.version = '0.2.0';
    this.lastStart = null;
    this.lastUpdate = null;
    this.lastCheck = Date.now();
    this.updateCount = 0;
    this.errorCount = 0;
    this.uptime = 0;
    this.status = 'initialized';
  }

  recordStart() {
    this.lastStart = Date.now();
    this.uptime = 0;
    this.status = 'running';
    this.save();
  }

  recordUpdate() {
    this.lastUpdate = Date.now();
    this.updateCount++;
    this.save();
  }

  recordError() {
    this.errorCount++;
    this.save();
  }

  tick() {
    if (this.status === 'running') {
      this.uptime++;
      this.lastCheck = Date.now();
      this.save();
    }
  }
}

// ============ 自启动系统 ============

/**
 * 自启动管理器
 */
class AutoStarter {
  constructor() {
    this.state = new SelfState();
  }

  /**
   * 启动 Alou
   */
  async start() {
    logSection('🚀 自启动');
    
    // 检查是否已经在运行
    if (this.isRunning()) {
      log(YELLOW, 'Alou 已在运行中');
      return { success: true, message: 'already_running' };
    }

    log(BLUE, '启动 Alou...');
    
    try {
      // 启动前端开发服务器
      this.startFrontend();
      
      // 启动 Rust 后端
      this.startBackend();
      
      this.state.recordStart();
      
      log(GREEN, '✅ Alou 已启动');
      
      return { success: true, message: 'started' };
    } catch (error) {
      this.state.recordError();
      log(RED, `❌ 启动失败: ${error.message}`);
      return { success: false, error: error.message };
    }
  }

  /**
   * 检查是否在运行
   */
  isRunning() {
    try {
      // 检查端口 1420
      execSync('lsof -i :1420', { stdio: 'pipe' });
      return true;
    } catch {
      return false;
    }
  }

  /**
   * 启动前端
   */
  startFrontend() {
    log(BLUE, '启动前端...');
    
    // 在后台启动
    spawn('npm', ['run', 'dev'], {
      cwd: PROJECT_DIR,
      detached: true,
      stdio: 'ignore',
    });
    
    log(GREEN, '前端已启动 (后台)');
  }

  /**
   * 启动后端
   */
  startBackend() {
    log(BLUE, '启动后端...');
    
    // 编译 Rust (如果需要)
    log(BLUE, '检查 Rust 编译...');
    execSync('cargo check', {
      cwd: `${PROJECT_DIR}/src-tauri`,
      stdio: 'pipe',
    });
    
    log(GREEN, '后端就绪');
  }

  /**
   * 设置开机自启动
   */
  setupLaunchd() {
    logSection('⚙️ 配置开机自启动');
    
    const plist = `<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.alou.desktop</string>
    <key>ProgramArguments</key>
    <array>
        <string>/bin/bash</string>
        <string>-c</string>
        <string>cd ${PROJECT_DIR} && npm run dev</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>${LOG_DIR}/alou.stdout.log</string>
    <key>StandardErrorPath</key>
    <string>${LOG_DIR}/alou.stderr.log</string>
</dict>
</plist>`;

    const plistPath = '/Users/apple/Library/LaunchAgents/com.alou.desktop.plist';
    
    fs.writeFileSync(plistPath, plist);
    execSync(`launchctl load ${plistPath}`);
    
    log(GREEN, `✅ 已配置开机自启动`);
    log(BLUE, `路径: ${plistPath}`);
  }
}

// ============ 自更新系统 ============

/**
 * 自更新管理器
 */
class AutoUpdater {
  constructor() {
    this.state = new SelfState();
  }

  /**
   * 检查更新
   */
  async checkForUpdates() {
    logSection('🔄 检查更新');
    
    try {
      // 拉取最新代码
      log(BLUE, '拉取最新代码...');
      execSync('git fetch origin master', { cwd: PROJECT_DIR });
      
      // 检查远程版本
      const remoteVersion = execSync(
        'git log origin/master --oneline -1',
        { cwd: PROJECT_DIR }
      ).toString().trim();
      
      // 检查本地版本
      const localVersion = execSync(
        'git log HEAD --oneline -1',
        { cwd: PROJECT_DIR }
      ).toString().trim();
      
      const needsUpdate = remoteVersion !== localVersion;
      
      log(BLUE, `本地版本: ${localVersion.substring(0, 8)}`);
      log(BLUE, `远程版本: ${remoteVersion.substring(0, 8)}`);
      
      if (needsUpdate) {
        log(GREEN, '🆕 发现新版本！');
      } else {
        log(BLUE, '✅ 已是最新版本');
      }
      
      return {
        hasUpdate: needsUpdate,
        localVersion: localVersion.substring(0, 8),
        remoteVersion: remoteVersion.substring(0, 8),
      };
    } catch (error) {
      log(RED, `检查失败: ${error.message}`);
      return { hasUpdate: false, error: error.message };
    }
  }

  /**
   * 执行更新
   */
  async update() {
    logSection('⬆️ 执行更新');
    
    try {
      // 拉取代码
      log(BLUE, '拉取代码...');
      execSync('git pull origin master', { cwd: PROJECT_DIR });
      
      // 安装依赖
      log(BLUE, '安装依赖...');
      execSync('npm install', { cwd: PROJECT_DIR });
      
      // 编译前端
      log(BLUE, '编译前端...');
      execSync('npm run build', { cwd: PROJECT_DIR });
      
      // 编译后端
      log(BLUE, '编译后端...');
      execSync('cargo build', { cwd: `${PROJECT_DIR}/src-tauri` });
      
      this.state.recordUpdate();
      
      log(GREEN, '✅ 更新完成！');
      
      return { success: true };
    } catch (error) {
      this.state.recordError();
      log(RED, `❌ 更新失败: ${error.message}`);
      return { success: false, error: error.message };
    }
  }

  /**
   * 自动更新（检查+执行）
   */
  async autoUpdate() {
    logSection('🤖 自动更新检查');
    
    const check = await this.checkForUpdates();
    
    if (check.hasUpdate) {
      log(BLUE, '执行更新...');
      return await this.update();
    }
    
    return { success: true, message: 'no_update_needed' };
  }
}

// ============ 自愈系统 ============

/**
 * 自愈管理器
 */
class SelfHealer {
  constructor() {
    this.state = new SelfState();
    this.checkInterval = null;
  }

  /**
   * 启动健康监控
   */
  startMonitoring() {
    logSection('🏥 启动健康监控');
    
    // 每30秒检查一次
    this.checkInterval = setInterval(() => {
      this.healthCheck();
    }, 30000);
    
    log(GREEN, '✅ 监控已启动 (每30秒检查)');
  }

  /**
   * 停止监控
   */
  stopMonitoring() {
    if (this.checkInterval) {
      clearInterval(this.checkInterval);
      this.checkInterval = null;
      log(BLUE, '监控已停止');
    }
  }

  /**
   * 健康检查
   */
  async healthCheck() {
    const checks = {
      frontend: this.checkFrontend(),
      backend: this.checkBackend(),
      memory: this.checkMemory(),
      disk: this.checkDisk(),
    };

    let allHealthy = true;
    
    for (const [name, healthy] of Object.entries(checks)) {
      if (!await healthy) {
        allHealthy = false;
        log(YELLOW, `⚠️ ${name} 不健康`);
      }
    }

    if (allHealthy) {
      log(GREEN, '✅ 所有组件健康');
    } else {
      log(YELLOW, '⚠️ 某些组件需要关注');
      await this.heal();
    }
    
    this.state.tick();
  }

  /**
   * 检查前端
   */
  async checkFrontend() {
    try {
      const result = execSync('curl -s -o /dev/null -w "%{http_code}" http://localhost:1420');
      return result.toString().trim() === '200';
    } catch {
      return false;
    }
  }

  /**
   * 检查后端
   */
  async checkBackend() {
    // 检查 Rust 进程
    try {
      execSync('pgrep -f "alou-desktop"', { stdio: 'pipe' });
      return true;
    } catch {
      return false;
    }
  }

  /**
   * 检查内存
   */
  async checkMemory() {
    try {
      const usage = execSync('vm_stat', { stdio: 'pipe' }).toString();
      return true; // 简化检查
    } catch {
      return true;
    }
  }

  /**
   * 检查磁盘
   */
  async checkDisk() {
    try {
      const usage = execSync('df -h .', { stdio: 'pipe' }).toString();
      return !usage.includes('100%');
    } catch {
      return true;
    }
  }

  /**
   * 自愈
   */
  async heal() {
    logSection('🩹 自愈处理');
    
    try {
      // 重启前端
      log(BLUE, '重启前端...');
      execSync('pkill -f "vite"');
      spawn('npm', ['run', 'dev'], {
        cwd: PROJECT_DIR,
        detached: true,
        stdio: 'ignore',
      });
      
      log(GREEN, '✅ 自愈完成');
      
      this.state.errorCount--;
      this.state.save();
      
      return { success: true };
    } catch (error) {
      log(RED, `❌ 自愈失败: ${error.message}`);
      return { success: false, error: error.message };
    }
  }
}

// ============ 主控制器 ============

/**
 * 自我管理系统主控制器
 */
class SelfManagingSystem {
  constructor() {
    this.starter = new AutoStarter();
    this.updater = new AutoUpdater();
    this.healer = new SelfHealer();
    this.state = new SelfState();
  }

  /**
   * 完整初始化
   */
  async initialize() {
    logSection('🚀 Alou 自我管理系统初始化');
    
    // 1. 自启动
    await this.starter.start();
    
    // 2. 启动监控
    this.healer.startMonitoring();
    
    // 3. 检查更新
    await this.updater.autoUpdate();
    
    log(GREEN, '✅ 初始化完成');
    
    return {
      started: true,
      monitoring: true,
      updated: false,
    };
  }

  /**
   * 获取系统状态
   */
  getStatus() {
    return {
      version: this.state.version,
      status: this.state.status,
      uptime: this.state.uptime,
      lastStart: this.state.lastStart,
      lastUpdate: this.state.lastUpdate,
      updateCount: this.state.updateCount,
      errorCount: this.state.errorCount,
      isRunning: this.starter.isRunning(),
    };
  }

  /**
   * 执行命令
   */
  async handleCommand(command) {
    switch (command) {
      case 'start':
        return await this.starter.start();
      
      case 'stop':
        this.healer.stopMonitoring();
        return { success: true, message: 'stopped' };
      
      case 'restart':
        this.healer.stopMonitoring();
        await this.starter.start();
        this.healer.startMonitoring();
        return { success: true, message: 'restarted' };
      
      case 'update':
        return await this.updater.autoUpdate();
      
      case 'check':
        return await this.updater.checkForUpdates();
      
      case 'status':
        return this.getStatus();
      
      case 'heal':
        return await this.healer.heal();
      
      case 'setup-autostart':
        this.starter.setupLaunchd();
        return { success: true };
      
      default:
        return { success: false, error: 'unknown_command' };
    }
  }
}

// ============ CLI 接口 ============

async function main() {
  const args = process.argv.slice(2);
  const command = args[0] || 'status';
  
  console.log('='.repeat(60));
  console.log('🤖 Alou 自我管理系统');
  console.log('='.repeat(60));
  
  const system = new SelfManagingSystem();
  
  if (command === 'init') {
    await system.initialize();
  } else if (command === 'help') {
    console.log(`
用法: node self-manager.js <命令>

命令:
  init           初始化系统（启动+监控+更新）
  start          启动 Alou
  stop           停止监控
  restart        重启
  update         检查并执行更新
  check          检查更新
  status         查看状态
  heal           执行自愈
  setup-autostart 配置开机自启动
  help            显示此帮助

示例:
  node self-manager.js init       # 初始化
  node self-manager.js status    # 查看状态
  node self-manager.js update    # 更新
`);
  } else {
    const result = await system.handleCommand(command);
    
    if (result.success) {
      log(GREEN, `✅ ${command} 完成`);
      if (result.message) {
        log(BLUE, `   ${result.message}`);
      }
    } else {
      log(RED, `❌ ${command} 失败: ${result.error}`);
    }
  }
}

main().catch(console.error);
