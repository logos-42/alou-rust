alou中参数传递和多轮调用的核心逻辑实现代码：

  1. 参数传递实现

  环境变量处理（tool-registry.ts）

    1 private processEnvironmentVariables(mcpServers: any, installDir: string): any {
    2   const processedServers = JSON.parse(JSON.stringify(mcpServers)); // 深拷贝
    3 
    4   for (const [serverName, serverConfig] of Object.entries(processedServers)) {
    5     if (config.args && Array.isArray(config.args)) {
    6       const newArgs: string[] = [];
    7 
    8       for (const arg of config.args) {
    9         // 处理 ALOU_INSTALL_DIR
   10         if (typeof arg === 'string' && arg.includes('${ALOU_INSTALL_DIR}')) {
   11           newArgs.push(this.resolveFetchPath(arg, installDir, serverName));
   12         }
   13         // 处理 OS_FILESYSTEM_PATHS_ARRAY - 展开为多个参数
   14         else if (typeof arg === 'string' && arg.includes('${OS_FILESYSTEM_PATHS_ARRAY}')) {
   15           const paths = this.getOSFilesystemPaths();
   16           const pathArray = paths.split(',').filter(path => path.trim() !== '');
   17           newArgs.push(...pathArray);
   18         }
   19         else {
   20           newArgs.push(arg);
   21         }
   22       }
   23 
   24       config.args = newArgs;
   25     }
   26   }
   27 
   28   return processedServers;
   29 }

  MCP服务器连接（mcp-client.ts）

    1 export async function connectToMcpServer(
    2   mcpServerName: string,
    3   mcpServerConfig: MCPServerConfig,
    4   debugMode: boolean,
    5   workspaceContext: WorkspaceContext,
    6 ): Promise<Client> {
    7   const mcpClient = new Client({
    8     name: 'qwen-code-mcp-client',
    9     version: '0.0.1',
   10   });
   11 
   12   // 创建传输层
   13   const transport = await createTransport(
   14     mcpServerName,
   15     mcpServerConfig,
   16     debugMode,
   17   );
   18 
   19   // 连接客户端
   20   await mcpClient.connect(transport, {
   21     timeout: mcpServerConfig.timeout ?? MCP_DEFAULT_TIMEOUT_MSEC,
   22   });
   23 
   24   return mcpClient;
   25 }

  2. 多轮工具调用实现

  对话循环核心逻辑（deepseek-client.ts）

     1 async chatWithTools(
     2   prompt: string,
     3   model?: string,
     4   maxIterations: number = 10
     5 ): Promise<string> {
     6   let currentPrompt = prompt;
     7   let iteration = 0;
     8   let conversationHistory: any[] = [];
     9 
    10   // 构建初始消息
    11   conversationHistory.push({
    12     role: 'user',
    13     content: currentPrompt
    14   });
    15 
    16   while (iteration < maxIterations) {
    17     iteration++;
    18 
    19     // 获取工具定义
    20     const toolSchemas = this.toolRegistry ? this.toolRegistry.getFunctionDeclarations() : undefined;
    21 
    22     try {
    23       // 生成响应
    24       const response = await this.generateContent(currentPrompt, toolSchemas, model, conversationHistory);
    25 
    26       // 添加AI响应到对话历史
    27       conversationHistory.push({
    28         role: 'assistant',
    29         content: response.content
    30       });
    31
    32       // 如果有工具调用，执行它们
    33       if (response.toolCalls && response.toolCalls.length > 0 && this.toolRegistry) {
    34         const toolResults = await this.executeToolCalls(response.toolCalls);
    35
    36         // 检查是否有工具执行失败
    37         const failedTools = toolResults.filter(result => !result.success);
    38         const successfulTools = toolResults.filter(result => result.success);
    39
    40         if (failedTools.length > 0) {
    41           // 构建重试提示
    42           const retryPrompt = `刚才的工具调用中有一些失败了。请分析失败的原因并尝试其他方法来解决用户的问题。
    43 失败的工具：
    44 ${failedTools.map(tool => `- ${tool.name}: ${tool.error}`).join('\n')}
    45
    46 成功的工具：
    47 ${successfulTools.map(tool => `- ${tool.name}: ${tool.content}`).join('\n')}
    48
    49 请重新思考并尝试其他方法。如果所有任务都已完成，请明确说明。`;
    50
    51           // 添加工具结果到对话历史
    52           conversationHistory.push({
    53             role: 'user',
    54             content: retryPrompt
    55           });
    56
    57           currentPrompt = retryPrompt;
    58           continue; // 继续下一轮对话
    59         } else {
    60           // 检查任务是否完成
    61           const completionIndicators = [
    62             '任务完成', '已完成', '完成', '结束', '完成所有',
    63             '所有任务', '任务结束', '工作完成', '执行完毕'
    64           ];
    65
    66           const responseText = response.content.toLowerCase();
    67           const isTaskComplete = completionIndicators.some(indicator =>
    68             responseText.includes(indicator.toLowerCase())
    69           );
    70
    71           if (isTaskComplete) {
    72             return response.content;
    73           } else {
    74             // 继续执行
    75             const continuePrompt = `请继续执行剩余的任务。如果所有任务都已完成，请明确说明"任务完成"。`;
    76
    77             conversationHistory.push({
    78               role: 'user',
    79               content: continuePrompt
    80             });
    81
    82             currentPrompt = continuePrompt;
    83             continue;
    84           }
    85         }
    86       } else {
    87         // 没有工具调用，直接返回响应
    88         return response.content;
    89       }
    90     } catch (error) {
    91       // 错误处理和重试逻辑
    92       let retryPrompt = `刚才出现了错误：${error}。请重新尝试解决用户的问题，可以考虑使用不同的方法或工具。`;
    93
    94       currentPrompt = retryPrompt;
    95       conversationHistory.push({
    96         role: 'user',
    97         content: currentPrompt
    98       });
    99     }
   100   }
   101
   102   return `经过 ${maxIterations} 轮尝试，仍然无法完全解决您的问题。`;
   103 }

  工具执行实现（deepseek-client.ts）

    1 async executeToolCalls(toolCalls: ToolCall[]): Promise<ToolExecutionResult[]> {
    2   if (!this.toolRegistry) {
    3     throw new Error('Tool registry not set');
    4   }
    5 
    6   const results: ToolExecutionResult[] = [];
    7 
    8   for (const toolCall of toolCalls) {
    9     let lastError: Error | null = null;
   10     let success = false;
   11     let resultContent = '';
   12 
   13     // 重试机制
   14     for (let attempt = 1; attempt <= this.maxRetries; attempt++) {
   15       try {
   16         const tool = this.toolRegistry.getTool(toolCall.function.name);
   17         if (!tool) {
   18           throw new Error(`Tool ${toolCall.function.name} not found`);
   19         }
   20 
   21         const args = JSON.parse(toolCall.function.arguments);
   22 
   23         // 创建超时控制器
   24         const controller = new AbortController();
   25         const timeoutId = setTimeout(() => {
   26           controller.abort();
   27         }, this.timeoutMs);
   28 
   29         try {
   30           // 使用buildAndExecute方法执行工具
   31           const result = await tool.buildAndExecute(args, controller.signal);
   32           clearTimeout(timeoutId);
   33
   34           // 提取结果内容
   35           if (result.llmContent) {
   36             if (Array.isArray(result.llmContent)) {
   37               for (const part of result.llmContent) {
   38                 if (part.text) {
   39                   resultContent += part.text + '\n';
   40                 }
   41               }
   42             } else if (typeof result.llmContent === 'string') {
   43               resultContent = result.llmContent;
   44             } else if (result.llmContent.text) {
   45               resultContent = result.llmContent.text;
   46             }
   47           }
   48
   49           if (!resultContent && result.returnDisplay) {
   50             resultContent = result.returnDisplay;
   51           }
   52
   53           success = true;
   54           break;
   55         } catch (executionError) {
   56           clearTimeout(timeoutId);
   57           throw executionError;
   58         }
   59       } catch (error) {
   60         lastError = error as Error;
   61
   62         if (attempt === this.maxRetries) {
   63           // 最后一次尝试失败
   64           resultContent = `工具执行失败: ${error}`;
   65         } else {
   66           // 等待一段时间后重试
   67           await new Promise(resolve => setTimeout(resolve, 1000 * attempt));
   68         }
   69       }
   70     }
   71
   72     results.push({
   73       tool_call_id: toolCall.id,
   74       name: toolCall.function.name,
   75       content: resultContent || (success ? '工具执行成功' : '工具执行失败'),
   76       success: success,
   77       error: success ? undefined : lastError?.message
   78     });
   79   }
   80
   81   return results;
   82 }

  MCP工具调用实现（mcp-tool.ts）

    1 class DiscoveredMCPToolInvocation extends BaseToolInvocation<ToolParams, ToolResult> {
    2   async execute(): Promise<ToolResult> {
    3     const functionCalls: FunctionCall[] = [
    4       {
    5         name: this.serverToolName,
    6         args: this.params,
    7       },
    8     ];
    9 
   10     // 调用MCP工具
   11     const rawResponseParts = await this.mcpTool.callTool(functionCalls);
   12     const transformedParts = transformMcpContentToParts(rawResponseParts);
   13 
   14     return {
   15       content: getStringifiedResultForDisplay(rawResponseParts),
   16       llmContent: transformedParts,
   17       returnDisplay: getStringifiedResultForDisplay(rawResponseParts),
   18     };
   19   }
   20 }

  这些代码展示了alou中参数传递和多轮工具调用的核心实现：

   1. 参数传递: 通过环境变量替换和MCP配置传递参数给服务器
   2. 多轮调用: 通过对话循环机制实现多轮工具调用
   3. 错误处理: 通过重试机制和错误分类处理各种异常情况
   4. 任务判断: 通过关键词检测判断任务是否完成