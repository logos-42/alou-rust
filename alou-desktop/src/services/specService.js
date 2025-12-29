/**
 * Spec Service - Specification management integration
 * 与 Tauri Spec SDK 模块通信
 */
import { invoke } from '@tauri-apps/api/core'

/**
 * Spec 类型
 */
export const SpecType = {
  Product: 'product',
  Technical: 'technical',
  Design: 'design',
  Api: 'api',
  UserStory: 'user_story',
  Tasks: 'tasks',
  Structure: 'structure',
}

/**
 * Spec 操作类型
 */
export const SpecOperation = {
  Create: 'create',
  Update: 'update',
  Get: 'get',
  Delete: 'delete',
  List: 'list',
  Validate: 'validate',
  GenerateFromTemplate: 'generate_from_template',
  Export: 'export',
  Import: 'import',
}

/**
 * 创建 Spec Service 类
 */
export class SpecService {
  /**
   * 创建新的规格文档
   * @param {Object} options - 创建选项
   * @param {string} options.specType - 规格类型
   * @param {Object} options.specData - 规格数据
   * @param {string} [options.templateId] - 模板ID（可选）
   * @returns {Promise<Object>} 创建的规格文档
   */
  async createSpec({ specType, specData, templateId }) {
    const request = {
      spec_type: specType,
      operation: SpecOperation.Create,
      spec_data: specData,
      template_id: templateId,
    }

    const response = await invoke('execute_spec', { request })
    
    if (!response.success) {
      throw new Error(response.error || '创建规格文档失败')
    }

    return response.spec
  }

  /**
   * 更新规格文档
   * @param {Object} options - 更新选项
   * @param {string} options.specId - 规格文档ID
   * @param {Object} options.specData - 更新的规格数据
   * @returns {Promise<Object>} 更新后的规格文档
   */
  async updateSpec({ specId, specData }) {
    const request = {
      spec_id: specId,
      operation: SpecOperation.Update,
      spec_data: specData,
    }

    const response = await invoke('execute_spec', { request })
    
    if (!response.success) {
      throw new Error(response.error || '更新规格文档失败')
    }

    return response.spec
  }

  /**
   * 获取规格文档
   * @param {string} specId - 规格文档ID
   * @returns {Promise<Object>} 规格文档
   */
  async getSpec(specId) {
    const request = {
      spec_id: specId,
      operation: SpecOperation.Get,
    }

    const response = await invoke('execute_spec', { request })
    
    if (!response.success) {
      throw new Error(response.error || '获取规格文档失败')
    }

    return response.spec
  }

  /**
   * 删除规格文档
   * @param {string} specId - 规格文档ID
   * @returns {Promise<boolean>} 删除成功
   */
  async deleteSpec(specId) {
    const request = {
      spec_id: specId,
      operation: SpecOperation.Delete,
    }

    const response = await invoke('execute_spec', { request })
    
    if (!response.success) {
      throw new Error(response.error || '删除规格文档失败')
    }

    return true
  }

  /**
   * 列出所有规格文档
   * @param {Object} options - 查询选项
   * @param {string} [options.specType] - 按类型过滤（可选）
   * @returns {Promise<Array>} 规格文档列表
   */
  async listSpecs({ specType } = {}) {
    const request = {
      spec_type: specType,
      operation: SpecOperation.List,
    }

    const response = await invoke('execute_spec', { request })
    
    if (!response.success) {
      throw new Error(response.error || '列出规格文档失败')
    }

    return response.specs || []
  }

  /**
   * 验证规格文档
   * @param {Object} options - 验证选项
   * @param {string} options.specId - 规格文档ID
   * @param {string} [options.specType] - 规格类型（可选）
   * @returns {Promise<Object>} 验证结果
   */
  async validateSpec({ specId, specType }) {
    const request = {
      spec_id: specId,
      spec_type: specType,
      operation: SpecOperation.Validate,
    }

    const response = await invoke('execute_spec', { request })
    
    if (!response.success) {
      throw new Error(response.error || '验证规格文档失败')
    }

    return response.validation_result
  }

  /**
   * 从模板生成规格文档
   * @param {Object} options - 生成选项
   * @param {string} options.templateId - 模板ID
   * @param {Object} options.specData - 规格数据
   * @returns {Promise<Object>} 生成的规格文档
   */
  async generateFromTemplate({ templateId, specData }) {
    const request = {
      template_id: templateId,
      operation: SpecOperation.GenerateFromTemplate,
      spec_data: specData,
    }

    const response = await invoke('execute_spec', { request })
    
    if (!response.success) {
      throw new Error(response.error || '从模板生成规格文档失败')
    }

    return response.spec
  }

  /**
   * 导出规格文档
   * @param {Object} options - 导出选项
   * @param {string} options.specId - 规格文档ID
   * @param {string} options.outputFormat - 输出格式（md, json, html, pdf）
   * @returns {Promise<Object>} 导出结果
   */
  async exportSpec({ specId, outputFormat }) {
    const request = {
      spec_id: specId,
      operation: SpecOperation.Export,
      output_format: outputFormat,
    }

    const response = await invoke('execute_spec', { request })
    
    if (!response.success) {
      throw new Error(response.error || '导出规格文档失败')
    }

    return response.export_result
  }

  /**
   * 导入规格文档
   * @param {Object} options - 导入选项
   * @param {Object} options.specData - 规格数据
   * @param {string} options.specType - 规格类型
   * @returns {Promise<Object>} 导入的规格文档
   */
  async importSpec({ specData, specType }) {
    const request = {
      spec_data: specData,
      spec_type: specType,
      operation: SpecOperation.Import,
    }

    const response = await invoke('execute_spec', { request })
    
    if (!response.success) {
      throw new Error(response.error || '导入规格文档失败')
    }

    return response.spec
  }

  /**
   * 获取规格模板列表
   * @returns {Promise<Array>} 规格模板列表
   */
  async getTemplates() {
    return await invoke('get_spec_templates')
  }

  /**
   * 加载模板内容
   * @param {string} templateId - 模板ID
   * @returns {Promise<string>} 模板内容
   */
  async loadTemplate(templateId) {
    return await invoke('load_template_content', { templateId })
  }

  /**
   * 创建产品需求规格
   * @param {Object} data - 产品需求数据
   * @returns {Promise<Object>} 创建的产品规格
   */
  async createProductSpec(data) {
    return this.createSpec({
      specType: SpecType.Product,
      specData: data,
      templateId: 'product',
    })
  }

  /**
   * 创建技术规格
   * @param {Object} data - 技术数据
   * @returns {Promise<Object>} 创建的技术规格
   */
  async createTechnicalSpec(data) {
    return this.createSpec({
      specType: SpecType.Technical,
      specData: data,
      templateId: 'tech',
    })
  }

  /**
   * 创建设计规格
   * @param {Object} data - 设计数据
   * @returns {Promise<Object>} 创建的设计规格
   */
  async createDesignSpec(data) {
    return this.createSpec({
      specType: SpecType.Design,
      specData: data,
      templateId: 'design',
    })
  }

  /**
   * 创建API规格
   * @param {Object} data - API数据
   * @returns {Promise<Object>} 创建的API规格
   */
  async createApiSpec(data) {
    return this.createSpec({
      specType: SpecType.Api,
      specData: data,
    })
  }

  /**
   * 创建用户故事
   * @param {Object} data - 用户故事数据
   * @returns {Promise<Object>} 创建的用户故事
   */
  async createUserStory(data) {
    return this.createSpec({
      specType: SpecType.UserStory,
      specData: data,
    })
  }

  /**
   * 创建任务规格
   * @param {Object} data - 任务数据
   * @returns {Promise<Object>} 创建的任务规格
   */
  async createTasksSpec(data) {
    return this.createSpec({
      specType: SpecType.Tasks,
      specData: data,
      templateId: 'tasks',
    })
  }

  /**
   * 创建结构规格
   * @param {Object} data - 结构数据
   * @returns {Promise<Object>} 创建的结构规格
   */
  async createStructureSpec(data) {
    return this.createSpec({
      specType: SpecType.Structure,
      specData: data,
      templateId: 'structure',
    })
  }
}

export default new SpecService()
