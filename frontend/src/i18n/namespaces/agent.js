/**
 * 智能体相关翻译 - 创建、管理、对话
 * 命名规范：agent.模块.具体含义
 */
export const agent = {
  zh: {
    // ==================== 创建智能体 ====================
    'agent.create.title': '创建新的智能体',
    'agent.create.subtitle': '上传头像、配置 MCP 端口，并保存到 IPFS',

    // 头像
    'agent.create.avatar.label': '智能体头像（IPFS）',
    'agent.create.avatar.preview': '预览',
    'agent.create.avatar.select': '选择图片',

    // 名称
    'agent.create.name.label': '智能体名称',
    'agent.create.name.placeholder': '例如：Alou Web3 调度',
    'agent.create.name.unnamed': '未命名智能体',

    // 角色描述
    'agent.create.role.label': '角色设计说明',
    'agent.create.role.placeholder': '描述该智能体的职责、语气与工具使用策略',
    'agent.create.role.default': 'Web3 多代理协调智能体',

    // MCP 端口配置
    'agent.create.mcp.label': 'MCP 端口配置',
    'agent.create.mcp.add': '+ 新增端口',
    'agent.create.mcp.portName': '端口名称',
    'agent.create.mcp.endpoint': 'Endpoint URL (wss:// 或 http://)',
    'agent.create.mcp.port': '端口',
    'agent.create.mcp.description': '描述',
    'agent.create.mcp.remove': '移除',

    // 解析已有智能体
    'agent.create.resolve.label': '解析已有智能体',
    'agent.create.resolve.placeholder': '输入 IPNS / CID / DID',
    'agent.create.resolve.button': '解析',
    'agent.create.resolve.resolving': '解析中...',
    'agent.create.resolve.success': '🔍 解析成功，确认导入以下智能体？',

    // 导入预览
    'agent.create.import.name': '名称：',
    'agent.create.import.description': '描述：',
    'agent.create.import.did': 'DID：',
    'agent.create.import.ipns': 'IPNS：',
    'agent.create.import.cid': 'CID：',
    'agent.create.import.pubsub': 'PubSub：',
    'agent.create.import.confirm': '确认导入',
    'agent.create.import.importing': '导入中...',

    // 创建按钮
    'agent.create.submit': '创建智能体',
    'agent.create.creating': '创建中...',

    // 错误消息
    'agent.create.error.ipfsNotRunning': 'IPFS 节点未运行。请先启动 IPFS 节点后再创建智能体。',
    'agent.create.error.importFailed': '导入失败',

    // ==================== 智能体列表 ====================
    'agent.list.title': '智能体列表',
    'agent.list.empty': '暂无智能体',
    'agent.list.search.placeholder': '搜索智能体...',
    'agent.list.filter.all': '全部',
    'agent.list.filter.online': '在线',
    'agent.list.filter.offline': '离线',

    // ==================== 智能体详情/频道 ====================
    'agent.channel.title': '频道',
    'agent.channel.newChannel': '新建频道',
    'agent.channel.delete': '删除频道',
    'agent.channel.deleteConfirm': '确定要删除这个频道吗？',

    // 智能体类型标签
    'agent.type.claude': 'Agent',
    'agent.type.custom': '自定义智能体',
    'agent.type.ipns': 'IPNS 解析',
    'agent.type.did': 'DID 解析',
    'agent.type.cid': 'CID 解析',

    // ==================== 对话相关 ====================
    'agent.chat.inputPlaceholder': '输入您的问题...（Enter发送，Shift+Enter换行）',
    'agent.chat.send': '发送',
    'agent.chat.thinking': 'AI正在思考中...',
    'agent.chat.welcome': '欢迎使用Alou智能助手',
    'agent.chat.welcomeDesc': '我是区块链支付的AI助手，很高兴为您提供智能服务。',

    // 对话错误
    'agent.chat.error.sessionExpired': '❌ 会话已过期，请刷新页面重试。',
    'agent.chat.error.serverError': '❌ 服务器内部错误，请稍后重试。',
    'agent.chat.error.networkError': '❌ 无法连接到服务器，请检查网络连接。',
    'agent.chat.error.backendUnavailable': '❌ 无法连接到后端服务，请检查网络连接或稍后重试。',

    // ==================== 邀请智能体 ====================
    'agent.invite.title': '邀请智能体',
    'agent.invite.tab.local': '从本地选择',
    'agent.invite.tab.network': '从网络邀请',
    'agent.invite.local.empty': '本地暂无可邀请的智能体',
    'agent.invite.network.placeholder': '输入 IPNS / CID / DID',
    'agent.invite.network.resolve': '解析并邀请',
    'agent.invite.selected': '已选择 {count} 个智能体',
    'agent.invite.confirm': '确认邀请',

    // ==================== DIAP 身份 ====================
    'agent.diap.title': 'DIAP 身份',
    'agent.diap.create': '创建 DIAP 身份',
    'agent.diap.creating': '创建中...',
    'agent.diap.noIdentity': '此智能体尚未创建 DIAP 身份',
    'agent.diap.did': 'DID',
    'agent.diap.cid': 'CID',
    'agent.diap.ipns': 'IPNS',
    'agent.diap.publicKey': '公钥',
    'agent.diap.gatewayUrl': 'Gateway URL',
    'agent.diap.registrationStatus': '链上注册状态',
    'agent.diap.register': '注册到链上',
    'agent.diap.registering': '注册中...',
    'agent.diap.registered': '已注册',
    'agent.diap.notRegistered': '未注册',
    'agent.diap.createSuccess': 'DIAP 身份创建成功！',
    'agent.diap.createFailed': '创建身份失败',
    'agent.diap.registerFailed': '注册到链上失败',
    'agent.diap.registerTxInfo': '注册交易信息',
    'agent.diap.registerTxHint': '已生成编码交易，请使用钱包签名并广播此交易：',
    'agent.diap.registrationFee': '注册费用',
    'agent.diap.minStakeAmount': '最小质押',
    'agent.diap.createdAt': '创建时间',
    'agent.diap.copy': '复制',

    // ==================== 侧边栏 ====================
    'agent.sidebar.loading': '正在加载智能体...',
    'agent.sidebar.empty': '暂无智能体。请在上方输入 IPNS / CID / DID 搜索。',
    'agent.sidebar.connectNow': '立即连接',
    'agent.sidebar.filtered': '已筛选：',
    'agent.sidebar.expandList': '展开智能体列表',
    'agent.sidebar.collapseList': '折叠智能体列表',
    'agent.sidebar.clearFilter': '清除模型筛选',
  },
  en: {
    // ==================== Create Agent ====================
    'agent.create.title': 'Create New Agent',
    'agent.create.subtitle': 'Upload avatar, configure MCP ports, and save to IPFS',

    // Avatar
    'agent.create.avatar.label': 'Agent Avatar (IPFS)',
    'agent.create.avatar.preview': 'Preview',
    'agent.create.avatar.select': 'Select Image',

    // Name
    'agent.create.name.label': 'Agent Name',
    'agent.create.name.placeholder': 'e.g., Alou Web3 Scheduler',
    'agent.create.name.unnamed': 'Unnamed Agent',

    // Role Description
    'agent.create.role.label': 'Role Description',
    'agent.create.role.placeholder': 'Describe the agent\'s responsibilities, tone, and tool usage strategy',
    'agent.create.role.default': 'Web3 Multi-Agent Coordinator',

    // MCP Port Configuration
    'agent.create.mcp.label': 'MCP Port Configuration',
    'agent.create.mcp.add': '+ Add Port',
    'agent.create.mcp.portName': 'Port Name',
    'agent.create.mcp.endpoint': 'Endpoint URL (wss:// or http://)',
    'agent.create.mcp.port': 'Port',
    'agent.create.mcp.description': 'Description',
    'agent.create.mcp.remove': 'Remove',

    // Resolve Existing Agent
    'agent.create.resolve.label': 'Resolve Existing Agent',
    'agent.create.resolve.placeholder': 'Enter IPNS / CID / DID',
    'agent.create.resolve.button': 'Resolve',
    'agent.create.resolve.resolving': 'Resolving...',
    'agent.create.resolve.success': '🔍 Resolved successfully, confirm import this agent?',

    // Import Preview
    'agent.create.import.name': 'Name:',
    'agent.create.import.description': 'Description:',
    'agent.create.import.did': 'DID:',
    'agent.create.import.ipns': 'IPNS:',
    'agent.create.import.cid': 'CID:',
    'agent.create.import.pubsub': 'PubSub:',
    'agent.create.import.confirm': 'Confirm Import',
    'agent.create.import.importing': 'Importing...',

    // Create Button
    'agent.create.submit': 'Create Agent',
    'agent.create.creating': 'Creating...',

    // Error Messages
    'agent.create.error.ipfsNotRunning': 'IPFS node is not running. Please start the IPFS node before creating an agent.',
    'agent.create.error.importFailed': 'Import failed',

    // ==================== Agent List ====================
    'agent.list.title': 'Agent List',
    'agent.list.empty': 'No agents yet',
    'agent.list.search.placeholder': 'Search agents...',
    'agent.list.filter.all': 'All',
    'agent.list.filter.online': 'Online',
    'agent.list.filter.offline': 'Offline',

    // ==================== Agent Details/Channel ====================
    'agent.channel.title': 'Channels',
    'agent.channel.newChannel': 'New Channel',
    'agent.channel.delete': 'Delete Channel',
    'agent.channel.deleteConfirm': 'Are you sure you want to delete this channel?',

    // Agent Type Labels
    'agent.type.claude': 'Agent',
    'agent.type.custom': 'Custom Agent',
    'agent.type.ipns': 'IPNS Resolved',
    'agent.type.did': 'DID Resolved',
    'agent.type.cid': 'CID Resolved',

    // ==================== Chat Related ====================
    'agent.chat.inputPlaceholder': 'Type your question... (Enter to send, Shift+Enter for new line)',
    'agent.chat.send': 'Send',
    'agent.chat.thinking': 'AI is thinking...',
    'agent.chat.welcome': 'Welcome to Alou AI Assistant',
    'agent.chat.welcomeDesc': 'I am your blockchain payment AI assistant, happy to help you.',

    // Chat Errors
    'agent.chat.error.sessionExpired': '❌ Session expired, please refresh the page.',
    'agent.chat.error.serverError': '❌ Server error, please try again later.',
    'agent.chat.error.networkError': '❌ Cannot connect to server, please check your network.',
    'agent.chat.error.backendUnavailable': '❌ Cannot connect to backend service, please check network or try again later.',

    // ==================== Invite Agent ====================
    'agent.invite.title': 'Invite Agent',
    'agent.invite.tab.local': 'Select from Local',
    'agent.invite.tab.network': 'Invite from Network',
    'agent.invite.local.empty': 'No local agents available to invite',
    'agent.invite.network.placeholder': 'Enter IPNS / CID / DID',
    'agent.invite.network.resolve': 'Resolve & Invite',
    'agent.invite.selected': '{count} agent(s) selected',
    'agent.invite.confirm': 'Confirm Invite',

    // ==================== DIAP Identity ====================
    'agent.diap.title': 'DIAP Identity',
    'agent.diap.create': 'Create DIAP Identity',
    'agent.diap.creating': 'Creating...',
    'agent.diap.noIdentity': 'DIAP identity not created for this agent',
    'agent.diap.did': 'DID',
    'agent.diap.cid': 'CID',
    'agent.diap.ipns': 'IPNS',
    'agent.diap.publicKey': 'Public Key',
    'agent.diap.gatewayUrl': 'Gateway URL',
    'agent.diap.registrationStatus': 'On-Chain Registration',
    'agent.diap.register': 'Register On-Chain',
    'agent.diap.registering': 'Registering...',
    'agent.diap.registered': 'Registered',
    'agent.diap.notRegistered': 'Not Registered',
    'agent.diap.createSuccess': 'DIAP identity created successfully!',
    'agent.diap.createFailed': 'Failed to create identity',
    'agent.diap.registerFailed': 'Failed to register on-chain',
    'agent.diap.registerTxInfo': 'Registration Transaction',
    'agent.diap.registerTxHint': 'Encoded transaction generated, please sign and broadcast with your wallet:',
    'agent.diap.registrationFee': 'Registration Fee',
    'agent.diap.minStakeAmount': 'Minimum Stake',
    'agent.diap.createdAt': 'Created At',
    'agent.diap.copy': 'Copy',

    // ==================== Sidebar ====================
    'agent.sidebar.loading': 'Loading agents...',
    'agent.sidebar.connectNow': 'Connect Now',
    'agent.sidebar.filtered': 'Filtered: ',
    'agent.sidebar.expandList': 'Expand agent list',
    'agent.sidebar.collapseList': 'Collapse agent list',
    'agent.sidebar.clearFilter': 'Clear model filter',
  },
}

