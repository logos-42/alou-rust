/**
 * IPFS Auto-Fix Skill - IPFS 自动修复技能
 * 
 * 自动检测和修复 IPFS 节点问题
 * 
 * @author Alou Team
 * @version 1.0.0
 * @license MIT
 */

import { invoke } from '@tauri-apps/api/core';

export interface IpfsAutoFixParams {
  action: 'check' | 'fix' | 'start' | 'stop' | 'restart' | 'diagnose';
  autoDownload?: boolean;
  maxRetries?: number;
}

export interface IpfsAutoFixResult {
  success: boolean;
  status?: 'running' | 'stopped' | 'error' | 'not_installed';
  nodeInfo?: {
    id?: string;
    version?: string;
    addresses?: string[];
    peerId?: string;
  };
  actions?: string[];
  message?: string;
  error?: string;
  diagnosis?: {
    kuboInstalled: boolean;
    nodeRunning: boolean;
    apiAccessible: boolean;
    portConflict: boolean;
    recommendations: string[];
  };
}

export class IpfsAutoFixSkill {
  name = 'ipfs-auto-fix';
  description = 'IPFS 自动修复技能，检测和修复 IPFS 节点问题';
  version = '1.0.0';
  category = 'system';

  /**
   * 执行技能
   */
  async execute(params: IpfsAutoFixParams): Promise<IpfsAutoFixResult> {
    const { action, autoDownload = true, maxRetries = 3 } = params;

    try {
      switch (action) {
        case 'check':
          return await this.checkStatus();
        case 'fix':
          return await this.autoFix(autoDownload, maxRetries);
        case 'start':
          return await this.startNode(autoDownload);
        case 'stop':
          return await this.stopNode();
        case 'restart':
          return await this.restartNode(autoDownload);
        case 'diagnose':
          return await this.diagnose();
        default:
          return {
            success: false,
            error: `未知操作: ${action}`,
          };
      }
    } catch (error) {
      return {
        success: false,
        error: error instanceof Error ? error.message : '未知错误',
      };
    }
  }

  /**
   * 检查 IPFS 状态
   */
  private async checkStatus(): Promise<IpfsAutoFixResult> {
    try {
      const nodeInfo = await invoke<any>('get_ipfs_info');
      
      return {
        success: true,
        status: 'running',
        nodeInfo: {
          id: nodeInfo.id || nodeInfo.ID,
          version: nodeInfo.version || nodeInfo.AgentVersion,
          addresses: nodeInfo.addresses || nodeInfo.Addresses || [],
          peerId: nodeInfo.id || nodeInfo.ID,
        },
        message: 'IPFS 节点运行正常',
      };
    } catch (error) {
      return {
        success: false,
        status: 'stopped',
        message: 'IPFS 节点未运行',
        error: error instanceof Error ? error.message : '未知错误',
      };
    }
  }

  /**
   * 自动修复 IPFS
   */
  private async autoFix(autoDownload: boolean, maxRetries: number): Promise<IpfsAutoFixResult> {
    const actions: string[] = [];
    let currentRetry = 0;

    console.log('[IpfsAutoFixSkill] 开始自动修复 IPFS...');

    // 1. 检查当前状态
    const statusCheck = await this.checkStatus();
    if (statusCheck.success && statusCheck.status === 'running') {
      return {
        success: true,
        status: 'running',
        nodeInfo: statusCheck.nodeInfo,
        actions: ['already_running'],
        message: 'IPFS 节点已经在运行',
      };
    }

    // 2. 尝试启动节点
    while (currentRetry < maxRetries) {
      console.log(`[IpfsAutoFixSkill] 尝试启动 IPFS (${currentRetry + 1}/${maxRetries})...`);
      
      try {
        const result = await invoke<any>('start_ipfs_node');
        actions.push('started');
        
        // 等待节点启动
        await this.sleep(2000);
        
        // 验证节点是否成功启动
        const verifyCheck = await this.checkStatus();
        if (verifyCheck.success && verifyCheck.status === 'running') {
          return {
            success: true,
            status: 'running',
            nodeInfo: verifyCheck.nodeInfo,
            actions,
            message: 'IPFS 节点已成功启动',
          };
        }
      } catch (error: any) {
        const errorMsg = String(error);
        
        // 3. 处理 Kubo 未安装的情况
        if (errorMsg.includes('not found') || errorMsg.includes('未找到')) {
          if (autoDownload && !actions.includes('downloaded_kubo')) {
            console.log('[IpfsAutoFixSkill] Kubo 未安装，开始下载...');
            try {
              await invoke('download_kubo_binary');
              actions.push('downloaded_kubo');
              console.log('[IpfsAutoFixSkill] Kubo 下载成功');
              // 下载后重试启动
              continue;
            } catch (downloadError) {
              return {
                success: false,
                status: 'not_installed',
                actions,
                error: 'Kubo 下载失败',
                message: '请手动安装 Kubo: https://docs.ipfs.tech/install/command-line/',
              };
            }
          } else {
            return {
              success: false,
              status: 'not_installed',
              actions,
              error: 'Kubo 未安装',
              message: '请手动安装 Kubo 或启用自动下载',
            };
          }
        }
        
        // 4. 处理端口冲突
        if (errorMsg.includes('端口') || errorMsg.includes('port') || errorMsg.includes('5001')) {
          console.log('[IpfsAutoFixSkill] 检测到端口冲突，尝试使用现有实例...');
          actions.push('port_conflict_detected');
          
          // 验证现有实例是否可用
          const verifyCheck = await this.checkStatus();
          if (verifyCheck.success && verifyCheck.status === 'running') {
            return {
              success: true,
              status: 'running',
              nodeInfo: verifyCheck.nodeInfo,
              actions: [...actions, 'using_existing_instance'],
              message: '使用现有的 IPFS 实例',
            };
          }
        }
        
        console.warn(`[IpfsAutoFixSkill] 启动失败 (${currentRetry + 1}/${maxRetries}):`, errorMsg);
      }
      
      currentRetry++;
      if (currentRetry < maxRetries) {
        await this.sleep(1000);
      }
    }

    // 5. 所有尝试都失败
    return {
      success: false,
      status: 'error',
      actions,
      error: '无法启动 IPFS 节点',
      message: `已尝试 ${maxRetries} 次，请检查日志或手动启动`,
    };
  }

  /**
   * 启动 IPFS 节点
   */
  private async startNode(autoDownload: boolean): Promise<IpfsAutoFixResult> {
    try {
      const result = await invoke<any>('start_ipfs_node');
      
      // 等待节点启动
      await this.sleep(2000);
      
      // 验证启动
      const statusCheck = await this.checkStatus();
      if (statusCheck.success && statusCheck.status === 'running') {
        return {
          success: true,
          status: 'running',
          nodeInfo: statusCheck.nodeInfo,
          actions: ['started'],
          message: 'IPFS 节点已启动',
        };
      }
      
      return {
        success: false,
        status: 'error',
        error: '节点启动后验证失败',
      };
    } catch (error: any) {
      const errorMsg = String(error);
      
      // 如果是 Kubo 未安装且允许自动下载
      if ((errorMsg.includes('not found') || errorMsg.includes('未找到')) && autoDownload) {
        console.log('[IpfsAutoFixSkill] Kubo 未安装，尝试下载...');
        try {
          await invoke('download_kubo_binary');
          // 下载后重试启动
          return await this.startNode(false);
        } catch (downloadError) {
          return {
            success: false,
            status: 'not_installed',
            error: 'Kubo 下载失败',
          };
        }
      }
      
      return {
        success: false,
        status: 'error',
        error: errorMsg,
      };
    }
  }

  /**
   * 停止 IPFS 节点
   */
  private async stopNode(): Promise<IpfsAutoFixResult> {
    try {
      await invoke('stop_ipfs_node');
      
      return {
        success: true,
        status: 'stopped',
        actions: ['stopped'],
        message: 'IPFS 节点已停止',
      };
    } catch (error) {
      return {
        success: false,
        error: error instanceof Error ? error.message : '停止失败',
      };
    }
  }

  /**
   * 重启 IPFS 节点
   */
  private async restartNode(autoDownload: boolean): Promise<IpfsAutoFixResult> {
    console.log('[IpfsAutoFixSkill] 重启 IPFS 节点...');
    
    // 先停止
    await this.stopNode();
    await this.sleep(1000);
    
    // 再启动
    return await this.startNode(autoDownload);
  }

  /**
   * 诊断 IPFS 问题
   */
  private async diagnose(): Promise<IpfsAutoFixResult> {
    const diagnosis = {
      kuboInstalled: false,
      nodeRunning: false,
      apiAccessible: false,
      portConflict: false,
      recommendations: [] as string[],
    };

    // 1. 检查 Kubo 是否安装
    try {
      await invoke('get_ipfs_info');
      diagnosis.kuboInstalled = true;
      diagnosis.nodeRunning = true;
      diagnosis.apiAccessible = true;
    } catch (error: any) {
      const errorMsg = String(error);
      
      if (errorMsg.includes('not found') || errorMsg.includes('未找到')) {
        diagnosis.kuboInstalled = false;
        diagnosis.recommendations.push('安装 Kubo: 运行自动修复或访问 https://docs.ipfs.tech/install/');
      } else if (errorMsg.includes('端口') || errorMsg.includes('port')) {
        diagnosis.kuboInstalled = true;
        diagnosis.portConflict = true;
        diagnosis.recommendations.push('端口 5001 被占用，请关闭其他 IPFS 实例或使用现有实例');
      } else {
        diagnosis.kuboInstalled = true;
        diagnosis.nodeRunning = false;
        diagnosis.recommendations.push('IPFS 节点未运行，尝试启动: 运行自动修复或手动启动');
      }
    }

    // 2. 检查 API 可访问性
    if (diagnosis.nodeRunning) {
      try {
        const nodeInfo = await invoke<any>('get_ipfs_info');
        diagnosis.apiAccessible = !!nodeInfo;
      } catch {
        diagnosis.apiAccessible = false;
        diagnosis.recommendations.push('IPFS API 不可访问，检查配置或重启节点');
      }
    }

    // 3. 生成建议
    if (diagnosis.kuboInstalled && diagnosis.nodeRunning && diagnosis.apiAccessible) {
      diagnosis.recommendations.push('✅ IPFS 运行正常，无需修复');
    }

    return {
      success: true,
      diagnosis,
      message: '诊断完成',
    };
  }

  /**
   * 睡眠函数
   */
  private sleep(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms));
  }
}

// 导出单例
export default new IpfsAutoFixSkill();
