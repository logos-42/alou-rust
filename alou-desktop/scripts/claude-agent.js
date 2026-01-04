#!/usr/bin/env node
/**
 * Claude Agent SDK 封装脚本
 * 从 Rust 后端调用，执行 Claude Agent SDK 查询
 * 
 * 支持通过环境变量重定向到自定义后端API
 * 
 * 使用方法:
 *   node claude-agent.js <input_json_file>
 * 
 * 输入: JSON 文件，包含查询参数
 * 输出: 结果写入 <input_json_file>.result.json，错误写入 <input_json_file>.error.json
 */

import { query } from '@anthropic-ai/claude-agent-sdk';
import fs from 'fs';
import path from 'path';
import os from 'os';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

/**
 * 加载配置文件
 */
function loadConfig() {
  const configPaths = [
    // 1. AI模型配置文件（新增）
    path.join(__dirname, '..', 'config', 'ai-models-config.json'),
    // 2. 用户主目录配置文件
    path.join(os.homedir(), '.alou', 'claude-config.json'),
    // 3. 项目配置文件
    path.join(__dirname, '..', 'config', 'claude-config.json'),
    // 4. 环境配置文件
    path.join(__dirname, '..', '.env.claude'),
  ];

  let config = {
    env: {
      // 默认值 - 指向 Workers 生产环境
      // 使用 Claude Agent SDK 兼容端点
      ANTHROPIC_BASE_URL: 'https://alou-edge.yuanjieliu65.workers.dev/api/claude-agent/query',
      ANTHROPIC_API_KEY: 'alou-backend-default-token',
      ANTHROPIC_DEFAULT_SONNET_MODEL: 'claude-3-5-sonnet-20241022',
      ANTHROPIC_DEFAULT_HAIKU_MODEL: 'claude-3-haiku-20240307',
      ANTHROPIC_DEFAULT_OPUS_MODEL: 'claude-3-opus-20240229',
    },
    routing: {
      // 模型路由映射 - 扩展支持更多模型
      // 注意：后端配置的是 deepseek，所以将 Claude 模型路由到 deepseek
      'claude-3-5-sonnet-20241022': 'deepseek',
      'claude-3-haiku-20240307': 'deepseek',
      'claude-3-opus-20240229': 'deepseek',
      'deepseek-chat': 'deepseek',
      'deepseek-reasoner': 'deepseek',
      'gpt-4o': 'openai',
      'gpt-4-turbo': 'openai',
      'gpt-3.5-turbo': 'openai',
      'kimi-k2': 'kimi',
      'qwen-max': 'qwen'
    },
    // 新增：AI模型配置
    ai_models: {},
    // 新增：路由规则
    routing_rules: {
      default_model: 'deepseek-chat',
      model_aliases: {},
      task_based_routing: {}
    }
  };

  // 尝试加载配置文件
  for (const configPath of configPaths) {
    try {
      if (fs.existsSync(configPath)) {
        const fileContent = fs.readFileSync(configPath, 'utf-8');
        const fileConfig = JSON.parse(fileContent);
        
        // 合并配置
        if (fileConfig.env) {
          config.env = { ...config.env, ...fileConfig.env };
        }
        if (fileConfig.routing) {
          config.routing = { ...config.routing, ...fileConfig.routing };
        }
        if (fileConfig.models) {
          config.ai_models = fileConfig.models;
        }
        if (fileConfig.routing_rules) {
          config.routing_rules = { ...config.routing_rules, ...fileConfig.routing_rules };
        }
        
        console.error(`[Claude Agent] 加载配置文件: ${configPath}`);
      }
    } catch (error) {
      console.error(`[Claude Agent] 加载配置文件失败 ${configPath}:`, error.message);
    }
  }

  return config;
}

/**
 * 设置环境变量
 */
function setupEnvironment(config) {
  // 设置环境变量
  Object.entries(config.env).forEach(([key, value]) => {
    if (value && !process.env[key]) {
      process.env[key] = value;
      console.error(`[Claude Agent] 设置环境变量: ${key}=${value.substring(0, 10)}...`);
    }
  });

  // 验证必要的环境变量
  if (!process.env.ANTHROPIC_BASE_URL) {
    console.error('[Claude Agent] 警告: ANTHROPIC_BASE_URL 未设置，将使用默认值');
  }

  if (!process.env.ANTHROPIC_API_KEY) {
    console.error('[Claude Agent] 警告: ANTHROPIC_API_KEY 未设置，将使用输入中的apiKey');
  }
}

/**
 * 根据模型特性调整请求
 */
function adjustRequestForModel(queryOptions, modelInfo) {
  const adjusted = { ...queryOptions };
  
  if (!modelInfo) {
    return adjusted;
  }
  
  // 如果模型不支持系统提示词，将系统提示词合并到用户消息中
  if (!modelInfo.supports_system_prompt && adjusted.systemPrompt) {
    console.error(`[Claude Agent] 模型不支持系统提示词，将其合并到用户消息中`);
    adjusted.prompt = `系统指令: ${adjusted.systemPrompt}\n\n用户请求: ${adjusted.prompt}`;
    delete adjusted.systemPrompt;
  }
  
  // 如果模型不支持工具调用，移除工具
  if (!modelInfo.supports_tools && adjusted.tools && adjusted.tools.length > 0) {
    console.error(`[Claude Agent] 模型不支持工具调用，移除工具`);
    delete adjusted.tools;
  }
  
  // 应用模型特定的token限制
  if (modelInfo.max_tokens && adjusted.maxTokens > modelInfo.max_tokens) {
    console.error(`[Claude Agent] 调整max_tokens从${adjusted.maxTokens}到${modelInfo.max_tokens}`);
    adjusted.maxTokens = modelInfo.max_tokens;
  }
  
  return adjusted;
}

async function main() {
  try {
    // 加载配置
    const config = loadConfig();
    
    // 设置环境变量
    setupEnvironment(config);

    // 从命令行参数读取输入文件路径
    const inputFile = process.argv[2];
    if (!inputFile) {
      throw new Error('缺少输入文件参数。使用方法: node claude-agent.js <input_json_file>');
    }

    // 检查输入文件是否存在
    if (!fs.existsSync(inputFile)) {
      throw new Error(`输入文件不存在: ${inputFile}`);
    }

    // 读取输入 JSON
    const inputContent = fs.readFileSync(inputFile, 'utf-8');
    let input;
    try {
      input = JSON.parse(inputContent);
    } catch (parseError) {
      throw new Error(`无法解析输入 JSON: ${parseError.message}`);
    }

    const {
      apiKey,
      prompt,
      systemPrompt,
      history = [],
      agentInfo,
      tools = [],
      model = 'claude-3-5-sonnet-20241022',
      maxTokens = 4096,
      temperature = 0.7,
    } = input;

    // 验证必需参数
    if (!prompt) {
      throw new Error('缺少必需参数: prompt');
    }

    // 确定使用的API密钥
    // 优先级: 1. 环境变量 2. 输入中的apiKey 3. 配置默认值
    const finalApiKey = process.env.ANTHROPIC_API_KEY || apiKey || config.env.ANTHROPIC_API_KEY;
    
    if (!finalApiKey) {
      throw new Error('缺少API密钥。请设置ANTHROPIC_API_KEY环境变量或在输入中提供apiKey');
    }

    // 构建查询选项
    const queryOptions = {
      prompt,
      ...(systemPrompt && { systemPrompt }),
      ...(history.length > 0 && { history }),
      ...(agentInfo && { agentInfo }),
      ...(tools.length > 0 && { tools }),
      model,
      maxTokens,
      temperature,
    };

    // 记录请求信息
    console.error(`[Claude Agent] 开始执行查询`);
    console.error(`[Claude Agent] 请求模型: ${model}`);
    
    // 检查模型是否支持
    const modelInfo = config.ai_models[model];
    if (modelInfo) {
      console.error(`[Claude Agent] 模型提供商: ${modelInfo.provider}`);
      console.error(`[Claude Agent] 支持系统提示词: ${modelInfo.supports_system_prompt}`);
      console.error(`[Claude Agent] 支持工具调用: ${modelInfo.supports_tools}`);
    } else {
      console.error(`[Claude Agent] 警告: 模型 ${model} 未在配置中定义`);
    }
    
    console.error(`[Claude Agent] 目标API: ${process.env.ANTHROPIC_BASE_URL || '默认Claude API'}`);
    console.error(`[Claude Agent] 路由目标: ${config.routing[model] || '未知'}`);
    
    // 根据模型特性调整请求
    const adjustedOptions = adjustRequestForModel(queryOptions, modelInfo);
    
    // 执行查询
    const result = await query(adjustedOptions, {
      apiKey: finalApiKey,
    });

    // 构建输出结果
    const output = {
      success: true,
      response: result.response || '',
      toolCalls: result.tool_calls || [],
      usage: result.usage || {
        input_tokens: 0,
        output_tokens: 0,
      },
      metadata: {
        backend_url: process.env.ANTHROPIC_BASE_URL,
        routed_to: config.routing[model] || 'unknown',
        model_used: model,
        model_provider: modelInfo?.provider || 'unknown',
        supports_tools: modelInfo?.supports_tools || false,
        supports_system_prompt: modelInfo?.supports_system_prompt || false,
        request_adjusted: !!(modelInfo && (!modelInfo.supports_system_prompt || !modelInfo.supports_tools)),
        timestamp: new Date().toISOString()
      }
    };

    // 输出结果到文件
    const outputFile = inputFile.replace(/\.json$/, '') + '.result.json';
    fs.writeFileSync(outputFile, JSON.stringify(output, null, 2), 'utf-8');

    // 输出结果文件路径到 stdout（Rust 会读取）
    console.log(outputFile);
    console.error(`[Claude Agent] 查询成功，结果已写入: ${outputFile}`);
  } catch (error) {
    // 错误处理
    const inputFile = process.argv[2];
    const errorFile = inputFile ? inputFile.replace(/\.json$/, '') + '.error.json' : 'error.json';
    
    const errorOutput = {
      success: false,
      error: error.message || '未知错误',
      stack: error.stack || undefined,
      metadata: {
        backend_url: process.env.ANTHROPIC_BASE_URL,
        timestamp: new Date().toISOString(),
      }
    };

    try {
      fs.writeFileSync(errorFile, JSON.stringify(errorOutput, null, 2), 'utf-8');
      console.error(`[Claude Agent] 错误已写入: ${errorFile}`);
    } catch (writeError) {
      console.error(`[Claude Agent] 无法写入错误文件: ${writeError.message}`);
    }

    // 输出错误信息到 stderr
    console.error(`[Claude Agent] 错误: ${error.message}`);
    if (error.stack) {
      console.error(`[Claude Agent] 堆栈: ${error.stack}`);
    }

    // 退出并返回错误代码
    process.exit(1);
  }
}

// 执行主函数
main().catch((error) => {
  console.error(`[Claude Agent] 未捕获的错误: ${error.message}`);
  process.exit(1);
});
