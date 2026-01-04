import React, { useState, useEffect, useCallback } from 'react';
import { useI18n } from '../hooks/useI18n';
import skillsService from '../services/skillsService';
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
  const [skillExamples, setSkillExamples] = useState([]);

  // 加载技能列表
  const loadSkills = useCallback(async () => {
    try {
      setIsLoading(true);
      const allSkills = skillsService.getAllSkills();
      setSkills(allSkills);

      // 加载技能示例
      const examples = skillsService.createSkillExamples();
      setSkillExamples(examples);

      onSkillEvent?.({
        type: 'skills_loaded',
        count: allSkills.length,
        skills: allSkills.map(s => s.name)
      });
    } catch (error) {
      console.error('[SkillsManager] 加载技能列表失败:', error);
      onSkillEvent?.({
        type: 'error',
        message: '加载技能列表失败',
        error: error.message
      });
    } finally {
      setIsLoading(false);
    }
  }, [onSkillEvent]);

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
  }, [sessionId, executingSkill, onSkillEvent]);

  // 发现新技能
  const discoverSkills = async () => {
    try {
      setIsLoading(true);
      const result = await skillsService.discoverSkills();
      
      if (result.success) {
        setSkills(skillsService.getAllSkills());
        onSkillEvent?.({
          type: 'skills_discovered',
          discovered: result.discovered,
          total: result.total
        });
      }
    } catch (error) {
      console.error('[SkillsManager] 发现技能失败:', error);
      onSkillEvent?.({
        type: 'discovery_error',
        error: error.message
      });
    } finally {
      setIsLoading(false);
    }
  };

  // 处理参数输入变化
  const handleParameterChange = (skillName, action, paramName, value) => {
    setSkillParameters(prev => ({
      ...prev,
      [`${skillName}.${action}`]: {
        ...prev[`${skillName}.${action}`],
        [paramName]: value
      }
    }));
  };

  // 获取参数值
  const getParameterValue = (skillName, action, paramName) => {
    return skillParameters[`${skillName}.${action}`]?.[paramName] || '';
  };

  // 渲染参数输入表单
  const renderParameterForm = (skillName, action, parametersDef) => {
    if (!parametersDef || !parametersDef.properties) {
      return <div className="no-parameters">此操作无需参数</div>;
    }

    const properties = parametersDef.properties;
    const required = parametersDef.required || [];

    return Object.entries(properties).map(([paramName, paramDef]) => (
      <div key={paramName} className="parameter-field">
        <label className="parameter-label">
          {paramName}
          {required.includes(paramName) && <span className="required">*</span>}
          {paramDef.description && (
            <span className="parameter-description">{paramDef.description}</span>
          )}
        </label>
        
        {paramDef.type === 'string' && (
          <input
            type="text"
            value={getParameterValue(skillName, action, paramName)}
            onChange={(e) => handleParameterChange(skillName, action, paramName, e.target.value)}
            className="parameter-input"
            placeholder={`输入 ${paramName}`}
          />
        )}
        
        {paramDef.type === 'number' && (
          <input
            type="number"
            value={getParameterValue(skillName, action, paramName)}
            onChange={(e) => handleParameterChange(skillName, action, paramName, e.target.value)}
            className="parameter-input"
            placeholder={`输入 ${paramName}`}
          />
        )}
        
        {paramDef.type === 'boolean' && (
          <label className="parameter-checkbox">
            <input
              type="checkbox"
              checked={getParameterValue(skillName, action, paramName) || false}
              onChange={(e) => handleParameterChange(skillName, action, paramName, e.target.checked)}
            />
            {paramDef.description || paramName}
          </label>
        )}
        
        {paramDef.type === 'array' && (
          <textarea
            value={getParameterValue(skillName, action, paramName) || ''}
            onChange={(e) => handleParameterChange(skillName, action, paramName, e.target.value)}
            className="parameter-textarea"
            placeholder={`输入 ${paramName}，每行一个项目`}
            rows={3}
          />
        )}
        
        {paramDef.type === 'object' && (
          <textarea
            value={JSON.stringify(getParameterValue(skillName, action, paramName) || {}, null, 2)}
            onChange={(e) => {
              try {
                const value = JSON.parse(e.target.value);
                handleParameterChange(skillName, action, paramName, value);
              } catch (error) {
                // 保持原始文本，等待有效JSON
              }
            }}
            className="parameter-textarea"
            placeholder={`输入 ${paramName} 的JSON对象`}
            rows={4}
          />
        )}
      </div>
    ));
  };

  // 获取执行结果
  const getExecutionResult = (skillName, action) => {
    return executionResults[`${skillName}.${action}`];
  };

  // 获取技能统计
  const getSkillStats = () => {
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
  };

  // 使用示例
  const useExample = (example) => {
    setSelectedSkill(example.skill);
    setSelectedAction(example.action);
    
    // 设置参数
    setSkillParameters(prev => ({
      ...prev,
      [`${example.skill}.${example.action}`]: example.parameters
    }));
  };

  // 初始化加载
  useEffect(() => {
    loadSkills();
  }, [loadSkills]);

  return (
    <div 
      className={`skills-manager ${className}`}
      data-theme={isDarkMode ? "dark" : "light"}
    >
    

      <div className="skills-header">
        <div className="skills-header-top">
          <h3>技能管理 </h3>
         
        </div>
        <div className="skills-actions">
          <button
            onClick={loadSkills}
            disabled={isLoading}
            className="btn-refresh"
            title="刷新技能列表"
          >
            {isLoading ? '刷新中...' : '🔄 刷新'}
          </button>
          <button
            onClick={discoverSkills}
            disabled={isLoading}
            className="btn-discover"
            title="发现新技能"
          >
            🔍 发现技能
          </button>
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
            <h4>可用技能 ({skills.length})</h4>
            {skills.length === 0 ? (
              <div className="no-skills">
                <p>暂无可用技能</p>
                <p>点击"发现技能"查找可用技能</p>
              </div>
            ) : (
              skills.map(skill => (
                <div
                  key={skill.name}
                  className={`skill-item ${selectedSkill === skill.name ? 'selected' : ''}`}
                  onClick={() => {
                    setSelectedSkill(skill.name);
                    setSelectedAction(null);
                  }}
                >
                  <div className="skill-icon">
                    {skill.category === 'automation' && '⚙️'}
                    {skill.category === 'development' && '💻'}
                    {skill.category === 'research' && '🔍'}
                    {skill.category === 'web3' && '⛓️'}
                    {!skill.category && '🔧'}
                  </div>
                  <div className="skill-info">
                    <div className="skill-name">{skill.name}</div>
                    <div className="skill-description">{skill.description}</div>
                    <div className="skill-meta">
                      <span className="skill-version">v{skill.version}</span>
                      <span className="skill-category">{skill.category}</span>
                    </div>
                  </div>
                </div>
              ))
            )}
          </div>

          {skillExamples.length > 0 && (
            <div className="skills-examples">
              <h4>使用示例</h4>
              {skillExamples.map((example, index) => (
                <div
                  key={index}
                  className="example-item"
                  onClick={() => useExample(example)}
                  title={`使用示例: ${example.description}`}
                >
                  <div className="example-icon">📋</div>
                  <div className="example-info">
                    <div className="example-title">{example.description}</div>
                    <div className="example-details">
                      {example.skill}.{example.action}
                    </div>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>

        <div className="skills-main">
          {selectedSkill ? (
            <div className="skill-details">
              <div className="skill-details-header">
                <h4>
                  {selectedSkill}
                  <span className="skill-category-badge">
                    {skills.find(s => s.name === selectedSkill)?.category || '未分类'}
                  </span>
                </h4>
                <button
                  onClick={() => {
                    setSelectedSkill(null);
                    setSelectedAction(null);
                  }}
                  className="btn-close"
                >
                  ✕
                </button>
              </div>

              <div className="skill-description-full">
                {skills.find(s => s.name === selectedSkill)?.description}
              </div>

              <div className="skill-actions">
                <h5>可用操作</h5>
                {Object.entries(skills.find(s => s.name === selectedSkill)?.actions || {}).map(([actionName, actionDef]) => (
                  <div
                    key={actionName}
                    className={`skill-action ${selectedAction === actionName ? 'selected' : ''}`}
                    onClick={() => setSelectedAction(actionName)}
                  >
                    <div className="action-header">
                      <div className="action-name">{actionName}</div>
                      <div className="action-status">
                        {executingSkill === `${selectedSkill}.${actionName}` && (
                          <span className="executing">执行中...</span>
                        )}
                        {getExecutionResult(selectedSkill, actionName)?.success && (
                          <span className="success">✅ 成功</span>
                        )}
                        {getExecutionResult(selectedSkill, actionName)?.success === false && (
                          <span className="error">❌ 失败</span>
                        )}
                      </div>
                    </div>
                    <div className="action-description">{actionDef.description}</div>
                  </div>
                ))}
              </div>

              {selectedAction && (
                <div className="skill-execution">
                  <h5>执行操作: {selectedAction}</h5>
                  
                  <div className="execution-form">
                    {renderParameterForm(
                      selectedSkill,
                      selectedAction,
                      skills.find(s => s.name === selectedSkill)?.actions[selectedAction]?.parameters
                    )}
                  </div>

                  <div className="execution-actions">
                    <button
                      onClick={async () => {
                        const params = skillParameters[`${selectedSkill}.${selectedAction}`] || {};
                        await executeSkill(selectedSkill, selectedAction, params);
                      }}
                      disabled={executingSkill === `${selectedSkill}.${selectedAction}`}
                      className="btn-execute"
                    >
                      {executingSkill === `${selectedSkill}.${selectedAction}` ? '执行中...' : '执行'}
                    </button>
                    
                    <button
                      onClick={() => {
                        setSkillParameters(prev => {
                          const newParams = { ...prev };
                          delete newParams[`${selectedSkill}.${selectedAction}`];
                          return newParams;
                        });
                      }}
                      className="btn-clear"
                    >
                      清除参数
                    </button>
                  </div>

                  {getExecutionResult(selectedSkill, selectedAction) && (
                    <div className="execution-result">
                      <h6>执行结果</h6>
                      <pre className="result-content">
                        {JSON.stringify(getExecutionResult(selectedSkill, selectedAction), null, 2)}
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
    </div>
  );
};

export default SkillsManager;
