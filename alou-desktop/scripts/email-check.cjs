/**
 * 邮箱自动检查工具
 * 
 * 功能：
 * - 检查 QQ 邮箱未读邮件
 * - 统计邮件数量
 * - 标记重要邮件
 * - 定期自动检查
 */

import { imap } from 'imap-simple';
import { simpleParser } from 'mailparser';

// QQ邮箱 IMAP 配置
const IMAP_CONFIG = {
  user: '2844169590@qq.com',
  password: 'wjjfbzhqnwthbijh', // QQ邮箱授权码
  host: 'imap.qq.com',
  port: 993,
  tls: true,
  authTimeout: 3000,
};

// 颜色输出
const GREEN = '\x1b[32m';
const YELLOW = '\x1b[33m';
const BLUE = '\x1b[34m';
const CYAN = '\x1b[36m';
const RESET = '\x1b[0m';

function log(color, msg) {
  console.log(`${color}${msg}${RESET}`);
}

function logSection(msg) {
  console.log(`\n${CYAN}=== ${msg} ===${RESET}\n`);
}

/**
 * 连接邮箱
 */
async function connectMailbox() {
  const config = { ...IMAP_CONFIG };
  
  try {
    const connection = await imap.connect(config);
    await connection.openBox('INBOX');
    return connection;
  } catch (error) {
    log(YELLOW, `连接失败: ${error.message}`);
    return null;
  }
}

/**
 * 获取未读邮件数量
 */
async function getUnreadCount(connection) {
  try {
    const searchCriteria = ['UNSEEN'];
    const fetchOptions = { bodies: ['HEADER'], struct: true };
    
    const messages = await connection.search(searchCriteria, fetchOptions);
    return messages.length;
  } catch (error) {
    log(YELLOW, `获取未读邮件失败: ${error.message}`);
    return -1;
  }
}

/**
 * 获取最近邮件
 */
async function getRecentEmails(connection, limit = 5) {
  try {
    const searchCriteria = ['ALL'];
    const fetchOptions = {
      bodies: ['HEADER', 'TEXT'],
      struct: true,
      limit,
    };
    
    const messages = await connection.search(searchCriteria, fetchOptions);
    
    const emails = [];
    for (const message of messages.slice(-limit)) {
      const header = message.parts[0];
      const subject = header.subject?.[0] || '(无主题)';
      const from = header.from?.[0] || '(未知)';
      const date = header.date?.[0] || '';
      
      emails.push({
        subject,
        from,
        date: new Date(date).toLocaleString('zh-CN'),
      });
    }
    
    return emails.reverse();
  } catch (error) {
    log(YELLOW, `获取邮件失败: ${error.message}`);
    return [];
  }
}

/**
 * 检查邮箱完整流程
 */
async function checkEmail(options = {}) {
  const { verbose = false, limit = 5 } = options;
  
  logSection('📧 邮箱检查');
  log(BLUE, `用户: ${IMAP_CONFIG.user}`);
  
  // 连接
  if (verbose) log(BLUE, '正在连接...');
  const connection = await connectMailbox();
  
  if (!connection) {
    return {
      success: false,
      error: '连接失败',
      unreadCount: 0,
      emails: [],
    };
  }
  
  // 获取未读数量
  if (verbose) log(BLUE, '获取未读邮件...');
  const unreadCount = await getUnreadCount(connection);
  
  if (unreadCount < 0) {
    connection.end();
    return {
      success: false,
      error: '获取失败',
      unreadCount: 0,
      emails: [],
    };
  }
  
  // 获取最近邮件
  const recentEmails = await getRecentEmails(connection, limit);
  
  // 关闭连接
  connection.end();
  
  // 返回结果
  const result = {
    success: true,
    unreadCount,
    recentEmails,
    checkedAt: new Date().toISOString(),
  };
  
  // 输出结果
  log(GREEN, `✅ 未读邮件: ${unreadCount} 封`);
  
  if (verbose && recentEmails.length > 0) {
    log(BLUE, `\n最近邮件:`);
    recentEmails.forEach((email, i) => {
      console.log(`  ${i + 1}. [${email.date}]`);
      console.log(`     来自: ${email.from}`);
      console.log(`     主题: ${email.subject}`);
    });
  }
  
  return result;
}

/**
 * 定期检查邮箱（自主循环调用）
 */
async function periodicCheck() {
  console.log('='.repeat(60));
  console.log('🤖 Alou 邮箱自动检查');
  console.log('='.repeat(60));
  
  const result = await checkEmail({ verbose: true });
  
  console.log('\n' + '='.repeat(60));
  console.log('📊 检查结果');
  console.log('='.repeat(60));
  
  if (result.success) {
    console.log(`未读邮件: ${result.unreadCount} 封`);
    console.log(`检查时间: ${new Date(result.checkedAt).toLocaleString('zh-CN')}`);
    
    if (result.unreadCount > 0) {
      console.log(`\n${YELLOW}有新邮件！建议回复。${RESET}`);
    }
  } else {
    console.log(`❌ 检查失败: ${result.error}`);
  }
  
  return result;
}

// 导出功能
module.exports = {
  checkEmail,
  periodicCheck,
  connectMailbox,
  getUnreadCount,
  getRecentEmails,
};

// CLI 运行
if (require.main === module) {
  const args = process.argv.slice(2);
  const verbose = args.includes('-v') || args.includes('--verbose');
  
  periodicCheck().catch(console.error);
}
