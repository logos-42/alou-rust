#!/usr/bin/env node
/**
 * Spec Agent Node.js Script
 * 处理规格文档管理相关请求
 */

import fs from 'fs/promises'
import path from 'path'
import crypto from 'crypto'
import os from 'os'
import { fileURLToPath } from 'url'

const __filename = fileURLToPath(import.meta.url)
const __dirname = path.dirname(__filename)

/**
 * Spec存储目录
 */
const SPEC_DIR = path.join(os.homedir(), '.alou', 'specs')

/**
 * 初始化Spec目录
 */
async function initSpecDir() {
  try {
    await fs.mkdir(SPEC_DIR, { recursive: true })
  } catch (error) {
    console.error('初始化Spec目录失败:', error)
  }
}

/**
 * 处理Spec请求
 */
async function handleSpecRequest(request) {
  const { spec_type, operation, spec_data, spec_id, template_id, output_format } = request

  await initSpecDir()

  switch (operation) {
    case 'create':
      return await createSpec(spec_type, spec_data, template_id)
    case 'update':
      return await updateSpec(spec_id, spec_data)
    case 'get':
      return await getSpec(spec_id)
    case 'delete':
      return await deleteSpec(spec_id)
    case 'list':
      return await listSpecs(spec_type)
    case 'validate':
      return await validateSpec(spec_id, spec_data)
    case 'generate_from_template':
      return await generateFromTemplate(template_id, spec_data)
    case 'export':
      return await exportSpec(spec_id, output_format)
    case 'import':
      return await importSpec(spec_data, spec_type)
    default:
      throw new Error(`不支持的 Spec 操作: ${operation}`)
  }
}

/**
 * 生成唯一ID
 */
function generateId() {
  return crypto.randomBytes(16).toString('hex')
}

/**
 * 获取规格文件路径
 */
function getSpecFilePath(specId) {
  return path.join(SPEC_DIR, `${specId}.json`)
}

/**
 * 创建规格文档
 */
async function createSpec(specType, specData, templateId) {
  const specId = generateId()
  const now = new Date().toISOString()

  let content = specData?.content || ''

  // 如果使用模板，从模板生成内容
  if (templateId) {
    content = await loadTemplateContent(templateId)
    if (specData) {
      // 替换模板变量
      Object.keys(specData).forEach(key => {
        const placeholder = `{{${key}}}`
        content = content.replace(new RegExp(placeholder, 'g'), specData[key])
      })
    }
  }

  const spec = {
    id: specId,
    spec_type: specType,
    title: specData?.title || '未命名规格',
    content: content,
    metadata: {
      version: specData?.version || '1.0.0',
      status: specData?.status || 'draft',
      author: specData?.author,
      tags: specData?.tags || [],
      template_id: templateId,
      related_specs: specData?.related_specs || [],
    },
    created_at: now,
    updated_at: now,
  }

  // 保存到文件
  const filePath = getSpecFilePath(specId)
  await fs.writeFile(filePath, JSON.stringify(spec, null, 2), 'utf-8')

  return spec
}

/**
 * 更新规格文档
 */
async function updateSpec(specId, specData) {
  const filePath = getSpecFilePath(specId)
  
  // 读取现有规格
  const existingContent = await fs.readFile(filePath, 'utf-8')
  const existingSpec = JSON.parse(existingContent)

  // 合并更新
  const updatedSpec = {
    ...existingSpec,
    title: specData?.title || existingSpec.title,
    content: specData?.content || existingSpec.content,
    metadata: {
      ...existingSpec.metadata,
      ...specData?.metadata,
    },
    updated_at: new Date().toISOString(),
  }

  // 保存更新后的规格
  await fs.writeFile(filePath, JSON.stringify(updatedSpec, null, 2), 'utf-8')

  return updatedSpec
}

/**
 * 获取规格文档
 */
async function getSpec(specId) {
  const filePath = getSpecFilePath(specId)
  const content = await fs.readFile(filePath, 'utf-8')
  return JSON.parse(content)
}

/**
 * 删除规格文档
 */
async function deleteSpec(specId) {
  const filePath = getSpecFilePath(specId)
  await fs.unlink(filePath)
  return true
}

/**
 * 列出规格文档
 */
async function listSpecs(specType) {
  const files = await fs.readdir(SPEC_DIR)
  const specs = []

  for (const file of files) {
    if (!file.endsWith('.json')) continue

    try {
      const filePath = path.join(SPEC_DIR, file)
      const content = await fs.readFile(filePath, 'utf-8')
      const spec = JSON.parse(content)

      // 过滤类型
      if (!specType || spec.spec_type === specType) {
        specs.push(spec)
      }
    } catch (error) {
      console.error(`读取规格文件失败: ${file}`, error)
    }
  }

  return specs
}

/**
 * 验证规格文档
 */
async function validateSpec(specId, specData) {
  const errors = []
  const warnings = []
  const suggestions = []

  try {
    let spec
    if (specId) {
      spec = await getSpec(specId)
    } else if (specData) {
      spec = specData
    } else {
      throw new Error('必须提供 specId 或 specData')
    }

    // 验证必需字段
    if (!spec.title || spec.title.trim() === '') {
      errors.push('规格标题不能为空')
    }

    if (!spec.content || spec.content.trim() === '') {
      errors.push('规格内容不能为空')
    }

    if (!spec.metadata.version) {
      errors.push('规格版本不能为空')
    }

    // 验证版本格式
    const versionRegex = /^\d+\.\d+\.\d+$/
    if (spec.metadata.version && !versionRegex.test(spec.metadata.version)) {
      warnings.push('版本号格式建议使用语义化版本格式 (如: 1.0.0)')
    }

    // 验证内容长度
    if (spec.content && spec.content.length < 100) {
      warnings.push('规格内容较为简短，可能需要补充更多细节')
    }

    // 验证标签
    if (spec.metadata.tags && spec.metadata.tags.length === 0) {
      suggestions.push('建议添加标签以便于分类和搜索')
    }

    // 根据规格类型进行特定验证
    switch (spec.spec_type) {
      case 'product':
        await validateProductSpec(spec, errors, warnings, suggestions)
        break
      case 'technical':
        await validateTechnicalSpec(spec, errors, warnings, suggestions)
        break
      case 'design':
        await validateDesignSpec(spec, errors, warnings, suggestions)
        break
      case 'api':
        await validateApiSpec(spec, errors, warnings, suggestions)
        break
    }

  } catch (error) {
    errors.push(`验证失败: ${error.message}`)
  }

  return {
    is_valid: errors.length === 0,
    errors,
    warnings,
    suggestions,
  }
}

/**
 * 验证产品规格
 */
async function validateProductSpec(spec, errors, warnings, suggestions) {
  const content = spec.content.toLowerCase()

  if (!content.includes('目标用户') && !content.includes('target user')) {
    warnings.push('产品规格建议包含目标用户描述')
  }

  if (!content.includes('核心功能') && !content.includes('core features')) {
    warnings.push('产品规格建议包含核心功能列表')
  }

  if (!content.includes('用户故事') && !content.includes('user story')) {
    suggestions.push('建议添加用户故事以更好描述需求')
  }
}

/**
 * 验证技术规格
 */
async function validateTechnicalSpec(spec, errors, warnings, suggestions) {
  const content = spec.content.toLowerCase()

  if (!content.includes('架构') && !content.includes('architecture')) {
    warnings.push('技术规格建议包含系统架构描述')
  }

  if (!content.includes('技术栈') && !content.includes('tech stack')) {
    warnings.push('技术规格建议包含技术栈信息')
  }

  if (!content.includes('数据库') && !content.includes('database')) {
    suggestions.push('建议包含数据库设计说明')
  }
}

/**
 * 验证设计规格
 */
async function validateDesignSpec(spec, errors, warnings, suggestions) {
  const content = spec.content.toLowerCase()

  if (!content.includes('ui') && !content.includes('ux')) {
    warnings.push('设计规格建议包含UI/UX说明')
  }

  if (!content.includes('交互') && !content.includes('interaction')) {
    suggestions.push('建议包含交互流程说明')
  }
}

/**
 * 验证API规格
 */
async function validateApiSpec(spec, errors, warnings, suggestions) {
  const content = spec.content.toLowerCase()

  if (!content.includes('endpoint') && !content.includes('接口')) {
    errors.push('API规格必须包含接口说明')
  }

  if (!content.includes('请求') && !content.includes('request')) {
    warnings.push('API规格建议包含请求参数说明')
  }

  if (!content.includes('响应') && !content.includes('response')) {
    warnings.push('API规格建议包含响应格式说明')
  }
}

/**
 * 从模板生成规格
 */
async function generateFromTemplate(templateId, specData) {
  const content = await loadTemplateContent(templateId)

  // 替换模板变量
  let filledContent = content
  if (specData) {
    Object.keys(specData).forEach(key => {
      const placeholder = `{{${key}}}`
      filledContent = filledContent.replace(new RegExp(placeholder, 'g'), specData[key])
    })
  }

  return createSpec(templateId, { content: filledContent, ...specData }, templateId)
}

/**
 * 导出规格
 */
async function exportSpec(specId, outputFormat) {
  const spec = await getSpec(specId)
  let outputContent = ''
  let fileExtension = ''

  switch (outputFormat) {
    case 'md':
    case 'markdown':
      outputContent = exportAsMarkdown(spec)
      fileExtension = 'md'
      break
    case 'json':
      outputContent = JSON.stringify(spec, null, 2)
      fileExtension = 'json'
      break
    case 'html':
      outputContent = exportAsHtml(spec)
      fileExtension = 'html'
      break
    default:
      throw new Error(`不支持的导出格式: ${outputFormat}`)
  }

  // 写入导出文件
  const exportDir = path.join(os.homedir(), '.alou', 'exports')
  await fs.mkdir(exportDir, { recursive: true })

  const exportFileName = `${specId}_${Date.now()}.${fileExtension}`
  const exportFilePath = path.join(exportDir, exportFileName)

  await fs.writeFile(exportFilePath, outputContent, 'utf-8')

  return {
    output_path: exportFilePath,
    format: outputFormat,
    size: outputContent.length,
  }
}

/**
 * 导出为Markdown
 */
function exportAsMarkdown(spec) {
  let md = `# ${spec.title}\n\n`
  md += `**类型:** ${spec.spec_type}\n\n`
  md += `**版本:** ${spec.metadata.version}\n\n`
  md += `**状态:** ${spec.metadata.status}\n\n`

  if (spec.metadata.author) {
    md += `**作者:** ${spec.metadata.author}\n\n`
  }

  if (spec.metadata.tags.length > 0) {
    md += `**标签:** ${spec.metadata.tags.join(', ')}\n\n`
  }

  md += `**创建时间:** ${spec.created_at}\n\n`
  md += `**更新时间:** ${spec.updated_at}\n\n`

  md += `---\n\n`
  md += spec.content

  return md
}

/**
 * 导出为HTML
 */
function exportAsHtml(spec) {
  const html = `<!DOCTYPE html>
<html lang="zh-CN">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>${spec.title}</title>
  <style>
    body { font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; max-width: 800px; margin: 0 auto; padding: 20px; }
    h1 { color: #333; border-bottom: 2px solid #007bff; padding-bottom: 10px; }
    .metadata { background: #f5f5f5; padding: 15px; border-radius: 5px; margin: 20px 0; }
    .metadata p { margin: 5px 0; }
    .content { line-height: 1.6; color: #555; }
  </style>
</head>
<body>
  <h1>${spec.title}</h1>
  <div class="metadata">
    <p><strong>类型:</strong> ${spec.spec_type}</p>
    <p><strong>版本:</strong> ${spec.metadata.version}</p>
    <p><strong>状态:</strong> ${spec.metadata.status}</p>
    ${spec.metadata.author ? `<p><strong>作者:</strong> ${spec.metadata.author}</p>` : ''}
    ${spec.metadata.tags.length > 0 ? `<p><strong>标签:</strong> ${spec.metadata.tags.join(', ')}</p>` : ''}
    <p><strong>创建时间:</strong> ${spec.created_at}</p>
    <p><strong>更新时间:</strong> ${spec.updated_at}</p>
  </div>
  <div class="content">
    ${spec.content.replace(/\n/g, '<br>')}
  </div>
</body>
</html>`

  return html
}

/**
 * 导入规格
 */
async function importSpec(specData, specType) {
  return await createSpec(specType, specData)
}

/**
 * 加载模板内容
 */
async function loadTemplateContent(templateId) {
  const templatePath = path.join(__dirname, '..', '.spec-workflow', 'templates', `${templateId}-template.md`)

  try {
    const content = await fs.readFile(templatePath, 'utf-8')
    return content
  } catch (error) {
    // 如果找不到模板文件，返回默认模板
    return getDefaultTemplate(templateId)
  }
}

/**
 * 获取默认模板
 */
function getDefaultTemplate(templateId) {
  const templates = {
    product: `# 产品需求规格

## 项目名称
{{project_name}}

## 目标用户
{{target_users}}

## 核心功能
{{core_features}}

## 用户故事
{{user_stories}}

## 非功能需求
{{non_functional_requirements}}
`,
    tech: `# 技术规格

## 系统架构
{{system_architecture}}

## 技术栈
{{tech_stack}}

## 数据库设计
{{database_design}}

## API设计
{{api_design}}

## 部署架构
{{deployment_architecture}}
`,
    design: `# 设计规格

## 设计原则
{{design_principles}}

## UI设计
{{ui_design}}

## UX设计
{{ux_design}}

## 交互流程
{{interaction_flow}}

## 视觉规范
{{visual_guidelines}}
`,
    tasks: `# 任务规格

## 任务概述
{{task_overview}}

## 任务列表
{{task_list}}

## 优先级
{{priority}}

## 时间估算
{{time_estimation}}

## 依赖关系
{{dependencies}}
`,
    structure: `# 结构规格

## 项目结构
{{project_structure}}

## 文件组织
{{file_organization}}

## 命名规范
{{naming_conventions}}

## 代码组织
{{code_organization}}
`,
  }

  return templates[templateId] || `# ${templateId} 规格模板\n\n{{content}}`
}

/**
 * 主函数
 */
async function main() {
  try {
    // 从命令行参数获取输入文件路径
    const inputPath = process.argv[2]
    if (!inputPath) {
      throw new Error('缺少输入文件路径')
    }

    // 读取请求
    const requestContent = await fs.readFile(inputPath, 'utf-8')
    const request = JSON.parse(requestContent)

    // 处理请求
    const result = await handleSpecRequest(request)

    // 生成结果文件路径
    const resultPath = inputPath.replace('.json', '.result.json')

    // 写入结果
    const resultContent = JSON.stringify({
      success: true,
      spec: result.id ? result : undefined,
      specs: Array.isArray(result) ? result : undefined,
      validation_result: result.is_valid !== undefined ? result : undefined,
      export_result: result.output_path ? result : undefined,
    }, null, 2)
    await fs.writeFile(resultPath, resultContent, 'utf-8')

    // 输出结果文件路径
    console.log(resultPath)
  } catch (error) {
    // 错误处理
    const inputPath = process.argv[2]
    const errorPath = inputPath.replace('.json', '.error.json')
    
    await fs.writeFile(errorPath, JSON.stringify({
      error: error.message,
      stack: error.stack,
    }, null, 2), 'utf-8')

    process.exit(1)
  }
}

main()
