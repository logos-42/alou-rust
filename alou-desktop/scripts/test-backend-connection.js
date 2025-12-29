// 测试后端连接
import axios from 'axios'

async function testConnection(url) {
  try {
    console.log(`测试连接: ${url}`)
    const response = await axios.get(url, { 
      timeout: 10000,
      headers: {
        'User-Agent': 'Alou-Desktop-Test/1.0'
      }
    })
    console.log(`✅ 成功: ${url}`)
    console.log(`   状态码: ${response.status}`)
    console.log(`   响应: ${JSON.stringify(response.data, null, 2)}`)
    return { success: true, url, data: response.data }
  } catch (error) {
    console.log(`❌ 失败: ${url}`)
    console.log(`   错误: ${error.code || '未知'} - ${error.message}`)
    
    if (error.response) {
      console.log(`   响应状态: ${error.response.status}`)
      console.log(`   响应数据: ${JSON.stringify(error.response.data, null, 2)}`)
    }
    
    return { success: false, url, error: error.message }
  }
}

async function testAllEndpoints() {
  console.log('🔍 测试后端连接...\n')
  
  const endpoints = [
    'https://alou-edge.yuanjieliu65.workers.dev/api/health',
    'https://alou-edge.yuanjieliu65.workers.dev/api/session',
    'http://localhost:8787/api/health',
    'http://localhost:3000/api/health',
  ]
  
  const results = []
  for (const url of endpoints) {
    const result = await testConnection(url)
    results.push(result)
    console.log('') // 空行分隔
  }
  
  console.log('📋 总结：')
  
  const successful = results.filter(r => r.success)
  const failed = results.filter(r => !r.success)
  
  console.log(`成功: ${successful.length}/${results.length}`)
  console.log(`失败: ${failed.length}/${results.length}`)
  
  if (successful.length > 0) {
    console.log('\n🎉 找到可用的后端服务器：')
    successful.forEach(r => {
      console.log(`  - ${r.url}`)
    })
    
    // 推荐配置
    const recommended = successful[0].url.replace('/api/health', '')
    console.log(`\n💡 建议配置：`)
    console.log(`在 .env.local 中添加：VITE_API_BASE_URL=${recommended}`)
  } else {
    console.log('\n⚠️  所有后端都不可用')
    console.log('💡 建议：')
    console.log('1. 检查网络连接')
    console.log('2. 启动本地后端服务器')
    console.log('3. 使用应用的离线模式')
  }
  
  return results
}

// 运行测试
testAllEndpoints().catch(console.error)
