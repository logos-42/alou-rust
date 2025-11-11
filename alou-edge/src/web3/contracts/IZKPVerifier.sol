// SPDX-License-Identifier: MIT
pragma solidity ^0.8.30;

/**
 * @title IZKPVerifier
 * @dev ZKP验证器接口合约
 * @notice 定义零知识证明验证的标准接口
 */
interface IZKPVerifier {
    /**
     * @dev 验证身份证明
     * @param proof ZKP证明数组
     * @param didDocument DID文档
     * @param publicKey 公钥
     * @return 验证是否成功
     */
    function verifyProof(
        uint256[8] calldata proof,
        string calldata didDocument,
        string calldata publicKey
    ) external view returns (bool);
    
    /**
     * @dev 验证声誉证明
     * @param proof ZKP证明数组
     * @param agent 智能体地址
     * @param reputation 声誉分数
     * @return 验证是否成功
     */
    function verifyReputationProof(
        uint256[8] calldata proof,
        address agent,
        uint256 reputation
    ) external view returns (bool);
    
    /**
     * @dev 验证批量证明
     * @param proofs 证明数组
     * @param agents 智能体地址数组
     * @param reputations 声誉分数数组
     * @return 验证结果数组
     */
    function verifyBatchProofs(
        uint256[8][] calldata proofs,
        address[] calldata agents,
        uint256[] calldata reputations
    ) external view returns (bool[] memory);
    
    /**
     * @dev 获取验证器版本信息
     * @return 版本字符串
     */
    function getVersion() external view returns (string memory);
    
    /**
     * @dev 检查验证器是否可用
     * @return 是否可用
     */
    function isAvailable() external view returns (bool);
}
