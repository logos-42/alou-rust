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
const APP_PATH = '/Applications/Alou.app';
const BUILD_OUTPUT = `${PROJECT_DIR}/src-tauri/target/release/bundle/macos/Alou.app`;

// 获取当前 Git 分支
function getCurrentBranch() {
  try {
    return execSync('git rev-parse --abbrev-ref HEAD', { cwd: PROJECT_DIR }).toString().trim();
  } catch {
    return 'main';
  }
}
const CURRENT_BRANCH = getCurrentBranch();

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
        const data = JSON.parse(fs.readFileSync(STATE_FILE, 'utf8'));
        Object.assign(this, data);
      } else {
        this.init();
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
   * 检查是否在运行
   */
  isRunning() {
    try {
      // 检查端口 1420 (开发模式)
      execSync('lsof -i :1420', { stdio: 'pipe' });
      return true;
    } catch {
      // 检查应用是否在运行 (生产模式)
      try {
        execSync('pgrep -f "Alou"', { stdio: 'pipe' });
        return true;
      } catch {
        return false;
      }
    }
  }

  /**
   * 停止 Alou
   */
  stop() {
    logSection('🛑 停止 Alou');
    
    try {
      // 停止开发服务器
      log(BLUE, '停止开发服务器...');
      execSync('pkill -f "vite"');
      execSync('pkill -f "npm run dev"');
      
      // 停止生产版本
      log(BLUE, '停止生产版本...');
      execSync('pkill -f "Alou"');
      
      log(GREEN, '✅ Alou 已停止');
      return { success: true };
    } catch (error) {
      log(RED, `停止失败: ${error.message}`);
      return { success: false, error: error.message };
    }
  }

  /**
   * 启动 Alou (开发模式)
   */
  async start() {
    logSection('🚀 自启动 (开发模式)');
    
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
   * 启动前端 (开发模式)
   */
  startFrontend() {
    log(BLUE, '启动前端 (开发模式)...');
    
    // 在后台启动
    spawn('npm', ['run', 'dev'], {
      cwd: PROJECT_DIR,
      detached: true,
      stdio: 'ignore',
    });
    
    log(GREEN, '前端已启动 (后台)');
  }

  /**
   * 启动后端 (开发模式)
   */
  startBackend() {
    log(BLUE, '启动后端 (开发模式)...');
    
    // 开发模式只是检查编译
    execSync('cargo check', {
      cwd: `${PROJECT_DIR}/src-tauri`,
      stdio: 'pipe',
    });
    
    log(GREEN, '后端就绪');
  }

  /**
   * 启动生产版本
   */
  launchProduction() {
    logSection('🚀 启动生产版本');
    
    if (fs.existsSync(APP_PATH)) {
      log(BLUE, `启动 ${APP_PATH}...`);
      spawn('open', [APP_PATH], {
        detached: true,
        stdio: 'ignore',
      });
      
      log(GREEN, '✅ Alou 已启动');
      return { success: true };
    } else {
      log(RED, `应用不存在: ${APP_PATH}`);
      return { success: false, error: 'app_not_found' };
    }
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
      log(BLUE, `拉取最新代码 (${CURRENT_BRANCH})...`);
      execSync(`git fetch origin ${CURRENT_BRANCH}`, { cwd: PROJECT_DIR });
      
      // 检查远程版本
      const remoteVersion = execSync(
        `git log origin/${CURRENT_BRANCH} --oneline -1`,
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
        return { 
          success: true, 
          hasUpdate: true, 
          message: 'update_available',
          localVersion: localVersion.substring(0, 8),
          remoteVersion: remoteVersion.substring(0, 8),
        };
      } else {
        log(BLUE, '✅ 已是最新版本');
        return { 
          success: true, 
          hasUpdate: false, 
          message: 'up_to_date',
          localVersion: localVersion.substring(0, 8),
          remoteVersion: remoteVersion.substring(0, 8),
        };
      }
    } catch (error) {
      log(RED, `检查失败: ${error.message}`);
      return { hasUpdate: false, error: error.message };
    }
  }

  /**
   * 开发模式更新 (热更新)
   */
  async devUpdate() {
    logSection('🔄 开发模式更新');
    
    try {
      // 拉取代码
      log(BLUE, `拉取代码 (${CURRENT_BRANCH})...`);
      execSync(`git pull origin ${CURRENT_BRANCH}`, { cwd: PROJECT_DIR });
      
      // 安装依赖
      log(BLUE, '安装依赖...');
      execSync('npm install', { cwd: PROJECT_DIR });
      
      // 编译前端
      log(BLUE, '编译前端...');
      execSync('npm run build', { cwd: PROJECT_DIR });
      
      // 检查后端
      log(BLUE, '检查后端...');
      execSync('cargo check', { cwd: `${PROJECT_DIR}/src-tauri` });
      
      this.state.recordUpdate();
      
      log(GREEN, '✅ 开发更新完成！');
      
      return { success: true, mode: 'dev' };
    } catch (error) {
      this.state.recordError();
      log(RED, `❌ 更新失败: ${error.message}`);
      return { success: false, error: error.message };
    }
  }

  /**
   * 生产模式更新 (完整打包)
   */
  async productionUpdate() {
    logSection('📦 生产模式更新');
    
    try {
      // 1. 拉取代码
      log(BLUE, `1/5 拉取代码 (${CURRENT_BRANCH})...`);
      execSync(`git pull origin ${CURRENT_BRANCH}`, { cwd: PROJECT_DIR });
      
      // 2. 安装依赖
      log(BLUE, '2/5 安装依赖...');
      execSync('npm install', { cwd: PROJECT_DIR });
      
      // 3. 编译前端
      log(BLUE, '3/5 编译前端...');
      execSync('npm run build', { cwd: PROJECT_DIR });
      
      // 4. 打包应用
      log(BLUE, '4/5 打包应用...');
      execSync('cargo tauri build', { 
        cwd: `${PROJECT_DIR}/src-tauri`,
        stdio: 'inherit' 
      });
      
      // 5. 记录更新
      this.state.recordUpdate();
      
      log(GREEN, '✅ 生产打包完成！');
      log(BLUE, `   输出: ${BUILD_OUTPUT}`);
      
      return { success: true, mode: 'production', buildPath: BUILD_OUTPUT };
    } catch (error) {
      this.state.recordError();
      log(RED, `❌ 打包失败: ${error.message}`);
      return { success: false, error: error.message };
    }
  }

  /**
   * 安装到 Applications
   */
  async install() {
    logSection('📥 安装到 Applications');
    
    try {
      if (!fs.existsSync(BUILD_OUTPUT)) {
        throw new Error(`构建不存在: ${BUILD_OUTPUT}`);
      }
      
      // 备份旧版本
      const backupPath = `${APP_PATH}.backup.${Date.now()}`;
      if (fs.existsSync(APP_PATH)) {
        log(BLUE, '备份旧版本...');
        execSync(`mv "${APP_PATH}" "${backupPath}"`);
      }
      
      // 复制新版本
      log(BLUE, '复制新版本...');
      execSync(`cp -R "${BUILD_OUTPUT}" "${APP_PATH}"`);
      
      // 设置权限
      log(BLUE, '设置权限...');
      execSync(`chmod -R +x "${APP_PATH}/Contents/MacOS/"`);
      
      // 清理备份
      if (fs.existsSync(backupPath)) {
        log(BLUE, '清理备份...');
        execSync(`rm -rf "${backupPath}"`);
      }
      
      log(GREEN, `✅ 已安装到 ${APP_PATH}`);
      
      return { success: true, appPath: APP_PATH };
    } catch (error) {
      log(RED, `❌ 安装失败: ${error.message}`);
      return { success: false, error: error.message };
    }
  }

  /**
   * 自动更新（开发模式）
   */
  async autoUpdate() {
    return await this.devUpdate();
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
    // Ensure state is loaded
    if (!this.state) {
      this.state = new SelfState();
    }
    
    // Reload from file
    try {
      this.state.load();
    } catch (e) {
      // If reload fails, ensure we have a valid state
      if (!this.state.version) {
        this.state.init();
      }
    }
    
    return {
      success: true,
      version: this.state.version || 'unknown',
      status: this.state.status || 'unknown',
      uptime: this.state.uptime || 0,
      lastStart: this.state.lastStart || null,
      lastUpdate: this.state.lastUpdate || null,
      updateCount: this.state.updateCount || 0,
      errorCount: this.state.errorCount || 0,
      isRunning: this.starter.isRunning(),
      branch: CURRENT_BRANCH,
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
        return this.starter.stop();
      
      case 'restart':
        this.healer.stopMonitoring();
        this.starter.stop();
        await new Promise(r => setTimeout(r, 2000));
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
      
      case 'full-update':
        // 完整更新：停止 → 打包 → 安装 → 启动
        logSection('🔄 完整更新流程');
        
        // 1. 停止
        log(BLUE, '步骤 1/5: 停止运行...');
        this.healer.stopMonitoring();
        const stopResult = this.starter.stop();
        if (!stopResult.success) {
          return stopResult;
        }
        
        await new Promise(r => setTimeout(r, 2000));
        
        // 2. 打包
        log(BLUE, '步骤 2/5: 打包新版本...');
        const buildResult = await this.updater.productionUpdate();
        if (!buildResult.success) {
          log(RED, '打包失败，尝试开发更新...');
          return await this.updater.devUpdate();
        }
        
        // 3. 安装
        log(BLUE, '步骤 3/5: 安装新版本...');
        const installResult = await this.updater.install();
        
        // 4. 启动
        log(BLUE, '步骤 4/5: 启动新版本...');
        const launchResult = this.starter.launchProduction();
        
        // 5. 完成
        log(BLUE, '步骤 5/5: 完成');
        this.healer.startMonitoring();
        
        log(GREEN, '✅ 完整更新完成！');
        
        return {
          success: true,
          message: 'full_update_complete',
          installed: installResult.success,
          launched: launchResult.success,
        };
      
      case 'install':
        return await this.updater.install();
      
      case 'launch':
        return this.starter.launchProduction();
      
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
  console.log(`📂 分支: ${CURRENT_BRANCH}`);
  console.log('='.repeat(60));
  
  const system = new SelfManagingSystem();
  
  if (command === 'init') {
    await system.initialize();
  } else if (command === 'help') {
    console.log(`
用法: node self-manager.cjs <命令>

命令:
  init           初始化系统（启动+监控+更新）
  start          启动 Alou (开发模式)
  stop           停止 Alou
  restart        重启
  update         开发模式更新 (热更新)
  full-update    生产模式更新 (打包+安装+启动)
  check          检查更新
  status         查看状态
  heal           执行自愈
  install        安装到 Applications
  launch         启动生产版本
  setup-autostart 配置开机自启动
  help           显示此帮助

示例:
  node self-manager.cjs init         # 初始化
  node self-manager.cjs status      # 查看状态
  node self-manager.cjs update      # 开发更新 (热更新)
  node self-manager.cjs full-update # 生产更新 (完整打包)
`);
  } else {
    try {
      const result = await system.handleCommand(command);
      
      if (result && result.success) {
        log(GREEN, `✅ ${command} 完成`);
        
        // Show additional info for status
        if (command === 'status' && result.version) {
          console.log(`   版本: ${result.version}`);
          console.log(`   状态: ${result.status}`);
          console.log(`   运行中: ${result.isRunning}`);
          console.log(`   分支: ${result.branch}`);
          console.log(`   更新次数: ${result.updateCount}`);
        }
        // Show additional info for check
        else if (command === 'check') {
          if (result.hasUpdate === false) {
            log(BLUE, `   本地: ${result.localVersion || 'unknown'}`);
            log(BLUE, `   远程: ${result.remoteVersion || 'unknown'}`);
          }
        }
        // Show message if present
        else if (result.message) {
          log(BLUE, `   ${result.message}`);
        }
      } else if (result && !result.success) {
        log(RED, `❌ ${command} 失败: ${result.error || result.message}`);
      }
    } catch (error) {
      log(RED, `❌ 执行错误: ${error.message}`);
    }
  }
}

main().catch(console.error);
