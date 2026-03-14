import React, { memo } from 'react';

/**
 * SkillItem - 技能列表项组件
 * 使用 React.memo 优化性能，避免不必要的重渲染
 */
const SkillItem = memo(({ skill, isSelected, onSelect }) => {
  const handleClick = () => {
    onSelect(skill.name);
  };

  const getSkillIcon = () => {
    switch (skill.category) {
      case 'automation':
        return '⚙️';
      case 'development':
        return '💻';
      case 'research':
        return '🔍';
      case 'web3':
        return '⛓️';
      default:
        return '🔧';
    }
  };

  return (
    <div
      className={`skill-item ${isSelected ? 'selected' : ''}`}
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
      aria-label={`选择技能: ${skill.name}`}
    >
      <div className="skill-icon">
        {getSkillIcon()}
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
  );
});

SkillItem.displayName = 'SkillItem';

export default SkillItem;
