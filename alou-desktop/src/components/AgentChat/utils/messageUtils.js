// 辅助函数：获取消息历史
export const getMessageHistory = (agentId, messagesByChannel) => {
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