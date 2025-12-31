/**
 * SDK 功能测试脚本
 * 用于测试 LSP 和 Spec 功能的基本功能
 */

const fs = require('fs').promises
const path = require('path')
const { spawn } = require('child_process')

/**
 * 测试 LSP 功能
 */
async function testLSP() {
  console.log('🧪 测试 LSP 功能...\n')

  const testRequest = {
    code: 'function test() {\n  console.log("Hello");\n  return 42;\n}',
    language: 'javascript',
    operation: 'completion',
    position: { line: 1, character: 10 }
  }

  const tempDir = path.join(__dirname, '.test_temp')
  await fs.mkdir(tempDir, { recursive: true })

  const inputFile = path.join(tempDir, 'lsp_test.json')
  await fs.writeFile(inputFile, JSON.stringify(testRequest, null, 2))

  try {
    const node = spawn('node', [path.join(__dirname, '../alou-desktop/scripts/lsp-agent.js'), inputFile], {
      stdio: 'pipe'
    })

    let stdout = ''
    let stderr = ''

    node.stdout.on('data', (data) => {
      stdout += data.toString()
    })

    node.stderr.on('data', (data) => {
      stderr += data.toString()
    })

    await new Promise((resolve, reject) => {
      node.on('close', (code) => {
        if (code === 0) {
          resolve()
        } else {
          reject(new Error(`Process exited with code ${code}\n${stderr}`))
        }
      })
    })

    // 读取结果文件
    const resultPath = path.join(tempDir, 'lsp_test.result.json')
    const resultContent = await fs.readFile(resultPath, 'utf-8')
    const result = JSON.parse(resultContent)

    if (result.success && result.result) {
      console.log('✅ LSP 补全测试通过')
      console.log(`   找到 ${result.result.items?.length || 0} 个补全建议\n`)
    } else {
      console.log('❌ LSP 补全测试失败\n')
    }

    // 清理
    await fs.rm(tempDir, { recursive: true, force: true })
  } catch (error) {
    console.log('❌ LSP 测试失败:', error.message, '\n')
    await fs.rm(tempDir, { recursive: true, force: true })
  }
}

/**
 * 测试 Spec 功能
 */
async function testSpec() {
  console.log('🧪 测试 Spec 功能...\n')

  const testRequest = {
    spec_type: 'product',
    operation: 'create',
    spec_data: {
      title: '测试产品',
      content: '# 测试规格\n\n这是一个测试规格文档',
      metadata: {
        version: '1.0.0',
        status: 'draft'
      }
    }
  }

  const tempDir = path.join(__dirname, '.test_temp')
  await fs.mkdir(tempDir, { recursive: true })

  const inputFile = path.join(tempDir, 'spec_test.json')
  await fs.writeFile(inputFile, JSON.stringify(testRequest, null, 2))

  try {
    const node = spawn('node', [path.join(__dirname, '../alou-desktop/scripts/spec-agent.js'), inputFile], {
      stdio: 'pipe'
    })

    let stdout = ''
    let stderr = ''

    node.stdout.on('data', (data) => {
      stdout += data.toString()
    })

    node.stderr.on('data', (data) => {
      stderr += data.toString()
    })

    await new Promise((resolve, reject) => {
      node.on('close', (code) => {
        if (code === 0) {
          resolve()
        } else {
          reject(new Error(`Process exited with code ${code}\n${stderr}`))
        }
      })
    })

    // 读取结果文件
    const resultPath = path.join(tempDir, 'spec_test.result.json')
    const resultContent = await fs.readFile(resultPath, 'utf-8')
    const result = JSON.parse(resultContent)

    if (result.success && result.spec) {
      console.log('✅ Spec 创建测试通过')
      console.log(`   规格ID: ${result.spec.id}`)
      console.log(`   规格标题: ${result.spec.title}\n`)
    } else {
      console.log('❌ Spec 创建测试失败\n')
    }

    // 清理
    await fs.rm(tempDir, { recursive: true, force: true })
  } catch (error) {
    console.log('❌ Spec 测试失败:', error.message, '\n')
    await fs.rm(tempDir, { recursive: true, force: true })
  }
}

/**
 * 运行所有测试
 */
async function runAllTests() {
  console.log('🚀 开始运行 SDK 功能测试\n')
  console.log('=' .repeat(50))

  await testLSP()
  await testSpec()

  console.log('=' .repeat(50))
  console.log('✨ 所有测试完成！\n')
}

// 运行测试
runAllTests().catch(console.error)
