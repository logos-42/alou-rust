// SPDX-License-Identifier: MIT
pragma solidity ^0.8.30;

import "./IEntryPoint.sol";
import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/security/ReentrancyGuard.sol";

/**
 * @title DIAPPaymaster
 * @dev DIAP Gas 赞助合约
 * @notice 为智能体操作支付 Gas 费用
 */
contract DIAPPaymaster is Ownable, ReentrancyGuard {
    
    // ============ Custom Errors ============
    
    error NotFromEntryPoint();
    error InvalidUserOp();
    error InsufficientDeposit();
    error QuotaExceeded();
    error NotWhitelisted();
    error InvalidQuota();
    error ZeroAddress();
    error GlobalQuotaExceeded();
    
    // ============ 状态变量 ============
    
    IEntryPoint public immutable ENTRY_POINT;
    
    // Gas 配额管理
    struct GasQuota {
        uint256 dailyQuota;       // 每日配额
        uint256 usedToday;        // 今日已使用
        uint256 lastResetTime;    // 上次重置时间
        bool isActive;            // 是否激活
    }
    
    mapping(address => GasQuota) public gasQuotas;
    
    // 白名单管理
    mapping(address => bool) public accountWhitelist;  // 账户白名单
    mapping(address => bool) public targetWhitelist;   // 目标合约白名单
    
    // 统计数据
    uint256 public totalSponsored;
    uint256 public totalOperations;
    
    // 配置参数
    uint256 public defaultDailyQuota;
    bool public requireWhitelist;
    
    // 全局限制（防止 DoS）
    uint256 public globalDailyLimit;
    uint256 public globalUsedToday;
    uint256 public globalLastResetTime;
    
    // ============ 事件定义 ============
    
    event UserOperationSponsored(
        address indexed account,
        uint256 actualGasCost,
        uint256 actualGasUsed
    );
    
    event QuotaUpdated(address indexed account, uint256 newQuota);
    event AccountWhitelisted(address indexed account);
    event AccountRemovedFromWhitelist(address indexed account);
    event TargetWhitelisted(address indexed target);
    event TargetRemovedFromWhitelist(address indexed target);
    event DefaultQuotaUpdated(uint256 newQuota);
    
    // ============ 修饰符 ============
    
    modifier onlyEntryPoint() {
        if (msg.sender != address(ENTRY_POINT)) revert NotFromEntryPoint();
        _;
    }
    
    // ============ 构造函数 ============
    
    constructor(IEntryPoint _entryPoint) {
        if (address(_entryPoint) == address(0)) revert ZeroAddress();
        
        ENTRY_POINT = _entryPoint;
        defaultDailyQuota = 0.01 ether;  // 默认每日 0.01 ETH
        requireWhitelist = true;
        globalDailyLimit = 1 ether;  // 全局每日限额 1 ETH
        globalLastResetTime = block.timestamp;
    }
    
    // ============ Paymaster 核心函数 ============
    
    /**
     * @dev 验证是否赞助用户操作
     * @param userOp 用户操作
     * @param userOpHash 用户操作哈希
     * @param maxCost 最大成本
     * @return context 上下文数据
     * @return validationData 验证数据
     */
    function validatePaymasterUserOp(
        IEntryPoint.UserOperation calldata userOp,
        bytes32 userOpHash,
        uint256 maxCost
    ) external onlyEntryPoint returns (bytes memory context, uint256 validationData) {
        (userOpHash);
        
        // 检查全局限额（防止 DoS）
        if (block.timestamp >= globalLastResetTime + 1 days) {
            globalUsedToday = 0;
            globalLastResetTime = block.timestamp;
        }
        
        if (globalUsedToday + maxCost > globalDailyLimit) {
            revert GlobalQuotaExceeded();
        }
        
        // 检查账户白名单
        if (requireWhitelist && !accountWhitelist[userOp.sender]) {
            revert NotWhitelisted();
        }
        
        // 解析目标地址（从 callData 中）
        address target = _parseTarget(userOp.callData);
        
        // 检查目标白名单
        if (requireWhitelist && target != address(0) && !targetWhitelist[target]) {
            revert NotWhitelisted();
        }
        
        // 检查 Gas 配额
        GasQuota storage quota = gasQuotas[userOp.sender];
        
        // 初始化配额（如果需要）
        if (!quota.isActive) {
            quota.dailyQuota = defaultDailyQuota;
            quota.isActive = true;
        }
        
        // 重置每日配额（如果需要）
        if (block.timestamp >= quota.lastResetTime + 1 days) {
            quota.usedToday = 0;
            quota.lastResetTime = block.timestamp;
        }
        
        // 检查配额是否充足
        if (quota.usedToday + maxCost > quota.dailyQuota) {
            revert QuotaExceeded();
        }
        
        // 检查 Paymaster 存款是否充足
        uint256 deposit = ENTRY_POINT.balanceOf(address(this));
        if (deposit < maxCost) {
            revert InsufficientDeposit();
        }
        
        // 更新全局使用量
        globalUsedToday += maxCost;
        
        // 返回上下文（用于 postOp）
        context = abi.encode(userOp.sender, maxCost);
        validationData = 0;  // 验证成功
    }
    
    /**
     * @dev 操作后处理
     * @param mode 模式
     * @param context 上下文
     * @param actualGasCost 实际 Gas 成本
     */
    function postOp(
        uint8 mode,
        bytes calldata context,
        uint256 actualGasCost
    ) external onlyEntryPoint {
        (mode);
        
        (address account, uint256 maxCost) = abi.decode(context, (address, uint256));
        (maxCost);
        
        // 更新配额
        GasQuota storage quota = gasQuotas[account];
        quota.usedToday += actualGasCost;
        
        // 更新统计
        totalSponsored += actualGasCost;
        totalOperations++;
        
        emit UserOperationSponsored(account, actualGasCost, actualGasCost / tx.gasprice);
    }
    
    /**
     * @dev 解析目标地址
     * @param callData 调用数据
     * @return 目标地址
     */
    function _parseTarget(bytes calldata callData) internal pure returns (address) {
        if (callData.length < 4) return address(0);
        
        // 假设 callData 格式为: execute(address,uint256,bytes)
        // 前 4 字节是函数选择器，接下来 32 字节是地址
        if (callData.length >= 36) {
            return address(uint160(uint256(bytes32(callData[4:36]))));
        }
        
        return address(0);
    }
    
    // ============ 配额管理 ============
    
    /**
     * @dev 设置账户配额
     * @param account 账户地址
     * @param dailyQuota 每日配额
     */
    function setGasQuota(address account, uint256 dailyQuota) external onlyOwner {
        if (account == address(0)) revert ZeroAddress();
        if (dailyQuota == 0) revert InvalidQuota();
        
        gasQuotas[account].dailyQuota = dailyQuota;
        gasQuotas[account].isActive = true;
        
        emit QuotaUpdated(account, dailyQuota);
    }
    
    /**
     * @dev 批量设置配额
     * @param accounts 账户地址数组
     * @param dailyQuotas 每日配额数组
     */
    function batchSetGasQuota(
        address[] calldata accounts,
        uint256[] calldata dailyQuotas
    ) external onlyOwner {
        require(accounts.length == dailyQuotas.length, "Length mismatch");
        require(accounts.length <= 100, "Batch too large");
        
        for (uint256 i = 0; i < accounts.length;) {
            if (accounts[i] != address(0) && dailyQuotas[i] > 0) {
                gasQuotas[accounts[i]].dailyQuota = dailyQuotas[i];
                gasQuotas[accounts[i]].isActive = true;
                emit QuotaUpdated(accounts[i], dailyQuotas[i]);
            }
            unchecked { ++i; }  // Gas 优化
        }
    }
    
    /**
     * @dev 设置默认配额
     * @param quota 默认配额
     */
    function setDefaultDailyQuota(uint256 quota) external onlyOwner {
        if (quota == 0) revert InvalidQuota();
        defaultDailyQuota = quota;
        emit DefaultQuotaUpdated(quota);
    }
    
    /**
     * @dev 设置全局每日限额
     * @param limit 全局限额
     */
    function setGlobalDailyLimit(uint256 limit) external onlyOwner {
        if (limit == 0) revert InvalidQuota();
        globalDailyLimit = limit;
    }
    
    // ============ 白名单管理 ============
    
    /**
     * @dev 添加账户到白名单
     * @param account 账户地址
     */
    function addAccountToWhitelist(address account) external onlyOwner {
        if (account == address(0)) revert ZeroAddress();
        accountWhitelist[account] = true;
        emit AccountWhitelisted(account);
    }
    
    /**
     * @dev 从白名单移除账户
     * @param account 账户地址
     */
    function removeAccountFromWhitelist(address account) external onlyOwner {
        accountWhitelist[account] = false;
        emit AccountRemovedFromWhitelist(account);
    }
    
    /**
     * @dev 批量添加账户到白名单
     * @param accounts 账户地址数组
     */
    function batchAddAccountToWhitelist(address[] calldata accounts) external onlyOwner {
        require(accounts.length <= 100, "Batch too large");
        
        for (uint256 i = 0; i < accounts.length;) {
            if (accounts[i] != address(0)) {
                accountWhitelist[accounts[i]] = true;
                emit AccountWhitelisted(accounts[i]);
            }
            unchecked { ++i; }  // Gas 优化
        }
    }
    
    /**
     * @dev 添加目标合约到白名单
     * @param target 目标地址
     */
    function addTargetToWhitelist(address target) external onlyOwner {
        if (target == address(0)) revert ZeroAddress();
        targetWhitelist[target] = true;
        emit TargetWhitelisted(target);
    }
    
    /**
     * @dev 从白名单移除目标合约
     * @param target 目标地址
     */
    function removeTargetFromWhitelist(address target) external onlyOwner {
        targetWhitelist[target] = false;
        emit TargetRemovedFromWhitelist(target);
    }
    
    /**
     * @dev 批量添加目标到白名单
     * @param targets 目标地址数组
     */
    function batchAddTargetToWhitelist(address[] calldata targets) external onlyOwner {
        require(targets.length <= 100, "Batch too large");
        
        for (uint256 i = 0; i < targets.length;) {
            if (targets[i] != address(0)) {
                targetWhitelist[targets[i]] = true;
                emit TargetWhitelisted(targets[i]);
            }
            unchecked { ++i; }  // Gas 优化
        }
    }
    
    /**
     * @dev 设置是否需要白名单
     * @param required 是否需要
     */
    function setRequireWhitelist(bool required) external onlyOwner {
        requireWhitelist = required;
    }
    
    // ============ 存款管理 ============
    
    /**
     * @dev 添加存款到 EntryPoint
     */
    function addDeposit() external payable onlyOwner {
        ENTRY_POINT.depositTo{value: msg.value}(address(this));
    }
    
    /**
     * @dev 从 EntryPoint 提取存款
     * @param withdrawAddress 提取地址
     * @param amount 提取数量
     */
    function withdrawDeposit(
        address payable withdrawAddress,
        uint256 amount
    ) external onlyOwner nonReentrant {
        ENTRY_POINT.withdrawTo(withdrawAddress, amount);
    }
    
    /**
     * @dev 获取存款余额
     */
    function getDeposit() external view returns (uint256) {
        return ENTRY_POINT.balanceOf(address(this));
    }
    
    // ============ 查询函数 ============
    
    /**
     * @dev 获取账户配额信息
     * @param account 账户地址
     */
    function getGasQuotaInfo(address account) external view returns (
        uint256 dailyQuota,
        uint256 usedToday,
        uint256 remainingToday,
        bool isActive
    ) {
        GasQuota memory quota = gasQuotas[account];
        
        uint256 remaining = 0;
        if (quota.isActive) {
            // 检查是否需要重置
            if (block.timestamp >= quota.lastResetTime + 1 days) {
                remaining = quota.dailyQuota;
            } else {
                remaining = quota.dailyQuota > quota.usedToday 
                    ? quota.dailyQuota - quota.usedToday 
                    : 0;
            }
        }
        
        return (
            quota.dailyQuota,
            quota.usedToday,
            remaining,
            quota.isActive
        );
    }
    
    /**
     * @dev 获取统计信息
     */
    function getStats() external view returns (
        uint256 _totalSponsored,
        uint256 _totalOperations,
        uint256 _deposit
    ) {
        return (
            totalSponsored,
            totalOperations,
            ENTRY_POINT.balanceOf(address(this))
        );
    }
    
    /**
     * @dev 检查账户是否在白名单
     */
    function isAccountWhitelisted(address account) external view returns (bool) {
        return accountWhitelist[account];
    }
    
    /**
     * @dev 检查目标是否在白名单
     */
    function isTargetWhitelisted(address target) external view returns (bool) {
        return targetWhitelist[target];
    }
    
    // ============ 接收 ETH ============
    
    receive() external payable {
        ENTRY_POINT.depositTo{value: msg.value}(address(this));
    }
}
