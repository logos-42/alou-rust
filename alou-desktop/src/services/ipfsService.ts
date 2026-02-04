/**
 * IPFS Service - 与本地 Kubo 节点通信
 * 通过 Tauri 命令与 Rust 后端交互
 */
import { invoke } from '@tauri-apps/api/core';

// IPFS 节点信息
export interface IpfsNodeInfo {
  id?: string;
  addresses?: string[];
  version?: string;
  peerId?: string;
}

// IPFS API 测试结果
export interface IpfsApiTestResult {
  success: boolean;
  apiUrl: string;
  error?: string;
}

// IPFS API 诊断结果
export interface IpfsDiagnosisResult {
  success: boolean;
  diagnosis?: {
    config_exists: boolean;
    api_enabled: boolean;
    api_address?: string;
    api_address_http?: string;
    api_accessible?: boolean;
    recommendations?: string[];
  };
  error?: string;
}

// IPFS API 就绪结果
export interface IpfsApiReadyResult {
  success: boolean;
  attempts?: number;
  method?: string;
  apiUrl?: string;
  error?: string;
  diagnosis?: IpfsDiagnosisResult['diagnosis'];
}

// 通用的服务响应类型
interface ServiceResult<T = any> {
  success: boolean;
  message?: string;
  error?: string;
  data?: T;
}

export class IpfsService {
  async downloadKubo(): Promise<ServiceResult<string>> {
    try {
      const result = await invoke('download_kubo_binary');
      return { success: true, message: String(result) };
    } catch (error) {
      console.error('Failed to download Kubo:', error);
      return { success: false, error: String(error) };
    }
  }

  async checkKuboInstalled(): Promise<boolean> {
    try {
      await invoke('get_ipfs_info');
      return true;
    } catch {
      try {
        await invoke('start_ipfs_node');
        return true;
      } catch {
        return false;
      }
    }
  }

  async startNode(autoDownload = true): Promise<ServiceResult<string>> {
    try {
      const result = await invoke('start_ipfs_node');
      return { success: true, message: String(result) };
    } catch (error: any) {
      const errorMsg = String(error);
      if (autoDownload && errorMsg.includes('not found')) {
        console.log('Kubo binary not found, downloading...');
        const downloadResult = await this.downloadKubo();
        if (downloadResult.success) {
          return this.startNode(false);
        }
        return { success: false, error: 'Failed to download Kubo binary' };
      }
      if ((errorMsg.includes('端口') && errorMsg.includes('5001')) || (errorMsg.includes('port') && errorMsg.includes('5001'))) {
        return { success: true, message: '使用已存在的 IPFS 实例', error: '检测到另一个 IPFS 实例' };
      }
      return { success: false, error: errorMsg };
    }
  }

  async stopNode(): Promise<ServiceResult<string>> {
    try {
      const result = await invoke('stop_ipfs_node');
      return { success: true, message: String(result) };
    } catch (error: any) {
      return { success: false, error: String(error) };
    }
  }

  async getNodeInfo(): Promise<ServiceResult<IpfsNodeInfo>> {
    try {
      const info = await invoke('get_ipfs_info');
      return { success: true, data: info as IpfsNodeInfo };
    } catch (error: any) {
      return { success: false, error: String(error) };
    }
  }

  async isNodeRunning(): Promise<boolean> {
    try {
      const result = await this.getNodeInfo();
      return result.success;
    } catch {
      return false;
    }
  }

  async getApiAddressFromConfig(): Promise<ServiceResult<string>> {
    try {
      const address = await invoke('get_ipfs_api_address');
      return { success: true, data: String(address) };
    } catch (error: any) {
      return { success: false, error: String(error) };
    }
  }

  async diagnoseApi(): Promise<IpfsDiagnosisResult> {
    try {
      const diagnosis = await invoke('diagnose_ipfs_api');
      return { success: true, diagnosis: diagnosis as IpfsDiagnosisResult['diagnosis'] };
    } catch (error: any) {
      return { success: false, error: String(error) };
    }
  }

  async testHttpApi(ipfsApiUrl?: string | null, useConfig = false): Promise<IpfsApiTestResult> {
    try {
      const apiUrl = ipfsApiUrl || import.meta.env.VITE_IPFS_API_URL || 'http://127.0.0.1:5001';
      const command = useConfig ? 'test_ipfs_api_with_config' : 'test_ipfs_api';
      const params = { ipfsApiUrl };
      const result = await invoke(command, params);
      if (typeof result === 'object' && result !== null) {
        const res = result as Record<string, any>;
        return { success: res.success || false, apiUrl: res.api_url || apiUrl, error: res.error };
      }
      return { success: result === true, apiUrl };
    } catch (error: any) {
      return { success: false, error: String(error), apiUrl: ipfsApiUrl || 'http://127.0.0.1:5001' };
    }
  }

  async waitForApiReady(maxRetries = 20, delayMs = 1000): Promise<IpfsApiReadyResult> {
    let apiUrl = import.meta.env.VITE_IPFS_API_URL || 'http://127.0.0.1:5001';
    const configAddress = await this.getApiAddressFromConfig();
    if (configAddress.success && configAddress.data) {
      apiUrl = configAddress.data;
    }

    for (let i = 0; i < maxRetries; i++) {
      const useConfig = i >= 3 && i % 5 === 0;
      const httpTest = await this.testHttpApi(apiUrl, useConfig);
      if (httpTest.success) {
        return { success: true, attempts: i + 1, method: 'http', apiUrl: httpTest.apiUrl };
      }
      if (i < maxRetries - 1) {
        await new Promise((resolve) => setTimeout(resolve, delayMs));
      }
    }

    const diagnosis = await this.diagnoseApi();
    let errorMsg = `IPFS HTTP API 在 ${maxRetries} 次尝试后仍未就绪。`;
    if (diagnosis.success && diagnosis.diagnosis) {
      const diag = diagnosis.diagnosis;
      errorMsg += ` 诊断: config_exists=${diag.config_exists}, api_enabled=${diag.api_enabled}`;
    }

    return { success: false, error: errorMsg, diagnosis: diagnosis.success ? diagnosis.diagnosis : undefined };
  }
}

export default new IpfsService();
