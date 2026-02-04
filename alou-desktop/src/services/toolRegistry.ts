/**
 * 工具注册表 - 确保所有工具都被正确注册和描述
 */

import { TOOL_DESCRIPTIONS } from './toolDescriptions';

// 定义所有可用的工具
export const ALL_TOOLS = {
  // 网络通信类
  'iroh': TOOL_DESCRIPTIONS.iroh,
  'message_passing': TOOL_DESCRIPTIONS.message_passing,
  'pubsub': TOOL_DESCRIPTIONS.pubsub,
  
  // 界面控制类
  'ui_control': TOOL_DESCRIPTIONS.ui_control,
  
  // 浏览器类
  'browser': TOOL_DESCRIPTIONS.browser,
  
  // 系统类
  'filesystem': TOOL_DESCRIPTIONS.filesystem,
  'bash': TOOL_DESCRIPTIONS.bash,
  'search': TOOL_DESCRIPTIONS.search,
  
  // 钱包类
  'wallet_manager': TOOL_DESCRIPTIONS.wallet_manager,
  
  // AI类
  'skills': TOOL_DESCRIPTIONS.skills,
  
  // 其他已知工具
  'plan': {
    name: "任务规划工具",
    description: "用于规划和管理任务的工具",
    categories: ["planning", "task"],
    usage: ["创建任务计划", "管理任务列表", "跟踪任务进度"]
  },
  
  'agent_collaboration': {
    name: "智能体协作工具",
    description: "用于多个智能体之间协作的工具",
    categories: ["collaboration", "multi-agent"],
    usage: ["协调多个智能体", "管理协作会话", "同步状态"]
  },
  
  'diap_registry': {
    name: "DIAP注册工具",
    description: "用于管理DIAP身份和注册的工具",
    categories: ["identity", "registry", "diap"],
    usage: ["注册DIAP身份", "管理身份信息", "验证身份"]
  },
  
  'email': {
    name: "邮件工具",
    description: "用于发送和接收邮件的工具",
    categories: ["communication", "email"],
    usage: ["发送邮件", "接收邮件", "管理邮件账户"]
  },
  
  'encryption': {
    name: "加密工具",
    description: "用于数据加密和解密的工具",
    categories: ["security", "encryption"],
    usage: ["加密数据", "解密数据", "管理密钥"]
  },
  
  'form_generator': {
    name: "表单生成工具",
    description: "用于生成和管理表单的工具",
    categories: ["ui", "forms", "generation"],
    usage: ["创建表单", "验证表单数据", "处理表单提交"]
  },
  
  'github': {
    name: "GitHub工具",
    description: "用于与GitHub交互的工具",
    categories: ["development", "github", "version-control"],
    usage: ["克隆仓库", "推送代码", "管理PR"]
  },
  
  'image_generation': {
    name: "图像生成工具",
    description: "用于生成和处理图像的工具",
    categories: ["media", "ai", "generation"],
    usage: ["生成图像", "编辑图像", "转换格式"]
  },
  
  'knowledge_base': {
    name: "知识库工具",
    description: "用于管理和查询知识库的工具",
    categories: ["knowledge", "search", "database"],
    usage: ["查询知识库", "更新知识", "管理文档"]
  },
  
  'location': {
    name: "位置工具",
    description: "用于获取和处理位置信息的工具",
    categories: ["location", "geolocation"],
    usage: ["获取当前位置", "查询地理位置", "计算距离"]
  },
  
  'media_player': {
    name: "媒体播放器工具",
    description: "用于播放和控制媒体的工具",
    categories: ["media", "player"],
    usage: ["播放音频", "播放视频", "控制播放"]
  },
  
  'mock_payment': {
    name: "模拟支付工具",
    description: "用于模拟支付流程的工具",
    categories: ["payment", "simulation"],
    usage: ["模拟支付", "测试支付流程", "验证支付逻辑"]
  },
  
  'qr_code': {
    name: "二维码工具",
    description: "用于生成和解析二维码的工具",
    categories: ["utilities", "qr", "encoding"],
    usage: ["生成二维码", "解析二维码", "验证内容"]
  },
  
  'screen_capture': {
    name: "屏幕捕捉工具",
    description: "用于捕捉和处理屏幕内容的工具",
    categories: ["utilities", "capture", "recording"],
    usage: ["截取屏幕", "录制视频", "分享截图"]
  },
  
  'social_media': {
    name: "社交媒体工具",
    description: "用于与社交媒体平台交互的工具",
    categories: ["social", "communication"],
    usage: ["发布内容", "获取动态", "管理账户"]
  },
  
  'system_info': {
    name: "系统信息工具",
    description: "用于获取系统信息的工具",
    categories: ["system", "info", "monitoring"],
    usage: ["获取硬件信息", "监控系统状态", "分析性能"]
  },
  
  'task_scheduler': {
    name: "任务调度工具",
    description: "用于调度和管理任务的工具",
    categories: ["scheduling", "automation"],
    usage: ["安排任务", "管理定时任务", "执行计划任务"]
  },
  
  'text_processing': {
    name: "文本处理工具",
    description: "用于处理和分析文本的工具",
    categories: ["text", "processing", "analysis"],
    usage: ["分析文本", "提取信息", "格式化文本"]
  },
  
  'transaction_builder': {
    name: "交易构建工具",
    description: "用于构建和管理区块链交易的工具",
    categories: ["blockchain", "transactions", "crypto"],
    usage: ["构建交易", "签名交易", "广播交易"]
  },
  
  'video_call': {
    name: "视频通话工具",
    description: "用于管理视频通话的工具",
    categories: ["communication", "video", "calls"],
    usage: ["发起通话", "管理通话", "共享屏幕"]
  },
  
  'voice_recognition': {
    name: "语音识别工具",
    description: "用于语音识别和处理的工具",
    categories: ["ai", "voice", "recognition"],
    usage: ["转录语音", "识别说话人", "处理音频"]
  },
  
  'web_search': {
    name: "网络搜索工具",
    description: "用于在网络上搜索信息的工具",
    categories: ["search", "web", "information"],
    usage: ["搜索网页", "获取信息", "分析结果"]
  },
  
  'workflow_automation': {
    name: "工作流自动化工具",
    description: "用于自动化工作流程的工具",
    categories: ["automation", "workflow", "process"],
    usage: ["创建工作流", "执行自动化", "管理流程"]
  }
};

/**
 * 获取所有工具名称
 */
export function getAllToolNames(): string[] {
  return Object.keys(ALL_TOOLS);
}

/**
 * 获取工具信息
 */
export function getToolInfo(toolName: string) {
  return ALL_TOOLS[toolName as keyof typeof ALL_TOOLS];
}

/**
 * 检查工具是否存在
 */
export function hasTool(toolName: string): boolean {
  return toolName in ALL_TOOLS;
}

/**
 * 根据类别获取工具
 */
export function getToolsByCategory(category: string): string[] {
  return Object.entries(ALL_TOOLS)
    .filter(([_, info]) => info.categories.includes(category))
    .map(([name, _]) => name);
}

/**
 * 获取所有类别
 */
export function getAllCategories(): string[] {
  const categories = new Set<string>();
  Object.values(ALL_TOOLS).forEach(info => {
    info.categories.forEach(cat => categories.add(cat));
  });
  return Array.from(categories);
}