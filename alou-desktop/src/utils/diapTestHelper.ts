/**
 * DIAP身份创建测试助手
 * 用于测试和验证DIAP身份创建流程
 */

import { invoke } from '@tauri-apps/api/core'
import { 
  setDiapIdentity, 
  getDiapIdentity, 
  hasDiapIdentity,
  getAllDiapIdentities 
} from './memoryStorage'

// IPFS状态接口
export interface IpfsStatus {
  daemon: unknown
  api: { success: boolean }
  ready: boolean
  error?: string
}

// DIAP身份接口
export interface TestDiapIdentity {
  did: string
  cid: string
  ipns: string
  public_key?: string
  gateway_url?: string
  [key: string]: unknown
}

// 创建参数接口
export interface CreationParams {
  agentName?: string
  agentDescription?: string
  ipfsApiUrl?: string
  ipfsGatewayUrl?: string
  [key: string]: unknown
}

// 测试结果接口
export interface TestResults {
  timestamp: string
  tests: {
    localStorage?: { success: boolean; testSessionId?: string; testIdentity?: TestDiapIdentity; error?: string }
    ipfsStatus?: IpfsStatus
    fullCreation?: { success: boolean; identity?: TestDiapIdentity; error?: string; isExisting?: boolean }
  }
  summary?: {
    allPassed: boolean
    passedCount: number
    totalCount: number
  }
}

// 可访问性测试结果
export interface AccessibilityTest {
  summary: string
}

// 创建结果接口
export interface CreationResult {
  success: boolean
  identity?: TestDiapIdentity
  ipfsStatus?: IpfsStatus
  accessibilityTest?: AccessibilityTest
  isExisting?: boolean
  error?: string
  sessionId?: string
}

// 全局窗口扩展
declare global {
  interface Window {
    DiapTestHelper: {
      test: DiapTestHelper
      testCreation: (sessionId: string, params?: CreationParams) => Promise<CreationResult>
      testStorage: () => Promise<{ success: boolean; testSessionId?: string; testIdentity?: TestDiapIdentity; error?: string }>
      testIpfs: () => Promise<IpfsStatus>
      runAll: (sessionId?: string | null) => Promise<TestResults>
      help: () => void
    }
  }
}

class DiapTestHelper {
  private testResults: unknown[]

  constructor() {
    this.testResults = []
  }

  /**
   * 测试IPFS节点状态
   */
  async testIpfsStatus(): Promise<IpfsStatus> {
    console.log('🧪 [DiapTest] 测试IPFS节点状态...')
    
    try {
      // 检查守护进程状态
      const daemonStatus = await invoke<unknown>('get_ipfs_daemon_status')
      console.log('📊 [DiapTest] IPFS守护进程状态:', daemonStatus)
      
      // 测试API连接
      const apiTest = await invoke<{ success: boolean }>('test_ipfs_api', { 
        ipfsApiUrl: 'http://localhost:5001' 
      })
      console.log('🔗 [DiapTest] IPFS API测试:', apiTest)
      
      // 获取节点信息
      if (apiTest.success) {
        const nodeInfo = await invoke<unknown>('get_ipfs_info')
        console.log('ℹ️ [DiapTest] IPFS节点信息:', nodeInfo)
      }
      
      return {
        daemon: daemonStatus,
        api: apiTest,
        ready: (daemonStatus as { process_running?: boolean }).process_running && apiTest.success
      }
    } catch (error) {
      console.error('❌ [DiapTest] IPFS状态测试失败:', error)
      return { ready: false, daemon: {}, api: { success: false }, error: (error as Error).message }
    }
  }

  /**
   * 测试完整的DIAP身份创建流程
   */
  async testFullDiapCreationFlow(sessionId: string, params: CreationParams = {}): Promise<CreationResult> {
    console.log('🚀 [DiapTest] 开始完整DIAP身份创建流程测试...')
    
    const testParams = {
      agentName: 'Test Agent',
      agentDescription: 'Test DIAP Identity Creation',
      ipfsApiUrl: 'http://localhost:5001',
      ipfsGatewayUrl: 'http://localhost:8080',
      ...params
    }
    
    try {
      // 步骤1: 检查IPFS状态
      console.log('📋 [DiapTest] 步骤1: 检查IPFS状态')
      const ipfsStatus = await this.testIpfsStatus()
      if (!ipfsStatus.ready) {
        throw new Error(`IPFS节点未就绪: ${ipfsStatus.error || 'Unknown error'}`)
      }
      
      // 步骤2: 检查是否已存在身份
      console.log('📋 [DiapTest] 步骤2: 检查现有身份')
      if (hasDiapIdentity(sessionId)) {
        const existing = getDiapIdentity(sessionId)
        console.log('⚠️ [DiapTest] 身份已存在:', existing)
        return { success: true, identity: existing as TestDiapIdentity, isExisting: true }
      }
      
      // 步骤3: 使用DiapIntegrationService创建身份
      console.log('📋 [DiapTest] 步骤3: 创建DIAP身份')
      const { default: diapIntegrationService } = await import('../services/diapIntegrationService')
      
      const result = await diapIntegrationService.createDiapIdentity(sessionId, testParams)
      
      // 步骤4: 验证创建结果
      console.log('📋 [DiapTest] 步骤4: 验证创建结果')
      const identity = result.identity
      
      if (!identity || !identity.did || !identity.cid || !identity.ipns) {
        throw new Error('创建的身份信息不完整')
      }
      
      // 步骤5: 验证本地存储
      console.log('📋 [DiapTest] 步骤5: 验证本地存储')
      const storedIdentity = getDiapIdentity(sessionId)
      if (!storedIdentity) {
        throw new Error('身份未正确保存到本地存储')
      }
      
      // 步骤6: 测试IPNS可访问性
      console.log('📋 [DiapTest] 步骤6: 测试IPNS可访问性')
      const accessibilityTest = await invoke<AccessibilityTest>('test_ipns_on_public_gateway', {
        ipnsName: identity.ipns
      })
      
      console.log('✅ [DiapTest] DIAP身份创建流程测试完成!')
      console.log('🎯 [DiapTest] 创建结果:', {
        did: identity.did,
        cid: identity.cid,
        ipns: identity.ipns,
        accessibility: accessibilityTest.summary
      })
      
      return {
        success: true,
        identity,
        ipfsStatus,
        accessibilityTest,
        isExisting: false
      }
      
    } catch (error) {
      console.error('❌ [DiapTest] DIAP身份创建流程测试失败:', error)
      return {
        success: false,
        error: (error as Error).message,
        sessionId
      }
    }
  }

  /**
   * 测试本地存储功能
   */
  async testLocalStorage(): Promise<{ success: boolean; testSessionId?: string; testIdentity?: TestDiapIdentity; error?: string }> {
    console.log('🧪 [DiapTest] 测试本地存储功能...')
    
    const testSessionId = `test_${Date.now()}`
    const testIdentity: TestDiapIdentity = {
      did: `did:test:${testSessionId}`,
      cid: `bafytest${testSessionId}`,
      ipns: `/ipns/k51test${testSessionId}`,
      public_key: `test_key_${testSessionId}`,
      gateway_url: 'http://localhost:8080'
    }
    
    try {
      // 测试存储
      console.log('💾 [DiapTest] 测试存储身份...')
      const stored = setDiapIdentity(testSessionId, testIdentity)
      if (!stored) {
        throw new Error('存储身份失败')
      }
      
      // 测试检查存在性
      console.log('🔍 [DiapTest] 测试检查存在性...')
      const exists = hasDiapIdentity(testSessionId)
      if (!exists) {
        throw new Error('检查存在性失败')
      }
      
      // 测试获取
      console.log('📤 [DiapTest] 测试获取身份...')
      const retrieved = getDiapIdentity(testSessionId)
      if (!retrieved || (retrieved as TestDiapIdentity).did !== testIdentity.did) {
        throw new Error('获取身份失败或数据不匹配')
      }
      
      // 测试获取所有身份
      console.log('📋 [DiapTest] 测试获取所有身份...')
      const allIdentities = getAllDiapIdentities()
      if (!allIdentities[testSessionId]) {
        throw new Error('获取所有身份中未找到测试身份')
      }
      
      console.log('✅ [DiapTest] 本地存储功能测试通过!')
      return { success: true, testSessionId, testIdentity }
      
    } catch (error) {
      console.error('❌ [DiapTest] 本地存储功能测试失败:', error)
      return { success: false, error: (error as Error).message }
    }
  }

  /**
   * 运行所有测试
   */
  async runAllTests(sessionId: string | null = null): Promise<TestResults> {
    console.log('🎯 [DiapTest] 开始运行所有DIAP测试...')
    
    const results: TestResults = {
      timestamp: new Date().toISOString(),
      tests: {}
    }
    
    // 测试1: 本地存储
    console.log('\n=== 测试1: 本地存储功能 ===')
    results.tests.localStorage = await this.testLocalStorage()
    
    // 测试2: IPFS状态
    console.log('\n=== 测试2: IPFS节点状态 ===')
    results.tests.ipfsStatus = await this.testIpfsStatus()
    
    // 测试3: 完整创建流程（如果提供了sessionId）
    if (sessionId) {
      console.log('\n=== 测试3: 完整DIAP身份创建流程 ===')
      results.tests.fullCreation = await this.testFullDiapCreationFlow(sessionId)
    }
    
    // 汇总结果
    const allPassed = Object.values(results.tests).every(test => test?.success)
    results.summary = {
      allPassed,
      passedCount: Object.values(results.tests).filter(test => test?.success).length,
      totalCount: Object.keys(results.tests).length
    }
    
    console.log('\n🏁 [DiapTest] 所有测试完成!')
    console.log('📊 [DiapTest] 测试汇总:', results.summary)
    
    if (allPassed) {
      console.log('🎉 [DiapTest] 所有测试通过!')
    } else {
      console.log('⚠️ [DiapTest] 部分测试失败，请检查错误信息')
    }
    
    return results
  }
}

// 创建全局实例
const diapTestHelper = new DiapTestHelper()

// 导出便捷函数
export const testDiapCreation = (sessionId: string, params?: CreationParams): Promise<CreationResult> => 
  diapTestHelper.testFullDiapCreationFlow(sessionId, params)

export const testDiapStorage = (): Promise<{ success: boolean; testSessionId?: string; testIdentity?: TestDiapIdentity; error?: string }> => 
  diapTestHelper.testLocalStorage()

export const testDiapIpfs = (): Promise<IpfsStatus> => 
  diapTestHelper.testIpfsStatus()

export const runAllDiapTests = (sessionId?: string | null): Promise<TestResults> => 
  diapTestHelper.runAllTests(sessionId)

// 添加到全局对象以便控制台调用
if (typeof window !== 'undefined') {
  window.DiapTestHelper = {
    test: diapTestHelper,
    testCreation: testDiapCreation,
    testStorage: testDiapStorage,
    testIpfs: testDiapIpfs,
    runAll: runAllDiapTests,
    help: () => {
      console.group('🧪 DiapTestHelper 使用帮助')
      console.log('DiapTestHelper.testCreation(sessionId, params) - 测试DIAP身份创建')
      console.log('DiapTestHelper.testStorage() - 测试本地存储功能')
      console.log('DiapTestHelper.testIpfs() - 测试IPFS节点状态')
      console.log('DiapTestHelper.runAll(sessionId) - 运行所有测试')
      console.log('')
      console.log('示例:')
      console.log('  DiapTestHelper.runAll("test_session_123")')
      console.log('  DiapTestHelper.testCreation("my_session", { agentName: "My Agent" })')
      console.groupEnd()
    }
  }
  
  console.log('🧪 DiapTestHelper 已加载!')
  console.log('💡 输入 DiapTestHelper.help() 查看使用帮助')
}

export default diapTestHelper
