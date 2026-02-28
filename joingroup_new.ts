  /**
   * 加入群聊
   * @param groupId - 群聊 ID
   * @param topic - 群聊主题（可选）
   * @returns 加入的群聊信息
   */
  async joinGroup(groupId: string, topic?: string | null): Promise<LocalGroup> {
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
        content: `${this.localIdentity.did} 加入了群聊`,
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
      throw new Error(`加入群聊失败：${error.message}`)
    }
  }

