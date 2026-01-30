// 消息类型
interface Message {
  type: 'user' | 'assistant' | string;
  content: string;
  timestamp: number;
}

// 历史消息类型
interface HistoryMessage {
  role: 'user' | 'assistant';
  content: string;
  timestamp: number;
}

/**
 * 辅助函数：获取消息历史
 * @param agentId - 智能体ID
 * @param messagesByChannel - 按频道分组的消息
 * @returns 最近10条消息历史
 */
export const getMessageHistory = (
  agentId: string,
  messagesByChannel: Record<string, Message[]>
): HistoryMessage[] => {
  const messages = messagesByChannel?.[agentId] || [];
  return messages
    .filter(msg => msg.type === 'user' || msg.type === 'assistant')
    .map(msg => ({
      role: msg.type === 'user' ? 'user' : 'assistant',
      content: msg.content || '',
      timestamp: msg.timestamp || Date.now(),
    }))
    .slice(-10); // 只保留最近10条消息
};

export default getMessageHistory;
