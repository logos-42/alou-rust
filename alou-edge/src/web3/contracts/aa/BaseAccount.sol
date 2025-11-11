// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8.30;

import "./IEntryPoint.sol";
import "@openzeppelin/contracts/utils/cryptography/ECDSA.sol";

/**
 * @title BaseAccount
 * @dev ERC-4337 账户基础合约
 * @notice 基于官方 account-abstraction 实现
 */
abstract contract BaseAccount {
    using ECDSA for bytes32;
    
    // ============ Custom Errors ============
    
    error NotFromEntryPoint();
    error NotFromEntryPointOrOwner();
    
    // ============ 修饰符 ============
    
    /**
     * @dev 确保调用者是 EntryPoint
     */
    modifier onlyEntryPoint() {
        if (msg.sender != address(entryPoint())) revert NotFromEntryPoint();
        _;
    }
    
    /**
     * @dev 确保调用者是 EntryPoint 或 Owner
     */
    modifier onlyEntryPointOrOwner() {
        if (msg.sender != address(entryPoint()) && msg.sender != owner()) {
            revert NotFromEntryPointOrOwner();
        }
        _;
    }
    
    // ============ 抽象函数 ============
    
    /**
     * @dev 返回 EntryPoint 地址
     */
    function entryPoint() public view virtual returns (IEntryPoint);
    
    /**
     * @dev 返回账户所有者
     */
    function owner() public view virtual returns (address);
    
    /**
     * @dev 验证用户操作签名
     * @param userOp 用户操作
     * @param userOpHash 用户操作哈希
     * @return validationData 验证数据（0 表示成功）
     */
    function _validateSignature(
        IEntryPoint.UserOperation calldata userOp,
        bytes32 userOpHash
    ) internal virtual returns (uint256 validationData);
    
    // ============ EntryPoint 调用函数 ============
    
    /**
     * @dev 验证用户操作
     * @param userOp 用户操作
     * @param userOpHash 用户操作哈希
     * @param missingAccountFunds 缺少的账户资金
     * @return validationData 验证数据
     */
    function validateUserOp(
        IEntryPoint.UserOperation calldata userOp,
        bytes32 userOpHash,
        uint256 missingAccountFunds
    ) external virtual onlyEntryPoint returns (uint256 validationData) {
        validationData = _validateSignature(userOp, userOpHash);
        _payPrefund(missingAccountFunds);
    }
    
    /**
     * @dev 支付预付款
     * @param missingAccountFunds 缺少的资金
     */
    function _payPrefund(uint256 missingAccountFunds) internal {
        if (missingAccountFunds != 0) {
            (bool success,) = payable(msg.sender).call{
                value: missingAccountFunds,
                gas: type(uint256).max
            }("");
            (success);
        }
    }
    
    /**
     * @dev 执行调用
     * @param dest 目标地址
     * @param value 发送的 ETH 数量
     * @param func 调用数据
     */
    function execute(
        address dest,
        uint256 value,
        bytes calldata func
    ) external onlyEntryPointOrOwner {
        _call(dest, value, func);
    }
    
    /**
     * @dev 批量执行调用
     * @param dest 目标地址数组
     * @param value ETH 数量数组
     * @param func 调用数据数组
     */
    function executeBatch(
        address[] calldata dest,
        uint256[] calldata value,
        bytes[] calldata func
    ) external onlyEntryPointOrOwner {
        require(dest.length == func.length && dest.length == value.length, "Length mismatch");
        for (uint256 i = 0; i < dest.length;) {
            _call(dest[i], value[i], func[i]);
            unchecked {
                ++i;  // Gas 优化
            }
        }
    }
    
    /**
     * @dev 内部调用函数
     */
    function _call(address target, uint256 value, bytes memory data) internal {
        (bool success, bytes memory result) = target.call{value: value}(data);
        if (!success) {
            assembly {
                revert(add(result, 32), mload(result))
            }
        }
    }
    
    /**
     * @dev 获取存款
     */
    function getDeposit() public view returns (uint256) {
        return entryPoint().balanceOf(address(this));
    }
    
    /**
     * @dev 添加存款
     */
    function addDeposit() public payable {
        entryPoint().depositTo{value: msg.value}(address(this));
    }
    
    /**
     * @dev 提取存款
     * @param withdrawAddress 提取地址
     * @param amount 提取数量
     */
    function withdrawDepositTo(
        address payable withdrawAddress,
        uint256 amount
    ) public onlyEntryPointOrOwner {
        entryPoint().withdrawTo(withdrawAddress, amount);
    }
    
    /**
     * @dev 接收 ETH
     */
    receive() external payable {}
}
