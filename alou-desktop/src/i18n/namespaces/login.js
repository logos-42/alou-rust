/**
 * 登录页面翻译
 * 命名规范：login.模块.具体含义
 */
export const login = {
  zh: {
    // ==================== 页面标题 ====================
    'login.title': '连接钱包',
    'login.subtitle.desktop': '使用手机钱包扫码连接，或在浏览器中使用钱包插件（桌面版）',
    'login.subtitle.browser': '选择您的加密钱包以安全登录（浏览器版）',

    // ==================== 钱包选项 ====================
    'login.wallet.walletconnect': 'WalletConnect',
    'login.wallet.walletconnect.desc': '扫码连接移动钱包',
    'login.wallet.local': '本地钱包',
    'login.wallet.local.desc': '导入私钥或创建新钱包',
    'login.wallet.metamask': 'MetaMask',
    'login.wallet.metamask.installed': '已安装',
    'login.wallet.metamask.notInstalled': '需要安装浏览器插件',
    'login.wallet.coinbase': 'Coinbase Wallet',
    'login.wallet.coinbase.desc': '安全易用的加密钱包',

    // ==================== 连接模式 ====================
    'login.mode.phoneScan': '手机扫码',
    'login.mode.localWallet': '本地钱包',
    'login.mode.selectHint': '💡 请选择连接方式：WalletConnect（手机扫码）或本地钱包',

    // ==================== 状态提示 ====================
    'login.status.metamaskDetected': '✅ 已检测到 MetaMask，点击上方按钮即可连接',
    'login.status.metamaskNotDetected': '⚠️ 未检测到 MetaMask 插件，请先安装 MetaMask 浏览器扩展',
    'login.status.connecting': '连接中...',

    // ==================== 错误消息 ====================
    'login.error.metamaskNotInstalled': '请先安装 MetaMask 浏览器插件',
    'login.error.connectionFailed': '连接失败',
    'login.error.userRejected': '您拒绝了连接请求，请在 MetaMask 中选择要连接的钱包',
    'login.error.pendingRequest': '请在 MetaMask 中确认连接请求（可能已有待处理的请求）',
    'login.error.internalError': 'MetaMask 内部错误，请刷新页面重试',
    'login.error.walletConnectComingSoon': 'WalletConnect 功能即将推出，请使用 MetaMask 浏览器插件',
    'login.error.coinbaseComingSoon': 'Coinbase Wallet 功能即将推出，请使用 MetaMask 浏览器插件',
    'login.error.notAvailableOnDesktop': '该连接方式在桌面版不可用，请使用 WalletConnect 或本地钱包',
    'login.error.loginFailed': '登录失败',
    'login.error.getAccountFailed': '请求钱包账户失败',
    'login.error.noAccountRetrieved': '未能获取钱包地址',

    // ==================== 安全提示 ====================
    'login.security.title': '安全提示',
    'login.security.tip1': '我们不会存储您的私钥或助记词',
    'login.security.tip2': '请确认您访问的是正确的网站',
    'login.security.tip3': '不要与他人分享您的钱包信息',

    // ==================== 帮助链接 ====================
    'login.help.noWallet': '没有钱包？',
    'login.help.downloadMetaMask': '下载 MetaMask',

    // ==================== 条款 ====================
    'login.terms.prefix': '连接钱包即表示您同意我们的',
    'login.terms.termsOfService': '服务条款',
    'login.terms.and': '和',
    'login.terms.privacyPolicy': '隐私政策',

    // ==================== 本地钱包表单 ====================
    'login.localWallet.title': '本地钱包',
    'login.localWallet.importPrivateKey': '导入私钥',
    'login.localWallet.createNew': '创建新钱包',
    'login.localWallet.privateKeyPlaceholder': '输入您的私钥（64位十六进制字符）',
    'login.localWallet.passwordPlaceholder': '设置密码（可选，用于加密存储）',
    'login.localWallet.importButton': '导入',
    'login.localWallet.createButton': '创建',

    // ==================== WalletConnect QR ====================
    'login.walletconnect.scanTitle': '扫描二维码',
    'login.walletconnect.scanDesc': '使用手机钱包 APP 扫描连接',
    'login.walletconnect.copyLink': '复制链接',
    'login.walletconnect.openInBrowser': '在浏览器中使用钱包插件登录',
    'login.walletconnect.generating': '正在生成二维码...',
    'login.walletconnect.expired': '二维码已过期，请刷新',
    'login.walletconnect.refreshQR': '刷新二维码',
  },
  en: {
    // ==================== Page Title ====================
    'login.title': 'Connect Wallet',
    'login.subtitle.desktop': 'Scan QR code with mobile wallet, or use wallet plugin in browser (Desktop)',
    'login.subtitle.browser': 'Select your crypto wallet to login securely (Browser)',

    // ==================== Wallet Options ====================
    'login.wallet.walletconnect': 'WalletConnect',
    'login.wallet.walletconnect.desc': 'Scan QR code to connect mobile wallet',
    'login.wallet.local': 'Local Wallet',
    'login.wallet.local.desc': 'Import private key or create new wallet',
    'login.wallet.metamask': 'MetaMask',
    'login.wallet.metamask.installed': 'Installed',
    'login.wallet.metamask.notInstalled': 'Browser extension required',
    'login.wallet.coinbase': 'Coinbase Wallet',
    'login.wallet.coinbase.desc': 'Secure and easy-to-use crypto wallet',

    // ==================== Connection Mode ====================
    'login.mode.phoneScan': 'Phone Scan',
    'login.mode.localWallet': 'Local Wallet',
    'login.mode.selectHint': '💡 Please select connection method: WalletConnect (phone scan) or Local Wallet',

    // ==================== Status Messages ====================
    'login.status.metamaskDetected': '✅ MetaMask detected, click the button above to connect',
    'login.status.metamaskNotDetected': '⚠️ MetaMask extension not detected, please install MetaMask browser extension first',
    'login.status.connecting': 'Connecting...',

    // ==================== Error Messages ====================
    'login.error.metamaskNotInstalled': 'Please install MetaMask browser extension first',
    'login.error.connectionFailed': 'Connection failed',
    'login.error.userRejected': 'You rejected the connection request. Please select a wallet in MetaMask',
    'login.error.pendingRequest': 'Please confirm the connection request in MetaMask (there may be a pending request)',
    'login.error.internalError': 'MetaMask internal error, please refresh the page and try again',
    'login.error.walletConnectComingSoon': 'WalletConnect coming soon, please use MetaMask browser extension',
    'login.error.coinbaseComingSoon': 'Coinbase Wallet coming soon, please use MetaMask browser extension',
    'login.error.notAvailableOnDesktop': 'This connection method is not available on desktop, please use WalletConnect or Local Wallet',
    'login.error.loginFailed': 'Login failed',
    'login.error.getAccountFailed': 'Failed to request wallet account',
    'login.error.noAccountRetrieved': 'Failed to retrieve wallet address',

    // ==================== Security Notice ====================
    'login.security.title': 'Security Notice',
    'login.security.tip1': 'We do not store your private keys or mnemonic phrases',
    'login.security.tip2': 'Please confirm you are visiting the correct website',
    'login.security.tip3': 'Do not share your wallet information with others',

    // ==================== Help Links ====================
    'login.help.noWallet': 'No wallet?',
    'login.help.downloadMetaMask': 'Download MetaMask',

    // ==================== Terms ====================
    'login.terms.prefix': 'By connecting your wallet, you agree to our',
    'login.terms.termsOfService': 'Terms of Service',
    'login.terms.and': 'and',
    'login.terms.privacyPolicy': 'Privacy Policy',

    // ==================== Local Wallet Form ====================
    'login.localWallet.title': 'Local Wallet',
    'login.localWallet.importPrivateKey': 'Import Private Key',
    'login.localWallet.createNew': 'Create New Wallet',
    'login.localWallet.privateKeyPlaceholder': 'Enter your private key (64 hex characters)',
    'login.localWallet.passwordPlaceholder': 'Set password (optional, for encrypted storage)',
    'login.localWallet.importButton': 'Import',
    'login.localWallet.createButton': 'Create',

    // ==================== WalletConnect QR ====================
    'login.walletconnect.scanTitle': 'Scan QR Code',
    'login.walletconnect.scanDesc': 'Use your mobile wallet app to scan and connect',
    'login.walletconnect.copyLink': 'Copy Link',
    'login.walletconnect.openInBrowser': 'Login with wallet plugin in browser',
    'login.walletconnect.generating': 'Generating QR code...',
    'login.walletconnect.expired': 'QR code expired, please refresh',
    'login.walletconnect.refreshQR': 'Refresh QR Code',
  },
}

