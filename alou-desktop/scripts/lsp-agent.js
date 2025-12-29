#!/usr/bin/env node
/**
 * LSP Agent Node.js Script
 * 处理语言服务器协议相关请求
 */

const fs = require('fs').promises
const path = require('path')
const os = require('os')

/**
 * 处理LSP请求
 */
async function handleLspRequest(request) {
  const { code, language, operation, position, project_root } = request

  switch (operation) {
    case 'completion':
      return await handleCompletion(code, language, position)
    case 'diagnostics':
      return await handleDiagnostics(code, language, project_root)
    case 'go_to_definition':
      return await handleGoToDefinition(code, language, position)
    case 'hover':
      return await handleHover(code, language, position)
    case 'find_references':
      return await handleFindReferences(code, language, position, project_root)
    case 'format':
      return await handleFormat(code, language)
    case 'rename':
      return await handleRename(code, language, position, request.spec_data)
    case 'document_symbols':
      return await handleDocumentSymbols(code, language)
    default:
      throw new Error(`不支持的 LSP 操作: ${operation}`)
  }
}

/**
 * 处理代码补全
 */
async function handleCompletion(code, language, position) {
  const line = code.split('\n')[position.line]
  const prefix = line.substring(0, position.character)

  let items = []

  // 基于语言的智能补全
  switch (language) {
    case 'javascript':
    case 'typescript':
      items = getJavaScriptCompletions(prefix, line)
      break
    case 'python':
      items = getPythonCompletions(prefix, line)
      break
    case 'rust':
      items = getRustCompletions(prefix, line)
      break
    case 'go':
      items = getGoCompletions(prefix, line)
      break
    default:
      items = getGenericCompletions(prefix)
  }

  return {
    items: items,
  }
}

/**
 * JavaScript/TypeScript 补全
 */
function getJavaScriptCompletions(prefix, line) {
  const items = []

  // 关键字
  const keywords = ['const', 'let', 'var', 'function', 'class', 'import', 'export', 'async', 'await', 'return', 'if', 'else', 'for', 'while', 'try', 'catch', 'throw', 'new', 'this', 'super', 'extends', 'static', 'typeof', 'instanceof']
  keywords.forEach(kw => {
    if (kw.startsWith(prefix)) {
      items.push({
        label: kw,
        kind: 'keyword',
        insert_text: kw + ' ',
      })
    }
  })

  // 常用对象和方法
  const objects = [
    { label: 'console.log', kind: 'function', detail: '输出到控制台', documentation: '在控制台打印消息' },
    { label: 'console.error', kind: 'function', detail: '输出错误', documentation: '在控制台打印错误' },
    { label: 'Array.from', kind: 'function', detail: '创建数组', documentation: '从可迭代对象创建数组' },
    { label: 'Object.keys', kind: 'function', detail: '获取键数组', documentation: '返回对象的键数组' },
    { label: 'Object.values', kind: 'function', detail: '获取值数组', documentation: '返回对象的值数组' },
    { label: 'Promise.resolve', kind: 'function', detail: '创建已解析的Promise', documentation: '创建一个已解析的Promise' },
  ]

  objects.forEach(obj => {
    if (obj.label.startsWith(prefix)) {
      items.push(obj)
    }
  })

  // 根据上下文提供智能建议
  if (line.match(/import\s+.*from\s+/)) {
    items.push({
      label: 'react',
      kind: 'module',
      detail: 'React 库',
    })
  }

  return items
}

/**
 * Python 补全
 */
function getPythonCompletions(prefix, line) {
  const items = []

  const keywords = ['def', 'class', 'import', 'from', 'as', 'if', 'else', 'elif', 'for', 'while', 'try', 'except', 'finally', 'with', 'return', 'yield', 'pass', 'break', 'continue', 'lambda', 'async', 'await', 'True', 'False', 'None']
  keywords.forEach(kw => {
    if (kw.startsWith(prefix)) {
      items.push({
        label: kw,
        kind: 'keyword',
        insert_text: kw + ' ',
      })
    }
  })

  const builtins = [
    { label: 'print', kind: 'function', detail: '打印输出', documentation: '打印对象到标准输出' },
    { label: 'len', kind: 'function', detail: '获取长度', documentation: '返回对象的长度' },
    { label: 'range', kind: 'function', detail: '生成范围', documentation: '生成数字序列' },
    { label: 'str', kind: 'function', detail: '字符串转换', documentation: '转换为字符串' },
    { label: 'int', kind: 'function', detail: '整数转换', documentation: '转换为整数' },
    { label: 'list', kind: 'function', detail: '列表转换', documentation: '转换为列表' },
    { label: 'dict', kind: 'function', detail: '字典转换', documentation: '转换为字典' },
    { label: 'open', kind: 'function', detail: '打开文件', documentation: '打开文件并返回文件对象' },
  ]

  builtins.forEach(bi => {
    if (bi.label.startsWith(prefix)) {
      items.push(bi)
    }
  })

  return items
}

/**
 * Rust 补全
 */
function getRustCompletions(prefix, line) {
  const items = []

  const keywords = ['fn', 'struct', 'enum', 'impl', 'pub', 'use', 'mod', 'let', 'mut', 'const', 'static', 'type', 'trait', 'where', 'for', 'while', 'loop', 'if', 'else', 'match', 'return', 'break', 'continue', 'unsafe', 'async', 'await', 'move', 'ref']
  keywords.forEach(kw => {
    if (kw.startsWith(prefix)) {
      items.push({
        label: kw,
        kind: 'keyword',
        insert_text: kw + ' ',
      })
    }
  })

  const types = [
    { label: 'String', kind: 'type', detail: '字符串类型', documentation: '堆分配的字符串' },
    { label: 'Vec', kind: 'type', detail: '向量类型', documentation: '动态大小的数组' },
    { label: 'HashMap', kind: 'type', detail: '哈希映射', documentation: '键值对映射' },
    { label: 'Option', kind: 'type', detail: '可选值', documentation: '可能存在或不存在的值' },
    { label: 'Result', kind: 'type', detail: '结果类型', documentation: '可能失败的操作结果' },
    { label: 'Box', kind: 'type', detail: '堆分配', documentation: '堆分配的指针' },
    { label: 'Rc', kind: 'type', detail: '引用计数', documentation: '引用计数智能指针' },
    { label: 'Arc', kind: 'type', detail: '原子引用计数', documentation: '线程安全的引用计数指针' },
  ]

  types.forEach(t => {
    if (t.label.startsWith(prefix)) {
      items.push(t)
    }
  })

  return items
}

/**
 * Go 补全
 */
function getGoCompletions(prefix, line) {
  const items = []

  const keywords = ['func', 'var', 'const', 'type', 'struct', 'interface', 'package', 'import', 'return', 'if', 'else', 'for', 'switch', 'case', 'default', 'break', 'continue', 'fallthrough', 'select', 'defer', 'go', 'range', 'chan', 'map']
  keywords.forEach(kw => {
    if (kw.startsWith(prefix)) {
      items.push({
        label: kw,
        kind: 'keyword',
        insert_text: kw + ' ',
      })
    }
  })

  const builtins = [
    { label: 'fmt.Println', kind: 'function', detail: '打印输出', documentation: '打印到标准输出' },
    { label: 'len', kind: 'function', detail: '获取长度', documentation: '返回长度' },
    { label: 'cap', kind: 'function', detail: '获取容量', documentation: '返回容量' },
    { label: 'make', kind: 'function', detail: '创建切片/映射/通道', documentation: '分配并初始化类型' },
    { label: 'new', kind: 'function', detail: '分配内存', documentation: '分配内存并返回指针' },
    { label: 'append', kind: 'function', detail: '追加元素', documentation: '向切片追加元素' },
    { label: 'copy', kind: 'function', detail: '复制切片', documentation: '复制源切片到目标切片' },
    { label: 'close', kind: 'function', detail: '关闭通道', documentation: '关闭通道' },
  ]

  builtins.forEach(bi => {
    if (bi.label.startsWith(prefix)) {
      items.push(bi)
    }
  })

  return items
}

/**
 * 通用补全
 */
function getGenericCompletions(prefix) {
  const items = [
    { label: 'function', kind: 'function', insert_text: 'function ' },
    { label: 'return', kind: 'keyword', insert_text: 'return ' },
    { label: 'if', kind: 'keyword', insert_text: 'if () {\n\n}' },
    { label: 'for', kind: 'keyword', insert_text: 'for () {\n\n}' },
    { label: 'class', kind: 'keyword', insert_text: 'class ' },
    { label: 'import', kind: 'keyword', insert_text: 'import ' },
  ]

  return items.filter(item => item.label.startsWith(prefix))
}

/**
 * 处理代码诊断
 */
async function handleDiagnostics(code, language, projectRoot) {
  const diagnostics = []

  // 简单的语法检查
  const lines = code.split('\n')
  lines.forEach((line, index) => {
    // 检查未闭合的括号
    const openBraces = (line.match(/\(/g) || []).length
    const closeBraces = (line.match(/\)/g) || []).length
    if (openBraces !== closeBraces) {
      diagnostics.push({
        range: {
          start: { line: index, character: 0 },
          end: { line: index, character: line.length },
        },
        severity: 'warning',
        message: '括号可能不匹配',
        source: 'lsp',
      })
    }

    // 检查未闭合的字符串
    const quotes = line.match(/"/g)
    if (quotes && quotes.length % 2 !== 0) {
      diagnostics.push({
        range: {
          start: { line: index, character: 0 },
          end: { line: index, character: line.length },
        },
        severity: 'error',
        message: '未闭合的字符串',
        source: 'lsp',
      })
    }
  })

  return {
    diagnostics,
  }
}

/**
 * 处理跳转到定义
 */
async function handleGoToDefinition(code, language, position) {
  // 简单实现：返回当前位置（实际需要解析代码）
  return {
    uri: 'file:///current/file.js',
    range: {
      start: position,
      end: { line: position.line, character: position.character + 5 },
    },
  }
}

/**
 * 处理悬停信息
 */
async function handleHover(code, language, position) {
  const line = code.split('\n')[position.line]
  const word = getWordAtPosition(line, position.character)

  if (!word) {
    return {
      contents: '',
    }
  }

  const documentation = getDocumentation(word, language)
  return {
    contents: documentation || `**${word}**\n\n暂无文档`,
  }
}

/**
 * 获取位置处的单词
 */
function getWordAtPosition(line, character) {
  const match = line.substring(0, character).match(/([a-zA-Z_][a-zA-Z0-9_]*)$/)
  return match ? match[1] : null
}

/**
 * 获取文档
 */
function getDocumentation(word, language) {
  const docs = {
    javascript: {
      'console.log': '打印消息到控制台',
      'Array.from': '从可迭代对象创建数组',
      'Object.keys': '获取对象的所有键',
      'Promise.resolve': '创建已解析的Promise',
    },
    python: {
      'print': '打印对象到标准输出',
      'len': '返回对象的长度',
      'range': '生成数字序列',
      'open': '打开文件并返回文件对象',
    },
    rust: {
      'Vec': '动态大小的数组',
      'String': '堆分配的UTF-8字符串',
      'Option': '可能存在或不存在的值',
      'Result': '可能失败的操作结果',
    },
  }

  return docs[language]?.[word]
}

/**
 * 处理查找引用
 */
async function handleFindReferences(code, language, position, projectRoot) {
  // 简单实现：返回空引用列表
  return {
    references: [],
  }
}

/**
 * 处理代码格式化
 */
async function handleFormat(code, language) {
  let formattedCode = code

  switch (language) {
    case 'javascript':
    case 'typescript':
      formattedCode = formatJavaScript(code)
      break
    case 'python':
      formattedCode = formatPython(code)
      break
    case 'rust':
      formattedCode = formatRust(code)
      break
    default:
      formattedCode = code
  }

  return {
    formatted_code: formattedCode,
  }
}

/**
 * 格式化JavaScript代码
 */
function formatJavaScript(code) {
  // 简单格式化：添加缩进和空格
  let formatted = code
  let indentLevel = 0

  formatted = formatted.replace(/\s*{\s*/g, ' {\n  ' + '  '.repeat(indentLevel++))
  formatted = formatted.replace(/\s*}\s*/g, '\n' + '  '.repeat(--indentLevel) + '}\n')
  formatted = formatted.replace(/\s*;\s*/g, ';\n  ' + '  '.repeat(indentLevel))

  return formatted
}

/**
 * 格式化Python代码
 */
function formatPython(code) {
  // Python使用缩进，简单处理
  return code
}

/**
 * 格式化Rust代码
 */
function formatRust(code) {
  // Rust格式化，简单处理
  return code
}

/**
 * 处理重命名符号
 */
async function handleRename(code, language, position, specData) {
  // 简单实现：返回无变更
  return {
    changes: [],
  }
}

/**
 * 处理文档符号
 */
async function handleDocumentSymbols(code, language) {
  const symbols = []
  const lines = code.split('\n')

  lines.forEach((line, index) => {
    // 检测函数定义
    if (language === 'javascript' || language === 'typescript') {
      const functionMatch = line.match(/(?:function|const|let)\s+(\w+)/)
      if (functionMatch) {
        symbols.push({
          name: functionMatch[1],
          kind: 'function',
          location: {
            uri: 'file:///current/file.js',
            range: {
              start: { line: index, character: 0 },
              end: { line: index, character: line.length },
            },
          },
        })
      }
    } else if (language === 'python') {
      const functionMatch = line.match(/def\s+(\w+)/)
      const classMatch = line.match(/class\s+(\w+)/)
      if (functionMatch) {
        symbols.push({
          name: functionMatch[1],
          kind: 'function',
          location: {
            uri: 'file:///current/file.py',
            range: {
              start: { line: index, character: 0 },
              end: { line: index, character: line.length },
            },
          },
        })
      } else if (classMatch) {
        symbols.push({
          name: classMatch[1],
          kind: 'class',
          location: {
            uri: 'file:///current/file.py',
            range: {
              start: { line: index, character: 0 },
              end: { line: index, character: line.length },
            },
          },
        })
      }
    } else if (language === 'rust') {
      const functionMatch = line.match(/(?:pub\s+)?fn\s+(\w+)/)
      const structMatch = line.match(/(?:pub\s+)?struct\s+(\w+)/)
      if (functionMatch) {
        symbols.push({
          name: functionMatch[1],
          kind: 'function',
          location: {
            uri: 'file:///current/file.rs',
            range: {
              start: { line: index, character: 0 },
              end: { line: index, character: line.length },
            },
          },
        })
      } else if (structMatch) {
        symbols.push({
          name: structMatch[1],
          kind: 'struct',
          location: {
            uri: 'file:///current/file.rs',
            range: {
              start: { line: index, character: 0 },
              end: { line: index, character: line.length },
            },
          },
        })
      }
    }
  })

  return {
    symbols,
  }
}

/**
 * 主函数
 */
async function main() {
  try {
    // 从命令行参数获取输入文件路径
    const inputPath = process.argv[2]
    if (!inputPath) {
      throw new Error('缺少输入文件路径')
    }

    // 读取请求
    const requestContent = await fs.readFile(inputPath, 'utf-8')
    const request = JSON.parse(requestContent)

    // 处理请求
    const result = await handleLspRequest(request)

    // 生成结果文件路径
    const resultPath = inputPath.replace('.json', '.result.json')

    // 写入结果
    const resultContent = JSON.stringify({
      success: true,
      result,
    }, null, 2)
    await fs.writeFile(resultPath, resultContent, 'utf-8')

    // 输出结果文件路径
    console.log(resultPath)
  } catch (error) {
    // 错误处理
    const inputPath = process.argv[2]
    const errorPath = inputPath.replace('.json', '.error.json')
    
    await fs.writeFile(errorPath, JSON.stringify({
      error: error.message,
      stack: error.stack,
    }, null, 2), 'utf-8')

    process.exit(1)
  }
}

main()
