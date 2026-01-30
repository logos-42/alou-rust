/**
 * rustMemoryStore 类型声明
 */

export interface StorageStats {
  totalItems: number;
  expiredItems: number;
  maxItems: number;
  memoryUsage: number;
  memoryUsageFormatted: string;
  usagePercentage: number;
}

/**
 * RustMemoryStorage 类
 */
export class RustMemoryStorage {
  isReady: boolean;
  constructor();
  init(): Promise<void>;
  waitForReady(): Promise<void>;
  setItem(key: string, value: any): Promise<void>;
  getItem(key: string): Promise<any>;
  removeItem(key: string): Promise<boolean>;
  clear(): Promise<void>;
  getKeys(): Promise<string[]>;
  getStats(): Promise<StorageStats | null>;
  cleanupExpired(): Promise<number>;
  cleanupLRU(keepCount?: number): Promise<number>;
  setExpiration(key: string, expiresInSeconds: number): Promise<void>;
  setItems(items: Record<string, any>): Promise<void>;
  getItems(keys: string[]): Promise<Record<string, any>>;
  removeItems(keys: string[]): Promise<void>;
}

export const rustMemoryStore: {
  // 基础 API
  setItem: (key: string, value: any) => Promise<void>;
  getItem: (key: string) => Promise<any>;
  removeItem: (key: string) => Promise<boolean>;
  clear: () => Promise<void>;
  key: (index: number) => Promise<string | null>;
  length: Promise<number>;
  keys: () => Promise<string[]>;
  values: () => Promise<any[]>;
  entries: () => Promise<[string, any][]>;

  // 群聊专用方法
  getGroupChats: (channelId: string) => Promise<any[]>;
  setGroupChats: (channelId: string, chats: any[]) => Promise<void>;
  getActiveActionId: (channelId: string) => Promise<any>;
  setActiveActionId: (channelId: string, actionId: any) => Promise<void>;

  // DIAP 群聊方法
  getDiapGroups: () => Promise<any[]>;
  setDiapGroup: (groupId: string, groupData: any) => Promise<void>;
  removeDiapGroup: (groupId: string) => Promise<void>;

  // 统计信息
  getStats: () => Promise<StorageStats | null>;

  // 清理方法
  cleanup: (options?: any) => Promise<number>;
  cleanupLRU: (keepCount?: number) => Promise<number>;

  // 等待就绪
  ready: () => Promise<void>;

  // 批量操作
  setItems: (items: Record<string, any>) => Promise<void>;
  getItems: (keys: string[]) => Promise<Record<string, any>>;
  removeItems: (keys: string[]) => Promise<void>;
};

export default typeof rustMemoryStore;
