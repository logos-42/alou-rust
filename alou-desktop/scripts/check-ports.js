// 检查 IPFS 相关端口状态
import net from 'net';

// 需要检查的端口
const ports = [
  { port: 5001, service: 'IPFS API (默认)' },
  { port: 5002, service: 'IPFS API (备用)' },
  { port: 8080, service: 'IPFS Gateway (默认)' },
  { port: 8081, service: 'IPFS Gateway (备用)' },
  { port: 4001, service: 'IPFS Swarm (默认)' },
  { port: 4002, service: 'IPFS Swarm (备用)' },
];

function checkPort(portInfo) {
  return new Promise((resolve) => {
    const server = net.createServer();
    
    server.once('error', (err) => {
      if (err.code === 'EADDRINUSE') {
        resolve({ ...portInfo, status: '占用', available: false });
      } else {
        resolve({ ...portInfo, status: '错误', available: false, error: err.code });
      }
    });
    
    server.once('listening', () => {
      server.close();
      resolve({ ...portInfo, status: '空闲', available: true });
    });
    
    server.listen(portInfo.port, '127.0.0.1');
  });
}

async function checkAllPorts() {
  console.log('🔍 检查 IPFS 相关端口状态...\n');
  
  const results = [];
  for (const portInfo of ports) {
    const result = await checkPort(portInfo);
    results.push(result);
    
    const statusIcon = result.available ? '✅' : '❌';
    console.log(`${statusIcon} 端口 ${portInfo.port.toString().padEnd(5)} - ${portInfo.service.padEnd(25)}: ${result.status}`);
  }
  
  console.log('\n📋 总结：');
  
  // 检查默认端口
  const defaultApiPort = results.find(r => r.port === 5001);
  const defaultGatewayPort = results.find(r => r.port === 8080);
  
  if (!defaultApiPort.available) {
    console.log('⚠️  IPFS API 默认端口 5001 被占用');
    
    // 推荐备用端口
    const availableApiPort = results.find(r => r.service.includes('API') && r.available);
    if (availableApiPort) {
      console.log(`   建议使用端口 ${availableApiPort.port} 作为 IPFS API`);
      console.log(`   在 .env.local 中添加：VITE_IPFS_API_URL=http://127.0.0.1:${availableApiPort.port}`);
    }
  }
  
  if (!defaultGatewayPort.available) {
    console.log('⚠️  IPFS Gateway 默认端口 8080 被占用');
    
    // 推荐备用端口
    const availableGatewayPort = results.find(r => r.service.includes('Gateway') && r.available);
    if (availableGatewayPort) {
      console.log(`   建议使用端口 ${availableGatewayPort.port} 作为 IPFS Gateway`);
      console.log(`   在 .env.local 中添加：VITE_IPFS_GATEWAY_URL=http://127.0.0.1:${availableGatewayPort.port}`);
    }
  }
  
  if (defaultApiPort.available && defaultGatewayPort.available) {
    console.log('✅ 所有默认端口都可用，可以直接启动 Alou');
  }
  
  console.log('\n💡 解决方案：');
  console.log('1. 停止占用端口的进程（推荐）');
  console.log('2. 修改 Alou 使用的端口（通过环境变量）');
  console.log('3. 查看 IPFS_PORT_CONFLICT_SOLUTION.md 获取详细解决方案');
}

// 运行检查
checkAllPorts().catch(console.error);
