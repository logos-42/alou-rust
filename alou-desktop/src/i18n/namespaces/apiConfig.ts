/**
 * API 配置相关翻译
 * 命名规范：apiConfig.模块.具体含义
 */
export const apiConfig = {
  zh: {
    // ==================== API 配置 ====================
    'apiConfig.title': '大模型 API 配置',
    'apiConfig.subtitle': '配置您的大模型 API Key 和模型参数',

    // API Key
    'apiConfig.apiKey.label': 'API Key',
    'apiConfig.apiKey.placeholder': '输入您的 API Key',
    'apiConfig.apiKey.show': '显示',
    'apiConfig.apiKey.hide': '隐藏',
    'apiConfig.apiKey.warning': 'API Key 将存储在本地浏览器中，请注意安全',

    // Provider
    'apiConfig.provider.label': '提供商 (Provider)',

    // Model
    'apiConfig.model.label': '模型 (Model)',
    'apiConfig.model.placeholder': '输入模型名称',
    'apiConfig.model.default': '默认值',
    'apiConfig.model.notSet': '未设置',

    // 按钮
    'apiConfig.cancel': '取消',
    'apiConfig.testConnection': '测试连接',
    'apiConfig.save': '保存',
    'apiConfig.verifying': '验证中...',
    'apiConfig.saving': '保存中...',

    // 错误消息
    'apiConfig.error.apiKeyRequired': '请输入 API Key',
    'apiConfig.error.modelRequired': '请输入模型名称',
    'apiConfig.error.saveFailed': '保存配置失败',
    'apiConfig.error.verifyFailed': 'API Key 验证失败',
    'apiConfig.error.networkError': '验证失败，请检查网络连接',

    // 成功消息
    'apiConfig.success.saved': '配置已保存',
    'apiConfig.success.verified': 'API Key 验证成功',
    'apiConfig.success.verifyNotEnabled': '验证功能暂未启用（配置已保存到本地）',

    // 提示信息
    'apiConfig.info.connectWallet': '连接钱包以获得更好的体验',
    'apiConfig.info.connectWalletDetail': '连接钱包后，您的 API 配置将安全地保存到云端，并可在不同设备间同步',
  },
  en: {
    // ==================== API Configuration ====================
    'apiConfig.title': 'LLM API Configuration',
    'apiConfig.subtitle': 'Configure your LLM API Key and model parameters',

    // API Key
    'apiConfig.apiKey.label': 'API Key',
    'apiConfig.apiKey.placeholder': 'Enter your API Key',
    'apiConfig.apiKey.show': 'Show',
    'apiConfig.apiKey.hide': 'Hide',
    'apiConfig.apiKey.warning': 'API Key will be stored locally in your browser, please keep it secure',

    // Provider
    'apiConfig.provider.label': 'Provider',

    // Model
    'apiConfig.model.label': 'Model',
    'apiConfig.model.placeholder': 'Enter model name',
    'apiConfig.model.default': 'Default',
    'apiConfig.model.notSet': 'Not set',

    // Buttons
    'apiConfig.cancel': 'Cancel',
    'apiConfig.testConnection': 'Test Connection',
    'apiConfig.save': 'Save',
    'apiConfig.verifying': 'Verifying...',
    'apiConfig.saving': 'Saving...',

    // Error messages
    'apiConfig.error.apiKeyRequired': 'Please enter API Key',
    'apiConfig.error.modelRequired': 'Please enter model name',
    'apiConfig.error.saveFailed': 'Failed to save configuration',
    'apiConfig.error.verifyFailed': 'API Key verification failed',
    'apiConfig.error.networkError': 'Verification failed, please check your network connection',

    // Success messages
    'apiConfig.success.saved': 'Configuration saved',
    'apiConfig.success.verified': 'API Key verified successfully',
    'apiConfig.success.verifyNotEnabled': 'Verification feature not enabled (configuration saved locally)',

    // Info messages
    'apiConfig.info.connectWallet': 'Connect wallet for better experience',
    'apiConfig.info.connectWalletDetail': 'After connecting your wallet, your API configuration will be securely saved to the cloud and synchronized across devices',
  },
}
