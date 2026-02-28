#!/usr/bin/env python3
"""
P0 问题修复脚本 - Python 版本
修复 3 个关键问题
"""

import re

# 文件路径
GROUP_CHAT_SERVICE = '/Users/apple/Downloads/alou/alou-desktop/src/services/localIpfsGroupChatService.ts'
COORDINATOR_SERVICE = '/Users/apple/Downloads/alou/alou-desktop/src/services/agentCoordinatorService.ts'

print('开始修复 P0 问题...\n')

# ==================== 修复问题 1 & 2: localIpfsGroupChatService.ts ====================
print('处理文件：localIpfsGroupChatService.ts')

with open(GROUP_CHAT_SERVICE, 'r', encoding='utf-8') as f:
    content = f.read()

# 问题 1: 在 loadGroupsFromKV 方法之前添加 loadGroupFromKV 方法
load_group_from_kv_method = '''
  /**
   * 从 Tauri KV 存储加载单个群聊
   * @param groupId - 群聊 ID
   * @returns 加载的群聊信息，如果不存在则返回 null
   */
  async loadGroupFromKV(groupId: string): Promise<LocalGroup | null> {
    try {
      const key = `group:${groupId}`
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

'''

# 查找 loadGroupsFromKV 方法的位置
load_groups_match = re.search(r'async loadGroupsFromKV\(\):', content)
if not load_groups_match:
    print('错误：找不到 loadGroupsFromKV 方法')
    exit(1)

insert_pos = load_groups_match.start()
content = content[:insert_pos] + load_group_from_kv_method + content[insert_pos:]
print('✓ 添加 loadGroupFromKV 方法')

# 问题 1: 修改 joinGroup 方法
old_join_group = r'''(  async joinGroup\(groupId: string, topic\?: string \| null\): Promise<LocalGroup> \{
    this\.log\(LogLevel\.INFO, '加入群聊:', \{ groupId \}\)
    
    if \(!this\.localIdentity\) \{
      throw new Error\('未设置本地身份，无法加入群聊'\)
    \}

    const group = this\.groups\.get\(groupId\)
    if \(!group\) \{
      throw new Error\('群聊不存在'\)
    \})'''

new_join_group = '''  async joinGroup(groupId: string, topic?: string | null): Promise<LocalGroup> {
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
      const groupTopic = topic || `diap/cluster_action/${groupId}`
      group = new LocalGroup({
        groupId,
        groupName: `群聊 ${groupId.slice(-8)}`,
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
    }'''

content = re.sub(old_join_group, new_join_group, content)
print('✓ 修复 joinGroup 方法 - 支持从 KV 加载和创建外部群聊')

# 问题 2: 修改 sendMessage 方法
# 修改注释
content = content.replace('// 发布到 IPFS PubSub', '// 先发送到 IPFS')

# 在 IPFS 发布后添加日志
old_ipfs_publish = '''await invoke('ipfs_pubsub_publish', {
        topic: messageObj.topic,
        message: JSON.stringify(messageObj.toJSON()),
        ipfs_api_url: DEFAULT_IPFS_API
      })

      // 保存到内存'''

new_ipfs_publish = '''await invoke('ipfs_pubsub_publish', {
        topic: messageObj.topic,
        message: JSON.stringify(messageObj.toJSON()),
        ipfs_api_url: DEFAULT_IPFS_API
      })

      this.log(LogLevel.INFO, 'IPFS 发布成功，准备保存消息:', { messageId: messageObj.id })

      // IPFS 发布成功后才保存
      // 保存到内存'''

content = content.replace(old_ipfs_publish, new_ipfs_publish)
print('✓ 修复 sendMessage 方法 - 先发送 IPFS 成功后才保存')

# 修改错误日志
content = content.replace(
    "this.log(LogLevel.ERROR, '发送消息失败，加入重试队列:', {",
    "this.log(LogLevel.ERROR, '发送消息失败，不保存到本地:', {"
)
print('✓ 更新错误日志描述')

# 保存文件
with open(GROUP_CHAT_SERVICE, 'w', encoding='utf-8') as f:
    f.write(content)
print('✓ 文件已保存：localIpfsGroupChatService.ts\n')

# ==================== 修复问题 3: agentCoordinatorService.ts ====================
print('处理文件：agentCoordinatorService.ts')

with open(COORDINATOR_SERVICE, 'r', encoding='utf-8') as f:
    content = f.read()

# 问题 3: 修改 sendTaskAssignment 方法
old_send_task = '''  private async sendTaskAssignment(task: TaskInfo, agent: AgentInfo): Promise<void> {
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
  }'''

new_send_task = '''  private async sendTaskAssignment(task: TaskInfo, agent: AgentInfo): Promise<void> {
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
  }'''

content = content.replace(old_send_task, new_send_task)
print('✓ 修复 sendTaskAssignment 方法 - 任务分配后更新状态')

# 保存文件
with open(COORDINATOR_SERVICE, 'w', encoding='utf-8') as f:
    f.write(content)
print('✓ 文件已保存：agentCoordinatorService.ts\n')

print('====================')
print('所有 P0 问题已修复完成！')
print('====================')
