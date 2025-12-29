/**
 * LSP Service - Language Server Protocol integration
 * 与 Tauri LSP SDK 模块通信
 */
import { invoke } from '@tauri-apps/api/core'

/**
 * LSP 操作类型
 */
export const LspOperation = {
  Completion: 'completion',
  Diagnostics: 'diagnostics',
  GoToDefinition: 'go_to_definition',
  Hover: 'hover',
  FindReferences: 'find_references',
  Format: 'format',
  Rename: 'rename',
  DocumentSymbols: 'document_symbols',
}

/**
 * 创建 LSP Service 类
 */
export class LspService {
  /**
   * 执行代码补全
   * @param {Object} options - 补全选项
   * @param {string} options.code - 代码内容
   * @param {string} options.language - 编程语言
   * @param {string} options.filePath - 文件路径（可选）
   * @param {Object} options.position - 光标位置
   * @param {number} options.position.line - 行号
   * @param {number} options.position.character - 字符位置
   * @returns {Promise<Object>} 补全结果
   */
  async getCompletions({ code, language, filePath, position }) {
    const request = {
      code,
      language,
      file_path: filePath,
      operation: LspOperation.Completion,
      position,
    }

    const response = await invoke('execute_lsp', { request })
    
    if (!response.success) {
      throw new Error(response.error || '代码补全失败')
    }

    return response.result
  }

  /**
   * 获取代码诊断信息（错误和警告）
   * @param {Object} options - 诊断选项
   * @param {string} options.code - 代码内容
   * @param {string} options.language - 编程语言
   * @param {string} options.filePath - 文件路径（可选）
   * @param {string} options.projectRoot - 项目根目录（可选）
   * @returns {Promise<Object>} 诊断结果
   */
  async getDiagnostics({ code, language, filePath, projectRoot }) {
    const request = {
      code,
      language,
      file_path: filePath,
      operation: LspOperation.Diagnostics,
      project_root: projectRoot,
    }

    const response = await invoke('execute_lsp', { request })
    
    if (!response.success) {
      throw new Error(response.error || '代码诊断失败')
    }

    return response.result
  }

  /**
   * 跳转到定义
   * @param {Object} options - 跳转选项
   * @param {string} options.code - 代码内容
   * @param {string} options.language - 编程语言
   * @param {Object} options.position - 光标位置
   * @param {string} options.projectRoot - 项目根目录（可选）
   * @returns {Promise<Object>} 定义位置
   */
  async goToDefinition({ code, language, position, projectRoot }) {
    const request = {
      code,
      language,
      operation: LspOperation.GoToDefinition,
      position,
      project_root: projectRoot,
    }

    const response = await invoke('execute_lsp', { request })
    
    if (!response.success) {
      throw new Error(response.error || '跳转到定义失败')
    }

    return response.result
  }

  /**
   * 获取悬停信息
   * @param {Object} options - 悬停选项
   * @param {string} options.code - 代码内容
   * @param {string} options.language - 编程语言
   * @param {Object} options.position - 光标位置
   * @returns {Promise<Object>} 悬停信息
   */
  async getHoverInfo({ code, language, position }) {
    const request = {
      code,
      language,
      operation: LspOperation.Hover,
      position,
    }

    const response = await invoke('execute_lsp', { request })
    
    if (!response.success) {
      throw new Error(response.error || '获取悬停信息失败')
    }

    return response.result
  }

  /**
   * 查找引用
   * @param {Object} options - 查找选项
   * @param {string} options.code - 代码内容
   * @param {string} options.language - 编程语言
   * @param {Object} options.position - 光标位置
   * @param {string} options.projectRoot - 项目根目录（可选）
   * @returns {Promise<Object>} 引用列表
   */
  async findReferences({ code, language, position, projectRoot }) {
    const request = {
      code,
      language,
      operation: LspOperation.FindReferences,
      position,
      project_root: projectRoot,
    }

    const response = await invoke('execute_lsp', { request })
    
    if (!response.success) {
      throw new Error(response.error || '查找引用失败')
    }

    return response.result
  }

  /**
   * 格式化代码
   * @param {Object} options - 格式化选项
   * @param {string} options.code - 代码内容
   * @param {string} options.language - 编程语言
   * @param {string} options.filePath - 文件路径（可选）
   * @returns {Promise<Object>} 格式化结果
   */
  async formatCode({ code, language, filePath }) {
    const request = {
      code,
      language,
      file_path: filePath,
      operation: LspOperation.Format,
    }

    const response = await invoke('execute_lsp', { request })
    
    if (!response.success) {
      throw new Error(response.error || '代码格式化失败')
    }

    return response.result
  }

  /**
   * 重命名符号
   * @param {Object} options - 重命名选项
   * @param {string} options.code - 代码内容
   * @param {string} options.language - 编程语言
   * @param {string} options.filePath - 文件路径
   * @param {Object} options.position - 光标位置
   * @param {string} options.newName - 新名称
   * @returns {Promise<Object>} 重命名结果
   */
  async renameSymbol({ code, language, filePath, position, newName }) {
    const request = {
      code,
      language,
      file_path: filePath,
      operation: LspOperation.Rename,
      position,
      spec_data: { new_name: newName },
    }

    const response = await invoke('execute_lsp', { request })
    
    if (!response.success) {
      throw new Error(response.error || '重命名符号失败')
    }

    return response.result
  }

  /**
   * 获取文档符号
   * @param {Object} options - 符号查询选项
   * @param {string} options.code - 代码内容
   * @param {string} options.language - 编程语言
   * @returns {Promise<Object>} 符号列表
   */
  async getDocumentSymbols({ code, language }) {
    const request = {
      code,
      language,
      operation: LspOperation.DocumentSymbols,
    }

    const response = await invoke('execute_lsp', { request })
    
    if (!response.success) {
      throw new Error(response.error || '获取文档符号失败')
    }

    return response.result
  }

  /**
   * 获取支持的语言列表
   * @returns {Promise<Array<string>>} 支持的语言列表
   */
  async getSupportedLanguages() {
    return await invoke('get_supported_languages')
  }
}

export default new LspService()
