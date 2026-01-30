import React, { useState, useEffect, useCallback, useMemo, memo } from 'react';
import { useI18n } from '../hooks/useI18n';
import skillsService from '../services/skillsService';
import agentSkillsStorage from '../services/agentSkillsStorage';
import skillGenerator from '../services/skillGenerator';
import ParameterField from './SkillsManager/ParameterField';
import SkillItem from './SkillsManager/SkillItem';
import SkillAction from './SkillsManager/SkillAction';
import { ErrorBoundary, ErrorMessage } from './SkillsManager/ErrorComponents';
import './SkillsManager.css';

/**
 * SkillsManager - 技能管理组件
 * 显示和管理所有可用的Skills，支持技能调用和执行
 */
const SkillsManager = ({
  sessionId,
  agentInfo = {},
  onSkillEvent,
  className = '',
  isDarkMode = false
}) => {
  const { t } = useI18n();
  const [skills, setSkills] = useState([]);
  const [selectedSkill, setSelectedSkill] = useState(null);
  const [selectedAction, setSelectedAction] = useState(null);
  const [isLoading, setIsLoading] = useState(false);
  const [executingSkill, setExecutingSkill] = useState(null);
  const [executionResults, setExecutionResults] = useState({});
  const [skillParameters, setSkillParameters] = useState({});
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedCategory, setSelectedCategory] = useState('all');
  const [showFilters, setShowFilters] = useState(false);
  const [errors, setErrors] = useState([]);
  const [agentSkillsConfig, setAgentSkillsConfig] = useState(null);
  const [showAIGenerator, setShowAIGenerator] = useState(false);
  const [aiGenerating, setAiGenerating] = useState(false);
  const [aiPrompt, setAiPrompt] = useState('');
  const [customSkills, setCustomSkills] = useState([]);

  // 使用 useCallback 优化错误处理
  const addError = useCallback((error) => {
    const errorWithId = {
      ...error,
      id: Date.now() + Math.random()
    };
    setErrors(prev => [...prev, errorWithId]);
    
    // 自动移除错误消息（5秒后）
    setTimeout(() => {
      setErrors(prev => prev.filter(e => e.id !== errorWithId.id));
    }, 5000);
  }, []);

  const removeError = useCallback((errorId) => {
    setErrors(prev => prev.filter(e => e.id !== errorId));
  }, []);

  const clearAllErrors = useCallback(() => {
    setErrors([]);
  }, []);

  // 加载技能列表
  const loadSkills = useCallback(async () => {
    try {
      setIsLoading(true);
      console.log('[SkillsManager] 开始加载技能...');
      
      // 使用 Agent Skills 系统加载技能
      await skillsService.initializeSkills();
      const allSkills = skillsService.getAllSkills();
      setSkills(allSkills);
      
      console.log(`[SkillsManager] 成功加载 ${allSkills.length} 个技能`);
      
      // 获取系统状态
      const systemStatus = await skillsService.getSystemStatus();
      console.log('[SkillsManager] Agent Skills 系统状态:', systemStatus);
      
      if (systemStatus.agent_skills_available) {
        addError({
          type: 'success',
          message: 'Agent Skills 系统已连接',
          suggestion: `发现 ${systemStatus.total_skills || 0} 个技能`
        });
      } else {
        addError({
          type: 'warning',
          message: 'Agent Skills 系统不可用',
          suggestion: '请检查后端服务或技能目录'
        });
      }

      onSkillEvent?.({
        type: 'skills_loaded',
        count: allSkills.length,
        skills: allSkills.map(s => s.name),
        agentSkillsAvailable: systemStatus.agent_skills_available
      });
    } catch (error) {
      console.error('[SkillsManager] 加载技能列表失败:', error);
      addError({
        type: 'network',
        message: '加载技能列表失败',
        error: error.message,
        suggestion: '请检查网络连接或稍后重试'
      });
      onSkillEvent?.({
        type: 'error',
        message: '加载技能列表失败',
        error: error.message
      });
    } finally {
      setIsLoading(false);
    }
  }, [onSkillEvent, addError]);

  // 执行技能
  const executeSkill = useCallback(async (skillName, action, parameters) => {
    if (!sessionId || executingSkill) return;

    try {
      setExecutingSkill(`${skillName}.${action}`);
      
      onSkillEvent?.({
        type: 'execution_started',
        skill: skillName,
        action,
        parameters
      });

      const result = await skillsService.executeSkill(skillName, action, parameters);
      
      // 保存执行结果
      setExecutionResults(prev => ({
        ...prev,
        [`${skillName}.${action}`]: result
      }));

      if (result.success) {
        onSkillEvent?.({
          type: 'execution_completed',
          skill: skillName,
          action,
          result: result.result,
          timestamp: result.timestamp
        });
      } else {
        onSkillEvent?.({
          type: 'execution_error',
          skill: skillName,
          action,
          error: result.error,
          timestamp: result.timestamp
        });
      }

      return result;
    } catch (error) {
      console.error('[SkillsManager] 执行技能异常:', error);
      const errorType = error.message.includes('验证失败') ? 'validation' : 'execution';
      addError({
        type: errorType,
        message: `技能执行失败: ${error.message}`,
        error: error.message,
        suggestion: errorType === 'validation' ? '请检查输入参数' : '请稍后重试或联系技术支持'
      });
      onSkillEvent?.({
        type: 'execution_error',
        skill: skillName,
        action,
        error: error.message
      });
      return {
        success: false,
        error: error.message
      };
    } finally {
      setExecutingSkill(null);
    }
  }, [sessionId, executingSkill, onSkillEvent, addError]);

  // 初始化加载智能体配置
  useEffect(() => {
    if (agentInfo?.id) {
      const config = agentSkillsStorage.getAgentSkillsConfig(agentInfo.id);
      setAgentSkillsConfig(config);
      setCustomSkills(agentSkillsStorage.getCustomSkills(agentInfo.id));
    }
  }, [agentInfo?.id]);

  // 使用 useMemo 缓存筛选后的技能列表（包含自定义技能）
  const filteredSkills = useMemo(() => {
    let allSkills = [...skills, ...customSkills];
    
    // 根据智能体配置过滤技能
    if (agentSkillsConfig && agentInfo?.id) {
      allSkills = allSkills.filter(skill => 
        agentSkillsStorage.isSkillEnabled(agentInfo.id, skill.name)
      );
    }
    
    // 按类别筛选
    if (selectedCategory !== 'all') {
      allSkills = allSkills.filter(skill => skill.category === selectedCategory);
    }
    
    // 按搜索关键词筛选
    if (searchQuery.trim()) {
      const query = searchQuery.toLowerCase();
      allSkills = allSkills.filter(skill => 
        skill.name.toLowerCase().includes(query) ||
        skill.description.toLowerCase().includes(query) ||
        (skill.category && skill.category.toLowerCase().includes(query))
      );
    }
    
    return allSkills;
  }, [skills, customSkills, agentSkillsConfig, agentInfo?.id, selectedCategory, searchQuery]);

  // 使用 useMemo 缓存可用类别列表
  const availableCategories = useMemo(() => {
    const categories = new Set(skills.map(skill => skill.category || 'uncategorized'));
    return ['all', ...Array.from(categories).sort()];
  }, [skills]);

  // 使用 useMemo 缓存技能统计信息
  const skillStats = useMemo(() => {
    const stats = skillsService.getStats();
    // 计算总操作数
    let totalActions = 0;
    skills.forEach(skill => {
      if (skill.actions) {
        totalActions += Object.keys(skill.actions).length;
      }
    });
    
    return {
      ...stats,
      totalActions
    };
  }, [skills]);

  // 使用 useMemo 缓存当前选中的技能定义
  const currentSkill = useMemo(() => 
    skills.find(s => s.name === selectedSkill), 
    [skills, selectedSkill]
  );

  // 使用 useMemo 缓存当前操作的参数定义
  const currentActionParams = useMemo(() => 
    currentSkill?.actions[selectedAction]?.parameters,
    [currentSkill, selectedAction]
  );

  // 使用 useMemo 缓存参数值
  const currentParameters = useMemo(() => 
    skillParameters[`${selectedSkill}.${selectedAction}`] || {},
    [skillParameters, selectedSkill, selectedAction]
  );

  // 使用 useMemo 缓存执行结果
  const currentExecutionResult = useMemo(() => {
    const result = executionResults[`${selectedSkill}.${selectedAction}`];
    return result;
  }, [executionResults, selectedSkill, selectedAction]);

  // 使用 useMemo 缓存当前执行状态
  const isCurrentExecuting = useMemo(() => 
    executingSkill === `${selectedSkill}.${selectedAction}`,
    [executingSkill, selectedSkill, selectedAction]
  );

  // AI 生成技能
  const handleAIGenerateSkill = useCallback(async () => {
    if (!aiPrompt.trim() || !agentInfo?.id) return;
    
    setAiGenerating(true);
    try {
      const result = await skillGenerator.generateSkill(aiPrompt, agentInfo, {
        sessionId,
        existingSkills: filteredSkills
      });
      
      if (result.success) {
        // 验证技能定义
        const validation = skillGenerator.validateSkillDefinition(result.skill);
        
        if (validation.isValid) {
          // 保存自定义技能
          agentSkillsStorage.addCustomSkill(agentInfo.id, result.skill);
          setCustomSkills(prev => [...prev, result.skill]);
          
          addError({
            type: 'success',
            message: 'AI 技能生成成功',
            suggestion: `已生成技能: ${result.skill.name}`
          });
          
          setShowAIGenerator(false);
          setAiPrompt('');
        } else {
          addError({
            type: 'validation',
            message: '生成的技能定义有误',
            suggestion: `错误: ${validation.errors.join(', ')}`
          });
        }
      } else {
        addError({
          type: 'execution',
          message: 'AI 技能生成失败',
          error: result.error,
          suggestion: result.suggestion
        });
      }
    } catch (error) {
      addError({
        type: 'execution',
        message: 'AI 技能生成异常',
        error: error.message,
        suggestion: '请稍后重试'
      });
    } finally {
      setAiGenerating(false);
    }
  }, [aiPrompt, agentInfo, sessionId, filteredSkills, addError]);

  // 切换技能启用状态
  const toggleSkillEnabled = useCallback((skillName) => {
    if (!agentInfo?.id) return;
    
    if (agentSkillsStorage.isSkillEnabled(agentInfo.id, skillName)) {
      agentSkillsStorage.disableSkill(agentInfo.id, skillName);
    } else {
      agentSkillsStorage.enableSkill(agentInfo.id, skillName);
    }
    
    // 更新配置状态
    const updatedConfig = agentSkillsStorage.getAgentSkillsConfig(agentInfo.id);
    setAgentSkillsConfig(updatedConfig);
  }, [agentInfo?.id]);

  // 删除自定义技能
  const deleteCustomSkill = useCallback((skillName) => {
    if (!agentInfo?.id) return;
    
    agentSkillsStorage.removeCustomSkill(agentInfo.id, skillName);
    setCustomSkills(prev => prev.filter(skill => skill.name !== skillName));
    
    addError({
      type: 'success',
      message: '自定义技能已删除',
      suggestion: `已删除技能: ${skillName}`
    });
  }, [agentInfo?.id, addError]);

  // 使用 useCallback 优化搜索和筛选处理
  const handleSearchChange = useCallback((value) => {
    setSearchQuery(value);
  }, []);

  const handleCategoryChange = useCallback((category) => {
    setSelectedCategory(category);
  }, []);

  const toggleFilters = useCallback(() => {
    setShowFilters(prev => !prev);
  }, []);

  const clearFilters = useCallback(() => {
    setSearchQuery('');
    setSelectedCategory('all');
  }, []);

  // 使用 useCallback 优化事件处理函数
  const handleSkillSelect = useCallback((skillName) => {
    setSelectedSkill(skillName);
    setSelectedAction(null);
  }, []);

  const handleActionSelect = useCallback((actionName) => {
    setSelectedAction(actionName);
  }, []);

  const handleCloseSkill = useCallback(() => {
    setSelectedSkill(null);
    setSelectedAction(null);
  }, []);

  const handleParameterChange = useCallback((skillName, action, paramName, value) => {
    setSkillParameters(prev => ({
      ...prev,
      [`${skillName}.${action}`]: {
        ...prev[`${skillName}.${action}`],
        [paramName]: value
      }
    }));
  }, []);

  const handleClearParameters = useCallback(() => {
    setSkillParameters(prev => {
      const newParams = { ...prev };
      delete newParams[`${selectedSkill}.${selectedAction}`];
      return newParams;
    });
  }, [selectedSkill, selectedAction]);

  const handleExecuteSkill = useCallback(async () => {
    const params = skillParameters[`${selectedSkill}.${selectedAction}`] || {};
    await executeSkill(selectedSkill, selectedAction, params);
  }, [selectedSkill, selectedAction, skillParameters, executeSkill]);

  // 渲染参数输入表单
  const renderParameterForm = useCallback((skillName, action, parametersDef) => {
    if (!parametersDef || !parametersDef.properties) {
      return <div className="no-parameters">此操作无需参数</div>;
    }

    const properties = parametersDef.properties;
    const required = parametersDef.required || [];

    return Object.entries(properties).map(([paramName, paramDef]) => (
      <ParameterField
        key={paramName}
        skillName={skillName}
        action={action}
        paramName={paramName}
        paramDef={paramDef}
        required={required.includes(paramName)}
        value={skillParameters[`${skillName}.${action}`]?.[paramName] || ''}
        onChange={handleParameterChange}
      />
    ));
  }, [skillParameters, handleParameterChange]);

  // 获取参数值
  const getParameterValue = (skillName, action, paramName) => {
    return skillParameters[`${skillName}.${action}`]?.[paramName] || '';
  };

  // 获取执行结果
  const getExecutionResult = (skillName, action) => {
    return executionResults[`${skillName}.${action}`];
  };

  // 初始化加载
  useEffect(() => {
    loadSkills();
  }, [loadSkills]);

  return (
    <ErrorBoundary isDarkMode={isDarkMode}>
      <div 
        className={`skills-manager ${className}`}
        data-theme={isDarkMode ? "dark" : "light"}
      >
        {/* 错误消息列表 */}
        {errors.length > 0 && (
          <div className="errors-container">
            {errors.map(error => (
              <ErrorMessage
                key={error.id}
                error={error}
                onRetry={() => {
                  removeError(error.id);
                  if (error.type === 'network') {
                    loadSkills();
                  }
                }}
                onDismiss={() => removeError(error.id)}
                isDarkMode={isDarkMode}
              />
            ))}
          </div>
        )}
    
        <div className="skills-header">
          <div className="skills-header-top">
            <h3>技能管理 </h3>
            <div className="skills-stats">
              <div className="stats-item">
                总计: {skills.length} 技能
              </div>
              {filteredSkills.length !== skills.length && (
                <div className="stats-item filtered">
                  显示: {filteredSkills.length}
                </div>
              )}
            </div>
          </div>
          <div className="skills-actions">
            <button
              onClick={toggleFilters}
              className={`btn-filters ${showFilters ? 'active' : ''}`}
              title="筛选技能"
            >
              🔧 筛选
            </button>
            <button
              onClick={() => setShowAIGenerator(true)}
              className="btn-ai-generate"
              title="AI 生成技能"
            >
              🤖 AI 生成
            </button>
          </div>
          
          {/* 搜索和筛选区域 */}
          <div className={`skills-filters ${showFilters ? 'show' : ''}`}>
            <div className="search-container">
              <input
                type="text"
                value={searchQuery}
                onChange={(e) => handleSearchChange(e.target.value)}
                className="search-input"
                placeholder="搜索技能名称、描述或类别..."
                aria-label="搜索技能"
              />
              {searchQuery && (
                <button
                  onClick={() => handleSearchChange('')}
                  className="btn-clear-search"
                  aria-label="清除搜索"
                >
                  ✕
                </button>
              )}
            </div>
            
            <div className="category-filters">
              <label className="filter-label">类别:</label>
              <div className="category-buttons">
                {availableCategories.map(category => (
                  <button
                    key={category}
                    onClick={() => handleCategoryChange(category)}
                    className={`category-btn ${selectedCategory === category ? 'active' : ''}`}
                    aria-label={`筛选类别: ${category}`}
                    aria-pressed={selectedCategory === category}
                  >
                    {category === 'all' ? '全部' : category}
                  </button>
                ))}
              </div>
            </div>
            
            {(searchQuery || selectedCategory !== 'all') && (
              <button
                onClick={clearFilters}
                className="btn-clear-filters"
              >
                清除筛选
              </button>
            )}
          </div>
        </div>

        {isLoading && (
          <div className="skills-loading">
            <div className="spinner"></div>
            加载中...
          </div>
        )}

        <div className="skills-content">
          <div className="skills-sidebar">
            <div className="skills-list">
              <h4>可用技能 ({filteredSkills.length})</h4>
              {filteredSkills.length === 0 ? (
                <div className="no-skills">
                  {skills.length === 0 ? (
                    <>
                      <p>暂无可用技能</p>
                      <p>点击"发现技能"查找可用技能</p>
                    </>
                  ) : (
                    <>
                      <p>没有找到匹配的技能</p>
                      <p>尝试调整搜索条件或筛选器</p>
                    </>
                  )}
                </div>
              ) : (
                filteredSkills.map(skill => (
                  <SkillItem
                    key={skill.name}
                    skill={skill}
                    isSelected={selectedSkill === skill.name}
                    onSelect={handleSkillSelect}
                  />
                ))
              )}
            </div>
          </div>

          <div className="skills-main">
            {selectedSkill ? (
              <div className="skill-details">
                <div className="skill-details-header">
                  <h4>
                    {selectedSkill}
                    <span className="skill-category-badge">
                      {currentSkill?.category || '未分类'}
                    </span>
                  </h4>
                  <button
                    onClick={handleCloseSkill}
                    className="btn-close"
                  >
                    ✕
                  </button>
                </div>

                <div className="skill-description-full">
                  {currentSkill?.description}
                </div>

                <div className="skill-actions">
                  <h5>可用操作</h5>
                  {Object.entries(currentSkill?.actions || {}).map(([actionName, actionDef]) => (
                    <SkillAction
                      key={actionName}
                      skillName={selectedSkill}
                      actionName={actionName}
                      actionDef={actionDef}
                      isSelected={selectedAction === actionName}
                      isExecuting={executingSkill === `${selectedSkill}.${actionName}`}
                      executionResult={getExecutionResult(selectedSkill, actionName)}
                      onSelect={handleActionSelect}
                    />
                  ))}
                </div>

                {selectedAction && (
                  <div className="skill-execution">
                    <h5>执行操作: {selectedAction}</h5>
                    
                    <div className="execution-form">
                      {renderParameterForm(
                        selectedSkill,
                        selectedAction,
                        currentActionParams
                      )}
                    </div>

                    <div className="execution-actions">
                      <button
                        onClick={handleExecuteSkill}
                        disabled={isCurrentExecuting}
                        className="btn-execute"
                      >
                        {isCurrentExecuting ? '执行中...' : '执行'}
                      </button>
                      
                      <button
                        onClick={handleClearParameters}
                        className="btn-clear"
                      >
                        清除参数
                      </button>
                    </div>

                    {currentExecutionResult && (
                      <div className="execution-result">
                        <h6>执行结果</h6>
                        <pre className="result-content">
                          {JSON.stringify(currentExecutionResult, null, 2)}
                        </pre>
                      </div>
                    )}
                  </div>
                )}
              </div>
            ) : (
              <div className="skills-welcome">
                <div className="welcome-icon">🎯</div>
                <h4>欢迎使用技能管理</h4>
                <p>从左侧选择技能开始使用</p>
                <p>技能是Claude可以调用的预定义功能模块</p>
                
                <div className="welcome-features">
                  <div className="feature">
                    <div className="feature-icon">⚙️</div>
                    <div className="feature-text">
                      <strong>自动化工作流</strong>
                      <p>创建和执行多步骤自动化任务</p>
                    </div>
                  </div>
                  <div className="feature">
                    <div className="feature-icon">🔧</div>
                    <div className="feature-text">
                      <strong>工具集成</strong>
                      <p>集成各种工具和服务的功能</p>
                    </div>
                  </div>
                  <div className="feature">
                    <div className="feature-icon">🚀</div>
                    <div className="feature-text">
                      <strong>一键执行</strong>
                      <p>通过简单界面快速执行复杂操作</p>
                    </div>
                  </div>
                </div>
              </div>
            )}
          </div>
        </div>
        
        {/* AI 生成技能面板 */}
        {showAIGenerator && (
          <div className="ai-generator-panel">
            <div className="ai-generator-header">
              <h4>🤖 AI 技能生成器</h4>
              <button
                onClick={() => setShowAIGenerator(false)}
                className="btn-close"
              >
                ✕
              </button>
            </div>
            <div className="ai-generator-content">
              <textarea
                value={aiPrompt}
                onChange={(e) => setAiPrompt(e.target.value)}
                placeholder="描述你想要创建的技能功能...\n\n例如：\n- 创建一个自动整理文件的工具\n- 生成代码审查报告\n- 分析网页内容并提取关键信息"
                className="ai-prompt-input"
                rows={6}
              />
              <div className="ai-generator-actions">
                <button
                  onClick={handleAIGenerateSkill}
                  disabled={aiGenerating || !aiPrompt.trim()}
                  className="btn-generate"
                >
                  {aiGenerating ? '生成中...' : '🚀 生成技能'}
                </button>
                <button
                  onClick={() => {
                    setShowAIGenerator(false);
                    setAiPrompt('');
                  }}
                  className="btn-cancel"
                >
                  取消
                </button>
              </div>
            </div>
          </div>
        )}
      </div>
    </ErrorBoundary>
  );
};

export default SkillsManager;
