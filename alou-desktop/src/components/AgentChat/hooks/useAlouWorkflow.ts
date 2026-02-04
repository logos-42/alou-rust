/**
 * useAlouWorkflow - Alou工作流Hook
 * 
 * 提供三种工作流模式：
 * 1. Interactive - 交互式模式，逐步引导 AI 完成任务
 * 2. Auto - 自动模式，自动分析并执行任务
 * 3. Parallel - 并行模式，拆分任务并行处理
 * 
 * 不依赖外部 CLI，直接使用现有的 AgentChat 工具调用
 */

import { useState, useCallback, useEffect, useRef } from 'react';
import { WorkflowService, WorkflowMode } from '@/services/workflowService';
import { generateInteractiveSystemPrompt, generateAutoSystemPrompt, generateParallelSystemPrompt } from '../utils/workflowPrompts';
import { analyzeTaskComplexity } from '../utils/workflowUtils';

interface WorkflowOptions {
  mode?: WorkflowMode;
  autoAnalyze?: boolean;
  maxSteps?: number;
  timeout?: number;
}

interface WorkflowProgress {
  currentStep: number;
  totalSteps: number;
  stepName: string;
  status: 'pending' | 'running' | 'completed' | 'failed';
  message?: string;
}

interface Message {
  id: string;
  role: 'user' | 'assistant' | 'system' | 'tool';
  content: string;
  timestamp: number;
  metadata?: Record<string, any>;
}

interface ToolCall {
  id: string;
  type: string;
  function: {
    name: string;
    arguments: string;
  };
}

interface UseAlouWorkflowParams {
  sendMessage: (message: string, options?: any) => Promise<any>;
  handleToolCalls: (toolCalls: ToolCall[], options?: any) => Promise<void>;
  appendMessage: (message: string | Message) => void;
  options?: WorkflowOptions;
}

export function useAlouWorkflow({
  sendMessage,
  handleToolCalls,
  appendMessage,
  options: _options = {}
}: UseAlouWorkflowParams) {
  const [isInitialized, setIsInitialized] = useState(false);
  const [currentMode, setCurrentMode] = useState(WorkflowMode.SMART);
  const [isProcessing, setIsProcessing] = useState(false);
  const [progress, setProgress] = useState<WorkflowProgress | null>(null);
  
  // 使用 ref 存储工作流服务，避免重复创建
  const workflowServiceRef = useRef<WorkflowService | null>(null);

  // 初始化工作流服务
  useEffect(() => {
    if (!workflowServiceRef.current) {
      workflowServiceRef.current = new WorkflowService();
      setIsInitialized(true);
    }
  }, []);

  // 切换工作流模式
  const switchMode = useCallback(async (mode: WorkflowMode) => {
    if (!workflowServiceRef.current) return;

    try {
      setIsProcessing(true);
      await workflowServiceRef.current.switchMode(mode);
      setCurrentMode(mode);
      
      const modeNames = {
        [WorkflowMode.INTERACTIVE]: '交互式',
        [WorkflowMode.AUTO]: '自动',
        [WorkflowMode.PARALLEL]: '并行',
        [WorkflowMode.SMART]: '智能',
      };
      
      appendMessage(`已切换到${modeNames[mode]}模式`);
    } catch (error) {
      console.error('[useAlouWorkflow] 切换模式失败:', error);
      appendMessage(`切换模式失败: ${(error as Error).message}`);
    } finally {
      setIsProcessing(false);
    }
  }, [appendMessage]);

  // 处理用户消息
  const processMessage = useCallback(async (message: string) => {
    if (!workflowServiceRef.current || !isInitialized) {
      return;
    }

    try {
      setIsProcessing(true);
      setProgress({
        currentStep: 0,
        totalSteps: 1,
        stepName: '分析任务',
        status: 'running',
      });

      // 分析任务复杂度
      const complexity = await analyzeTaskComplexity(message);
      
      // 根据复杂度和当前模式选择最佳工作流
      let targetMode = currentMode;
      if (currentMode === WorkflowMode.SMART) {
        targetMode = complexity.recommendedMode as WorkflowMode;
      }

      // 生成系统提示
      let systemPrompt: string;
      switch (targetMode) {
        case WorkflowMode.INTERACTIVE:
          systemPrompt = generateInteractiveSystemPrompt();
          break;
        case WorkflowMode.AUTO:
          systemPrompt = generateAutoSystemPrompt();
          break;
        case WorkflowMode.PARALLEL:
          systemPrompt = generateParallelSystemPrompt();
          break;
        default:
          systemPrompt = generateAutoSystemPrompt();
      }

      // 发送消息给 AI
      const response = await sendMessage(message, {
        systemPrompt,
        mode: 'alou',
        workflowMode: targetMode,
      });

      // 处理工具调用
      if (response.toolCalls && response.toolCalls.length > 0) {
        setProgress({
          currentStep: 1,
          totalSteps: response.toolCalls.length,
          stepName: '执行工具调用',
          status: 'running',
        });

        await handleToolCalls(response.toolCalls, {
          onProgress: (step: number, total: number, stepName: string) => {
            setProgress({
              currentStep: step,
              totalSteps: total,
              stepName,
              status: 'running',
            });
          },
        });
      }

      setProgress({
        currentStep: 1,
        totalSteps: 1,
        stepName: '完成',
        status: 'completed',
      });

    } catch (error) {
      console.error('[useAlouWorkflow] 处理消息失败:', error);
      
      setProgress({
        currentStep: 0,
        totalSteps: 1,
        stepName: '错误',
        status: 'failed',
        message: (error as Error).message,
      });

      appendMessage(`处理失败: ${(error as Error).message}`);
    } finally {
      setIsProcessing(false);
      setTimeout(() => setProgress(null), 2000);
    }
  }, [currentMode, isInitialized, sendMessage, handleToolCalls, appendMessage]);

  // 获取工作流统计信息
  const getWorkflowStats = useCallback(() => {
    if (!workflowServiceRef.current) return null;

    return workflowServiceRef.current.getStats();
  }, []);

  // 重置工作流状态
  const resetWorkflow = useCallback(() => {
    if (!workflowServiceRef.current) return;

    workflowServiceRef.current.reset();
    setProgress(null);
    setIsProcessing(false);
  }, []);

  // 获取当前模式信息
  const getModeInfo = useCallback(() => {
    const modeDescriptions = {
      [WorkflowMode.INTERACTIVE]: '交互式模式 - 逐步引导完成任务',
      [WorkflowMode.AUTO]: '自动模式 - 自动分析并执行任务',
      [WorkflowMode.PARALLEL]: '并行模式 - 拆分任务并行处理',
      [WorkflowMode.SMART]: '智能模式 - 根据任务复杂度自动选择最佳模式',
    };

    return {
      currentMode,
      description: modeDescriptions[currentMode],
      isProcessing,
      progress,
    };
  }, [currentMode, isProcessing, progress]);

  return {
    // 状态
    isInitialized,
    currentMode,
    isProcessing,
    progress,
    
    // 方法
    switchMode,
    processMessage,
    getWorkflowStats,
    resetWorkflow,
    getModeInfo,
  };
}
