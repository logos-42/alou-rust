import React, { memo, useState } from 'react';

/**
 * ErrorBoundary - React 错误边界组件
 * 捕获并显示组件树中的错误
 */
class ErrorBoundary extends React.Component {
  constructor(props) {
    super(props);
    this.state = { hasError: false, error: null, errorInfo: null };
  }

  static getDerivedStateFromError(error) {
    return { hasError: true };
  }

  componentDidCatch(error, errorInfo) {
    this.setState({
      error: error,
      errorInfo: errorInfo
    });
    
    console.error('[SkillsManager ErrorBoundary] 捕获到错误:', error, errorInfo);
  }

  render() {
    if (this.state.hasError) {
      return (
        <ErrorDisplay 
          error={this.state.error}
          errorInfo={this.state.errorInfo}
          onRetry={() => {
            this.setState({ hasError: false, error: null, errorInfo: null });
          }}
          isDarkMode={this.props.isDarkMode}
        />
      );
    }

    return this.props.children;
  }
}

/**
 * ErrorDisplay - 错误显示组件
 */
const ErrorDisplay = memo(({ error, errorInfo, onRetry, isDarkMode }) => {
  const [showDetails, setShowDetails] = useState(false);

  const getErrorMessage = (error) => {
    if (!error) return '未知错误';
    
    if (error.name === 'ChunkLoadError') {
      return '加载资源失败，请刷新页面重试';
    }
    
    if (error.message.includes('Network Error')) {
      return '网络连接失败，请检查网络连接';
    }
    
    if (error.message.includes('timeout')) {
      return '请求超时，请稍后重试';
    }
    
    return error.message || '操作失败';
  };

  const getErrorSuggestion = (error) => {
    if (!error) return '请稍后重试或联系技术支持';
    
    if (error.name === 'ChunkLoadError') {
      return '建议刷新页面或清除浏览器缓存';
    }
    
    if (error.message.includes('Network Error')) {
      return '请检查网络连接，确保可以访问服务器';
    }
    
    if (error.message.includes('timeout')) {
      return '请检查网络连接或稍后重试';
    }
    
    return '请检查输入参数或稍后重试';
  };

  return (
    <div className={`error-display ${isDarkMode ? 'dark' : ''}`}>
      <div className="error-icon">⚠️</div>
      <div className="error-content">
        <h3>出现错误</h3>
        <p className="error-message">{getErrorMessage(error)}</p>
        <p className="error-suggestion">{getErrorSuggestion(error)}</p>
        
        <div className="error-actions">
          <button onClick={onRetry} className="btn-retry">
            🔄 重试
          </button>
          <button 
            onClick={() => setShowDetails(!showDetails)} 
            className="btn-details"
          >
            {showDetails ? '隐藏详情' : '显示详情'}
          </button>
        </div>
        
        {showDetails && (
          <div className="error-details">
            <div className="error-stack">
              <strong>错误信息:</strong>
              <pre>{error?.toString()}</pre>
            </div>
            {errorInfo && (
              <div className="error-stack">
                <strong>组件堆栈:</strong>
                <pre>{errorInfo.componentStack}</pre>
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
});

ErrorDisplay.displayName = 'ErrorDisplay';

/**
 * ErrorMessage - 错误消息组件
 * 用于显示操作错误
 */
const ErrorMessage = memo(({ error, onRetry, onDismiss, isDarkMode }) => {
  if (!error) return null;

  const getErrorType = (error) => {
    if (error.type === 'validation') return '验证错误';
    if (error.type === 'network') return '网络错误';
    if (error.type === 'execution') return '执行错误';
    return '错误';
  };

  const getErrorIcon = (error) => {
    if (error.type === 'validation') return '⚠️';
    if (error.type === 'network') return '🌐';
    if (error.type === 'execution') return '❌';
    return '❌';
  };

  return (
    <div className={`error-message ${isDarkMode ? 'dark' : ''}`}>
      <div className="error-header">
        <span className="error-icon">{getErrorIcon(error)}</span>
        <span className="error-type">{getErrorType(error)}</span>
        {onDismiss && (
          <button onClick={onDismiss} className="btn-dismiss" aria-label="关闭错误">
            ✕
          </button>
        )}
      </div>
      <div className="error-body">
        <p className="error-text">{error.message || '操作失败'}</p>
        {error.suggestion && (
          <p className="error-suggestion">{error.suggestion}</p>
        )}
      </div>
      {onRetry && (
        <div className="error-footer">
          <button onClick={onRetry} className="btn-retry-small">
            🔄 重试
          </button>
        </div>
      )}
    </div>
  );
});

ErrorMessage.displayName = 'ErrorMessage';

export { ErrorBoundary, ErrorDisplay, ErrorMessage };
