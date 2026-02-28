const fs = require('fs');

const filePath = '/Users/apple/Downloads/alou/alou-desktop/src/services/localIpfsGroupChatService.ts';
let content = fs.readFileSync(filePath, 'utf-8');

// 问题 1: 替换 joinGroup 方法
const oldJoinGroup = `  async joinGroup(groupId: string, topic?: string | null): Promise<LocalGroup> {
    this.log(LogLevel.INFO, '加入群聊:', { groupId })

    if (!this.localIdentity) {
      throw new Error('未设置本地身份，无法加入群聊')
    }

    const group = this.groups.get(groupId)
    if (!group) {
      throw new Error('群聊不存在')
    }

    const groupTopic = topic || group.topic

    try {
      // 订阅群聊消息
      await this.subscribeToGroup(groupId, groupTopic)

      // 如果用户不在群聊成员列表中，添加进去
      if (!group.members.includes(this.localIdentity.did)) {
        group.members.push(this.localIdentity.did)
        await this.saveGroupToKV(group)
      }

      // 发送加入消息
      const joinMessage = new LocalGroupMessage({
        groupId,
        topic: groupTopic,
        from: this.localIdentity.did,
        fromName: this.localIdentity.name || this.localIdentity.did,
        content: \`\${this.localIdentity.did} 加入了群聊\`,
        type: MessageTypes.JOIN,
        metadata: {
          type: 'member_joined',
          member: this.localIdentity.did
        }
      })

      await this.sendMessage(joinMessage)

      this.log(LogLevel.INFO, '加入群聊成功:', { groupId, member: this.localIdentity.did })
      return group

    } catch (error: any) {
      this.log(LogLevel.ERROR, '加入群聊失败:', { groupId, error: error.message })
      throw new Error(\`加入群聊失败：\${error.message}\`)
    }
  }`;

const newJoinGroup = `  async joinGroup(groupId: string, topic?: string | null): Promise<LocalGroup> {
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
    }

    const groupTopic = topic || group.topic

    try {
      // 订阅群聊消息
      await this.subscribeToGroup(groupId, groupTopic)

      // 如果用户不在群聊成员列表中，添加进去
      if (!group.members.includes(this.localIdentity.did)) {
        group.members.push(this.localIdentity.did)
        await this.saveGroupToKV(group)
      }

      // 发送加入消息
      const joinMessage = new LocalGroupMessage({
        groupId,
        topic: groupTopic,
        from: this.localIdentity.did,
        fromName: this.localIdentity.name || this.localIdentity.did,
        content: \`\${this.localIdentity.did} 加入了群聊\`,
        type: MessageTypes.JOIN,
        metadata: {
          type: 'member_joined',
          member: this.localIdentity.did
        }
      })

      await this.sendMessage(joinMessage)

      this.log(LogLevel.INFO, '加入群聊成功:', { groupId, member: this.localIdentity.did })
      return group

    } catch (error: any) {
      this.log(LogLevel.ERROR, '加入群聊失败:', { groupId, error: error.message })
      throw new Error(\`加入群聊失败：\${error.message}\`)
    }
  }`;

// 问题 2: 替换 sendMessage 方法中的逻辑
const oldSendMessage = `    try {
      // 发布到 IPFS PubSub
      await invoke('ipfs_pubsub_publish', {
        topic: messageObj.topic,
        message: JSON.stringify(messageObj.toJSON()),
        ipfs_api_url: DEFAULT_IPFS_API
      })

      // 保存到内存
      this.saveMessageToMemory(messageObj.groupId, messageObj)

      // 保存到 KV 存储
      await this.saveMessageToKV(messageObj.groupId, messageObj)

      // 保存到 LocalStorage 备份
      this.saveMessageToLocalStorage(messageObj.groupId, messageObj)

      // 记录消息 ID
      this.recordMessageId(messageObj.groupId, messageObj)

      // 标记为已发送
      messageObj.delivered = true

      // 如果需要等待确认
      if (waitForAck) {
        await this.waitForAck(messageObj.id)
      }

      this.log(LogLevel.INFO, '消息发送成功:', { messageId: messageObj.id })
      return true

    } catch (error: any) {
      this.log(LogLevel.ERROR, '发送消息失败，加入重试队列:', {
        messageId: messageObj.id,
        error: error.message
      })

      // 加入重试队列
      this.addToRetryQueue(messageObj)

      throw new Error(\`发送消息失败：\${error.message}\`)
    }`;

const newSendMessage = `    try {
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
      await this.saveMessageToKV(messageObj.groupId, messageObj)

      // 保存到 LocalStorage 备份
      this.saveMessageToLocalStorage(messageObj.groupId, messageObj)

      // 记录消息 ID
      this.recordMessageId(messageObj.groupId, messageObj)

      // 标记为已发送
      messageObj.delivered = true

      // 如果需要等待确认
      if (waitForAck) {
        await this.waitForAck(messageObj.id)
      }

      this.log(LogLevel.INFO, '消息发送成功:', { messageId: messageObj.id })
      return true

    } catch (error: any) {
      this.log(LogLevel.ERROR, '发送消息失败，不保存到本地:', {
        messageId: messageObj.id,
        error: error.message
      })

      // 加入重试队列
      this.addToRetryQueue(messageObj)

      throw new Error(\`发送消息失败：\${error.message}\`)
    }`;

// 执行替换
let replaced = false;
if (content.includes(oldJoinGroup)) {
    content = content.replace(oldJoinGroup, newJoinGroup);
    replaced = true;
    console.log('joinGroup method replaced');
} else {
    console.log('WARNING: oldJoinGroup not found');
}

if (content.includes(oldSendMessage)) {
    content = content.replace(oldSendMessage, newSendMessage);
    replaced = true;
    console.log('sendMessage method replaced');
} else {
    console.log('WARNING: oldSendMessage not found');
}

if (replaced) {
    fs.writeFileSync(filePath, content, 'utf-8');
    console.log('File updated successfully');
} else {
    console.log('No replacements made');
}
