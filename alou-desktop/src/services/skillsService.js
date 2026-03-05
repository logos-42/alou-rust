// ============================================
// Skills Service - 技能管理和服务
// 用于加载、管理和执行所有可用技能
// ============================================

import apiClient from './api'

class SkillsService {
  constructor() {
    this.skills = []
    this.initialized = false
    this.skillInstances = new Map()
  }

  /**
   * 初始化技能系统
   * 从后端 API 加载所有可用技能
   */
  async initializeSkills() {
    if (this.initialized) {
      console.log('[SkillsService] 技能系统已初始化')
      return
    }

    try {
      console.log('[SkillsService] 正在初始化技能系统...')

      // 尝试从后端 API 加载技能
      try {
        const response = await apiClient.get('/skills')
        this.skills = response.data.skills || []
        console.log(`[SkillsService] 从 API 加载了 ${this.skills.length} 个技能`)
      } catch (apiError) {
        console.warn('[SkillsService] API 加载失败，使用本地技能:', apiError.message)
        // API 不可用，使用本地技能
        await this.loadLocalSkills()
      }

      // 初始化技能实例
      this.initializeSkillInstances()

      this.initialized = true
      console.log(`[SkillsService] 技能系统初始化完成，共 ${this.skills.length} 个技能`)
    } catch (error) {
      console.error('[SkillsService] 初始化技能系统失败:', error)
      // 即使失败也标记为已初始化，避免重复尝试
      this.initialized = true
      throw error
    }
  }

  /**
   * 加载本地技能（从 skills 目录）
   */
  async loadLocalSkills() {
    try {
      // 动态导入 WorkflowSkill
      const { WorkflowSkill } = await import('../skills/WorkflowSkill')
      const workflowSkill = new WorkflowSkill()
      
      // 动态导入 IpfsAutoFixSkill
      let ipfsAutoFixSkill = null
      try {
        const ipfsAutoFixModule = await import('../skills/builtin/ipfs-auto-fix/skill')
        ipfsAutoFixSkill = ipfsAutoFixModule.default
      } catch (error) {
        console.warn('[SkillsService] IPFS Auto-Fix Skill 加载失败:', error)
      }

      this.skills = [
        workflowSkill.getDefinition(),
        ...(ipfsAutoFixSkill ? [{
          name: 'ipfs-auto-fix',
          displayName: 'IPFS 自动修复',
          description: 'IPFS 自动修复技能，检测和修复 IPFS 节点问题',
          version: '1.0.0',
          category: 'system',
          actions: [
            {
              name: 'check',
              displayName: '检查状态',
              description: '检查 IPFS 节点状态',
              parameters: {}
            },
            {
              name: 'fix',
              displayName: '自动修复',
              description: '自动检测并修复 IPFS 问题',
              parameters: {
                autoDownload: {
                  type: 'boolean',
                  description: '是否自动下载 Kubo',
                  default: true
                },
                maxRetries: {
                  type: 'number',
                  description: '最大重试次数',
                  default: 3
                }
              }
            },
            {
              name: 'start',
              displayName: '启动节点',
              description: '启动 IPFS 节点',
              parameters: {
                autoDownload: {
                  type: 'boolean',
                  description: '是否自动下载 Kubo',
                  default: true
                }
              }
            },
            {
              name: 'stop',
              displayName: '停止节点',
              description: '停止 IPFS 节点',
              parameters: {}
            },
            {
              name: 'restart',
              displayName: '重启节点',
              description: '重启 IPFS 节点',
              parameters: {
                autoDownload: {
                  type: 'boolean',
                  description: '是否自动下载 Kubo',
                  default: true
                }
              }
            },
            {
              name: 'diagnose',
              displayName: '诊断问题',
              description: '诊断 IPFS 问题并提供建议',
              parameters: {}
            }
          ],
          instance: ipfsAutoFixSkill
        }] : [])
      ]

      console.log('[SkillsService] 加载本地技能成功，共', this.skills.length, '个')
    } catch (error) {
      console.error('[SkillsService] 加载本地技能失败:', error)
      this.skills = []
    }
  }

  /**
   * 初始化技能实例
   */
  initializeSkillInstances() {
    this.skillInstances.clear()

    this.skills.forEach(skill => {
      try {
        // 如果技能有类定义，创建实例
        if (skill.instance) {
          this.skillInstances.set(skill.name, skill.instance)
        }
      } catch (error) {
        console.error(`[SkillsService] 初始化技能实例失败 ${skill.name}:`, error)
      }
    })
  }

  /**
   * 获取所有技能
   * @returns {Array} 技能列表
   */
  getAllSkills() {
    return [...this.skills]
  }

  /**
   * 根据名称获取技能
   * @param {string} skillName - 技能名称
   * @returns {Object|null} 技能定义
   */
  getSkill(skillName) {
    return this.skills.find(skill => skill.name === skillName) || null
  }

  /**
   * 获取技能的所有操作
   * @param {string} skillName - 技能名称
   * @returns {Object} 操作定义
   */
  getActions(skillName) {
    const skill = this.getSkill(skillName)
    return skill?.actions || {}
  }

  /**
   * 获取特定操作的参数定义
   * @param {string} skillName - 技能名称
   * @param {string} actionName - 操作名称
   * @returns {Object} 参数定义
   */
  getActionParameters(skillName, actionName) {
    const actions = this.getActions(skillName)
    return actions[actionName]?.parameters || {}
  }

  /**
   * 执行技能操作
   * @param {string} skillName - 技能名称
   * @param {string} actionName - 操作名称
   * @param {Object} parameters - 参数对象
   * @returns {Promise<Object>} 执行结果
   */
  async executeSkill(skillName, actionName, parameters = {}) {
    if (!this.initialized) {
      await this.initializeSkills()
    }

    const skill = this.getSkill(skillName)
    if (!skill) {
      throw new Error(`技能不存在: ${skillName}`)
    }

    const action = skill.actions[actionName]
    if (!action) {
      throw new Error(`技能操作不存在: ${skillName}.${actionName}`)
    }

    try {
      console.log(`[SkillsService] 执行技能: ${skillName}.${actionName}`, parameters)

      // 验证参数
      this.validateParameters(skillName, actionName, parameters)

      // 尝试通过 API 执行
      try {
        const response = await apiClient.post(`/skills/${skillName}/${actionName}`, parameters)
        console.log(`[SkillsService] 通过 API 执行成功`)

        return {
          success: true,
          result: response.data,
          timestamp: new Date().toISOString()
        }
      } catch (apiError) {
        // API 失败，尝试本地执行
        console.warn('[SkillsService] API 执行失败，尝试本地执行:', apiError.message)

        const localResult = await this.executeLocally(skillName, actionName, parameters)

        return {
          success: true,
          result: localResult,
          source: 'local',
          timestamp: new Date().toISOString()
        }
      }
    } catch (error) {
      console.error(`[SkillsService] 执行技能失败: ${skillName}.${actionName}`, error)

      return {
        success: false,
        error: error.message,
        timestamp: new Date().toISOString()
      }
    }
  }

  /**
   * 本地执行技能
   * @param {string} skillName - 技能名称
   * @param {string} actionName - 操作名称
   * @param {Object} parameters - 参数对象
   * @returns {Promise<Object>} 执行结果
   */
  async executeLocally(skillName, actionName, parameters) {
    const skillInstance = this.skillInstances.get(skillName)

    if (skillInstance) {
      // 对于 IPFS Auto-Fix skill，使用 execute 方法
      if (skillName === 'ipfs-auto-fix' && typeof skillInstance.execute === 'function') {
        return await skillInstance.execute({
          action: actionName,
          ...parameters
        })
      }
      
      // 对于其他 skills，使用 action 方法
      if (typeof skillInstance[actionName] === 'function') {
        return await skillInstance[actionName](parameters)
      }
    }

    // 默认实现：返回参数（用于测试）
    return {
      message: `本地执行 ${skillName}.${actionName}`,
      parameters
    }
  }

  /**
   * 验证参数
   * @param {string} skillName - 技能名称
   * @param {string} actionName - 操作名称
   * @param {Object} parameters - 参数对象
   * @throws {Error} 参数验证失败
   */
  validateParameters(skillName, actionName, parameters) {
    const paramDef = this.getActionParameters(skillName, actionName)

    // 检查必需参数
    if (paramDef.required) {
      const missing = paramDef.required.filter(key => !(key in parameters))

      if (missing.length > 0) {
        throw new Error(`缺少必需参数: ${missing.join(', ')}`)
      }
    }

    // 检查参数类型
    if (paramDef.properties) {
      Object.entries(parameters).forEach(([key, value]) => {
        const propDef = paramDef.properties[key]

        if (propDef) {
          this.validateParameterType(key, value, propDef.type)
        }
      })
    }
  }

  /**
   * 验证单个参数类型
   * @param {string} key - 参数名
   * @param {any} value - 参数值
   * @param {string} expectedType - 期望类型
   * @throws {Error} 类型验证失败
   */
  validateParameterType(key, value, expectedType) {
    const actualType = Array.isArray(value) ? 'array' : typeof value

    // 简化类型检查
    const typeMap = {
      string: 'string',
      number: 'number',
      boolean: 'boolean',
      array: 'array',
      object: 'object'
    }

    if (typeMap[expectedType] !== actualType) {
      throw new Error(`参数 ${key} 类型错误: 期望 ${expectedType}, 实际 ${actualType}`)
    }
  }

  /**
   * 获取系统状态
   * @returns {Promise<Object>} 系统状态
   */
  async getSystemStatus() {
    try {
      const response = await apiClient.get('/skills/status')

      return {
        agent_skills_available: response.data.available || false,
        total_skills: this.skills.length,
        initialized: this.initialized,
        api_connected: true,
        ...response.data
      }
    } catch (error) {
      return {
        agent_skills_available: false,
        total_skills: this.skills.length,
        initialized: this.initialized,
        api_connected: false,
        error: error.message
      }
    }
  }

  /**
   * 获取统计信息
   * @returns {Object} 统计数据
   */
  getStats() {
    let totalActions = 0
    const categories = new Set()

    this.skills.forEach(skill => {
      if (skill.actions) {
        totalActions += Object.keys(skill.actions).length
      }

      if (skill.category) {
        categories.add(skill.category)
      }
    })

    return {
      totalSkills: this.skills.length,
      totalActions,
      categories: Array.from(categories),
      initialized: this.initialized
    }
  }

  /**
   * 根据类别获取技能
   * @param {string} category - 类别名称
   * @returns {Array} 技能列表
   */
  getSkillsByCategory(category) {
    if (category === 'all') {
      return this.getAllSkills()
    }

    return this.skills.filter(skill => skill.category === category)
  }

  /**
   * 搜索技能
   * @param {string} query - 搜索关键词
   * @returns {Array} 匹配的技能列表
   */
  searchSkills(query) {
    const lowerQuery = query.toLowerCase()

    return this.skills.filter(skill =>
      skill.name.toLowerCase().includes(lowerQuery) ||
      skill.description.toLowerCase().includes(lowerQuery) ||
      (skill.category && skill.category.toLowerCase().includes(lowerQuery))
    )
  }

  /**
   * 重置技能系统
   */
  reset() {
    this.skills = []
    this.skillInstances.clear()
    this.initialized = false
    console.log('[SkillsService] 技能系统已重置')
  }
}

// 导出单例
export default new SkillsService()
