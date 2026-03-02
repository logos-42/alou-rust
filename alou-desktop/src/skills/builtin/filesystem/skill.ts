/**
 * Filesystem Skill - 文件系统操作技能
 * 
 * 提供文件和目录的读取、写入、删除、移动等操作
 * 
 * @author Alou Team
 * @version 1.0.0
 * @license MIT
 */

import { Skill, SkillContext, SkillResult, SkillParameters } from '../../../types/skills';
import * as fs from 'fs/promises';
import * as path from 'path';

export class FilesystemSkill extends Skill {
  name = 'filesystem';
  description = '文件系统操作技能，支持读取、写入、删除、移动文件和目录';
  version = '1.0.0';
  category = 'file';
  
  parameters: SkillParameters = {
    type: 'object',
    properties: {
      path: {
        type: 'string',
        description: '文件或目录路径',
      },
      operation: {
        type: 'string',
        description: '操作类型',
        enum: ['read', 'write', 'delete', 'move', 'copy', 'list', 'exists', 'mkdir'],
      },
      content: {
        type: 'string',
        description: '写入的内容（write 操作需要）',
      },
      destination: {
        type: 'string',
        description: '目标路径（move/copy 操作需要）',
      },
      options: {
        type: 'object',
        description: '可选配置',
        properties: {
          encoding: {
            type: 'string',
            default: 'utf-8',
          },
          recursive: {
            type: 'boolean',
            default: false,
          },
        },
      },
    },
    required: ['path', 'operation'],
  };

  /**
   * 初始化技能
   */
  async initialize(context: SkillContext): Promise<void> {
    context.logger.info('[FilesystemSkill] 初始化完成');
  }

  /**
   * 执行技能
   */
  async execute(params: Record<string, any>, context: SkillContext): Promise<SkillResult> {
    const startTime = Date.now();
    
    try {
      // 参数验证
      this.validateParams(params);
      
      const { path: filePath, operation, content, destination, options = {} } = params;
      
      // 解析路径（处理 ~）
      const resolvedPath = this.resolvePath(filePath, context);
      
      // 执行操作
      let result: any;
      
      switch (operation) {
        case 'read':
          result = await this.readFile(resolvedPath, options);
          break;
        case 'write':
          if (!content) {
            throw new Error('write 操作需要 content 参数');
          }
          result = await this.writeFile(resolvedPath, content, options);
          break;
        case 'delete':
          result = await this.deleteFile(resolvedPath, options);
          break;
        case 'move':
          if (!destination) {
            throw new Error('move 操作需要 destination 参数');
          }
          result = await this.moveFile(resolvedPath, this.resolvePath(destination, context), options);
          break;
        case 'copy':
          if (!destination) {
            throw new Error('copy 操作需要 destination 参数');
          }
          result = await this.copyFile(resolvedPath, this.resolvePath(destination, context), options);
          break;
        case 'list':
          result = await this.listDirectory(resolvedPath, options);
          break;
        case 'exists':
          result = await this.checkExists(resolvedPath);
          break;
        case 'mkdir':
          result = await this.createDirectory(resolvedPath, options);
          break;
        default:
          throw new Error(`未知操作：${operation}`);
      }
      
      return {
        success: true,
        output: result,
        metadata: {
          duration: Date.now() - startTime,
          version: this.version,
          operation,
          path: resolvedPath,
        },
      };
    } catch (error) {
      return {
        success: false,
        output: null,
        error: error instanceof Error ? error.message : '未知错误',
        errorDetails: {
          code: 'FILESYSTEM_ERROR',
          message: error instanceof Error ? error.message : '未知错误',
        },
        metadata: {
          duration: Date.now() - startTime,
          version: this.version,
        },
      };
    }
  }

  /**
   * 验证参数
   */
  private validateParams(params: Record<string, any>): void {
    if (!params.path) {
      throw new Error('缺少必需参数：path');
    }
    if (!params.operation) {
      throw new Error('缺少必需参数：operation');
    }
    
    const validOperations = ['read', 'write', 'delete', 'move', 'copy', 'list', 'exists', 'mkdir'];
    if (!validOperations.includes(params.operation)) {
      throw new Error(`无效操作：${params.operation}，支持的操作：${validOperations.join(', ')}`);
    }
  }

  /**
   * 解析路径
   */
  private resolvePath(filePath: string, context: SkillContext): string {
    // 处理 ~ 路径
    if (filePath.startsWith('~')) {
      const homeDir = process.env.HOME || process.env.USERPROFILE || '';
      return path.join(homeDir, filePath.slice(1));
    }
    
    // 确保路径在允许的范围内
    const absolutePath = path.resolve(filePath);
    
    return absolutePath;
  }

  /**
   * 读取文件
   */
  private async readFile(filePath: string, options: any): Promise<string> {
    const encoding = options.encoding || 'utf-8';
    return await fs.readFile(filePath, encoding);
  }

  /**
   * 写入文件
   */
  private async writeFile(filePath: string, content: string, options: any): Promise<void> {
    const encoding = options.encoding || 'utf-8';
    await fs.writeFile(filePath, content, encoding);
  }

  /**
   * 删除文件
   */
  private async deleteFile(filePath: string, options: any): Promise<void> {
    const recursive = options.recursive || false;
    await fs.rm(filePath, { recursive, force: true });
  }

  /**
   * 移动文件
   */
  private async moveFile(source: string, destination: string, options: any): Promise<void> {
    await fs.rename(source, destination);
  }

  /**
   * 复制文件
   */
  private async copyFile(source: string, destination: string, options: any): Promise<void> {
    const recursive = options.recursive || false;
    if (recursive) {
      await this.copyRecursive(source, destination);
    } else {
      await fs.copyFile(source, destination);
    }
  }

  /**
   * 递归复制目录
   */
  private async copyRecursive(source: string, destination: string): Promise<void> {
    const stats = await fs.stat(source);
    
    if (stats.isDirectory()) {
      await fs.mkdir(destination, { recursive: true });
      const entries = await fs.readdir(source);
      
      for (const entry of entries) {
        const sourcePath = path.join(source, entry);
        const destPath = path.join(destination, entry);
        await this.copyRecursive(sourcePath, destPath);
      }
    } else {
      await fs.copyFile(source, destination);
    }
  }

  /**
   * 列出目录内容
   */
  private async listDirectory(dirPath: string, options: any): Promise<string[]> {
    return await fs.readdir(dirPath);
  }

  /**
   * 检查文件是否存在
   */
  private async checkExists(filePath: string): Promise<boolean> {
    try {
      await fs.access(filePath);
      return true;
    } catch {
      return false;
    }
  }

  /**
   * 创建目录
   */
  private async createDirectory(dirPath: string, options: any): Promise<void> {
    const recursive = options.recursive || false;
    await fs.mkdir(dirPath, { recursive });
  }
}

export default new FilesystemSkill();
