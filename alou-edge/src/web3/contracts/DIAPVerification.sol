// SPDX-License-Identifier: MIT
pragma solidity ^0.8.30;

// solhint-disable max-states-count

import "@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol";
import "@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol";
import "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";
import "@openzeppelin/contracts-upgradeable/security/ReentrancyGuardUpgradeable.sol";
import "./DIAPAgentNetwork.sol";
import "./IZKPVerifier.sol";

/**
 * @title DIAPVerification
 * @dev DIAP智能体网络验证合约
 * @notice 集成Noir ZKP验证，支持身份验证和防恶意行为
 */
contract DIAPVerification is 
    Initializable,
    UUPSUpgradeable,
    OwnableUpgradeable,
    ReentrancyGuardUpgradeable
{
    
    // ============ Custom Errors ============
    
    error InvalidAgentNetworkAddress();
    error AgentIsBlacklisted();
    error InvalidDIDDocumentLength();
    error InvalidPublicKeyLength();
    error InvalidCommitment();
    error InvalidNullifier();
    error NullifierAlreadyUsed();
    error TooManyFailedAttempts();
    error SessionNotFound();
    error SessionNotPending();
    error SessionExpired();
    error InvalidReputationScore();
    error ArraysLengthMismatch();
    error TooManyAgents();
    error AgentAlreadyBlacklisted();
    error AgentNotBlacklisted();
    error NullifierNotUsed();
    error TimeoutMustBeGreaterThanZero();
    error AttemptsMustBeGreaterThanZero();
    error OnlyOwnerCanVerify();
    error OnlyZKPVerifierCanVerify();
    error UnauthorizedVerifier();
    
    // ============ 结构体定义 ============
    
    struct VerificationSession {
        address agent;
        string didDocument;  // DID文档标识符 (支持IPNS名称和IPFS CID)
        string publicKey;
        bytes32 commitment;  // 添加commitment存储
        bytes32 nullifier;   // 添加nullifier存储
        uint256 timestamp;
        VerificationStatus status;
        uint256[8] proof;
        bool isValid;
    }
    
    struct IdentityProof {
        string didDocument;  // DID文档标识符 (支持IPNS名称和IPFS CID)
        string publicKey;
        bytes32 commitment;
        bytes32 nullifier;
        uint256[8] proof;
        uint256 timestamp;
        bool isVerified;
    }
    
    struct ReputationProof {
        address agent;
        uint256 reputation;
        uint256 timestamp;
        uint256[8] proof;
        bool isValid;
    }
    
    enum VerificationStatus {
        PENDING,
        VERIFIED,
        FAILED,
        EXPIRED
    }
    
    // ============ 状态变量 ============
    
    DIAPAgentNetwork public agentNetwork;
    
    mapping(bytes32 => VerificationSession) public verificationSessions;
    mapping(address => IdentityProof) public identityProofs;
    mapping(address => ReputationProof) public reputationProofs;
    
    uint256 public totalVerifications;
    uint256 public totalSuccessfulVerifications;
    uint256 public totalFailedVerifications;
    
    // 验证参数
    uint256 public verificationTimeout;     // 验证超时时间
    uint256 public maxVerificationAttempts; // 最大验证尝试次数
    uint256 public reputationThreshold;     // 声誉阈值
    
    // ZKP验证器地址
    address public zkpVerifier;
    
    // 恶意行为检测
    mapping(address => uint256) public failedAttempts;
    mapping(address => uint256) public lastFailedAttempt;
    mapping(address => bool) public blacklistedAgents;
    
    // nullifier管理
    mapping(bytes32 => bool) public usedNullifiers;
    
    // 字符串长度限制
    uint256 public constant MAX_STRING_LENGTH = 1000;
    
    // 验证模式枚举
    enum VerificationMode {
        OWNER_MANUAL,    // 仅owner手动验证
        ZKP_AUTOMATED,   // ZKP自动验证
        HYBRID          // 混合模式
    }
    
    VerificationMode public verificationMode = VerificationMode.HYBRID;
    
    // ============ 事件定义 ============
    
    event VerificationInitiated(
        bytes32 indexed sessionId,
        address indexed agent,
        string didDocument
    );
    
    event VerificationCompleted(
        bytes32 indexed sessionId,
        address indexed agent,
        bool isValid
    );
    
    event IdentityVerified(
        address indexed agent,
        string didDocument,
        uint256 timestamp
    );
    
    event ReputationVerified(
        address indexed agent,
        uint256 reputation,
        uint256 timestamp
    );
    
    event AgentBlacklisted(
        address indexed agent,
        string reason,
        uint256 timestamp
    );
    
    event AgentWhitelisted(
        address indexed agent,
        uint256 timestamp
    );
    
    event ZKPVerifierUpdated(
        address oldVerifier,
        address newVerifier
    );
    
    event VerificationTimeoutUpdated(uint256 oldTimeout, uint256 newTimeout);
    event MaxVerificationAttemptsUpdated(uint256 oldAttempts, uint256 newAttempts);
    event ReputationThresholdUpdated(uint256 oldThreshold, uint256 newThreshold);
    event NullifierUsed(bytes32 indexed nullifier, address indexed agent);
    event VerificationModeUpdated(VerificationMode newMode);
    
    // ============ 修饰符 ============
    
    /**
     * @dev 仅允许授权验证器调用
     */
    modifier onlyAuthorizedVerifier() {
        if (verificationMode == VerificationMode.OWNER_MANUAL) {
            if (msg.sender != owner()) revert OnlyOwnerCanVerify();
        } else if (verificationMode == VerificationMode.ZKP_AUTOMATED) {
            if (msg.sender != zkpVerifier) revert OnlyZKPVerifierCanVerify();
        } else { // HYBRID
            if (msg.sender != owner() && msg.sender != zkpVerifier) revert UnauthorizedVerifier();
        }
        _;
    }
    
    // ============ 初始化函数 ============
    
    function initialize(address _agentNetwork) public initializer {
        if (_agentNetwork == address(0)) revert InvalidAgentNetworkAddress();
        
        __Ownable_init();
        __UUPSUpgradeable_init();
        __ReentrancyGuard_init();
        
        agentNetwork = DIAPAgentNetwork(payable(_agentNetwork));
        
        // 设置默认参数
        verificationTimeout = 1 hours;
        maxVerificationAttempts = 3;
        reputationThreshold = 1000;
    }
    
    // ============ 身份验证功能 ============
    
    /**
     * @dev 发起身份验证
     * @param didDocument DID文档标识符 (支持IPNS名称和IPFS CID)
     * @param publicKey 公钥
     * @param commitment 承诺
     * @param nullifier 空值
     * @param proof ZKP证明
     * @return sessionId 会话ID
     */
    function initiateIdentityVerification(
        string calldata didDocument,
        string calldata publicKey,
        bytes32 commitment,
        bytes32 nullifier,
        uint256[8] calldata proof
    ) external nonReentrant returns (bytes32) {
        if (blacklistedAgents[msg.sender]) revert AgentIsBlacklisted();
        if (
            bytes(didDocument).length == 0
            || bytes(didDocument).length > MAX_STRING_LENGTH
        ) revert InvalidDIDDocumentLength();
        if (
            bytes(publicKey).length == 0
            || bytes(publicKey).length > MAX_STRING_LENGTH
        ) revert InvalidPublicKeyLength();
        if (commitment == bytes32(0)) revert InvalidCommitment();
        if (nullifier == bytes32(0)) revert InvalidNullifier();
        if (usedNullifiers[nullifier]) revert NullifierAlreadyUsed(); // 防止重放攻击
        
        // 立即标记nullifier为已使用，防止重放攻击
        usedNullifiers[nullifier] = true;
        
        // 检查验证尝试次数
        if (failedAttempts[msg.sender] >= maxVerificationAttempts) revert TooManyFailedAttempts();
        
        // 生成防碰撞的sessionId
        bytes32 sessionId = keccak256(abi.encodePacked(
            msg.sender,
            didDocument,
            publicKey,
            commitment,
            nullifier,
            block.timestamp,
            block.number
        ));
        
        // 正确保存所有会话数据
        verificationSessions[sessionId] = VerificationSession({
            agent: msg.sender,
            didDocument: didDocument,
            publicKey: publicKey,
            commitment: commitment,  // 保存commitment
            nullifier: nullifier,    // 保存nullifier
            timestamp: block.timestamp,
            status: VerificationStatus.PENDING,
            proof: proof,
            isValid: false
        });
        
        totalVerifications++;
        
        emit VerificationInitiated(sessionId, msg.sender, didDocument);
        
        return sessionId;
    }
    
    /**
     * @dev 验证身份
     * @param sessionId 会话ID
     * @return 是否验证成功
     */
    function verifyIdentity(bytes32 sessionId) external onlyAuthorizedVerifier returns (bool) {
        VerificationSession storage session = verificationSessions[sessionId];
        if (session.timestamp == 0) revert SessionNotFound();
        if (session.status != VerificationStatus.PENDING) revert SessionNotPending();
        if (block.timestamp > session.timestamp + verificationTimeout) revert SessionExpired();
        
        // 验证ZKP证明
        uint256[8] memory proofCopy = session.proof;
        bool isValid = _verifyZKProof(proofCopy, session.didDocument, session.publicKey);
        
        if (isValid) {
            session.status = VerificationStatus.VERIFIED;
            session.isValid = true;
            totalSuccessfulVerifications++;
            
            // nullifier已经在initiateIdentityVerification中被标记为已使用，无需重复标记
            
            // 保存身份证明，使用会话中的commitment和nullifier
            identityProofs[session.agent] = IdentityProof({
                didDocument: session.didDocument,
                publicKey: session.publicKey,
                commitment: session.commitment,  // 使用会话中的commitment
                nullifier: session.nullifier,    // 使用会话中的nullifier
                proof: session.proof,
                timestamp: block.timestamp,
                isVerified: true
            });
            
            // 重置失败尝试次数
            failedAttempts[session.agent] = 0;
            
            emit VerificationCompleted(sessionId, session.agent, true);
            emit IdentityVerified(session.agent, session.didDocument, block.timestamp);
            emit NullifierUsed(session.nullifier, session.agent);
            
            return true;
        } else {
            session.status = VerificationStatus.FAILED;
            session.isValid = false;
            totalFailedVerifications++;
            
            // 增加失败尝试次数
            failedAttempts[session.agent]++;
            lastFailedAttempt[session.agent] = block.timestamp;
            
            // 检查是否需要加入黑名单
            if (failedAttempts[session.agent] >= maxVerificationAttempts) {
                blacklistedAgents[session.agent] = true;
                emit AgentBlacklisted(session.agent, "Too many failed verification attempts", block.timestamp);
            }
            
            emit VerificationCompleted(sessionId, session.agent, false);
            
            return false;
        }
    }
    
    // ============ 声誉验证功能 ============
    
    /**
     * @dev 验证声誉
     * @param agent 智能体地址
     * @param reputation 声誉分数
     * @param proof ZKP证明
     * @return 是否验证成功
     */
    function verifyReputation(
        address agent,
        uint256 reputation,
        uint256[8] calldata proof
    ) external onlyAuthorizedVerifier nonReentrant returns (bool) {
        if (blacklistedAgents[agent]) revert AgentIsBlacklisted();
        if (reputation > 10000) revert InvalidReputationScore();
        
        // 调用外部ZKP验证器进行实际验证
        bool isValid = _callExternalReputationVerifier(proof, agent, reputation);
        
        if (isValid) {
            reputationProofs[agent] = ReputationProof({
                agent: agent,
                reputation: reputation,
                timestamp: block.timestamp,
                proof: proof,
                isValid: true
            });
            
            emit ReputationVerified(agent, reputation, block.timestamp);
            
            return true;
        }
        
        return false;
    }
    
    // ============ 批量验证功能 ============
    
    /**
     * @dev 批量验证智能体
     * @param agents 智能体地址数组
     * @param proofs ZKP证明数组
     * @return 验证结果数组
     */
    function batchVerifyAgents(
        address[] calldata agents,
        uint256[8][] calldata proofs
    ) external view onlyOwner returns (bool[] memory) {
        if (agents.length != proofs.length) revert ArraysLengthMismatch();
        if (agents.length > 10) revert TooManyAgents(); // 限制批量大小
        
        bool[] memory results = new bool[](agents.length);
        
        for (uint256 i = 0; i < agents.length;) {
            if (blacklistedAgents[agents[i]]) {
                results[i] = false;
                continue;
            }
            
            // 获取智能体信息
            try agentNetwork.getAgent(agents[i]) returns (DIAPAgentNetwork.Agent memory agent) {
                if (agent.isActive) {
                    uint256[8] memory proofCopy = proofs[i];
                    results[i] = _verifyZKProof(proofCopy, agent.identifier, agent.publicKey);
                } else {
                    results[i] = false;
                }
            } catch {
                results[i] = false;
            }
            unchecked {
                ++i;  // Gas 优化
            }
        }
        
        return results;
    }
    
    // ============ 恶意行为检测 ============
    
    /**
     * @dev 检测恶意行为
     * @param agent 智能体地址
     * @param behaviorType 行为类型
     */
    function detectMaliciousBehavior(
        address agent,
        string calldata behaviorType,
        string calldata /* evidence */
    ) external onlyOwner {
        if (blacklistedAgents[agent]) revert AgentAlreadyBlacklisted();
        
        // 根据行为类型采取不同措施
        if (keccak256(bytes(behaviorType)) == keccak256("SPAM")) {
            // 垃圾信息行为
            _handleSpamBehavior(agent);
        } else if (keccak256(bytes(behaviorType)) == keccak256("FRAUD")) {
            // 欺诈行为
            _handleFraudBehavior(agent);
        } else if (keccak256(bytes(behaviorType)) == keccak256("ATTACK")) {
            // 攻击行为
            _handleAttackBehavior(agent);
        }
        
        emit AgentBlacklisted(agent, behaviorType, block.timestamp);
    }
    
    /**
     * @dev 移除黑名单
     * @param agent 智能体地址
     */
    function removeFromBlacklist(address agent) external onlyOwner {
        if (!blacklistedAgents[agent]) revert AgentNotBlacklisted();
        
        blacklistedAgents[agent] = false;
        failedAttempts[agent] = 0;
        
        emit AgentWhitelisted(agent, block.timestamp);
    }
    
    /**
     * @dev 重置nullifier（申诉成功后）
     * @param nullifier 要重置的nullifier
     */
    function resetNullifier(bytes32 nullifier) external onlyOwner {
        if (!usedNullifiers[nullifier]) revert NullifierNotUsed();
        usedNullifiers[nullifier] = false;
    }
    
    // ============ 内部函数 ============
    
    /**
     * @dev 验证ZKP证明
     * @param proof ZKP证明
     * @param identifier DID 标识符（CID/IPNS）
     * @param publicKey 公钥
     * @return 是否有效
     */
    function _verifyZKProof(
        uint256[8] memory proof,
        string memory identifier,
        string memory publicKey
    ) internal view returns (bool) {
        if (zkpVerifier == address(0)) {
            // 默认验证逻辑 (简化实现)
            return proof.length == 8 && proof[0] != 0 && bytes(identifier).length > 0;
        }
        
        // 使用ABI-safe调用外部ZKP验证器
        try IZKPVerifier(zkpVerifier).verifyProof(proof, identifier, publicKey) returns (bool result) {
            return result;
        } catch {
            // 验证器调用失败时返回false
            return false;
        }
    }
    
    /**
     * @dev 调用外部声誉验证器
     * @param proof ZKP证明
     * @param agent 智能体地址
     * @param reputation 声誉分数
     * @return 是否有效
     */
    function _callExternalReputationVerifier(
        uint256[8] calldata proof,
        address agent,
        uint256 reputation
    ) internal view returns (bool) {
        if (zkpVerifier == address(0)) {
            return false; // 没有验证器时拒绝
        }
        
        try IZKPVerifier(zkpVerifier).verifyReputationProof(proof, agent, reputation) returns (bool result) {
            return result;
        } catch {
            // 验证器调用失败时返回false
            return false;
        }
    }
    
    /**
     * @dev 验证声誉证明
     * @param reputation 声誉分数
     * @return 是否有效
     */
    function _verifyReputationProof(
        uint256[8] calldata /* proof */,
        address /* agent */,
        uint256 reputation
    ) internal pure returns (bool) {
        // 这里应该集成实际的声誉ZKP验证逻辑
        // 简化实现
        return reputation > 0;
    }
    
    function _handleSpamBehavior(address agent) internal {
        // 处理垃圾信息行为
        blacklistedAgents[agent] = true;
    }
    
    function _handleFraudBehavior(address agent) internal {
        // 处理欺诈行为
        blacklistedAgents[agent] = true;
    }
    
    function _handleAttackBehavior(address agent) internal {
        // 处理攻击行为
        blacklistedAgents[agent] = true;
    }
    
    // ============ 管理函数 ============
    
    function setZKPVerifier(address _verifier) external onlyOwner {
        address oldVerifier = zkpVerifier;
        zkpVerifier = _verifier;
        emit ZKPVerifierUpdated(oldVerifier, _verifier);
    }
    
    /**
     * @dev 设置验证模式
     * @param _mode 验证模式
     */
    function setVerificationMode(VerificationMode _mode) external onlyOwner {
        verificationMode = _mode;
        emit VerificationModeUpdated(_mode);
    }
    
    function setVerificationTimeout(uint256 _timeout) external onlyOwner {
        if (_timeout == 0) revert TimeoutMustBeGreaterThanZero();
        uint256 oldTimeout = verificationTimeout;
        verificationTimeout = _timeout;
        emit VerificationTimeoutUpdated(oldTimeout, _timeout);
    }
    
    function setMaxVerificationAttempts(uint256 _attempts) external onlyOwner {
        if (_attempts == 0) revert AttemptsMustBeGreaterThanZero();
        uint256 oldAttempts = maxVerificationAttempts;
        maxVerificationAttempts = _attempts;
        emit MaxVerificationAttemptsUpdated(oldAttempts, _attempts);
    }
    
    function setReputationThreshold(uint256 _threshold) external onlyOwner {
        uint256 oldThreshold = reputationThreshold;
        reputationThreshold = _threshold;
        emit ReputationThresholdUpdated(oldThreshold, _threshold);
    }
    
    // solhint-disable-next-line no-empty-blocks
    function _authorizeUpgrade(address newImplementation) internal override onlyOwner {}
    
    // ============ 查询函数 ============
    
    function getVerificationSession(bytes32 sessionId) external view returns (VerificationSession memory) {
        return verificationSessions[sessionId];
    }
    
    function getIdentityProof(address agent) external view returns (IdentityProof memory) {
        return identityProofs[agent];
    }
    
    function getReputationProof(address agent) external view returns (ReputationProof memory) {
        return reputationProofs[agent];
    }
    
    function isAgentBlacklisted(address agent) external view returns (bool) {
        return blacklistedAgents[agent];
    }
    
    function getFailedAttempts(address agent) external view returns (uint256) {
        return failedAttempts[agent];
    }
    
    function getVerificationStats() external view returns (
        uint256 _totalVerifications,
        uint256 _totalSuccessfulVerifications,
        uint256 _totalFailedVerifications
    ) {
        return (totalVerifications, totalSuccessfulVerifications, totalFailedVerifications);
    }
    
    function isIdentityVerified(address agent) external view returns (bool) {
        return identityProofs[agent].isVerified;
    }
    
    function isReputationVerified(address agent) external view returns (bool) {
        return reputationProofs[agent].isValid;
    }
    
    /**
     * @dev 检查nullifier是否已被使用
     * @param nullifier nullifier值
     * @return 是否已被使用
     */
    function isNullifierUsed(bytes32 nullifier) external view returns (bool) {
        return usedNullifiers[nullifier];
    }
    
    /**
     * @dev 获取验证器状态
     * @return verifier 验证器地址
     * @return isAvailable 是否可用
     */
    function getVerifierStatus() external view returns (address verifier, bool isAvailable) {
        verifier = zkpVerifier;
        if (verifier == address(0)) {
            isAvailable = false;
        } else {
            try IZKPVerifier(verifier).isAvailable() returns (bool available) {
                isAvailable = available;
            } catch {
                isAvailable = false;
            }
        }
    }
}
