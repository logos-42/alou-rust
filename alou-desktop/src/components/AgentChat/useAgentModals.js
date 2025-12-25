import { useState, useCallback } from 'react'

/**
 * Hook for managing agent-related modal states
 * 管理智能体相关的模态框状态
 * 
 * @param {Function} recordInteraction - 记录交互的函数
 * @returns {Object} 模态框相关的状态和方法
 */
export const useAgentModals = ({ recordInteraction }) => {
  const [isCreateAgentModalOpen, setCreateAgentModalOpen] = useState(false)
  const [isImportAgentModalOpen, setImportAgentModalOpen] = useState(false)
  const [showDetailPanel, setShowDetailPanel] = useState(false)
  
  // 创建频道/智能体
  const createChannel = useCallback(() => {
    setCreateAgentModalOpen(true)
    recordInteraction('open_create_agent_modal')
  }, [recordInteraction])
  
  // 导入频道/智能体
  const importChannel = useCallback(() => {
    setImportAgentModalOpen(true)
    recordInteraction('open_import_agent_modal')
  }, [recordInteraction])
  
  // 关闭创建模态框
  const closeCreateAgentModal = useCallback(() => {
    setCreateAgentModalOpen(false)
  }, [])
  
  // 关闭导入模态框
  const closeImportAgentModal = useCallback(() => {
    setImportAgentModalOpen(false)
  }, [])
  
  // 打开详情面板
  const openDetailPanel = useCallback(() => {
    setShowDetailPanel(true)
  }, [])
  
  // 关闭详情面板
  const closeDetailPanel = useCallback(() => {
    setShowDetailPanel(false)
  }, [])
  
  return {
    // 状态
    isCreateAgentModalOpen,
    isImportAgentModalOpen,
    showDetailPanel,
    
    // 方法
    createChannel,
    importChannel,
    closeCreateAgentModal,
    closeImportAgentModal,
    openDetailPanel,
    closeDetailPanel,
  }
}

