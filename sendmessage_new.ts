    try {
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

      throw new Error(`发送消息失败：${error.message}`)
    }
  }

