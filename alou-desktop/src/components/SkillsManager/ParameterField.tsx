import React, { memo } from 'react';

/**
 * ParameterField - 参数输入字段组件
 * 使用 React.memo 优化性能，避免不必要的重渲染
 */
const ParameterField = memo(({ 
  skillName, 
  action, 
  paramName, 
  paramDef, 
  required, 
  value, 
  onChange 
}) => {
  const handleChange = (e) => {
    const newValue = paramDef.type === 'boolean' ? e.target.checked : e.target.value;
    onChange(skillName, action, paramName, newValue);
  };

  const handleObjectChange = (e) => {
    try {
      const parsedValue = JSON.parse(e.target.value);
      onChange(skillName, action, paramName, parsedValue);
    } catch (error) {
      // 保持原始文本，等待有效JSON
    }
  };

  const renderInput = () => {
    switch (paramDef.type) {
      case 'string':
        return (
          <input
            type="text"
            value={value || ''}
            onChange={handleChange}
            className="parameter-input"
            placeholder={`输入 ${paramName}`}
          />
        );
      
      case 'number':
        return (
          <input
            type="number"
            value={value || ''}
            onChange={handleChange}
            className="parameter-input"
            placeholder={`输入 ${paramName}`}
          />
        );
      
      case 'boolean':
        return (
          <label className="parameter-checkbox">
            <input
              type="checkbox"
              checked={value || false}
              onChange={handleChange}
            />
            {paramDef.description || paramName}
          </label>
        );
      
      case 'array':
        return (
          <textarea
            value={value || ''}
            onChange={handleChange}
            className="parameter-textarea"
            placeholder={`输入 ${paramName}，每行一个项目`}
            rows={3}
          />
        );
      
      case 'object':
        return (
          <textarea
            value={JSON.stringify(value || {}, null, 2)}
            onChange={handleObjectChange}
            className="parameter-textarea"
            placeholder={`输入 ${paramName} 的JSON对象`}
            rows={4}
          />
        );
      
      default:
        return (
          <input
            type="text"
            value={value || ''}
            onChange={handleChange}
            className="parameter-input"
            placeholder={`输入 ${paramName}`}
          />
        );
    }
  };

  return (
    <div className="parameter-field">
      <label className="parameter-label">
        {paramName}
        {required && <span className="required">*</span>}
        {paramDef.description && (
          <span className="parameter-description">{paramDef.description}</span>
        )}
      </label>
      {renderInput()}
    </div>
  );
});

ParameterField.displayName = 'ParameterField';

export default ParameterField;
