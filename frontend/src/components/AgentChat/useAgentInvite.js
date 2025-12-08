import { useState, useCallback } from 'react'

/**
 * 智能体邀请功能 Hook
 * 管理邀请模态框状态和相关操作
 */
export const useAgentInvite = ({
  deleteChannel,
  resolveExistingAgentTarget,
  recordInteraction,
}) => {
  const [isInviteModalOpen, setInviteModalOpen] = useState(false)
  const [inviteTargetChannel, setInviteTargetChannel] = useState(null)

  // 打开邀请模态框
  const handleInviteToChannel = useCallback((channel) => {
    setInviteTargetChannel(channel)
    setInviteModalOpen(true)
    recordInteraction('open_invite_modal', { channelId: channel.id, name: channel.name })
  }, [recordInteraction])

  // 关闭邀请模态框
  const closeInviteModal = useCallback(() => {
    setInviteModalOpen(false)
    setInviteTargetChannel(null)
  }, [])

  // 删除频道
  const handleDeleteChannel = useCallback((channel) => {
    deleteChannel(channel)
  }, [deleteChannel])

  // 处理邀请提交
  const handleInviteSubmit = useCallback(async (channel, agents, source) => {
    // TODO: 实现群组邀请逻辑
    console.log('[useAgentInvite] 邀请智能体到群组:', { channel, agents, source })
    recordInteraction('invite_agents', {
      channelId: channel.id,
      agentCount: agents.length,
      source,
    })
    // 群组功能的完整实现需要后端支持
  }, [recordInteraction])

  return {
    // 状态
    isInviteModalOpen,
    inviteTargetChannel,
    
    // 操作
    handleInviteToChannel,
    closeInviteModal,
    handleDeleteChannel,
    handleInviteSubmit,
    resolveExistingAgentTarget,
  }
}

