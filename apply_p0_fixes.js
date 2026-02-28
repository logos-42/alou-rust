#!/usr/bin/env node
/**
 * P0 问题修复脚本
 * 修复 3 个关键问题：
 * 1. joinGroup() - 支持从 KV 加载和创建外部群聊
 * 2. sendMessage() - 先发送 IPFS 成功后才保存
 * 3. sendTaskAssignment() - 任务分配后更新状态
 */

const fs = require('fs');
const path = require('path');

// 文件路径
const groupChatServicePath = path.join(__dirname, 'alou-desktop/src/services/localIpfsGroupChatService.ts');
const coordinatorServicePath = path.join(__dirname, 'alou-desktop/src/services/agentCoordinatorService.ts');

console.log('开始修复 P0 问题...\n');

// ==================== 修复问题 1 & 2: localIpfsGroupChatService.ts ====================
console.log('处理文件：localIpfsGroupChatService.ts');
let groupChatContent = fs.readFileSync(groupChatServicePath, 'utf-8');

// 问题 1: 在 loadGroupsFromKV 方法之前添加 loadGroupFromKV 方法
const loadGroupFromKVMethod = `
  /**
   * 从 Tauri KV 存储加载单个群聊
   * @param groupId - 群聊 ID
   * @returns 加载的群聊信息，如果不存在则返回 null
   */
  async loadGroupFromKV(groupId: string): Promise<LocalGroup | null> {
    try {
      const key = \`group:\${groupId}\`
      const value: string | null = await invoke('kv_get', { key })
      
      if (value) {
        const group = LocalGroup.fromJSON(JSON.parse(value))
        this.log(LogLevel.INFO, '从 KV 加载群聊成功:', { groupId })
        
        // 保存到内存
        this.groups.set(groupId, group)
        
        // 初始化消息 ID 集合
        this.messageIdSet.set(groupId, new Set())
        this.messageHashSet.set(groupId, new Set())
        
        return group
      }
      
      this.log(LogLevel.DEBUG, 'KV 中未找到群聊:', { groupId })
      return null
    } catch (error: any) {
      this.log(LogLevel.WARN, '从 KV 加载群聊失败:', { groupId, error: error.message })
      return null
    }
  }

`;

// 查找 loadGroupsFromKV 方法的位置
const loadGroupsFromKVIndex = groupChatContent.indexOf('async loadGroupsFromKV(): Promise<void>');
if (loadGroupsFromKVIndex === -1) {
  console.error('错误：找不到 loadGroupsFromKV 方法');
  process.exit(1);
}

// 在 loadGroupsFromKV 之前插入 loadGroupFromKV
const beforeLoadGroups = groupChatContent.substring(0, loadGroupsFromKVIndex);
const afterLoadGroups = groupChatContent.substring(loadGroupsFromKVIndex);
groupChatContent = beforeLoadGroups + loadGroupFromKVMethod + afterLoadGroups;
console.log('✓ 添加 loadGroupFromKV 方法');

// 问题 1: 修改 joinGroup 方法
const oldJoinGroupPattern = `  async joinGroup(groupId: string, topic?: string | null): Promise<LocalGroup> {
    this.log(LogLevel.INFO, '加入群聊:', { groupId })
    
    if (!this.localIdentity) {
      throw new Error('未设置本地身份，无法加入群聊')
    }

    const group = this.groups.get(groupId)
    if (!group) {
      throw new Error('群聊不存在')
    }`;

const newJoinGroupCode = `  async joinGroup(groupId: string, topic?: string | null): Promise<LocalGroup> {
    this.log(LogLevel.INFO, '加入群聊:', { groupId })

    if (!this.localIdentity) {
      throw new Error('未设置本地身份，无法加入群聊')
    }

    // 1. 首先尝试从内存获取群聊
    let group = this.groups.get(groupId)
    
    // 2. 如果内存中没有，尝试从 KV 加载
    if (!group) {
      this.log(LogLevel.INFO, '内存中未找到群聊，尝试从 KV 加载:', { groupId })
      group = await this.loadGroupFromKV(groupId)
    }
    
    // 3. 如果 KV 中也没有，创建新的群聊对象（用于加入外部创建的群聊）
    if (!group) {
      this.log(LogLevel.INFO, 'KV 中也未找到群聊，创建新群聊对象:', { groupId })
      const groupTopic = topic || \`diap/cluster_action/\${groupId}\`
      group = new LocalGroup({
        groupId,
        groupName: \`群聊 \${groupId.slice(-8)}\`,
        description: '',
        topic: groupTopic,
        members: [],
        creator: 'unknown',
        createdAt: Date.now(),
        metadata: {
          isPublic: true,
          externalGroup: true
        }
      })
      
      // 保存到新创建的群聊到 KV
      await this.saveGroupToKV(group)
    }`;

if (!groupChatContent.includes(oldJoinGroupPattern)) {
  console.error('错误：找不到旧的 joinGroup 代码');
  process.exit(1);
}

groupChatContent = groupChatContent.replace(oldJoinGroupPattern, newJoinGroupCode);
console.log('✓ 修复 joinGroup 方法 - 支持从 KV 加载和创建外部群聊');

// 问题 2: 修改 sendMessage 方法
const oldSendMessagePattern = `    try {
      // 发布到 IPFS PubSub
      await invoke('ipfs_pubsub_publish', {
        topic: messageObj.topic,
        message: JSON.stringify(messageObj.toJSON()),
        ipfs_api_url: DEFAULT_IPFS_API
      })

      // 保存到内存
      this.saveMessageToMemory(messageObj.groupId, messageObj)
      
      // 保存到 KV 存储
      await this.saveMessageToKV(messageObj.groupId, messageObj)`;

const newSendMessageCode = `    try {
      // 先发送到 IPFS
      await invoke('ipfs_pubsub_publish', {
        topic: messageObj.topic,
        message: JSON.stringify(messageObj.toJSON()),
        ipfs_api_url: DEFAULT_IPFS_API
      })

      this.log(LogLevel.INFO, 'IPFS 发布成功，准备保存消息:', { messageId: messageObj.id })

      // IPFS 发布成功后才保存
      // 保存到内存
      this.saveMessageToMemory(messageObj.groupId, messageObj)

      // 保存到 KV 存储
      await this.saveMessageToKV(messageObj.groupId, messageObj)`;

if (!groupChatContent.includes(oldSendMessagePattern)) {
  console.error('错误：找不到旧的 sendMessage 代码');
  process.exit(1);
}

groupChatContent = groupChatContent.replace(oldSendMessagePattern, newSendMessageCode);
console.log('✓ 修复 sendMessage 方法 - 先发送 IPFS 成功后才保存');

// 修改错误日志
const oldErrorLog = "this.log(LogLevel.ERROR, '发送消息失败，加入重试队列:', {";
const newErrorLog = "this.log(LogLevel.ERROR, '发送消息失败，不保存到本地:', {";

if (groupChatContent.includes(oldErrorLog)) {
  groupChatContent = groupChatContent.replace(oldErrorLog, newErrorLog);
  console.log('✓ 更新错误日志描述');
}

// 保存文件
fs.writeFileSync(groupChatServicePath, groupChatContent, 'utf-8');
console.log('✓ 文件已保存：localIpfsGroupChatService.ts\n');

// ==================== 修复问题 3: agentCoordinatorService.ts ====================
console.log('处理文件：agentCoordinatorService.ts');
let coordinatorContent = fs.readFileSync(coordinatorServicePath, 'utf-8');

// 问题 3: 修改 sendTaskAssignment 方法
const oldSendTaskAssignment = `  private async sendTaskAssignment(task: TaskInfo, agent: AgentInfo): Promise<void> {
    const assignmentMessage = new PubSubMessage({
      type: MessageType.TASK_ASSIGN,
      from: task.requesterId,
      to: agent.did || agent.id,
      content: task.content,
      topic: agent.pubsubTopics[0] || pubsubService.generateAgentTopic(agent.id),
      metadata: {
        taskId: task.id,
        taskType: task.type,
        priority: task.priority,
        assignedAt: task.assignedAt,
      },
    });

    try {
      await pubsubService.publish(assignmentMessage.topic, assignmentMessage);
    } catch (error: any) {
      this.log('发送任务分配消息失败:', { error: error.message });
    }
  }`;

const newSendTaskAssignment = `  private async sendTaskAssignment(task: TaskInfo, agent: AgentInfo): Promise<void> {
    const assignmentMessage = new PubSubMessage({
      type: MessageType.TASK_ASSIGN,
      from: task.requesterId,
      to: agent.did || agent.id,
      content: task.content,
      topic: agent.pubsubTopics[0] || pubsubService.generateAgentTopic(agent.id),
      metadata: {
        taskId: task.id,
        taskType: task.type,
        priority: task.priority,
        assignedAt: task.assignedAt,
      },
    });

    try {
      await pubsubService.publish(assignmentMessage.topic, assignmentMessage);
      this.log('任务分配消息发送成功:', { taskId: task.id, agentId: agent.id });
    } catch (error: any) {
      this.log('发送任务分配消息失败:', { error: error.message });
    }

    // 发送任务分配后，更新任务状态为已分配
    this.updateTaskStatus(task.id, TaskStatus.ASSIGNED, null, undefined);
    this.log('任务状态已更新为已分配:', { taskId: task.id, assignee: agent.id });
  }`;

if (!coordinatorContent.includes(oldSendTaskAssignment)) {
  console.error('错误：找不到旧的 sendTaskAssignment 代码');
  process.exit(1);
}

coordinatorContent = coordinatorContent.replace(oldSendTaskAssignment, newSendTaskAssignment);
console.log('✓ 修复 sendTaskAssignment 方法 - 任务分配后更新状态');

// 保存文件
fs.writeFileSync(coordinatorServicePath, coordinatorContent, 'utf-8');
console.log('✓ 文件已保存：agentCoordinatorService.ts\n');

console.log('====================');
console.log('所有 P0 问题已修复完成！');
console.log('====================');
