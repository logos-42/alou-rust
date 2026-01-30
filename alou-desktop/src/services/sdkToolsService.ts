/**
 * SDK工具服务 - 简化的LSP和Spec工具调用接口
 * 为智能体提供简化的代码分析和规格管理工具
 */

import { invoke } from '@tauri-apps/api/core'

/**
 * LSP工具操作类型
 */
export enum LspToolOperation {
  COMPLETION = 'completion',
  DIAGNOSTICS = 'diagnostics',
  FORMAT = 'format',
  HOVER = 'hover',
  DEFINITION = 'go_to_definition',
  REFERENCES = 'find_references',
  RENAME = 'rename',
  SYMBOLS = 'document_symbols'
}

/**
 * Spec工具操作类型
 */
export enum SpecToolOperation {
  CREATE = 'create',
  UPDATE = 'update',
  GET = 'get',
  DELETE = 'delete',
  LIST = 'list',
  VALIDATE = 'validate',
  GENERATE = 'generate_from_template',
  EXPORT = 'export',
  IMPORT = 'import'
}

/**
 * Spec类型
 */
export enum SpecType {
  PRODUCT = 'product',
  TECHNICAL = 'technical',
  DESIGN = 'design',
  API = 'api',
  USER_STORY = 'user_story',
  TASKS = 'tasks',
  STRUCTURE = 'structure'
}

/**
 * LSP工具选项
 */
export interface LspToolOptions {
  code: string
  language: string
  operation: string
  position?: { line: number; character: number } | null
  filePath?: string | null
}

/**
 * LSP执行结果
 */
export interface LspToolResult {
  success: boolean
  result?: any
  operation: string
  error?: string
}

/**
 * Spec工具选项
 */
export interface SpecToolOptions {
  specType: string
  operation: string
  specData?: Record<string, any> | null
  specId?: string | null
  templateId?: string | null
  outputFormat?: string | null
}

/**
 * Spec执行结果
 */
export interface SpecToolResult {
  success: boolean
  spec?: any
  specs?: any[]
  validation_result?: any
  export_result?: any
  operation: string
  error?: string
}

/**
 * 可用工具
 */
export interface AvailableTool {
  name: string
  description: string
  category: string
  parameters: Record<string, {
    type: string
    description: string
    required: boolean
  }>
}

/**
 * SDK工具服务类
 */
export class SdkToolsService {
  /**
   * 执行LSP工具操作
   * @param options - 工具选项
   * @returns 执行结果
   */
  static async executeLspTool(options: LspToolOptions): Promise<LspToolResult> {
    try {
      const { code, language, operation, position, filePath } = options
      
      const request = {
        code,
        language,
        operation,
        position: position || null,
        file_path: filePath || null,
        project_root: null
      }

      const response = await invoke<{ success: boolean; result?: any; error?: string }>('execute_lsp', { request })
      
      if (!response.success) {
        throw new Error(response.error || 'LSP工具执行失败')
      }

      return {
        success: true,
        result: response.result,
        operation
      }
    } catch (error) {
      console.error('[SdkToolsService] LSP工具执行失败:', error)
      return {
        success: false,
        error: (error as Error).message,
        operation: options.operation
      }
    }
  }

  /**
   * 获取代码补全建议
   * @param code - 代码内容
   * @param language - 编程语言
   * @param position - 位置信息
   * @returns 补全结果
   */
  static async getCodeCompletions(
    code: string,
    language: string,
    position: { line: number; character: number }
  ): Promise<LspToolResult> {
    return this.executeLspTool({
      code,
      language,
      operation: LspToolOperation.COMPLETION,
      position
    })
  }

  /**
   * 诊断代码问题
   * @param code - 代码内容
   * @param language - 编程语言
   * @returns 诊断结果
   */
  static async diagnoseCode(code: string, language: string): Promise<LspToolResult> {
    return this.executeLspTool({
      code,
      language,
      operation: LspToolOperation.DIAGNOSTICS
    })
  }

  /**
   * 格式化代码
   * @param code - 代码内容
   * @param language - 编程语言
   * @returns 格式化结果
   */
  static async formatCode(code: string, language: string): Promise<LspToolResult> {
    return this.executeLspTool({
      code,
      language,
      operation: LspToolOperation.FORMAT
    })
  }

  /**
   * 获取代码悬停信息
   * @param code - 代码内容
   * @param language - 编程语言
   * @param position - 位置信息
   * @returns 悬停信息
   */
  static async getHoverInfo(
    code: string,
    language: string,
    position: { line: number; character: number }
  ): Promise<LspToolResult> {
    return this.executeLspTool({
      code,
      language,
      operation: LspToolOperation.HOVER,
      position
    })
  }

  /**
   * 执行Spec工具操作
   * @param options - 工具选项
   * @returns 执行结果
   */
  static async executeSpecTool(options: SpecToolOptions): Promise<SpecToolResult> {
    try {
      const { specType, operation, specData, specId, templateId, outputFormat } = options
      
      const request = {
        spec_type: specType,
        operation,
        spec_data: specData || null,
        spec_id: specId || null,
        template_id: templateId || null,
        output_format: outputFormat || null
      }

      const response = await invoke<{
        success: boolean
        spec?: any
        specs?: any[]
        validation_result?: any
        export_result?: any
        error?: string
      }>('execute_spec', { request })
      
      if (!response.success) {
        throw new Error(response.error || 'Spec工具执行失败')
      }

      return {
        success: true,
        spec: response.spec,
        specs: response.specs,
        validation_result: response.validation_result,
        export_result: response.export_result,
        operation
      }
    } catch (error) {
      console.error('[SdkToolsService] Spec工具执行失败:', error)
      return {
        success: false,
        error: (error as Error).message,
        operation: options.operation
      }
    }
  }

  /**
   * 创建规格文档
   * @param specType - 规格类型
   * @param specData - 规格数据
   * @param templateId - 模板ID（可选）
   * @returns 创建结果
   */
  static async createSpec(
    specType: string,
    specData: Record<string, any>,
    templateId: string | null = null
  ): Promise<SpecToolResult> {
    return this.executeSpecTool({
      specType,
      operation: SpecToolOperation.CREATE,
      specData,
      templateId
    })
  }

  /**
   * 获取规格文档
   * @param specType - 规格类型
   * @param specId - 规格ID
   * @returns 规格文档
   */
  static async getSpec(specType: string, specId: string): Promise<SpecToolResult> {
    return this.executeSpecTool({
      specType,
      operation: SpecToolOperation.GET,
      specId
    })
  }

  /**
   * 列出所有规格文档
   * @param specType - 规格类型
   * @returns 规格列表
   */
  static async listSpecs(specType: string): Promise<SpecToolResult> {
    return this.executeSpecTool({
      specType,
      operation: SpecToolOperation.LIST
    })
  }

  /**
   * 验证规格文档
   * @param specType - 规格类型
   * @param specData - 规格数据
   * @returns 验证结果
   */
  static async validateSpec(specType: string, specData: Record<string, any>): Promise<SpecToolResult> {
    return this.executeSpecTool({
      specType,
      operation: SpecToolOperation.VALIDATE,
      specData
    })
  }

  /**
   * 获取支持的语言列表
   * @returns 语言列表
   */
  static async getSupportedLanguages(): Promise<string[]> {
    try {
      const languages = await invoke<string[]>('get_supported_languages')
      return languages
    } catch (error) {
      console.error('[SdkToolsService] 获取支持的语言失败:', error)
      return []
    }
  }

  /**
   * 获取规格模板
   * @returns 模板列表
   */
  static async getSpecTemplates(): Promise<any[]> {
    try {
      const templates = await invoke<any[]>('get_spec_templates')
      return templates
    } catch (error) {
      console.error('[SdkToolsService] 获取规格模板失败:', error)
      return []
    }
  }

  /**
   * 获取所有可用的工具
   * @returns 工具列表
   */
  static getAvailableTools(): AvailableTool[] {
    return [
      {
        name: 'code_completion',
        description: '获取代码补全建议',
        category: 'development',
        parameters: {
          code: { type: 'string', description: '代码内容', required: true },
          language: { type: 'string', description: '编程语言', required: true },
          position: { type: 'object', description: '光标位置 {line, character}', required: true }
        }
      },
      {
        name: 'code_diagnostics',
        description: '诊断代码问题',
        category: 'development',
        parameters: {
          code: { type: 'string', description: '代码内容', required: true },
          language: { type: 'string', description: '编程语言', required: true }
        }
      },
      {
        name: 'code_format',
        description: '格式化代码',
        category: 'development',
        parameters: {
          code: { type: 'string', description: '代码内容', required: true },
          language: { type: 'string', description: '编程语言', required: true }
        }
      },
      {
        name: 'code_hover',
        description: '获取代码悬停信息',
        category: 'development',
        parameters: {
          code: { type: 'string', description: '代码内容', required: true },
          language: { type: 'string', description: '编程语言', required: true },
          position: { type: 'object', description: '光标位置 {line, character}', required: true }
        }
      },
      {
        name: 'create_spec',
        description: '创建规格文档',
        category: 'documentation',
        parameters: {
          specType: { type: 'string', description: '规格类型', required: true },
          specData: { type: 'object', description: '规格数据', required: true },
          templateId: { type: 'string', description: '模板ID', required: false }
        }
      },
      {
        name: 'get_spec',
        description: '获取规格文档',
        category: 'documentation',
        parameters: {
          specType: { type: 'string', description: '规格类型', required: true },
          specId: { type: 'string', description: '规格ID', required: true }
        }
      },
      {
        name: 'list_specs',
        description: '列出所有规格文档',
        category: 'documentation',
        parameters: {
          specType: { type: 'string', description: '规格类型', required: true }
        }
      },
      {
        name: 'validate_spec',
        description: '验证规格文档',
        category: 'documentation',
        parameters: {
          specType: { type: 'string', description: '规格类型', required: true },
          specData: { type: 'object', description: '规格数据', required: true }
        }
      }
    ]
  }

  /**
   * 执行工具
   * @param toolName - 工具名称
   * @param parameters - 工具参数
   * @returns 执行结果
   */
  static async executeTool(toolName: string, parameters: Record<string, any>): Promise<any> {
    switch (toolName) {
      case 'code_completion':
        return this.getCodeCompletions(
          parameters.code,
          parameters.language,
          parameters.position
        )
      
      case 'code_diagnostics':
        return this.diagnoseCode(
          parameters.code,
          parameters.language
        )
      
      case 'code_format':
        return this.formatCode(
          parameters.code,
          parameters.language
        )
      
      case 'code_hover':
        return this.getHoverInfo(
          parameters.code,
          parameters.language,
          parameters.position
        )
      
      case 'create_spec':
        return this.createSpec(
          parameters.specType,
          parameters.specData,
          parameters.templateId
        )
      
      case 'get_spec':
        return this.getSpec(
          parameters.specType,
          parameters.specId
        )
      
      case 'list_specs':
        return this.listSpecs(parameters.specType)
      
      case 'validate_spec':
        return this.validateSpec(
          parameters.specType,
          parameters.specData
        )
      
      default:
        throw new Error(`未知的工具: ${toolName}`)
    }
  }
}

export default SdkToolsService
