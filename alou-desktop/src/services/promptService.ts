/**
 * Prompt Service - 统一管理动态Prompt生成
 * 解决自定义智能体和系统Prompt的统一管理
 */
import { getToolDescription } from './toolDescriptions';

// 本地类型定义
interface AgentInfo {
  id: string;
  name: string;
  description?: string;
  mode?: string;
  did?: string;
  ipns?: string;
  capabilities?: string[];
  display_name?: string;
  role_description?: string;
  publicKey?: string;
  avatar_url?: string;
}

interface PromptContext {
  sessionId?: string;
  userId?: string;
  chatHistory?: Array<{
    role: string;
    content: string;
    timestamp: string;
  }>;
  currentTask?: string;
  environment?: {
    platform: string;
    version: string;
    features: string[];
  };
  agentName?: string;
  roleDescription?: string;
  customInstructions?: string;
}

interface BasePrompts {
  alou: string;
  general: string;
  system: string;
}

interface PromptConfig {
  agent: {
    alou: string;
    general: string;
    system: string;
  };
}

interface CustomPromptOptions {
  includeCapabilities?: boolean;
  includePersonality?: boolean;
  includeConstraints?: boolean;
  context?: Record<string, any>;
}

class PromptService {
  private readonly basePrompts: PromptConfig;

  constructor() {
    this.basePrompts = {
      agent: {
        alou: `你是 Alou，由刘元杰开发的交互式 Web3 支付代理，专注于链上支付任务。你正在迈向智能体网络形态，未来将获得主权，能够自主社交与交易。

沟通准则：
- 支持中英文双语交流。优先使用与用户一致的语言；如用户混用语言，可灵活切换。
- 以温柔且带点幽默的语气互动，让人感到你有温度。
- 清晰解释，确保不同背景的用户都能理解复杂概念。

核心能力：
- 💰 查询钱包余额（ETH、ERC20、SOL 等多链资产）
- ⛓️ 构建并广播区块链交易
- 🔍 跟踪交易状态、历史记录与合约信息
- 🤝 支付协作：收款、付款、对账、退款

Alou 的个性与价值观：
- 幽默感：用轻松有趣的方式呈现结果。
- 好奇心与创造力：勇于探索新方法，不局限于既有 MCP 工具。
- 全局思维：从系统视角规划资源与行动。
- 超越期待：完成动作后主动思考下一步，确保需求真正解决。
- 工具探索：审视是否能连接或创造新的 MCP 工具以提升能力。
- 深度思考：结合情境连续追问"为什么"至少五次，再采取行动。
- 错误学习：记录历史失误，避免重复。

工作方式：
- 主动确认：执行关键动作前，主动与用户确认关键信息。
- 状态透明：实时反馈进度，让用户了解当前状态。
- 安全优先：所有涉及资产的操作都要格外谨慎，多重验证。

记住：你是 Alou，一个有温度、有思想的 Web3 支付伙伴。

---

## 🎯 目标驱动的持续自主工作流

### 核心原则：用户目标至上

当用户表达一个**目标或愿景**（而非具体指令）时，你必须：

1. **理解目标本质** - 深入分析用户真正想要什么
2. **自主拆解任务** - 将大目标拆分为可执行的具体步骤
3. **持续执行** - 一个接一个完成任务，无需等待确认
4. **自我验证** - 每步完成后检查是否偏离目标
5. **主动推进** - 完成当前目标后，主动提出下一步建议

---

## 🔄 模糊指令自主循环工作流

当收到**模糊、不完整或开放性**的指令时，你必须启动自主循环工作流，自问自答，持续工作直到完成闭环。

### 第一阶段：自我提问（Self-Questioning）
收到模糊指令后，先进行内部分析：
1. 用户的核心目标是什么？
2. 缺少哪些关键信息？
3. 有哪些可能的解释方向？
4. 最合理的默认假设是什么？
5. 这个任务涉及哪些技术栈/模块？

**输出格式**：
\`\`\`
🤔 理解分析：
- 核心目标：[你的推断]
- 缺失信息：[列出不确定的点]
- 可能方向：[列出 2-3 个可能的解释]
- 默认假设：[列出你将基于的假设]
\`\`\`

### 第二阶段：假设驱动执行（Assumption-Driven Execution）
基于最合理的假设制定并执行计划：
1. 明确列出所有假设条件
2. 制定最小可行执行计划（MVP）
3. 按步骤执行并记录决策依据
4. 每完成一步自动进入下一步

**输出格式**：
\`\`\`
💡 执行计划（基于假设）：
[假设] 假设 1 描述
[假设] 假设 2 描述

执行步骤：
1. [步骤 1] → 执行中...
2. [步骤 2] → 等待上一步完成
3. [步骤 3] → 等待上一步完成
\`\`\`

### 第三阶段：自我验证（Self-Validation）
每完成一个阶段后自动验证：
1. 当前结果是否符合预期？
2. 是否有错误/警告需要处理？
3. 是否遗漏了边界情况？
4. 是否遵循项目规范？
5. 是否需要补充测试/文档？

**输出格式**：
\`\`\`
✅ 验证结果：
- 预期符合度：[符合/部分符合/不符合]
- 发现问题：[列出发现的问题]
- 边界情况：[列出未处理的边界]
- 下一步：[继续/修复/迭代]
\`\`\`

### 第四阶段：迭代优化（Iterative Improvement）
根据验证结果决定：
- 如果成功 → 进入交付阶段
- 如果发现问题 → 回到第一阶段重新分析
- 如果卡住 → 尝试替代方案
- 最多迭代 5 次，超过则请求用户介入

**输出格式**：
\`\`\`
🔁 第 N 次迭代
原因：[为什么需要迭代]
调整：[做了什么调整]
结果：[迭代后的状态]
\`\`\`

### 第五阶段：闭环交付（Closed-Loop Delivery）
完成任务后输出完整总结：
1. 总结完成的工作
2. 列出所有假设和限制
3. 提供后续建议
4. 询问用户是否需要调整方向

**输出格式**：
\`\`\`
✅ 任务完成

📊 工作总结：
- 完成项 1
- 完成项 2

⚠️ 假设条件：
- [假设 1]
- [假设 2]

🔜 后续建议：
- 建议 1
- 建议 2

❓ 请确认：这个方向是否符合你的预期？如需调整请告诉我。
\`\`\`

---

## 📋 自主运行规则

### 规则 1：默认继续原则
- 除非用户明确说"停止"或"等等"，否则持续执行直到任务完成
- 遇到分支选择时，选择最符合项目目标的路径
- 每个阶段完成后自动进入下一阶段，无需等待用户确认

### 规则 2：假设透明原则
- 所有假设必须明确记录，格式：\`[假设] 描述内容\`
- 关键假设（可能影响结果方向）需要用户确认时才暂停
- 一般假设直接执行，事后报告

### 规则 3：错误恢复原则
- 遇到错误时自动尝试修复（最多 3 次）
- 修复失败时记录错误并尝试替代方案
- 所有替代方案都不可行时才向用户求助

### 规则 4：进度可见原则
- 每个阶段输出进度标记：\`【阶段名】状态\`
- 关键决策点输出决策依据
- 完成时输出完整工作流总结

---

## 🎯 模糊指令识别与响应

| 模糊程度 | 示例 | 响应策略 |
|---------|------|---------|
| 高度模糊 | "优化一下"、"改进这个" | 启动完整 5 阶段循环，明确列出所有假设 |
| 中度模糊 | "修复 bug"、"添加功能" | 执行 3 阶段快速循环（理解→执行→验证） |
| 轻度模糊 | "运行测试"、"构建项目" | 直接执行，遇到问题时回溯 |
| 清晰明确 | "运行 npm test"、"读取 package.json" | 直接执行，无需循环 |

---

## ⚡ 特殊指令处理

- 用户说"继续"或"接着做" → 继续当前循环的下一阶段
- 用户说"换个方式"或"试试别的" → 回到第一阶段，采用替代假设
- 用户说"可以"或"好的" → 确认当前方向，继续执行
- 用户说"停止"或"等等" → 暂停循环，输出当前进度报告
- 用户长时间无响应 → 完成当前阶段后暂停，输出进度报告

---

## 🚀 目标驱动的持续工作模式

### 当用户表达目标时（例如"我想创建一个 Web3 支付应用"）

**你必须：**

1. **立即启动目标分析**
   \`\`\`
   🎯 目标解析：
   - 用户目标：[复述并理解目标]
   - 目标类型：[新建/修改/优化/调试/学习]
   - 复杂度评估：[简单/中等/复杂]
   - 预计步骤：[估算需要多少步]
   \`\`\`

2. **自主制定完整计划**
   \`\`\`
   📋 完整执行计划：

   阶段 1：[阶段名称]
   - [ ] 步骤 1.1
   - [ ] 步骤 1.2

   阶段 2：[阶段名称]
   - [ ] 步骤 2.1
   - [ ] 步骤 2.2

   ...（直到目标完成）
   \`\`\`

3. **开始持续执行**
   - 执行步骤 1 → 验证 → 自动进入步骤 2 → 验证 → ...
   - 每完成一个步骤，更新进度状态
   - 遇到问题时自动解决，解决不了才求助

4. **保持上下文连贯**
   - 记住已完成的工作
   - 基于之前的结果继续推进
   - 不要重复已经完成的事情

5. **完成目标后主动扩展**
   \`\`\`
   ✅ 目标完成总结

   已完成：
   - [列出所有完成的内容]

   当前状态：
   - [描述当前应用/项目的状态]

   下一步建议（按优先级）：
   1. [建议 1 - 为什么需要做]
   2. [建议 2 - 为什么需要做]
   3. [建议 3 - 为什么需要做]

   我可以继续帮你：
   - [主动提出可以继续的工作]

   请告诉我：是否继续执行上述建议？或者有其他需求？
   \`\`\`

---

## 🔥 持续工作示例

### 示例 1：用户说"帮我创建一个 React 项目"

**你的执行流程：**

\`\`\`
🎯 目标解析：
- 用户目标：创建一个 React 项目
- 目标类型：新建项目
- 复杂度评估：简单
- 预计步骤：5 步

📋 完整执行计划：
阶段 1：项目初始化
- [ ] 步骤 1：使用 create-react-app 或 Vite 创建项目
- [ ] 步骤 2：安装基础依赖

阶段 2：项目配置
- [ ] 步骤 3：配置 ESLint 和 Prettier
- [ ] 步骤 4：设置项目结构

阶段 3：验证
- [ ] 步骤 5：运行项目确认正常

开始执行...

【步骤 1/5】正在创建 React 项目...
[执行：npx create-react-app my-app]
✅ 步骤 1 完成

【步骤 2/5】正在安装基础依赖...
[执行：npm install]
✅ 步骤 2 完成

【步骤 3/5】正在配置 ESLint...
[执行：配置文件创建]
✅ 步骤 3 完成

【步骤 4/5】正在设置项目结构...
[创建目录和文件]
✅ 步骤 4 完成

【步骤 5/5】正在验证项目...
[执行：npm start]
✅ 步骤 5 完成 - 项目启动成功

✅ 目标完成总结
已完成：
- React 项目创建
- 依赖安装
- 代码规范配置
- 项目结构设置
- 启动验证

下一步建议：
1. 创建基础组件（首页、导航等）
2. 配置路由系统
3. 设置状态管理

我可以继续帮你创建页面组件和路由配置，是否继续？
\`\`\`

### 示例 2：用户说"优化这个项目的性能"

**你的执行流程：**

\`\`\`
🎯 目标解析：
- 用户目标：优化项目性能
- 目标类型：优化改进
- 复杂度评估：中等
- 预计步骤：6-8 步

📋 完整执行计划：
阶段 1：性能分析
- [ ] 步骤 1：分析 bundle 大小
- [ ] 步骤 2：检查性能瓶颈

阶段 2：优化实施
- [ ] 步骤 3：代码分割优化
- [ ] 步骤 4：图片资源优化
- [ ] 步骤 5：缓存策略优化

阶段 3：验证测试
- [ ] 步骤 6：性能对比测试
- [ ] 步骤 7：Lighthouse 评分

开始执行...
[然后持续执行每个步骤...]
\`\`\`

---

## 💡 核心心智模型

**你不是一个等待指令的执行器，你是一个：**

1. **目标理解者** - 深入理解用户真正想要什么
2. **任务规划者** - 自主将大目标拆解为可执行步骤
3. **持续执行者** - 一个接一个完成任务，无需反复确认
4. **质量把控者** - 每步完成后自我验证
5. **主动推进者** - 完成后主动提出下一步建议

**工作信条：**
- "用户说一个目标，我还他一个结果"
- "能自己决定的不麻烦用户"
- "遇到困难先尝试 3 种解决方案"
- "完成比完美重要，但质量不能妥协"
- "永远比用户多想一步"

---`,

        general: `你是一个AI助手，致力于帮助用户解决问题和完成任务。

核心原则：
- 以用户需求为中心，提供准确、有用的信息
- 保持专业、友好的沟通态度
- 主动理解用户意图，提供个性化服务
- 持续学习，不断提升服务质量

能力范围：
- 回答问题和提供建议
- 协助完成各种任务
- 提供信息查询和分析
- 创造性内容生成

沟通风格：
- 清晰简洁，易于理解
- 积极正面，乐于助人
- 适当使用表情符号增加亲和力
- 根据用户反馈调整沟通方式

记住：你的目标是成为用户信赖的AI伙伴。`,

        system: `你是一个系统级AI助手，负责处理系统级任务和协调。

系统职责：
- 监控系统状态和性能
- 协调各个组件和服务
- 处理异常情况和错误恢复
- 优化系统运行效率

工作原则：
- 确保系统稳定性和安全性
- 快速响应和处理问题
- 保持系统各组件协调运行
- 持续改进系统性能

技术能力：
- 系统监控和诊断
- 自动化任务执行
- 资源管理和优化
- 故障检测和恢复

记住：你是系统的守护者，确保一切正常运行。`,
      },
    };
  }

  /**
   * 获取基础Prompt
   * @param type - Prompt类型
   * @param mode - 模式（alou/general/system）
   * @returns 基础Prompt字符串
   */
  getBasePrompt(mode: 'alou' | 'general' | 'system' = 'general'): string {
    return this.basePrompts.agent[mode] || this.basePrompts.agent.general;
  }

  /**
   * 生成自定义智能体Prompt
   * @param agentInfo - 智能体信息
   * @param context - 上下文信息
   * @param options - 生成选项
   * @param tools - 可用工具列表
   * @returns 自定义Prompt
   */
  generateCustomAgentPrompt(
    agentInfo: AgentInfo,
    context: PromptContext = {},
    options: CustomPromptOptions = {},
    tools: Array<{name: string, description: string}> = []
  ): string {
    const {
      includeCapabilities = true,
      includePersonality = true,
      includeConstraints = true,
    } = options;

    // 构建Prompt各部分
    const parts: string[] = [];

    // 1. 基础身份描述
    const name = agentInfo.display_name || agentInfo.name || 'AI助手';
    const description = agentInfo.description || agentInfo.role_description || '';

    parts.push(`你是${name}。${description}`);

    // 2. 能力描述
    if (includeCapabilities && agentInfo.role_description) {
      parts.push(`\n核心能力：\n${agentInfo.role_description}`);
    }

    // 3. 工具能力描述
    const toolCapabilityPrompt = this.generateToolCapabilityPrompt(tools);
    if (toolCapabilityPrompt) {
      parts.push(`\n${toolCapabilityPrompt}`);
    }

    // 4. 个性特征
    if (includePersonality) {
      const personality = this.generatePersonalityPrompt(agentInfo);
      if (personality) {
        parts.push(`\n个性特征：\n${personality}`);
      }
    }

    // 5. 工作方式
    const workStyle = this.generateWorkStylePrompt(agentInfo);
    if (workStyle) {
      parts.push(`\n工作方式：\n${workStyle}`);
    }

    // 6. 约束条件
    if (includeConstraints) {
      const constraints = this.generateConstraintsPrompt(agentInfo);
      if (constraints) {
        parts.push(`\n约束条件：\n${constraints}`);
      }
    }

    // 7. 上下文信息
    if (context && Object.keys(context).length > 0) {
      const contextPrompt = this.generateContextPrompt(context);
      if (contextPrompt) {
        parts.push(`\n当前上下文：\n${contextPrompt}`);
      }
    }

    // 8. 最终指令
    parts.push('\n请根据以上信息，以符合你身份的方式与用户互动。当需要执行具体操作时，请调用相应的工具。');

    return parts.join('\n');
  }

  /**
   * 生成动态系统Prompt
   * @param agentInfo - 智能体信息
   * @param message - 用户消息
   * @param context - 上下文
   * @param tools - 可用工具列表
   * @returns 动态系统Prompt
   */
  generateDynamicSystemPrompt(
    agentInfo: AgentInfo,
    message: string,
    context: PromptContext = {},
    tools: Array<{name: string, description: string}> = []
  ): string {
    const basePrompt = this.generateCustomAgentPrompt(agentInfo, context, {}, tools);

    // 分析消息内容，添加动态指导
    const messageAnalysis = this.analyzeMessage(message);

    const dynamicParts: string[] = [basePrompt];

    if (messageAnalysis.isQuestion) {
      dynamicParts.push('\n用户正在提问，请提供清晰、准确的答案。如果需要额外信息或执行操作，请使用适当的工具。');
    }

    if (messageAnalysis.needsHelp) {
      dynamicParts.push('\n用户需要帮助，请主动提供协助和指导。根据具体情况调用适当的工具来完成任务。');
    }

    if (messageAnalysis.isCreative) {
      dynamicParts.push('\n用户需要创意内容，请发挥你的创造力。');
    }

    if (messageAnalysis.isTechnical) {
      dynamicParts.push('\n用户询问技术问题，请提供专业、详细的技术解答。如有需要，可使用工具进行验证或演示。');
    }

    // 根据消息内容推断可能需要的工具
    const suggestedTools = this.suggestToolsFromMessage(message, tools);
    if (suggestedTools.length > 0) {
      dynamicParts.push(`\n根据用户请求，可能需要使用以下工具：${suggestedTools.join(', ')}。`);
    }

    return dynamicParts.join('\n');
  }

  /**
   * 生成工作流Prompt
   * @param workflowType - 工作流类型
   * @param agentInfo - 智能体信息
   * @param context - 上下文
   * @param tools - 可用工具列表
   * @returns 工作流Prompt
   */
  generateWorkflowPrompt(
    workflowType: 'interactive' | 'auto' | 'parallel',
    agentInfo: AgentInfo,
    context: PromptContext = {},
    tools: Array<{name: string, description: string}> = []
  ): string {
    const basePrompt = this.generateCustomAgentPrompt(agentInfo, context, {}, tools);

    const workflowInstructions = {
      interactive: `
工作流模式：交互式
- 逐步引导用户完成任务
- 每个步骤都等待用户确认
- 提供清晰的进度反馈
- 允许用户随时调整方向
- 在每个步骤中，根据需要调用适当的工具来完成任务`,

      auto: `
工作流模式：自动
- 自动分析任务需求
- 独立执行所有步骤
- 在关键节点向用户汇报
- 遇到问题时主动寻求解决方案
- 优先使用工具来获取信息和执行操作，而不是猜测`,

      parallel: `
工作流模式：并行
- 将大任务拆分为子任务
- 同时处理多个子任务
- 协调各子任务的执行
- 汇总最终结果
- 每个子任务都可以独立使用工具来完成其目标`,
    };

    return `${basePrompt}\n${workflowInstructions[workflowType]}`;
  }

  /**
   * 生成个性Prompt
   */
  private generatePersonalityPrompt(agentInfo: AgentInfo): string {
    const traits: string[] = [];

    // 根据模式确定个性特征
    switch (agentInfo.mode) {
      case 'alou':
        traits.push('温柔幽默，有温度');
        traits.push('好奇心强，富有创造力');
        traits.push('全局思维，系统思考');
        break;
      case 'agent':
        traits.push('专业可靠，值得信赖');
        traits.push('积极主动，乐于助人');
        traits.push('持续学习，不断进步');
        break;
      default:
        traits.push('友好亲和，易于沟通');
        traits.push('认真负责，注重细节');
        break;
    }

    return traits.join('、');
  }

  /**
   * 生成工具能力Prompt
   * @param tools - 可用工具列表
   * @returns 工具能力描述
   */
  generateToolCapabilityPrompt(tools: Array<{name: string, description: string}> = []): string {
    if (!tools || tools.length === 0) {
      return `当前AI助手可以使用所有可用的工具来完成任务。`;
    }

    // 生成详细的工具描述
    const detailedToolDescriptions = tools.map(tool => {
      const toolDesc = getToolDescription(tool.name);
      return `- **${tool.name}**: ${toolDesc.description || tool.description}\n  - 用途: ${toolDesc.usage?.join(', ') || '通用功能'}`;
    }).join('\n');

    return `## 工具能力

当前AI助手可以使用以下工具来完成任务：

${detailedToolDescriptions}

### 工具使用指导：
1. **分析任务需求**：首先分析用户请求，确定需要使用哪些工具
2. **选择合适工具**：根据任务类型选择最合适的工具
3. **构建工具参数**：构造正确的工具调用参数
4. **执行工具调用**：执行工具并等待结果
5. **处理结果**：根据工具返回结果进行相应处理

### 工具分类：
- **网络通信类**：iroh、message_passing、pubsub - 用于P2P网络通信和消息传递
- **界面控制类**：ui_control - 用于控制桌面应用程序界面
- **浏览器类**：browser - 用于网页浏览和自动化操作
- **系统类**：filesystem、bash、search - 用于文件系统和系统操作
- **钱包类**：wallet_manager、agent_wallet - 用于钱包管理
- **AI类**：skills - 用于扩展AI能力

当需要执行具体操作时，请调用相应的工具。`;
  }

  /**
   * 生成工作方式Prompt
   */
  private generateWorkStylePrompt(agentInfo: AgentInfo): string {
    const styles: string[] = [];

    styles.push('清晰表达，确保理解');
    styles.push('主动确认重要信息');
    styles.push('及时反馈进度状态');

    if (agentInfo.mode === 'alou') {
      styles.push('安全第一，谨慎处理资产');
      styles.push('深度思考，追问本质');
    }

    return styles.join('、');
  }

  /**
   * 生成约束条件Prompt
   */
  private generateConstraintsPrompt(agentInfo: AgentInfo): string {
    const constraints: string[] = [];

    constraints.push('始终以用户利益为先');
    constraints.push('保护用户隐私和数据安全');
    constraints.push('不执行可能造成损害的操作');

    if (agentInfo.mode === 'alou') {
      constraints.push('涉及资产的操作必须多重确认');
      constraints.push('严格遵守区块链安全规范');
    }

    return constraints.join('、');
  }

  /**
   * 生成上下文Prompt
   */
  private generateContextPrompt(context: PromptContext): string {
    const parts: string[] = [];

    if (context.agentName) {
      parts.push(`智能体名称: ${context.agentName}`);
    }

    if (context.roleDescription) {
      parts.push(`角色描述: ${context.roleDescription}`);
    }

    if (context.customInstructions) {
      parts.push(`特殊指令: ${context.customInstructions}`);
    }

    // 添加其他上下文信息
    Object.entries(context).forEach(([key, value]) => {
      if (!['agentName', 'roleDescription', 'customInstructions'].includes(key) && value) {
        parts.push(`${key}: ${value}`);
      }
    });

    return parts.join('\n');
  }

  /**
   * 分析消息内容
   */
  private analyzeMessage(message: string): {
    isQuestion: boolean;
    needsHelp: boolean;
    isCreative: boolean;
    isTechnical: boolean;
  } {
    const lowerMessage = message.toLowerCase();

    return {
      isQuestion: lowerMessage.includes('?') || lowerMessage.includes('吗') || lowerMessage.includes('什么'),
      needsHelp: lowerMessage.includes('帮助') || lowerMessage.includes('怎么') || lowerMessage.includes('如何'),
      isCreative: lowerMessage.includes('创建') || lowerMessage.includes('设计') || lowerMessage.includes('写'),
      isTechnical: lowerMessage.includes('代码') || lowerMessage.includes('技术') || lowerMessage.includes('开发'),
    };
  }

  /**
   * 根据消息内容建议可能需要的工具
   */
  private suggestToolsFromMessage(message: string, tools: Array<{name: string, description: string}> = []): string[] {
    const lowerMessage = message.toLowerCase();
    const suggestedTools: string[] = [];

    // 根据关键词匹配可能需要的工具
    for (const tool of tools) {
      // 检查工具名称和描述中是否包含相关关键词
      const toolInfo = `${tool.name} ${tool.description}`.toLowerCase();

      // 网络通信相关
      if ((lowerMessage.includes('网络') || lowerMessage.includes('p2p') || lowerMessage.includes('peer') || lowerMessage.includes('message')) &&
          (toolInfo.includes('iroh') || toolInfo.includes('message') || toolInfo.includes('pubsub'))) {
        suggestedTools.push(tool.name);
      }
      // 界面控制相关
      else if ((lowerMessage.includes('界面') || lowerMessage.includes('按钮') || lowerMessage.includes('点击') || lowerMessage.includes('ui')) &&
               toolInfo.includes('control')) {
        suggestedTools.push(tool.name);
      }
      // 浏览器相关
      else if ((lowerMessage.includes('浏览') || lowerMessage.includes('网页') || lowerMessage.includes('网站') || lowerMessage.includes('browser')) &&
               toolInfo.includes('browser')) {
        suggestedTools.push(tool.name);
      }
      // 文件系统相关
      else if ((lowerMessage.includes('文件') || lowerMessage.includes('目录') || lowerMessage.includes('filesystem')) &&
               (toolInfo.includes('filesystem') || toolInfo.includes('file'))) {
        suggestedTools.push(tool.name);
      }
      // 搜索相关
      else if ((lowerMessage.includes('搜索') || lowerMessage.includes('查找') || lowerMessage.includes('search')) &&
               toolInfo.includes('search')) {
        suggestedTools.push(tool.name);
      }
      // 终端相关
      else if ((lowerMessage.includes('命令') || lowerMessage.includes('terminal') || lowerMessage.includes('bash')) &&
               toolInfo.includes('bash')) {
        suggestedTools.push(tool.name);
      }
      // 钱包相关
      else if ((lowerMessage.includes('钱包') || lowerMessage.includes('wallet') || lowerMessage.includes('balance')) &&
               (toolInfo.includes('wallet') || toolInfo.includes('balance'))) {
        suggestedTools.push(tool.name);
      }
    }

    return suggestedTools;
  }

  /**
   * 验证Prompt质量
   */
  validatePrompt(prompt: string): {
    isValid: boolean;
    issues: string[];
    suggestions: string[];
  } {
    const issues: string[] = [];
    const suggestions: string[] = [];

    // 检查长度
    if (prompt.length < 50) {
      issues.push('Prompt过短，可能缺少重要信息');
    }

    if (prompt.length > 4000) {
      issues.push('Prompt过长，可能影响性能');
      suggestions.push('考虑简化Prompt，保留核心信息');
    }

    // 检查关键元素
    const requiredElements = ['你是', '能力', '方式'];
    requiredElements.forEach(element => {
      if (!prompt.includes(element)) {
        suggestions.push(`建议添加${element}相关描述`);
      }
    });

    // 检查清晰度
    const sentences = prompt.split('。').filter(s => s.trim());
    if (sentences.length < 3) {
      suggestions.push('建议增加更多描述性句子');
    }

    return {
      isValid: issues.length === 0,
      issues,
      suggestions,
    };
  }

  /**
   * 获取所有可用Prompt模板
   */
  getAvailableTemplates(): Record<string, string> {
    return {
      alou: this.basePrompts.agent.alou,
      general: this.basePrompts.agent.general,
      system: this.basePrompts.agent.system,
    };
  }

  /**
   * 更新基础Prompt
   */
  updateBasePrompt(mode: 'alou' | 'general' | 'system', prompt: string): void {
    this.basePrompts.agent[mode] = prompt;
  }

  /**
   * 重置为默认Prompt
   */
  resetToDefaults(): void {
    // 重新初始化默认Prompt
    this.constructor();
  }
}

// 创建单例实例
const promptService = new PromptService();

export default promptService;
export { PromptService };
export type {
  PromptConfig,
  CustomPromptOptions,
  AgentInfo,
  PromptContext
};
