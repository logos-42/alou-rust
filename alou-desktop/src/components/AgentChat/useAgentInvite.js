import { useState, useCallback } from 'react'
import clusterActionService from '@/services/clusterActionService'
import useClusterActionStore from '@/stores/clusterActionStore'

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

  // 处理邀请提交 - 创建群聊并显示
  const handleInviteSubmit = useCallback(async (channel, agents, source) => {
    try {
      if (!channel || !agents || agents.length === 0) {
        return
      }
      
      recordInteraction('invite_agents', {
        channelId: channel.id,
        agentCount: agents.length,
        source,
      })

      // 创建群聊集群行动
      const { addAction, setActiveAction } = useClusterActionStore.getState()
      
      // 构建群聊描述
      const agentNames = agents.map(a => a.display_name || a.name || 'Agent').join(', ')
      const groupDescription = `群聊: ${channel.name} + ${agentNames}`

      // 静默创建群聊（不输出日志）

      // 内联的本地群聊创建函数
      const createLocalGroupChatInline = () => {
        const actionId = `local_group_${Date.now()}`
        
        const localAction = {
          action_id: actionId,
          description: groupDescription,
          status: 'Active',
          created_at: new Date().toISOString(),
          agents: [
            {
              id: channel.meta?.ipns || channel.meta?.cid || channel.meta?.did || channel.id,
              name: channel.name,
            },
            ...agents.map(a => ({
              id: a.ipns || a.cid || a.did || a.id,
              name: a.display_name || a.name,
            })),
          ],
          metadata: {
            type: 'group_chat',
            channel_id: channel.id,
            channel_name: channel.name,
            local: true,
          },
        }
        
        addAction(localAction)
        setActiveAction(actionId)
        
        // 触发群聊显示事件
        window.dispatchEvent(
          new CustomEvent('cluster-action-created', {
            detail: { actionId },
          }),
        )
        
        return actionId
      }

      let actionId = null

      try {
        // 尝试调用后端创建集群行动
        const userId = typeof window !== 'undefined' ? localStorage.getItem('user_id') || 'user' : 'user'
        const createResult = await clusterActionService.createClusterAction(
          groupDescription,
          userId,
          { 
            type: 'group_chat',
            channel_id: channel.id,
            channel_name: channel.name,
            invited_agents: agents.map(a => ({
              id: a.ipns || a.cid || a.did || a.id,
              name: a.display_name || a.name,
            })),
          },
        )

        if (createResult?.action) {
          const action = createResult.action
          actionId = action.action_id
          addAction(action)
          setActiveAction(actionId)
          
          // 创建集群行动后立即执行，以创建 pubsub topic
          try {
            const walletAddress =
              typeof window !== 'undefined' ? localStorage.getItem('wallet_address') : null
            const chain =
              typeof window !== 'undefined' ? localStorage.getItem('wallet_chain_id') : null
            
            // 执行集群行动（这会创建 pubsub topic）
            await clusterActionService.executeClusterAction(
              actionId,
              walletAddress,
              chain || undefined,
            )
          } catch (executeError) {
            // 执行失败时静默处理，不影响群聊创建
            console.warn('[useAgentInvite] 执行集群行动失败（不影响群聊创建）:', executeError)
          }
          
          // 触发群聊显示事件
          window.dispatchEvent(
            new CustomEvent('cluster-action-created', {
              detail: { actionId },
            }),
          )
        } else {
          // 后端创建失败，使用本地模拟（静默处理）
          actionId = createLocalGroupChatInline()
        }
      } catch (apiError) {
        // API 调用失败（通常是 404），静默使用本地模拟
        actionId = createLocalGroupChatInline()
      }
      
      // 触发邀请成功事件
      if (typeof window !== 'undefined') {
        window.dispatchEvent(
          new CustomEvent('agents-invited', {
            detail: { channel, agents, source, actionId },
          }),
        )
      }
    } catch (error) {
      // 静默处理错误
      throw error
    }
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

