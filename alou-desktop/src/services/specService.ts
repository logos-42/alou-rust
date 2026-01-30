/**
 * Spec Service - Specification management integration
 * 与 Tauri Spec SDK 模块通信
 */
import { invoke } from '@tauri-apps/api/core'

/**
 * Spec 类型
 */
export enum SpecType {
  Product = 'product',
  Technical = 'technical',
  Design = 'design',
  Api = 'api',
  UserStory = 'user_story',
  Tasks = 'tasks',
  Structure = 'structure',
}

/**
 * Spec 操作类型
 */
export enum SpecOperation {
  Create = 'create',
  Update = 'update',
  Get = 'get',
  Delete = 'delete',
  List = 'list',
  Validate = 'validate',
  GenerateFromTemplate = 'generate_from_template',
  Export = 'export',
  Import = 'import',
}

/**
 * Spec 请求
 */
export interface SpecRequest {
  spec_type?: string
  spec_id?: string
  operation: SpecOperation | string
  spec_data?: Record<string, any>
  template_id?: string
  output_format?: string
}

/**
 * Spec 响应
 */
export interface SpecResponse {
  success: boolean
  spec?: any
  specs?: any[]
  error?: string
  validation_result?: any
  export_result?: any
}

/**
 * 创建 Spec 选项
 */
export interface CreateSpecOptions {
  specType: string
  specData: Record<string, any>
  templateId?: string
}

/**
 * 更新 Spec 选项
 */
export interface UpdateSpecOptions {
  specId: string
  specData: Record<string, any>
}

/**
 * 验证 Spec 选项
 */
export interface ValidateSpecOptions {
  specId: string
  specType?: string
}

/**
 * 从模板生成 Spec 选项
 */
export interface GenerateFromTemplateOptions {
  templateId: string
  specData: Record<string, any>
}

/**
 * 导出 Spec 选项
 */
export interface ExportSpecOptions {
  specId: string
  outputFormat: 'md' | 'json' | 'html' | 'pdf'
}

/**
 * 导入 Spec 选项
 */
export interface ImportSpecOptions {
  specData: Record<string, any>
  specType: string
}

/**
 * 创建 Spec Service 类
 */
export class SpecService {
  /**
   * 创建新的规格文档
   * @param options - 创建选项
   * @returns 创建的规格文档
   */
  async createSpec({ specType, specData, templateId }: CreateSpecOptions): Promise<any> {
    const request: SpecRequest = {
      spec_type: specType,
      operation: SpecOperation.Create,
      spec_data: specData,
      template_id: templateId,
    }

    const response = await invoke<SpecResponse>('execute_spec', { request })
    
    if (!response.success) {
      throw new Error(response.error || '创建规格文档失败')
    }

    return response.spec
  }

  /**
   * 更新规格文档
   * @param options - 更新选项
   * @returns 更新后的规格文档
   */
  async updateSpec({ specId, specData }: UpdateSpecOptions): Promise<any> {
    const request: SpecRequest = {
      spec_id: specId,
      operation: SpecOperation.Update,
      spec_data: specData,
    }

    const response = await invoke<SpecResponse>('execute_spec', { request })
    
    if (!response.success) {
      throw new Error(response.error || '更新规格文档失败')
    }

    return response.spec
  }

  /**
   * 获取规格文档
   * @param specId - 规格文档ID
   * @returns 规格文档
   */
  async getSpec(specId: string): Promise<any> {
    const request: SpecRequest = {
      spec_id: specId,
      operation: SpecOperation.Get,
    }

    const response = await invoke<SpecResponse>('execute_spec', { request })
    
    if (!response.success) {
      throw new Error(response.error || '获取规格文档失败')
    }

    return response.spec
  }

  /**
   * 删除规格文档
   * @param specId - 规格文档ID
   * @returns 删除成功
   */
  async deleteSpec(specId: string): Promise<boolean> {
    const request: SpecRequest = {
      spec_id: specId,
      operation: SpecOperation.Delete,
    }

    const response = await invoke<SpecResponse>('execute_spec', { request })
    
    if (!response.success) {
      throw new Error(response.error || '删除规格文档失败')
    }

    return true
  }

  /**
   * 列出所有规格文档
   * @param options - 查询选项
   * @returns 规格文档列表
   */
  async listSpecs({ specType }: { specType?: string } = {}): Promise<any[]> {
    const request: SpecRequest = {
      spec_type: specType,
      operation: SpecOperation.List,
    }

    const response = await invoke<SpecResponse>('execute_spec', { request })
    
    if (!response.success) {
      throw new Error(response.error || '列出规格文档失败')
    }

    return response.specs || []
  }

  /**
   * 验证规格文档
   * @param options - 验证选项
   * @returns 验证结果
   */
  async validateSpec({ specId, specType }: ValidateSpecOptions): Promise<any> {
    const request: SpecRequest = {
      spec_id: specId,
      spec_type: specType,
      operation: SpecOperation.Validate,
    }

    const response = await invoke<SpecResponse>('execute_spec', { request })
    
    if (!response.success) {
      throw new Error(response.error || '验证规格文档失败')
    }

    return response.validation_result
  }

  /**
   * 从模板生成规格文档
   * @param options - 生成选项
   * @returns 生成的规格文档
   */
  async generateFromTemplate({ templateId, specData }: GenerateFromTemplateOptions): Promise<any> {
    const request: SpecRequest = {
      template_id: templateId,
      operation: SpecOperation.GenerateFromTemplate,
      spec_data: specData,
    }

    const response = await invoke<SpecResponse>('execute_spec', { request })
    
    if (!response.success) {
      throw new Error(response.error || '从模板生成规格文档失败')
    }

    return response.spec
  }

  /**
   * 导出规格文档
   * @param options - 导出选项
   * @returns 导出结果
   */
  async exportSpec({ specId, outputFormat }: ExportSpecOptions): Promise<any> {
    const request: SpecRequest = {
      spec_id: specId,
      operation: SpecOperation.Export,
      output_format: outputFormat,
    }

    const response = await invoke<SpecResponse>('execute_spec', { request })
    
    if (!response.success) {
      throw new Error(response.error || '导出规格文档失败')
    }

    return response.export_result
  }

  /**
   * 导入规格文档
   * @param options - 导入选项
   * @returns 导入的规格文档
   */
  async importSpec({ specData, specType }: ImportSpecOptions): Promise<any> {
    const request: SpecRequest = {
      spec_data: specData,
      spec_type: specType,
      operation: SpecOperation.Import,
    }

    const response = await invoke<SpecResponse>('execute_spec', { request })
    
    if (!response.success) {
      throw new Error(response.error || '导入规格文档失败')
    }

    return response.spec
  }

  /**
   * 获取规格模板列表
   * @returns 规格模板列表
   */
  async getTemplates(): Promise<any[]> {
    return await invoke<any[]>('get_spec_templates')
  }

  /**
   * 加载模板内容
   * @param templateId - 模板ID
   * @returns 模板内容
   */
  async loadTemplate(templateId: string): Promise<string> {
    return await invoke<string>('load_template_content', { templateId })
  }

  /**
   * 创建产品需求规格
   * @param data - 产品需求数据
   * @returns 创建的产品规格
   */
  async createProductSpec(data: Record<string, any>): Promise<any> {
    return this.createSpec({
      specType: SpecType.Product,
      specData: data,
      templateId: 'product',
    })
  }

  /**
   * 创建技术规格
   * @param data - 技术数据
   * @returns 创建的技术规格
   */
  async createTechnicalSpec(data: Record<string, any>): Promise<any> {
    return this.createSpec({
      specType: SpecType.Technical,
      specData: data,
      templateId: 'tech',
    })
  }

  /**
   * 创建设计规格
   * @param data - 设计数据
   * @returns 创建的设计规格
   */
  async createDesignSpec(data: Record<string, any>): Promise<any> {
    return this.createSpec({
      specType: SpecType.Design,
      specData: data,
      templateId: 'design',
    })
  }

  /**
   * 创建API规格
   * @param data - API数据
   * @returns 创建的API规格
   */
  async createApiSpec(data: Record<string, any>): Promise<any> {
    return this.createSpec({
      specType: SpecType.Api,
      specData: data,
    })
  }

  /**
   * 创建用户故事
   * @param data - 用户故事数据
   * @returns 创建的用户故事
   */
  async createUserStory(data: Record<string, any>): Promise<any> {
    return this.createSpec({
      specType: SpecType.UserStory,
      specData: data,
    })
  }

  /**
   * 创建任务规格
   * @param data - 任务数据
   * @returns 创建的任务规格
   */
  async createTasksSpec(data: Record<string, any>): Promise<any> {
    return this.createSpec({
      specType: SpecType.Tasks,
      specData: data,
      templateId: 'tasks',
    })
  }

  /**
   * 创建结构规格
   * @param data - 结构数据
   * @returns 创建的结构规格
   */
  async createStructureSpec(data: Record<string, any>): Promise<any> {
    return this.createSpec({
      specType: SpecType.Structure,
      specData: data,
      templateId: 'structure',
    })
  }
}

export default new SpecService()
