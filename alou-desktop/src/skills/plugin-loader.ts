/**
 * 插件加载器
 * 
 * 负责管理 ~/.alou/plugins/ 目录下的用户自定义插件
 * 
 * 注意：这个模块需要在 Node.js 环境中运行（如 Tauri 主进程）
 */

import { Skill, PluginInfo } from './skill-sdk';

interface PluginConfig {
  id: string;
  name: string;
  version: string;
  description: string;
  author?: string;
  skills: string[];
  enabled: boolean;
}

interface LoadedPlugin {
  info: PluginConfig;
  skills: Skill[];
  enabled: boolean;
}

export class PluginLoader {
  private plugins: Map<string, LoadedPlugin> = new Map();
  private initialized: boolean = false;
  
  async initialize(): Promise<void> {
    if (this.initialized) return;
    
    console.log('[PluginLoader] 插件系统初始化...');
    this.initialized = true;
  }
  
  async loadPlugins(): Promise<void> {
    await this.initialize();
    console.log('[PluginLoader] 插件加载完成（需要在 Node 环境中运行）');
  }
  
  getAllSkills(): Skill[] {
    const skills: Skill[] = [];
    for (const plugin of this.plugins.values()) {
      if (plugin.enabled) {
        skills.push(...plugin.skills);
      }
    }
    return skills;
  }
  
  getSkillByName(name: string): Skill | undefined {
    for (const plugin of this.plugins.values()) {
      if (plugin.enabled) {
        const skill = plugin.skills.find(s => s.name === name);
        if (skill) return skill;
      }
    }
    return undefined;
  }
  
  getPlugins(): PluginInfo[] {
    return Array.from(this.plugins.values())
      .filter(p => p.enabled)
      .map(p => ({
        id: p.info.id,
        name: p.info.name,
        version: p.info.version,
        description: p.info.description,
        author: p.info.author,
        skills: p.info.skills,
      }));
  }
  
  setPluginEnabled(pluginId: string, enabled: boolean): boolean {
    const plugin = this.plugins.get(pluginId);
    if (!plugin) return false;
    plugin.enabled = enabled;
    return true;
  }
}

export default new PluginLoader();
