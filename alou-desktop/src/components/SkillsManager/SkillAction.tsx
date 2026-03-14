import React, { memo } from 'react';

/**
 * SkillAction - 技能操作项组件
 * 使用 React.memo 优化性能，避免不必要的重渲染
 */
const SkillAction = memo(({ 
  skillName, 
  actionName, 
  actionDef, 
  isSelected, 
  isExecuting, 
  executionResult, 
  onSelect 
}) => {
  const handleClick = () => {
    onSelect(actionName);
  };

  const getStatusIndicator = () => {
    if (isExecuting) {
      return <span className="executing">执行中...</span>;
    }
    
    if (executionResult?.success) {
      return <span className="success">✅ 成功</span>;
    }
    
    if (executionResult?.success === false) {
      return <span className="error">❌ 失败</span>;
    }
    
    return null;
  };

  return (
    <div
      className={`skill-action ${isSelected ? 'selected' : ''}`}
      onClick={handleClick}
      role="button"
      tabIndex={0}
      onKeyDown={(e) => {
        if (e.key === 'Enter' || e.key === ' ') {
          e.preventDefault();
          handleClick();
        }
      }}
      aria-pressed={isSelected}
      aria-label={`选择操作: ${actionName}`}
    >
      <div className="action-header">
        <div className="action-name">{actionName}</div>
        <div className="action-status">
          {getStatusIndicator()}
        </div>
      </div>
      <div className="action-description">{actionDef.description}</div>
    </div>
  );
});

SkillAction.displayName = 'SkillAction';

export default SkillAction;
