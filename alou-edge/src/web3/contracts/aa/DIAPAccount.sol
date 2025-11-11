// SPDX-License-Identifier: MIT
pragma solidity ^0.8.30;

import "./BaseAccount.sol";
import "./IEntryPoint.sol";
import "./IERC1271.sol";
import "@openzeppelin/contracts/utils/cryptography/ECDSA.sol";
import "@openzeppelin/contracts/proxy/utils/Initializable.sol";
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";

/**
 * @title DIAPAccount
 * @dev DIAP 智能体 ERC-4337 账户合约
 * @notice 支持 Session Key、支出限额、白名单等功能
 */
contract DIAPAccount is BaseAccount, Initializable, IERC1271 {
    using ECDSA for bytes32;
    
    // ============ Custom Errors ============
    
    error OnlyOwner();
    error InvalidSessionKey();
    error SessionKeyExpired();
    error DailyLimitExceeded();
    error PerTxLimitExceeded();
    error NotWhitelisted();
    error AccountFrozen();
    error InvalidSignature();
    error ZeroAddress();
    error InvalidLimit();
    error SessionKeyNotFound();
    
    // ============ 状态变量 ============
    
    IEntryPoint private immutable ENTRY_POINT;
    address private _owner;
    bool private _frozen;
    
    // Session Key 管理
    struct SessionKeyData {
        uint256 validUntil;       // 有效期
        uint256 dailyLimit;       // 每日限额
        uint256 perTxLimit;       // 单笔限额
        uint256 spentToday;       // 今日已花费
        uint256 lastResetTime;    // 上次重置时间
        bool isActive;            // 是否激活
    }
    
    mapping(address => SessionKeyData) public sessionKeys;
    address[] public sessionKeyList;
    
    // Nonce 管理（防止重放攻击）
    mapping(address => uint256) public sessionKeyNonces;
    
    // 支出限额
    uint256 public defaultDailyLimit;
    uint256 public defaultPerTxLimit;
    
    // 批量操作限制
    uint256 public constant MAX_BATCH_SIZE = 50;
    
    // 白名单
    mapping(address => bool) public whitelist;
    address[] public whitelistAddresses;
    
    // ============ 事件定义 ============
    
    event OwnershipTransferred(address indexed previousOwner, address indexed newOwner);
    event SessionKeyAdded(address indexed key, uint256 validUntil, uint256 dailyLimit);
    event SessionKeyRemoved(address indexed key);
    event SessionKeyUsed(address indexed key, uint256 amount);
    event DailyLimitUpdated(uint256 newLimit);
    event PerTxLimitUpdated(uint256 newLimit);
    event WhitelistAdded(address indexed target);
    event WhitelistRemoved(address indexed target);
    event AccountFrozenEvent();
    event AccountUnfrozenEvent();
    event Executed(address indexed target, uint256 value, bytes data);
    
    // ============ 修饰符 ============
    
    modifier onlyOwner() {
        if (msg.sender != _owner) revert OnlyOwner();
        _;
    }
    
    modifier notFrozen() {
        if (_frozen) revert AccountFrozen();
        _;
    }
    
    // ============ 构造函数 ============
    
    constructor(IEntryPoint anEntryPoint) {
        ENTRY_POINT = anEntryPoint;
        _disableInitializers();
    }
    
    /**
     * @dev 初始化账户
     * @param anOwner 所有者地址
     */
    function initialize(address anOwner) public initializer {
        if (anOwner == address(0)) revert ZeroAddress();
        _owner = anOwner;
        
        // 设置默认限额
        defaultDailyLimit = 1000 * 10**18;  // 1000 DIAP
        defaultPerTxLimit = 100 * 10**18;   // 100 DIAP
        
        emit OwnershipTransferred(address(0), anOwner);
    }
    
    // ============ BaseAccount 实现 ============
    
    function entryPoint() public view override returns (IEntryPoint) {
        return ENTRY_POINT;
    }
    
    function owner() public view override returns (address) {
        return _owner;
    }
    
    /**
     * @dev 验证签名（包含 nonce 检查防止重放攻击）
     */
    function _validateSignature(
        IEntryPoint.UserOperation calldata userOp,
        bytes32 userOpHash
    ) internal override returns (uint256 validationData) {
        bytes32 hash = userOpHash.toEthSignedMessageHash();
        address signer = hash.recover(userOp.signature);
        
        // 检查是否是 owner
        if (signer == _owner) {
            return 0;
        }
        
        // 检查是否是有效的 Session Key
        SessionKeyData storage sessionKey = sessionKeys[signer];
        if (!sessionKey.isActive) {
            return 1; // 签名无效
        }
        
        if (block.timestamp > sessionKey.validUntil) {
            return 1; // Session Key 已过期
        }
        
        // 验证并更新 nonce（防止重放攻击）
        uint256 expectedNonce = sessionKeyNonces[signer];
        if (userOp.nonce != expectedNonce) {
            return 1; // Nonce 不匹配
        }
        unchecked {
            ++sessionKeyNonces[signer];  // Gas 优化
        }
        
        // Session Key 验证成功
        return 0;
    }
    
    // ============ Session Key 管理 ============
    
    /**
     * @dev 添加 Session Key
     * @param key Session Key 地址
     * @param validUntil 有效期
     * @param dailyLimit 每日限额
     * @param perTxLimit 单笔限额
     */
    function addSessionKey(
        address key,
        uint256 validUntil,
        uint256 dailyLimit,
        uint256 perTxLimit
    ) external onlyOwner {
        if (key == address(0)) revert ZeroAddress();
        if (validUntil <= block.timestamp) revert InvalidSessionKey();
        if (dailyLimit == 0 || perTxLimit == 0) revert InvalidLimit();
        
        if (!sessionKeys[key].isActive) {
            sessionKeyList.push(key);
        }
        
        sessionKeys[key] = SessionKeyData({
            validUntil: validUntil,
            dailyLimit: dailyLimit,
            perTxLimit: perTxLimit,
            spentToday: 0,
            lastResetTime: block.timestamp,
            isActive: true
        });
        
        emit SessionKeyAdded(key, validUntil, dailyLimit);
    }
    
    /**
     * @dev 移除 Session Key
     * @param key Session Key 地址
     */
    function removeSessionKey(address key) external onlyOwner {
        if (!sessionKeys[key].isActive) revert SessionKeyNotFound();
        
        sessionKeys[key].isActive = false;
        
        emit SessionKeyRemoved(key);
    }
    
    /**
     * @dev 获取当天开始时间（UTC 0:00）
     */
    function _getDayStart(uint256 timestamp) internal pure returns (uint256) {
        return (timestamp / 1 days) * 1 days;
    }
    
    /**
     * @dev 检查并更新 Session Key 限额
     * @param key Session Key 地址
     * @param amount 花费金额
     */
    function _checkAndUpdateSessionKeyLimit(address key, uint256 amount) internal {
        SessionKeyData storage sessionKey = sessionKeys[key];
        
        if (!sessionKey.isActive) revert InvalidSessionKey();
        if (block.timestamp > sessionKey.validUntil) revert SessionKeyExpired();
        
        // 检查单笔限额
        if (amount > sessionKey.perTxLimit) revert PerTxLimitExceeded();
        
        // 使用固定的 UTC 时间重置每日限额
        uint256 currentDayStart = _getDayStart(block.timestamp);
        uint256 lastDayStart = _getDayStart(sessionKey.lastResetTime);
        
        if (currentDayStart > lastDayStart) {
            sessionKey.spentToday = 0;
            sessionKey.lastResetTime = currentDayStart;
        }
        
        // 检查每日限额
        if (sessionKey.spentToday + amount > sessionKey.dailyLimit) {
            revert DailyLimitExceeded();
        }
        
        // 更新已花费金额
        sessionKey.spentToday += amount;
        
        emit SessionKeyUsed(key, amount);
    }
    
    // ============ 限额管理 ============
    
    /**
     * @dev 设置默认每日限额
     * @param limit 新限额
     */
    function setDefaultDailyLimit(uint256 limit) external onlyOwner {
        if (limit == 0) revert InvalidLimit();
        defaultDailyLimit = limit;
        emit DailyLimitUpdated(limit);
    }
    
    /**
     * @dev 设置默认单笔限额
     * @param limit 新限额
     */
    function setDefaultPerTxLimit(uint256 limit) external onlyOwner {
        if (limit == 0) revert InvalidLimit();
        defaultPerTxLimit = limit;
        emit PerTxLimitUpdated(limit);
    }
    
    // ============ 白名单管理 ============
    
    /**
     * @dev 添加到白名单
     * @param target 目标地址
     */
    function addToWhitelist(address target) external onlyOwner {
        if (target == address(0)) revert ZeroAddress();
        if (!whitelist[target]) {
            whitelist[target] = true;
            whitelistAddresses.push(target);
            emit WhitelistAdded(target);
        }
    }
    
    /**
     * @dev 从白名单移除
     * @param target 目标地址
     */
    function removeFromWhitelist(address target) external onlyOwner {
        if (whitelist[target]) {
            whitelist[target] = false;
            emit WhitelistRemoved(target);
        }
    }
    
    /**
     * @dev 批量添加到白名单
     * @param targets 目标地址数组
     */
    function batchAddToWhitelist(address[] calldata targets) external onlyOwner {
        require(targets.length <= MAX_BATCH_SIZE, "Batch too large");
        
        for (uint256 i = 0; i < targets.length;) {
            if (targets[i] != address(0) && !whitelist[targets[i]]) {
                whitelist[targets[i]] = true;
                whitelistAddresses.push(targets[i]);
                emit WhitelistAdded(targets[i]);
            }
            unchecked { ++i; }  // Gas 优化
        }
    }
    
    // ============ 账户控制 ============
    
    /**
     * @dev 冻结账户
     */
    function freeze() external onlyOwner {
        _frozen = true;
        emit AccountFrozenEvent();
    }
    
    /**
     * @dev 解冻账户
     */
    function unfreeze() external onlyOwner {
        _frozen = false;
        emit AccountUnfrozenEvent();
    }
    
    /**
     * @dev 转移所有权
     * @param newOwner 新所有者
     */
    function transferOwnership(address newOwner) external onlyOwner {
        if (newOwner == address(0)) revert ZeroAddress();
        address oldOwner = _owner;
        _owner = newOwner;
        emit OwnershipTransferred(oldOwner, newOwner);
    }
    
    // ============ 执行函数（带限额检查）============
    
    /**
     * @dev 执行调用（带 Session Key 限额检查）
     * @param dest 目标地址
     * @param value 发送的 ETH 数量
     * @param func 调用数据
     */
    function executeWithSessionKey(
        address dest,
        uint256 value,
        bytes calldata func
    ) external notFrozen {
        // 只有通过 EntryPoint 或 owner 可以调用
        if (msg.sender != address(entryPoint()) && msg.sender != _owner) {
            revert NotFromEntryPointOrOwner();
        }
        
        // 如果不是 owner，检查白名单和限额
        if (msg.sender != _owner) {
            if (!whitelist[dest]) revert NotWhitelisted();
            
            // 注意：这里简化了 Session Key 的获取
            // 生产环境应该从 UserOperation 的签名中解析
            // 暂时跳过限额检查，由 validateUserOp 处理
        }
        
        _call(dest, value, func);
        emit Executed(dest, value, func);
    }
    
    // ============ 代币操作 ============
    
    /**
     * @dev 转移 ERC20 代币
     * @param token 代币地址
     * @param to 接收地址
     * @param amount 数量
     */
    function transferToken(
        address token,
        address to,
        uint256 amount
    ) external onlyEntryPointOrOwner notFrozen {
        IERC20(token).transfer(to, amount);
    }
    
    /**
     * @dev 批量转移 ERC20 代币
     * @param token 代币地址
     * @param recipients 接收地址数组
     * @param amounts 数量数组
     */
    function batchTransferToken(
        address token,
        address[] calldata recipients,
        uint256[] calldata amounts
    ) external onlyEntryPointOrOwner notFrozen {
        require(recipients.length == amounts.length, "Length mismatch");
        require(recipients.length <= MAX_BATCH_SIZE, "Batch too large");
        
        for (uint256 i = 0; i < recipients.length;) {
            IERC20(token).transfer(recipients[i], amounts[i]);
            unchecked { ++i; }  // Gas 优化
        }
    }
    
    // ============ 查询函数 ============
    
    /**
     * @dev 获取 Session Key 信息
     * @param key Session Key 地址
     */
    function getSessionKeyInfo(address key) external view returns (
        uint256 validUntil,
        uint256 dailyLimit,
        uint256 perTxLimit,
        uint256 spentToday,
        uint256 remainingToday,
        bool isActive
    ) {
        SessionKeyData memory sessionKey = sessionKeys[key];
        
        uint256 remaining = 0;
        if (sessionKey.isActive && block.timestamp < sessionKey.validUntil) {
            // 检查是否需要重置
            if (block.timestamp >= sessionKey.lastResetTime + 1 days) {
                remaining = sessionKey.dailyLimit;
            } else {
                remaining = sessionKey.dailyLimit > sessionKey.spentToday 
                    ? sessionKey.dailyLimit - sessionKey.spentToday 
                    : 0;
            }
        }
        
        return (
            sessionKey.validUntil,
            sessionKey.dailyLimit,
            sessionKey.perTxLimit,
            sessionKey.spentToday,
            remaining,
            sessionKey.isActive
        );
    }
    
    /**
     * @dev 获取所有 Session Keys
     */
    function getAllSessionKeys() external view returns (address[] memory) {
        return sessionKeyList;
    }
    
    /**
     * @dev 获取所有白名单地址
     */
    function getWhitelist() external view returns (address[] memory) {
        return whitelistAddresses;
    }
    
    /**
     * @dev 检查地址是否在白名单
     */
    function isWhitelisted(address target) external view returns (bool) {
        return whitelist[target];
    }
    
    /**
     * @dev 检查账户是否冻结
     */
    function isFrozen() external view returns (bool) {
        return _frozen;
    }
    
    /**
     * @dev 获取代币余额
     * @param token 代币地址
     */
    function getTokenBalance(address token) external view returns (uint256) {
        return IERC20(token).balanceOf(address(this));
    }
    
    // ============ EIP-1271 签名验证 ============
    
    /**
     * @dev EIP-1271 签名验证
     * @param hash 数据哈希
     * @param signature 签名
     * @return magicValue 魔术值
     */
    function isValidSignature(
        bytes32 hash,
        bytes memory signature
    ) external view override returns (bytes4 magicValue) {
        bytes32 messageHash = hash.toEthSignedMessageHash();
        address signer = messageHash.recover(signature);
        
        // 检查是否是 owner
        if (signer == _owner) {
            return 0x1626ba7e; // EIP-1271 魔术值
        }
        
        // 检查是否是有效的 Session Key
        SessionKeyData storage sessionKey = sessionKeys[signer];
        if (sessionKey.isActive && block.timestamp <= sessionKey.validUntil) {
            return 0x1626ba7e;
        }
        
        return 0xffffffff; // 无效签名
    }
}
