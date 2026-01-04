#!/usr/bin/env node
/**
 * 简单网络连接测试
 */

import { exec } from 'child_process';
import { promisify } from 'util';

const execAsync = promisify(exec);

async function testNetwork() {
  console.log('=== 网络连接测试 ===\n');
  
  const testUrls = [
    'https://alou-edge.yuanjieliu65.workers.dev',
    'https://alou-edge.yuanjieliu65.workers.dev/api/health',
    'https://cloudflare.com',
    'https://google.com'
  ];
  
  for (const url of testUrls) {
    console.log(`测试: ${url}`);
    
    try {
      // 使用 curl 进行简单的 HTTP 请求
      const { stdout, stderr } = await execAsync(`curl -s -o /dev/null -w "%{http_code}" -I ${url}`, {
        timeout: 10000
      });
      
      console.log(`   HTTP 状态码: ${stdout.trim()}`);
      
      if (stderr) {
        console.log(`   错误输出: ${stderr}`);
      }
      
    } catch (error) {
      console.log(`   错误: ${error.message}`);
      
      if (error.code === 'ETIMEDOUT') {
        console.log('   超时 - 可能是网络问题或目标不可达');
      } else if (error.code === 'ENOTFOUND') {
        console.log('   域名解析失败 - 检查 DNS 设置');
      }
    }
    
    console.log('');
  }
  
  console.log('=== 测试完成 ===\n');
  console.log('建议:');
  console.log('1. 如果 Workers 返回 200，说明服务正常');
  console.log('2. 如果返回 5xx，可能是服务端错误');
  console.log('3. 如果返回 4xx，可能是请求格式问题');
  console.log('4. 如果连接失败，检查网络和防火墙设置');
}

testNetwork().catch(console.error);
