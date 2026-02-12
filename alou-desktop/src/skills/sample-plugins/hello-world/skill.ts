/**
 * 示例技能 - Hello World
 * 
 * 这是一个最简单的插件示例，展示了如何创建自定义技能
 */

import { Skill, PluginContext, SkillResult } from '../../skill-sdk';

class HelloWorldSkill extends Skill {
  name = 'hello_world';
  description = '输出 Hello World 和当前时间';
  version = '1.0.0';
  category = 'utility';
  
  parameters = {
    type: 'object',
    properties: {
      name: {
        type: 'string',
        description: '要问候的名字',
        default: 'World',
      },
      useEmoji: {
        type: 'boolean',
        description: '是否使用表情',
        default: true,
      },
    },
    required: ['name'],
  };
  
  async execute(params: Record<string, any>, context: PluginContext): Promise<SkillResult> {
    const { name, useEmoji = true } = params;
    
    const emoji = useEmoji ? '👋 ' : '';
    const greeting = `${emoji}你好，${name}！当前时间是 ${new Date().toLocaleTimeString()}`;
    
    // 记录日志
    context.logger.info(`问候用户: ${name}`);
    
    // 返回结果
    return {
      success: true,
      output: greeting,
      metadata: {
        timestamp: new Date().toISOString(),
        greetedName: name,
      },
    };
  }
}

export default new HelloWorldSkill();
