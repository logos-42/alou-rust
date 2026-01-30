/**
 * LSP Service - Language Server Protocol integration
 * 与 Tauri LSP SDK 模块通信
 */
import { invoke } from '@tauri-apps/api/core'

/**
 * LSP 操作类型
 */
export enum LspOperation {
  Completion = 'completion',
  Diagnostics = 'diagnostics',
  GoToDefinition = 'go_to_definition',
  Hover = 'hover',
  FindReferences = 'find_references',
  Format = 'format',
  Rename = 'rename',
  DocumentSymbols = 'document_symbols',
}

/**
 * 光标位置
 */
export interface Position {
  line: number
  character: number
}

/**
 * LSP 请求
 */
export interface LspRequest {
  code: string
  language: string
  file_path?: string
  operation: LspOperation | string
  position?: Position
  project_root?: string
  spec_data?: Record<string, any>
}

/**
 * LSP 响应
 */
export interface LspResponse {
  success: boolean
  result?: any
  error?: string
}

/**
 * 代码补全选项
 */
export interface GetCompletionsOptions {
  code: string
  language: string
  filePath?: string
  position: Position
}

/**
 * 代码诊断选项
 */
export interface GetDiagnosticsOptions {
  code: string
  language: string
  filePath?: string
  projectRoot?: string
}

/**
 * 跳转到定义选项
 */
export interface GoToDefinitionOptions {
  code: string
  language: string
  position: Position
  projectRoot?: string
}

/**
 * 获取悬停信息选项
 */
export interface GetHoverInfoOptions {
  code: string
  language: string
  position: Position
}

/**
 * 查找引用选项
 */
export interface FindReferencesOptions {
  code: string
  language: string
  position: Position
  projectRoot?: string
}

/**
 * 格式化代码选项
 */
export interface FormatCodeOptions {
  code: string
  language: string
  filePath?: string
}

/**
 * 重命名符号选项
 */
export interface RenameSymbolOptions {
  code: string
  language: string
  filePath: string
  position: Position
  newName: string
}

/**
 * 获取文档符号选项
 */
export interface GetDocumentSymbolsOptions {
  code: string
  language: string
}

/**
 * 创建 LSP Service 类
 */
export class LspService {
  /**
   * 执行代码补全
   * @param options - 补全选项
   * @returns 补全结果
   */
  async getCompletions({ code, language, filePath, position }: GetCompletionsOptions): Promise<any> {
    const request: LspRequest = {
      code,
      language,
      file_path: filePath,
      operation: LspOperation.Completion,
      position,
    }

    const response = await invoke<LspResponse>('execute_lsp', { request })
    
    if (!response.success) {
      throw new Error(response.error || '代码补全失败')
    }

    return response.result
  }

  /**
   * 获取代码诊断信息（错误和警告）
   * @param options - 诊断选项
   * @returns 诊断结果
   */
  async getDiagnostics({ code, language, filePath, projectRoot }: GetDiagnosticsOptions): Promise<any> {
    const request: LspRequest = {
      code,
      language,
      file_path: filePath,
      operation: LspOperation.Diagnostics,
      project_root: projectRoot,
    }

    const response = await invoke<LspResponse>('execute_lsp', { request })
    
    if (!response.success) {
      throw new Error(response.error || '代码诊断失败')
    }

    return response.result
  }

  /**
   * 跳转到定义
   * @param options - 跳转选项
   * @returns 定义位置
   */
  async goToDefinition({ code, language, position, projectRoot }: GoToDefinitionOptions): Promise<any> {
    const request: LspRequest = {
      code,
      language,
      operation: LspOperation.GoToDefinition,
      position,
      project_root: projectRoot,
    }

    const response = await invoke<LspResponse>('execute_lsp', { request })
    
    if (!response.success) {
      throw new Error(response.error || '跳转到定义失败')
    }

    return response.result
  }

  /**
   * 获取悬停信息
   * @param options - 悬停选项
   * @returns 悬停信息
   */
  async getHoverInfo({ code, language, position }: GetHoverInfoOptions): Promise<any> {
    const request: LspRequest = {
      code,
      language,
      operation: LspOperation.Hover,
      position,
    }

    const response = await invoke<LspResponse>('execute_lsp', { request })
    
    if (!response.success) {
      throw new Error(response.error || '获取悬停信息失败')
    }

    return response.result
  }

  /**
   * 查找引用
   * @param options - 查找选项
   * @returns 引用列表
   */
  async findReferences({ code, language, position, projectRoot }: FindReferencesOptions): Promise<any> {
    const request: LspRequest = {
      code,
      language,
      operation: LspOperation.FindReferences,
      position,
      project_root: projectRoot,
    }

    const response = await invoke<LspResponse>('execute_lsp', { request })
    
    if (!response.success) {
      throw new Error(response.error || '查找引用失败')
    }

    return response.result
  }

  /**
   * 格式化代码
   * @param options - 格式化选项
   * @returns 格式化结果
   */
  async formatCode({ code, language, filePath }: FormatCodeOptions): Promise<any> {
    const request: LspRequest = {
      code,
      language,
      file_path: filePath,
      operation: LspOperation.Format,
    }

    const response = await invoke<LspResponse>('execute_lsp', { request })
    
    if (!response.success) {
      throw new Error(response.error || '代码格式化失败')
    }

    return response.result
  }

  /**
   * 重命名符号
   * @param options - 重命名选项
   * @returns 重命名结果
   */
  async renameSymbol({ code, language, filePath, position, newName }: RenameSymbolOptions): Promise<any> {
    const request: LspRequest = {
      code,
      language,
      file_path: filePath,
      operation: LspOperation.Rename,
      position,
      spec_data: { new_name: newName },
    }

    const response = await invoke<LspResponse>('execute_lsp', { request })
    
    if (!response.success) {
      throw new Error(response.error || '重命名符号失败')
    }

    return response.result
  }

  /**
   * 获取文档符号
   * @param options - 符号查询选项
   * @returns 符号列表
   */
  async getDocumentSymbols({ code, language }: GetDocumentSymbolsOptions): Promise<any> {
    const request: LspRequest = {
      code,
      language,
      operation: LspOperation.DocumentSymbols,
    }

    const response = await invoke<LspResponse>('execute_lsp', { request })
    
    if (!response.success) {
      throw new Error(response.error || '获取文档符号失败')
    }

    return response.result
  }

  /**
   * 获取支持的语言列表
   * @returns 支持的语言列表
   */
  async getSupportedLanguages(): Promise<string[]> {
    return await invoke<string[]>('get_supported_languages')
  }
}

export default new LspService()
