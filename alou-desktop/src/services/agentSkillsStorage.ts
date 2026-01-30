/**
 * AgentSkillsStorage - 智能体技能配置本地存储服务
 * 为每个智能体保存独立的技能配置和偏好设置
 */

import type {
  AgentSkillsConfig,
  AllAgentConfigs,
  SkillSettings,
  SkillPreferences,
  CustomSkill
} from '@shared/types/services';

class AgentSkillsStorage {
  private storageKey: string;
  private globalSettingsKey: string;

  constructor() {
    this.storageKey = 'alou-agent-skills';
    this.globalSettingsKey = 'alou-skills-global-settings';
  }

  /**
   * 获取智能体的技能配置
   * @param agentId - 智能体ID
   * @returns 技能配置
   */
  getAgentSkillsConfig(agentId: string): AgentSkillsConfig {
    try {
      const allConfigs = this.getAllConfigs();
      return allConfigs[agentId] || this.getDefaultConfig();
    } catch (error) {
      console.error('[AgentSkillsStorage] 获取智能体技能配置失败:', error);
      return this.getDefaultConfig();
    }
  }

  /**
   * 保存智能体的技能配置
   * @param agentId - 智能体ID
   * @param config - 技能配置
   */
  saveAgentSkillsConfig(agentId: string, config: Partial<AgentSkillsConfig>): void {
    try {
      const allConfigs = this.getAllConfigs();
      allConfigs[agentId] = {
        ...this.getDefaultConfig(),
        ...config,
        updatedAt: new Date().toISOString(),
        agentId
      };
      
      localStorage.setItem(this.storageKey, JSON.stringify(allConfigs));
      console.log(`[AgentSkillsStorage] 已保存智能体 ${agentId} 的技能配置`);
    } catch (error) {
      console.error('[AgentSkillsStorage] 保存智能体技能配置失败:', error);
    }
  }

  /**
   * 删除智能体的技能配置
   * @param agentId - 智能体ID
   */
  removeAgentSkillsConfig(agentId: string): void {
    try {
      const allConfigs = this.getAllConfigs();
      delete allConfigs[agentId];
      localStorage.setItem(this.storageKey, JSON.stringify(allConfigs));
      console.log(`[AgentSkillsStorage] 已删除智能体 ${agentId} 的技能配置`);
    } catch (error) {
      console.error('[AgentSkillsStorage] 删除智能体技能配置失败:', error);
    }
  }

  /**
   * 获取所有智能体的技能配置
   * @returns 所有配置
   */
  getAllConfigs(): AllAgentConfigs {
    try {
      const stored = localStorage.getItem(this.storageKey);
      return stored ? JSON.parse(stored) : {};
    } catch (error) {
      console.error('[AgentSkillsStorage] 获取所有配置失败:', error);
      return {};
    }
  }

  /**
   * 获取默认技能配置
   * @returns 默认配置
   */
  getDefaultConfig(): AgentSkillsConfig {
    return {
      enabledSkills: [], // 启用的技能列表
      disabledSkills: [], // 禁用的技能列表
      skillSettings: {}, // 技能特定设置
      customSkills: [], // 自定义技能
      preferences: {
        autoExecute: false,
        showExamples: true,
        enableNotifications: true,
        defaultCategory: 'all'
      },
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString()
    };
  }

  /**
   * 启用技能
   * @param agentId - 智能体ID
   * @param skillName - 技能名称
   */
  enableSkill(agentId: string, skillName: string): void {
    const config = this.getAgentSkillsConfig(agentId);
    if (!config.enabledSkills.includes(skillName)) {
      config.enabledSkills.push(skillName);
    }
    config.disabledSkills = config.disabledSkills.filter(name => name !== skillName);
    this.saveAgentSkillsConfig(agentId, config);
  }

  /**
   * 禁用技能
   * @param agentId - 智能体ID
   * @param skillName - 技能名称
   */
  disableSkill(agentId: string, skillName: string): void {
    const config = this.getAgentSkillsConfig(agentId);
    if (!config.disabledSkills.includes(skillName)) {
      config.disabledSkills.push(skillName);
    }
    config.enabledSkills = config.enabledSkills.filter(name => name !== skillName);
    this.saveAgentSkillsConfig(agentId, config);
  }

  /**
   * 检查技能是否启用
   * @param agentId - 智能体ID
   * @param skillName - 技能名称
   * @returns 是否启用
   */
  isSkillEnabled(agentId: string, skillName: string): boolean {
    const config = this.getAgentSkillsConfig(agentId);
    if (config.disabledSkills.includes(skillName)) {
      return false;
    }
    if (config.enabledSkills.length > 0) {
      return config.enabledSkills.includes(skillName);
    }
    return true; // 默认启用
  }

  /**
   * 保存技能设置
   * @param agentId - 智能体ID
   * @param skillName - 技能名称
   * @param settings - 技能设置
   */
  saveSkillSettings(agentId: string, skillName: string, settings: Partial<SkillSettings>): void {
    const config = this.getAgentSkillsConfig(agentId);
    config.skillSettings[skillName] = {
      ...config.skillSettings[skillName],
      ...settings,
      updatedAt: new Date().toISOString()
    };
    this.saveAgentSkillsConfig(agentId, config);
  }

  /**
   * 获取技能设置
   * @param agentId - 智能体ID
   * @param skillName - 技能名称
   * @returns 技能设置
   */
  getSkillSettings(agentId: string, skillName: string): SkillSettings {
    const config = this.getAgentSkillsConfig(agentId);
    return config.skillSettings[skillName] || {};
  }

  /**
   * 添加自定义技能
   * @param agentId - 智能体ID
   * @param skill - 技能定义
   */
  addCustomSkill(agentId: string, skill: Omit<CustomSkill, 'createdAt' | 'updatedAt'>): void {
    const config = this.getAgentSkillsConfig(agentId);
    const existingIndex = config.customSkills.findIndex(s => s.name === skill.name);
    
    if (existingIndex >= 0) {
      config.customSkills[existingIndex] = {
        ...skill,
        updatedAt: new Date().toISOString()
      };
    } else {
      config.customSkills.push({
        ...skill,
        createdAt: new Date().toISOString(),
        updatedAt: new Date().toISOString()
      });
    }
    
    this.saveAgentSkillsConfig(agentId, config);
  }

  /**
   * 删除自定义技能
   * @param agentId - 智能体ID
   * @param skillName - 技能名称
   */
  removeCustomSkill(agentId: string, skillName: string): void {
    const config = this.getAgentSkillsConfig(agentId);
    config.customSkills = config.customSkills.filter(skill => skill.name !== skillName);
    this.saveAgentSkillsConfig(agentId, config);
  }

  /**
   * 获取自定义技能列表
   * @param agentId - 智能体ID
   * @returns 自定义技能列表
   */
  getCustomSkills(agentId: string): CustomSkill[] {
    const config = this.getAgentSkillsConfig(agentId);
    return config.customSkills || [];
  }

  /**
   * 更新偏好设置
   * @param agentId - 智能体ID
   * @param preferences - 偏好设置
   */
  updatePreferences(agentId: string, preferences: Partial<SkillPreferences>): void {
    const config = this.getAgentSkillsConfig(agentId);
    config.preferences = {
      ...config.preferences,
      ...preferences
    };
    this.saveAgentSkillsConfig(agentId, config);
  }

  /**
   * 获取偏好设置
   * @param agentId - 智能体ID
   * @returns 偏好设置
   */
  getPreferences(agentId: string): SkillPreferences {
    const config = this.getAgentSkillsConfig(agentId);
    return config.preferences || this.getDefaultConfig().preferences;
  }

  /**
   * 清理过期数据
   * @param daysOld - 保留天数
   */
  cleanupOldData(daysOld: number = 30): void {
    try {
      const allConfigs = this.getAllConfigs();
      const cutoffDate = new Date();
      cutoffDate.setDate(cutoffDate.getDate() - daysOld);
      
      let cleanedCount = 0;
      Object.keys(allConfigs).forEach(agentId => {
        const config = allConfigs[agentId];
        const lastUpdated = new Date(config.updatedAt);
        
        if (lastUpdated < cutoffDate) {
          delete allConfigs[agentId];
          cleanedCount++;
        }
      });
      
      if (cleanedCount > 0) {
        localStorage.setItem(this.storageKey, JSON.stringify(allConfigs));
        console.log(`[AgentSkillsStorage] 清理了 ${cleanedCount} 个过期配置`);
      }
    } catch (error) {
      console.error('[AgentSkillsStorage] 清理过期数据失败:', error);
    }
  }

  /**
   * 导出配置
   * @param agentId - 智能体ID
   * @returns JSON字符串
   */
  exportConfig(agentId: string): string | null {
    try {
      const config = this.getAgentSkillsConfig(agentId);
      return JSON.stringify(config, null, 2);
    } catch (error) {
      console.error('[AgentSkillsStorage] 导出配置失败:', error);
      return null;
    }
  }

  /**
   * 导入配置
   * @param agentId - 智能体ID
   * @param configJson - 配置JSON字符串
   * @returns 是否成功
   */
  importConfig(agentId: string, configJson: string): boolean {
    try {
      const config = JSON.parse(configJson) as AgentSkillsConfig;
      this.saveAgentSkillsConfig(agentId, config);
      return true;
    } catch (error) {
      console.error('[AgentSkillsStorage] 导入配置失败:', error);
      return false;
    }
  }
}

// 创建单例实例
const agentSkillsStorage = new AgentSkillsStorage();

export default agentSkillsStorage;
