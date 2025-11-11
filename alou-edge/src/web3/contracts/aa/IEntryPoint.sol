// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8.30;

/**
 * @title IEntryPoint
 * @dev ERC-4337 EntryPoint 接口
 * @notice 官方 EntryPoint 接口定义
 */
interface IEntryPoint {
    
    struct UserOperation {
        address sender;
        uint256 nonce;
        bytes initCode;
        bytes callData;
        uint256 callGasLimit;
        uint256 verificationGasLimit;
        uint256 preVerificationGas;
        uint256 maxFeePerGas;
        uint256 maxPriorityFeePerGas;
        bytes paymasterAndData;
        bytes signature;
    }
    
    function handleOps(UserOperation[] calldata ops, address payable beneficiary) external;
    
    function handleAggregatedOps(
        UserOperation[] calldata opsPerAggregator,
        address payable beneficiary
    ) external;
    
    function getUserOpHash(UserOperation calldata userOp) external view returns (bytes32);
    
    function depositTo(address account) external payable;
    
    function balanceOf(address account) external view returns (uint256);
    
    function withdrawTo(address payable withdrawAddress, uint256 withdrawAmount) external;
}
