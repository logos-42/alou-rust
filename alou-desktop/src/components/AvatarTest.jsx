import React, { useState, useEffect } from 'react'
import imageProxyService from '@/services/imageProxyService'

const AvatarTest = () => {
  const [testResults, setTestResults] = useState([])
  const [isLoading, setIsLoading] = useState(false)

  const testUrls = [
    'https://avatars.githubusercontent.com/u/16309930?v=4',
    'https://github.com/identicons/jasonlong.png',
    'https://via.placeholder.com/150/0000FF/808080?text=Test'
  ]

  const testAvatarLoading = async () => {
    setIsLoading(true)
    setTestResults([])
    
    console.log('[AvatarTest] 开始测试头像加载')
    console.log('[AvatarTest] 环境检测:', {
      isTauri: typeof window !== 'undefined' && window.__TAURI__,
      hasTauriHttp: typeof window !== 'undefined' && window.__TAURI__ && window.__TAURI__.http,
      userAgent: navigator.userAgent
    })

    const results = []
    
    for (const url of testUrls) {
      try {
        console.log(`[AvatarTest] 测试URL: ${url}`)
        const startTime = Date.now()
        
        const dataUrl = await imageProxyService.getImageDataUrl(url)
        const endTime = Date.now()
        
        results.push({
          url,
          success: !!dataUrl,
          dataUrl: dataUrl ? dataUrl.substring(0, 100) + '...' : null,
          size: dataUrl ? dataUrl.length : 0,
          time: endTime - startTime,
          error: null
        })
        
        console.log(`[AvatarTest] 成功: ${url}, 大小: ${dataUrl?.length}, 时间: ${endTime - startTime}ms`)
      } catch (error) {
        console.error(`[AvatarTest] 失败: ${url}`, error)
        results.push({
          url,
          success: false,
          dataUrl: null,
          size: 0,
          time: 0,
          error: error.message
        })
      }
    }
    
    setTestResults(results)
    setIsLoading(false)
  }

  return (
    <div style={{ padding: '20px', fontFamily: 'monospace' }}>
      <h2>Avatar Loading Test</h2>
      
      <div style={{ marginBottom: '20px' }}>
        <h3>Environment Info:</h3>
        <ul>
          <li>Is Tauri: {typeof window !== 'undefined' && window.__TAURI__ ? 'Yes' : 'No'}</li>
          <li>Has Tauri HTTP: {typeof window !== 'undefined' && window.__TAURI__ && window.__TAURI__.http ? 'Yes' : 'No'}</li>
          <li>User Agent: {navigator.userAgent}</li>
        </ul>
      </div>

      <button 
        onClick={testAvatarLoading} 
        disabled={isLoading}
        style={{ 
          padding: '10px 20px', 
          fontSize: '16px', 
          marginBottom: '20px',
          backgroundColor: isLoading ? '#ccc' : '#007bff',
          color: 'white',
          border: 'none',
          borderRadius: '4px',
          cursor: isLoading ? 'not-allowed' : 'pointer'
        }}
      >
        {isLoading ? 'Testing...' : 'Test Avatar Loading'}
      </button>

      {testResults.length > 0 && (
        <div>
          <h3>Test Results:</h3>
          {testResults.map((result, index) => (
            <div key={index} style={{ 
              marginBottom: '15px', 
              padding: '10px', 
              border: '1px solid #ddd',
              borderRadius: '4px',
              backgroundColor: result.success ? '#d4edda' : '#f8d7da'
            }}>
              <div><strong>URL:</strong> {result.url}</div>
              <div><strong>Success:</strong> {result.success ? 'Yes' : 'No'}</div>
              {result.success && (
                <>
                  <div><strong>Size:</strong> {result.size} bytes</div>
                  <div><strong>Time:</strong> {result.time}ms</div>
                  <div><strong>Data URL Preview:</strong> {result.dataUrl}</div>
                  <div style={{ marginTop: '10px' }}>
                    <img 
                      src={result.dataUrl.replace('...', '')} 
                      alt="Test Avatar" 
                      style={{ width: '50px', height: '50px', borderRadius: '50%' }}
                      onLoad={() => console.log(`[AvatarTest] 图片显示成功: ${result.url}`)}
                      onError={(e) => console.error(`[AvatarTest] 图片显示失败: ${result.url}`, e)}
                    />
                  </div>
                </>
              )}
              {result.error && (
                <div><strong>Error:</strong> {result.error}</div>
              )}
            </div>
          ))}
        </div>
      )}
    </div>
  )
}

export default AvatarTest