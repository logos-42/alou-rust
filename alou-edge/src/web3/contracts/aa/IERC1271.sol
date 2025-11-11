// SPDX-License-Identifier: MIT
pragma solidity ^0.8.30;

/**
 * @title IERC1271
 * @dev EIP-1271 标准签名验证接口
 * @notice 用于智能合约账户的签名验证
 */
interface IERC1271 {
    /**
     * @dev 验证签名是否有效
     * @param hash 要验证的数据哈希
     * @param signature 签名数据
     * @return magicValue 如果签名有效返回 0x1626ba7e，否则返回其他值
     */
    function isValidSignature(
        bytes32 hash,
        bytes memory signature
    ) external view returns (bytes4 magicValue);
}
