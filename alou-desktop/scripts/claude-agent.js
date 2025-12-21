#!/usr/bin/env node
/**
 * Claude Agent SDK 封装脚本
 * 从 Rust 后端调用，执行 Claude Agent SDK 查询
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
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

async function main() {
  try {
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
    if (!apiKey) {
      throw new Error('缺少必需参数: apiKey');
    }
    if (!prompt) {
      throw new Error('缺少必需参数: prompt');
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

    // 执行查询
    console.error(`[Claude Agent] 开始执行查询，模型: ${model}`);
    const result = await query(queryOptions, {
      apiKey,
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

