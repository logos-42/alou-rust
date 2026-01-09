/**
 * SDK Agent Tools - LSP和Spec工具配置
 * 为智能体提供代码分析和规格管理工具
 */

import lspService from './lspService'
import specService from './specService'

/**
 * LSP工具配置
 */
export const LSP_TOOLS = {
  CODE_COMPLETION: {
    name: "code_completion",
    description: "获取代码补全建议",
    category: "development",
    parameters: {
      type: "object",
      properties: {
        code: {
          type: "string",
          description: "代码内容"
        },
        language: {
          type: "string",
          description: "编程语言",
          enum: ["javascript", "typescript", "python", "rust", "go", "java", "csharp", "cpp", "html", "css", "json", "markdown"]
        },
        position: {
          type: "object",
          description: "光标位置",
          properties: {
            line: {
              type: "number",
              description: "行号（从0开始）"
            },
            character: {
              type: "number",
              description: "字符位置（从0开始）"
            }
          },
          required: ["line", "character"]
        }
      },
      required: ["code", "language", "position"]
    }
  },

  CODE_DIAGNOSTICS: {
    name: "code_diagnostics",
    description: "诊断代码问题",
    category: "development",
    parameters: {
      type: "object",
      properties: {
        code: {
          type: "string",
          description: "代码内容"
        },
        language: {
          type: "string",
          description: "编程语言",
          enum: ["javascript", "typescript", "python", "rust", "go", "java", "csharp", "cpp", "html", "css", "json", "markdown"]
        }
      },
      required: ["code", "language"]
    }
  },

  CODE_FORMAT: {
    name: "code_format",
    description: "格式化代码",
    category: "development",
    parameters: {
      type: "object",
      properties: {
        code: {
          type: "string",
          description: "代码内容"
        },
        language: {
          type: "string",
          description: "编程语言",
          enum: ["javascript", "typescript", "python", "rust", "go", "java", "csharp", "cpp", "html", "css", "json", "markdown"]
        }
      },
      required: ["code", "language"]
    }
  },

  CODE_HOVER: {
    name: "code_hover",
    description: "获取代码悬停信息",
    category: "development",
    parameters: {
      type: "object",
      properties: {
        code: {
          type: "string",
          description: "代码内容"
        },
        language: {
          type: "string",
          description: "编程语言",
          enum: ["javascript", "typescript", "python", "rust", "go", "java", "csharp", "cpp", "html", "css", "json", "markdown"]
        },
        position: {
          type: "object",
          description: "光标位置",
          properties: {
            line: {
              type: "number",
              description: "行号（从0开始）"
            },
            character: {
              type: "number",
              description: "字符位置（从0开始）"
            }
          },
          required: ["line", "character"]
        }
      },
      required: ["code", "language", "position"]
    }
  }
}

/**
 * Spec工具配置
 */
export const SPEC_TOOLS = {
  CREATE_SPEC: {
    name: "create_spec",
    description: "创建规格文档",
    category: "documentation",
    parameters: {
      type: "object",
      properties: {
        specType: {
          type: "string",
          description: "规格类型",
          enum: ["product", "technical", "design", "api", "user_story", "tasks", "structure"]
        },
        specData: {
          type: "object",
          description: "规格数据"
        },
        templateId: {
          type: "string",
          description: "模板ID（可选）"
        }
      },
      required: ["specType", "specData"]
    }
  },

  GET_SPEC: {
    name: "get_spec",
    description: "获取规格文档",
    category: "documentation",
    parameters: {
      type: "object",
      properties: {
        specType: {
          type: "string",
          description: "规格类型",
          enum: ["product", "technical", "design", "api", "user_story", "tasks", "structure"]
        },
        specId: {
          type: "string",
          description: "规格ID"
        }
      },
      required: ["specType", "specId"]
    }
  },

  LIST_SPECS: {
    name: "list_specs",
    description: "列出所有规格文档",
    category: "documentation",
    parameters: {
      type: "object",
      properties: {
        specType: {
          type: "string",
          description: "规格类型",
          enum: ["product", "technical", "design", "api", "user_story", "tasks", "structure"]
        }
      },
      required: ["specType"]
    }
  },

  VALIDATE_SPEC: {
    name: "validate_spec",
    description: "验证规格文档",
    category: "documentation",
    parameters: {
      type: "object",
      properties: {
        specType: {
          type: "string",
          description: "规格类型",
          enum: ["product", "technical", "design", "api", "user_story", "tasks", "structure"]
        },
        specData: {
          type: "object",
          description: "规格数据"
        }
      },
      required: ["specType", "specData"]
    }
  }
}

/**
 * 所有SDK工具
 */
export const SDK_AGENT_TOOLS = {
  ...LSP_TOOLS,
  ...SPEC_TOOLS
}

/**
 * SDK工具类别
 */
export const SDK_TOOL_CATEGORIES = {
  DEVELOPMENT: [
    "code_completion",
    "code_diagnostics", 
    "code_format",
    "code_hover"
  ],
  DOCUMENTATION: [
    "create_spec",
    "get_spec",
    "list_specs",
    "validate_spec"
  ]
}

/**
 * 执行LSP工具
 */
export async function executeLspTool(toolName, parameters) {
  try {
    switch (toolName) {
      case 'code_completion':
        return await lspService.getCompletions({
          code: parameters.code,
          language: parameters.language,
          position: parameters.position
        })
      
      case 'code_diagnostics':
        return await lspService.getDiagnostics({
          code: parameters.code,
          language: parameters.language
        })
      
      case 'code_format':
        return await lspService.formatCode({
          code: parameters.code,
          language: parameters.language
        })
      
      case 'code_hover':
        return await lspService.getHoverInfo({
          code: parameters.code,
          language: parameters.language,
          position: parameters.position
        })
      
      default:
        throw new Error(`未知的LSP工具: ${toolName}`)
    }
  } catch (error) {
    console.error(`[executeLspTool] ${toolName} 执行失败:`, error)
    throw error
  }
}

/**
 * 执行Spec工具
 */
export async function executeSpecTool(toolName, parameters) {
  try {
    switch (toolName) {
      case 'create_spec':
        return await specService.createSpec({
          specType: parameters.specType,
          specData: parameters.specData,
          templateId: parameters.templateId
        })
      
      case 'get_spec':
        return await specService.getSpec(parameters.specId)
      
      case 'list_specs':
        return await specService.listSpecs({ specType: parameters.specType })
      
      case 'validate_spec':
        return await specService.validateSpec({
          specId: parameters.specId,
          specType: parameters.specType
        })
      
      default:
        throw new Error(`未知的Spec工具: ${toolName}`)
    }
  } catch (error) {
    console.error(`[executeSpecTool] ${toolName} 执行失败:`, error)
    throw error
  }
}

/**
 * 执行SDK工具
 */
export async function executeSdkTool(toolName, parameters) {
  // 检查是LSP工具还是Spec工具
  if (Object.keys(LSP_TOOLS).some(key => LSP_TOOLS[key].name === toolName)) {
    return await executeLspTool(toolName, parameters)
  } else if (Object.keys(SPEC_TOOLS).some(key => SPEC_TOOLS[key].name === toolName)) {
    return await executeSpecTool(toolName, parameters)
  } else {
    throw new Error(`未知的SDK工具: ${toolName}`)
  }
}

/**
 * 获取所有可用的SDK工具
 */
export function getAllSdkTools() {
  return Object.values(SDK_AGENT_TOOLS)
}

/**
 * 获取指定类别的SDK工具
 */
export function getSdkToolsByCategory(category) {
  const tools = SDK_TOOL_CATEGORIES[category.toUpperCase()] || []
  return tools.map(toolName => SDK_AGENT_TOOLS[toolName.toUpperCase()]).filter(Boolean)
}

export default {
  SDK_AGENT_TOOLS,
  SDK_TOOL_CATEGORIES,
  executeSdkTool,
  getAllSdkTools,
  getSdkToolsByCategory
}

