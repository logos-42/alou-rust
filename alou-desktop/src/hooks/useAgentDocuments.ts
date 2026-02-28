/**
 * Agent文档管理Hook
 * 用于初始化和管理agent的文档
 */

import { useEffect } from 'react';
import { agentDocumentService } from '@/services/agentDocumentService';

export function useAgentDocuments() {
  useEffect(() => {
    // 初始化文档服务
    agentDocumentService.initialize();

    // 清理
    return () => {
      agentDocumentService.cleanup();
    };
  }, []);

  /**
   * 初始化agent的文档
   */
  const initializeDocuments = async (agentId: string, agentInfo?: any) => {
    await agentDocumentService.initializeAgentDocuments(agentId, agentInfo);
  };

  /**
   * 读取agent文档
   */
  const getDocument = async (agentId: string, documentType: string) => {
    return await agentDocumentService.getAgentDocument(agentId, documentType);
  };

  /**
   * 读取所有文档
   */
  const getAllDocuments = async (agentId: string) => {
    return await agentDocumentService.getAgentDocuments(agentId);
  };

  /**
   * 更新文档
   */
  const updateDocument = async (agentId: string, documentType: string, content: string) => {
    return await agentDocumentService.updateAgentDocument(agentId, documentType, content);
  };

  /**
   * 获取文档绝对路径
   */
  const getDocumentPath = async (agentId: string, documentType: string) => {
    return await agentDocumentService.getDocumentAbsolutePath(agentId, documentType);
  };

  return {
    initializeDocuments,
    getDocument,
    getAllDocuments,
    updateDocument,
    getDocumentPath,
  };
}
