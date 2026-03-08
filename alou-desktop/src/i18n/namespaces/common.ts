/**
 * 通用翻译 - 在多个页面/组件中共用的文案
 * 命名规范：common.模块.具体含义
 */
export const common = {
  zh: {
    // 应用基础
    'common.appTitle': 'Alou智能助手',
    'common.appName': 'Alou',

    // 通用操作
    'common.confirm': '确认',
    'common.cancel': '取消',
    'common.save': '保存',
    'common.delete': '删除',
    'common.edit': '编辑',
    'common.close': '关闭',
    'common.back': '返回',
    'common.next': '下一步',
    'common.previous': '上一步',
    'common.submit': '提交',
    'common.reset': '重置',
    'common.refresh': '刷新',
    'common.search': '搜索',
    'common.filter': '筛选',
    'common.sort': '排序',
    'common.more': '更多',
    'common.less': '收起',
    'common.expand': '展开',
    'common.collapse': '收起',
    'common.copy': '复制',
    'common.copied': '已复制',
    'common.loading': '加载中...',
    'common.retry': '重试',
    'common.remove': '移除',
    'common.add': '添加',
    'common.create': '创建',
    'common.update': '更新',
    'common.import': '导入',
    'common.export': '导出',

    // 通用状态
    'common.status.online': '在线',
    'common.status.offline': '离线',
    'common.status.connecting': '连接中...',
    'common.status.connected': '已连接',
    'common.status.disconnected': '未连接',
    'common.status.error': '错误',
    'common.status.success': '成功',
    'common.status.pending': '等待中',
    'common.status.processing': '处理中',

    // 通用提示
    'common.error': '抱歉，发生了错误',
    'common.networkError': '请检查网络连接或稍后重试',
    'common.unknownError': '未知错误',
    'common.noData': '暂无数据',
    'common.empty': '空',

    // 时间相关
    'common.time.justNow': '刚刚',
    'common.time.minutesAgo': '分钟前',
    'common.time.hoursAgo': '小时前',
    'common.time.daysAgo': '天前',
    'common.time.today': '今天',
    'common.time.yesterday': '昨天',

    // 导航
    'common.nav.home': '首页',
    'common.nav.settings': '设置',
    'common.nav.help': '帮助',
    'common.nav.about': '关于',

    // 主题
    'common.theme.dark': '深色模式',
    'common.theme.light': '浅色模式',
    'common.theme.auto': '跟随系统',
    'common.theme.title': '主题',
    'common.theme.dayNight': '白天/黑夜模式',
    'common.theme.day': '白天',
    'common.theme.night': '黑夜',
    'common.theme.switchToDay': '切换到白天模式',
    'common.theme.switchToNight': '切换到黑夜模式',

    // 设置面板
    'common.settings.title': '设置',
    'common.settings.apiConfig.title': '大模型 API 配置',
    'common.settings.apiConfig.label': '配置 API 密钥和模型',
    'common.settings.apiConfig.open': '打开 API 配置',
    'common.settings.background.title': '聊天背景',
    'common.settings.background.label': '背景图片',
    'common.settings.background.select': '选择图片',
    'common.settings.background.remove': '移除背景',
    'common.settings.background.preview': '背景预览',

    // 语言
    'common.language': '语言',
    'common.language.zh': '中文',
    'common.language.en': 'English',

    // Bot Gateway 设置
    'common.gateway.title': 'Bot Gateway',
    'common.gateway.subtitle': '配置即时通讯工具 Bot，远程调用 Alou 工具',
    'common.gateway.enabled': '已启用',
    'common.gateway.disabled': '已禁用',
    'common.gateway.status': '状态',
    'common.gateway.running': '运行中',
    'common.gateway.stopped': '已停止',
    'common.gateway.port': '监听端口',
    'common.gateway.platforms': '启用平台',
    'common.gateway.none': '无',
    'common.gateway.configure': '配置',
    'common.gateway.testConnection': '测试连接',
    'common.gateway.save': '保存配置',
    'common.gateway.saving': '保存中...',
    'common.gateway.startService': '启动服务',
    'common.gateway.stopService': '停止服务',

    // 平台名称
    'common.gateway.platform.telegram': 'Telegram',
    'common.gateway.platform.feishu': '飞书',
    'common.gateway.platform.discord': 'Discord',
    'common.gateway.platform.qq': 'QQ',

    // Telegram 配置
    'common.gateway.telegram.enabled': '启用 Telegram Bot',
    'common.gateway.telegram.botToken': 'Bot Token',
    'common.gateway.telegram.tokenPlaceholder': '123456789:ABCdefGHIjklMNOpqrsTUVwxyz',
    'common.gateway.telegram.usePolling': '使用轮询模式 (无需 Webhook)',
    'common.gateway.telegram.allowedUserIds': '允许的用户 IDs',
    'common.gateway.telegram.allowedChatIds': '允许的聊天 IDs',
    'common.gateway.telegram.hint': '提示：从 BotFather 获取 Bot Token',

    // 飞书配置
    'common.gateway.feishu.enabled': '启用飞书 Bot',
    'common.gateway.feishu.appId': 'App ID',
    'common.gateway.feishu.appSecret': 'App Secret',
    'common.gateway.feishu.verifyToken': '验证 Token',
    'common.gateway.feishu.encryptKey': '加密 Key (可选)',
    'common.gateway.feishu.appIdPlaceholder': 'cli_a1b2c3d4e5f6g7h8',
    'common.gateway.feishu.hint': '提示：在飞书开放平台创建应用获取配置',

    // Discord 配置
    'common.gateway.discord.enabled': '启用 Discord Bot',
    'common.gateway.discord.botToken': 'Bot Token',
    'common.gateway.discord.tokenPlaceholder': 'Your Discord Bot Token',
    'common.gateway.discord.allowedUserIds': '允许的用户 IDs',
    'common.gateway.discord.allowedGuildIds': '允许的服务器 IDs',
    'common.gateway.discord.allowedChannelIds': '允许的频道 IDs',
    'common.gateway.discord.hint': '提示：在 Discord Developer Portal 创建应用获取 Bot Token',

    // QQ 配置
    'common.gateway.qq.enabled': '启用 QQ Bot (OneBot)',
    'common.gateway.qq.wsUrl': 'WebSocket URL',
    'common.gateway.qq.wsUrlPlaceholder': 'ws://127.0.0.1:8080',
    'common.gateway.qq.accessToken': 'Access Token',
    'common.gateway.qq.allowedUserIds': '允许的用户 IDs',
    'common.gateway.qq.allowedGroupIds': '允许的群 IDs',
    'common.gateway.qq.hint': '提示：需要运行 OneBot 兼容的 QQ 机器人框架（如 go-cqhttp）',

    // 通用配置
    'common.gateway.general.enabled': '启用 Bot Gateway',
    'common.gateway.general.commandPrefix': '命令前缀',
    'common.gateway.general.publicDomain': '公网域名 (可选)',

    // 日志
    'common.gateway.logs.title': '日志',
    'common.gateway.logs.noLogs': '暂无日志',

    // 成功消息
    'common.gateway.success.saved': '配置已保存',
  },
  en: {
    // App basics
    'common.appTitle': 'Alou AI Assistant',
    'common.appName': 'Alou',

    // Common actions
    'common.confirm': 'Confirm',
    'common.cancel': 'Cancel',
    'common.save': 'Save',
    'common.delete': 'Delete',
    'common.edit': 'Edit',
    'common.close': 'Close',
    'common.back': 'Back',
    'common.next': 'Next',
    'common.previous': 'Previous',
    'common.submit': 'Submit',
    'common.reset': 'Reset',
    'common.refresh': 'Refresh',
    'common.search': 'Search',
    'common.filter': 'Filter',
    'common.sort': 'Sort',
    'common.more': 'More',
    'common.less': 'Less',
    'common.expand': 'Expand',
    'common.collapse': 'Collapse',
    'common.copy': 'Copy',
    'common.copied': 'Copied',
    'common.loading': 'Loading...',
    'common.retry': 'Retry',
    'common.remove': 'Remove',
    'common.add': 'Add',
    'common.create': 'Create',
    'common.update': 'Update',
    'common.import': 'Import',
    'common.export': 'Export',

    // Common status
    'common.status.online': 'Online',
    'common.status.offline': 'Offline',
    'common.status.connecting': 'Connecting...',
    'common.status.connected': 'Connected',
    'common.status.disconnected': 'Disconnected',
    'common.status.error': 'Error',
    'common.status.success': 'Success',
    'common.status.pending': 'Pending',
    'common.status.processing': 'Processing',

    // Common messages
    'common.error': 'Sorry, an error occurred',
    'common.networkError': 'Please check your network connection or try again later',
    'common.unknownError': 'Unknown error',
    'common.noData': 'No data',
    'common.empty': 'Empty',

    // Time related
    'common.time.justNow': 'Just now',
    'common.time.minutesAgo': ' minutes ago',
    'common.time.hoursAgo': ' hours ago',
    'common.time.daysAgo': ' days ago',
    'common.time.today': 'Today',
    'common.time.yesterday': 'Yesterday',

    // Navigation
    'common.nav.home': 'Home',
    'common.nav.settings': 'Settings',
    'common.nav.help': 'Help',
    'common.nav.about': 'About',

    // Theme
    'common.theme.dark': 'Dark Mode',
    'common.theme.light': 'Light Mode',
    'common.theme.auto': 'System',
    'common.theme.title': 'Theme',
    'common.theme.dayNight': 'Day/Night Mode',
    'common.theme.day': 'Day',
    'common.theme.night': 'Night',
    'common.theme.switchToDay': 'Switch to Day Mode',
    'common.theme.switchToNight': 'Switch to Night Mode',

    // Settings Panel
    'common.settings.title': 'Settings',
    'common.settings.apiConfig.title': 'LLM API Configuration',
    'common.settings.apiConfig.label': 'Configure API Key and Model',
    'common.settings.apiConfig.open': 'Open API Configuration',
    'common.settings.background.title': 'Chat Background',
    'common.settings.background.label': 'Background Image',
    'common.settings.background.select': 'Select Image',
    'common.settings.background.remove': 'Remove Background',
    'common.settings.background.preview': 'Background Preview',

    // Language
    'common.language': 'Language',
    'common.language.zh': '中文',
    'common.language.en': 'English',

    // Bot Gateway Settings
    'common.gateway.title': 'Bot Gateway',
    'common.gateway.subtitle': 'Configure IM bots to remotely call Alou tools',
    'common.gateway.enabled': 'Enabled',
    'common.gateway.disabled': 'Disabled',
    'common.gateway.status': 'Status',
    'common.gateway.running': 'Running',
    'common.gateway.stopped': 'Stopped',
    'common.gateway.port': 'Port',
    'common.gateway.platforms': 'Enabled Platforms',
    'common.gateway.none': 'None',
    'common.gateway.configure': 'Configure',
    'common.gateway.testConnection': 'Test Connection',
    'common.gateway.save': 'Save Configuration',
    'common.gateway.saving': 'Saving...',
    'common.gateway.startService': 'Start Service',
    'common.gateway.stopService': 'Stop Service',

    // Platform Names
    'common.gateway.platform.telegram': 'Telegram',
    'common.gateway.platform.feishu': 'Feishu',
    'common.gateway.platform.discord': 'Discord',
    'common.gateway.platform.qq': 'QQ',

    // Telegram Configuration
    'common.gateway.telegram.enabled': 'Enable Telegram Bot',
    'common.gateway.telegram.botToken': 'Bot Token',
    'common.gateway.telegram.tokenPlaceholder': '123456789:ABCdefGHIjklMNOpqrsTUVwxyz',
    'common.gateway.telegram.usePolling': 'Use Polling Mode (No Webhook Required)',
    'common.gateway.telegram.allowedUserIds': 'Allowed User IDs',
    'common.gateway.telegram.allowedChatIds': 'Allowed Chat IDs',
    'common.gateway.telegram.hint': 'Tip: Get Bot Token from BotFather',

    // Feishu Configuration
    'common.gateway.feishu.enabled': 'Enable Feishu Bot',
    'common.gateway.feishu.appId': 'App ID',
    'common.gateway.feishu.appSecret': 'App Secret',
    'common.gateway.feishu.verifyToken': 'Verify Token',
    'common.gateway.feishu.encryptKey': 'Encrypt Key (Optional)',
    'common.gateway.feishu.appIdPlaceholder': 'cli_a1b2c3d4e5f6g7h8',
    'common.gateway.feishu.hint': 'Tip: Create app in Feishu Open Platform to get credentials',

    // Discord Configuration
    'common.gateway.discord.enabled': 'Enable Discord Bot',
    'common.gateway.discord.botToken': 'Bot Token',
    'common.gateway.discord.tokenPlaceholder': 'Your Discord Bot Token',
    'common.gateway.discord.allowedUserIds': 'Allowed User IDs',
    'common.gateway.discord.allowedGuildIds': 'Allowed Guild IDs',
    'common.gateway.discord.allowedChannelIds': 'Allowed Channel IDs',
    'common.gateway.discord.hint': 'Tip: Create app in Discord Developer Portal to get Bot Token',

    // QQ Configuration
    'common.gateway.qq.enabled': 'Enable QQ Bot (OneBot)',
    'common.gateway.qq.wsUrl': 'WebSocket URL',
    'common.gateway.qq.wsUrlPlaceholder': 'ws://127.0.0.1:8080',
    'common.gateway.qq.accessToken': 'Access Token',
    'common.gateway.qq.allowedUserIds': 'Allowed User IDs',
    'common.gateway.qq.allowedGroupIds': 'Allowed Group IDs',
    'common.gateway.qq.hint': 'Tip: OneBot-compatible QQ bot framework required (e.g., go-cqhttp)',

    // General Configuration
    'common.gateway.general.enabled': 'Enable Bot Gateway',
    'common.gateway.general.commandPrefix': 'Command Prefix',
    'common.gateway.general.publicDomain': 'Public Domain (Optional)',

    // Logs
    'common.gateway.logs.title': 'Logs',
    'common.gateway.logs.noLogs': 'No logs',

    // Success messages
    'common.gateway.success.saved': 'Configuration saved',
  },
}
