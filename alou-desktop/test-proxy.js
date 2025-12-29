// 测试Vite代理配置
const http = require('http');

// 测试代理服务器是否正常工作
const testProxy = async () => {
  console.log('测试Vite代理配置...');
  
  const options = {
    hostname: 'localhost',
    port: 1420,
    path: '/api/health',
    method: 'GET',
    headers: {
      'Origin': 'http://localhost:1420',
      'Content-Type': 'application/json',
    }
  };

  return new Promise((resolve, reject) => {
    const req = http.request(options, (res) => {
      console.log(`状态码: ${res.statusCode}`);
      console.log(`响应头:`, res.headers);
      
      let data = '';
      res.on('data', (chunk) => {
        data += chunk;
      });
      
      res.on('end', () => {
        console.log('响应体:', data);
        resolve({ statusCode: res.statusCode, headers: res.headers, body: data });
      });
    });

    req.on('error', (error) => {
      console.error('请求错误:', error.message);
      reject(error);
    });

    req.end();
  });
};

// 运行测试
testProxy()
  .then(result => {
    console.log('\n✅ 代理测试完成');
    console.log('CORS头检查:');
    console.log('- Access-Control-Allow-Origin:', result.headers['access-control-allow-origin']);
    console.log('- Access-Control-Allow-Methods:', result.headers['access-control-allow-methods']);
    console.log('- Access-Control-Allow-Headers:', result.headers['access-control-allow-headers']);
    
    if (result.headers['access-control-allow-origin'] === 'http://localhost:1420') {
      console.log('\n🎉 CORS配置正确！');
    } else {
      console.log('\n⚠️ CORS头可能未正确设置');
    }
  })
  .catch(error => {
    console.error('\n❌ 代理测试失败:', error.message);
    console.log('请确保Vite开发服务器正在运行 (npm run dev)');
  });
