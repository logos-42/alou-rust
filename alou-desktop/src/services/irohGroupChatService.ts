/**
 * Iroh 群聊服务
 * 用于同步和管理通过 Iroh 工具创建的群聊
 */

import { invoke } from '@tauri-apps/api/core';

export interface IrohGroup {
  group_id: string;
  group_name: string;
  ticket: string;
  description?: string;
  members: string[];
  created_at: number;
  created_by: string;
}

export interface IrohGroupMessage {
  id: string;
  group_id: string;
  sender: string;
  content: string;
  timestamp: number;
}

class IrohGroupChatService {
  /**
   * 获取所有 Iroh 群聊
   */
  async listGroups(): Promise<IrohGroup[]> {
    try {
      const result = await invoke<any>('execute_tool', {
        toolId: 'iroh',
        args: JSON.stringify({
          action: 'list_groups'
        })
      });

      if (result.success && result.data) {
        return result.data as IrohGroup[];
      }

      return [];
    } catch (error) {
      console.error('[IrohGroupChatService] 获取群聊列表失败:', error);
      return [];
    }
  }

  /**
   * 获取群聊信息
   */
  async getGroupInfo(groupId: string): Promise<IrohGroup | null> {
    try {
      const result = await invoke<any>('execute_tool', {
        toolId: 'iroh',
        args: JSON.stringify({
          action: 'get_group_info',
          group_id: groupId
        })
      });

      if (result.success && result.data) {
        return result.data as IrohGroup;
      }

      return null;
    } catch (error) {
      console.error('[IrohGroupChatService] 获取群聊信息失败:', error);
      return null;
    }
  }

  /**
   * 加入群聊
   */
  async joinGroup(groupId: string): Promise<IrohGroup | null> {
    try {
      const result = await invoke<any>('execute_tool', {
        toolId: 'iroh',
        args: JSON.stringify({
          action: 'join_group',
          group_id: groupId
        })
      });

      if (result.success && result.data) {
        return result.data as IrohGroup;
      }

      return null;
    } catch (error) {
      console.error('[IrohGroupChatService] 加入群聊失败:', error);
      return null;
    }
  }

  /**
   * 发送群聊消息
   */
  async sendMessage(groupId: string, message: string): Promise<boolean> {
    try {
      const result = await invoke<any>('execute_tool', {
        toolId: 'iroh',
        args: JSON.stringify({
          action: 'send_group_message',
          group_id: groupId,
          message
        })
      });

      return result.success;
    } catch (error) {
      console.error('[IrohGroupChatService] 发送消息失败:', error);
      return false;
    }
  }
}

export const irohGroupChatService = new IrohGroupChatService();
export default irohGroupChatService;
