#!/usr/bin/env node
/**
 * 简单测试 Workers 健康状态
 */

async function testWorkersHealth() {
  console.log('=== 测试 Workers 健康状态 ===\n');
  
  const workersUrl = 'https://alou-edge.yuanjieliu65.workers.dev';
  
  console.log('测试端点:', workersUrl);
  
  // 测试健康检查
  try {
    console.log('\n1. 发送健康检查请求...');
    const response = await fetch(`${workersUrl}/api/health`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({}),
    });
    
    console.log('   响应状态:', response.status, response.statusText);
    
    if (response.ok) {
      const data = await response.json();
      console.log('   ✅ 健康检查成功');
      console.log('   响应数据:', JSON.stringify(data, null, 2));
    } else {
      console.log('   ❌ 健康检查失败');
      const errorText = await response.text();
      console.log('   错误响应:', errorText.substring(0, 200));
    }
  } catch (error) {
    console.log('   ❌ 请求错误:', error.message);
    
    // 检查网络连接
    if (error.code === 'ETIMEDOUT' || error.code === 'ECONNREFUSED') {
      console.log('   🔍 网络连接问题:');
      console.log('   - 检查 Workers 域名是否正确');
      console.log('   - 检查网络防火墙设置');
      console.log('   - 尝试使用浏览器访问:', workersUrl);
    }
  }
  
  console.log('\n=== 测试完成 ===');
}

// 使用动态导入来避免 CommonJS/ESM 问题
import('node-fetch').then(({ default: fetch }) => {
  testWorkersHealth();
}).catch(error => {
  console.error('无法加载 node-fetch:', error);
  process.exit(1);
});
