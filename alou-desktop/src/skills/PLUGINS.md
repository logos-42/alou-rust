# Alou 插件系统

## 概述

Alou 支持用户自定义插件，每个人都可以根据自己的需求创建插件来扩展功能。

## 插件目录结构

```
~/.alou/plugins/
├── my-plugin/           # 插件文件夹
│   ├── config.json     # 插件配置
│   ├── skill.ts        # 技能实现
│   └── README.md       # 插件说明
└── another-plugin/
    └── ...
```

## 创建插件

### 1. 创建插件文件夹

```bash
mkdir -p ~/.alou/plugins/my-plugin
```

### 2. 创建配置文件 `config.json`

```json
{
  "id": "my-plugin",
  "name": "我的插件",
  "version": "1.0.0",
  "description": "这是一个示例插件",
  "author": "你的名字",
  "skills": ["skill.ts"],
  "enabled": true
}
```

### 3. 创建技能文件 `skill.ts`

```typescript
import { Skill, PluginContext, SkillResult } from '@alou/skill-sdk';

class MySkill extends Skill {
  name = 'my_skill';
  description = '我的自定义技能';
  version = '1.0.0';
  category = 'utility';
  
  parameters = {
    type: 'object',
    properties: {
      input: {
        type: 'string',
        description: '输入参数',
      },
    },
    required: ['input'],
  };
  
  async execute(params: Record<string, any>, context: PluginContext): Promise<SkillResult> {
    // 实现技能逻辑
    return {
      success: true,
      output: `处理结果: ${params.input}`,
    };
  }
}

export default new MySkill();
```

## 使用插件

### 方式1：通过对话

```
用户: 请使用 my_skill 技能处理 "Hello"
Alou: 处理结果: Hello
```

### 方式2：代码调用

```typescript
import pluginLoader from './plugin-loader';

// 获取技能
const skill = pluginLoader.getSkillByName('my_skill');

// 执行技能
const result = await skill.execute(
  { input: 'Hello' },
  context
);

console.log(result.output);
```

## 插件配置说明

| 字段 | 必填 | 说明 |
|------|------|------|
| id | 是 | 插件唯一标识 |
| name | 是 | 插件名称 |
| version | 是 | 版本号 |
| description | 是 | 描述 |
| skills | 是 | 技能文件列表 |
| enabled | 否 | 是否启用，默认 true |

## 技能属性

| 属性 | 说明 |
|------|------|
| name | 技能名称（唯一） |
| description | 技能描述 |
| version | 版本号 |
| category | 分类：utility, automation, data, etc. |
| parameters | 参数定义 |
| enabled | 是否启用 |

## 示例插件

查看 `sample-plugins/hello-world/` 目录了解完整的示例实现。

## 注意事项

1. 插件文件使用 TypeScript 编写
2. 需要从 `@alou/skill-sdk` 导入类型
3. 技能名称必须唯一
4. 修改插件后需要重启 Alou
