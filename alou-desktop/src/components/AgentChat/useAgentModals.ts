import { useState, useCallback } from 'react'

/**
 * Hook参数类型
 */
interface UseAgentModalsParams {
  recordInteraction: (action: string, data?: unknown, label?: string) => void;
}

/**
 * Hook返回类型
 */
interface UseAgentModalsReturn {
  isCreateAgentModalOpen: boolean;
  isImportAgentModalOpen: boolean;
  showDetailPanel: boolean;
  createChannel: () => void;
  importChannel: () => void;
  closeCreateAgentModal: () => void;
  closeImportAgentModal: () => void;
  openDetailPanel: () => void;
  closeDetailPanel: () => void;
}

/**
 * Hook for managing agent-related modal states
 * 管理智能体相关的模态框状态
 * 
 * @param recordInteraction - 记录交互的函数
 * @returns 模态框相关的状态和方法
 */
export const useAgentModals = ({ recordInteraction }: UseAgentModalsParams): UseAgentModalsReturn => {
  const [isCreateAgentModalOpen, setCreateAgentModalOpen] = useState<boolean>(false)
  const [isImportAgentModalOpen, setImportAgentModalOpen] = useState<boolean>(false)
  const [showDetailPanel, setShowDetailPanel] = useState<boolean>(false)
  
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

export default useAgentModals
