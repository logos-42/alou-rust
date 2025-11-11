// SPDX-License-Identifier: MIT
pragma solidity ^0.8.30;

// solhint-disable max-states-count

import "@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol";
import "@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol";
import "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";
import "@openzeppelin/contracts-upgradeable/security/ReentrancyGuardUpgradeable.sol";
import "@openzeppelin/contracts-upgradeable/security/PausableUpgradeable.sol";
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";

/**
 * @title DIAPAgentNetwork
 * @dev 去中心化智能体网络主合约
 * @notice 基于DIAP协议实现智能体身份验证、通信和治理
 */
contract DIAPAgentNetwork is 
    Initializable, 
    UUPSUpgradeable, 
    OwnableUpgradeable, 
    ReentrancyGuardUpgradeable,
    PausableUpgradeable 
{
    
    // ============ Custom Errors ============
    
    error InvalidTokenAddress();
    error AgentAlreadyRegistered();
    error InsufficientStakeAmount();
    error StakeAmountTooLarge();
    error InvalidDIDDocument();
    error InvalidIdentifierFormat();
    error MustUseIPNSOrIPFS();
    error DIDAlreadyExists();
    error InsufficientTokenBalance();
    error InsufficientTokenAllowance();
    error TokenTransferFailed();
    error AgentNotRegistered();
    error LockPeriodNotEnded();
    error AgentAlreadyVerified();
    error VerificationContractNotSet();
    error InvalidZKPProof();
    error TargetAgentNotActive();
    error InvalidMessageCID();
    error InvalidConsumerAddress();
    error CannotCreateServiceForSelf();
    error ConsumerNotActive();
    error ConsumerNotVerified();
    error InvalidPrice();
    error ServiceTypeRequired();
    error PriceTooHigh();
    error NotServiceProvider();
    error ServiceAlreadyCompleted();
    error InvalidResultCID();
    error ServiceNotFound();
    error ServiceExpired();
    error TotalEarningsOverflow();
    error TotalServicesOverflow();
    error ReputationOverflow();
    error InsufficientContractBalance();
    error InvalidReputationScore();
    error FeeRateTooHigh();
    error InvalidVerificationContractAddress();
    error InvalidTreasuryAddress();
    error TreasuryAddressNotSet();
    error NoFeesToWithdraw();
    error AgentNotActive();
    
    // ============ 枚举定义 ============
    
    /**
     * @dev 标识符类型枚举
     */
    enum IdentifierType {
        UNKNOWN,      // 未知类型
        IPFS_CID,     // IPFS内容标识符
        IPNS_NAME     // IPNS名称（推荐）
    }
    
    // ============ 结构体定义 ============
    
    struct Agent {
        string identifier;        // CID 标识符（CID 或 IPNS 名称）
                                  // 建议使用 IPNS 名称指向可更新的文档
                                  // 示例: k51qzi5uqu5dlvj2baxnqndepeb86cbk3ng7n3i46uzyxzyqj2xjonzllnv0v8
        string publicKey;          // 公钥
        uint128 stakedAmount;     // 质押代币数量 (打包优化)
        uint128 totalEarnings;    // 总收益 (打包优化)
        uint64 reputation;        // 声誉分数 (0-65535，打包优化)
        uint64 registrationTime;  // 注册时间 (打包优化)
        uint64 lastActivity;      // 最后活动时间 (打包优化)
        uint32 totalServices;     // 提供服务次数 (打包优化)
        bool isActive;            // 是否活跃 (打包优化)
        bool isVerified;          // 是否通过ZKP验证 (打包优化)
        // ERC-4337 支持
        address aaAccount;        // ERC-4337 账户地址
        bool isAAAccount;         // 是否使用 AA 账户
    }
    
    struct Message {
        address fromAgent;
        address toAgent;
        string messageCID;        // IPFS消息CID
        uint256 timestamp;
        bool isVerified;          // ZKP验证状态
        uint256 fee;             // 消息费用
    }
    
    struct Service {
        address provider;
        address consumer;
        string serviceType;       // 服务类型
        uint256 price;           // 服务价格
        uint256 timestamp;
        bool isCompleted;        // 是否完成
        string resultCID;        // 结果CID
    }
    
    // ============ 状态变量 ============
    
    IERC20 public token; // DIAP代币合约
    address public verification; // DIAPVerification合约地址
    address public accountFactory; // DIAPAccountFactory合约地址
    
    mapping(address => Agent) public agents;
    mapping(string => address) public identifierToAgent;  // CID/IPNS 标识符到地址映射
    mapping(bytes32 => Message) public messages;
    mapping(uint256 => Service) public services;
    
    // Gas优化：可枚举的智能体列表
    address[] public agentList;
    mapping(address => uint256) public agentIndex; // 智能体地址到索引的映射
    
    uint256 public totalAgents;
    uint256 public totalMessages;
    uint256 public totalServices;
    uint256 public totalVolume;  // 总交易量
    
    // 网络参数
    uint256 public registrationFee;     // 注册费用
    uint256 public messageFee;          // 消息费用
    uint256 public serviceFeeRate;      // 服务费用率 (基点)
    uint256 public minStakeAmount;      // 最小质押数量
    uint256 public reputationThreshold; // 声誉阈值
    uint256 public lockPeriod;          // 锁定期
    uint256 public totalStaked;         // 总质押量
    
    // 奖励参数
    uint256 public rewardRate;          // 奖励率
    uint256 public lastRewardTime;      // 上次奖励时间
    uint256 public totalRewards;        // 总奖励
    
    // Gas优化：分批奖励分配
    uint256 public constant BATCH_SIZE = 50; // 每批处理的智能体数量
    uint256 public lastProcessedIndex;       // 上次处理的索引
    uint256 public pendingRewards;           // 待分配奖励
    
    // 混合存储：链上存储关键数据，链下存储详细数据
    mapping(address => string) public agentMetadataCID; // 智能体详细元数据CID（链下）
    mapping(uint256 => string) public serviceMetadataCID; // 服务详细元数据CID（链下）
    
    // 费用分配
    uint256 public accumulatedFees;  // 累积的注册费和消息费
    address public treasuryAddress;  // 金库地址
    
    // ============ 事件定义 ============
    
    event AgentRegistered(
        address indexed agent, 
        string identifier,  // 支持IPNS名称和IPFS CID
        uint256 stakedAmount
    );
    
    event AgentRegisteredWithIPNS(
        address indexed agent,
        string ipnsName,
        uint256 stakedAmount,
        uint256 timestamp
    );
    
    event AgentUnstaked(
        address indexed agent,
        uint256 stakedAmount
    );
    
    event AgentVerified(
        address indexed agent, 
        bool isVerified
    );
    
    event MessageSent(
        bytes32 indexed messageId, 
        address from, 
        address to, 
        uint256 fee
    );
    
    event ServiceCreated(
        uint256 indexed serviceId, 
        address provider, 
        address consumer, 
        uint256 price
    );
    
    event ServiceCompleted(
        uint256 indexed serviceId, 
        string resultCID, 
        uint256 reward
    );
    
    event ReputationUpdated(
        address indexed agent, 
        uint256 oldReputation, 
        uint256 newReputation
    );
    
    event RewardsDistributed(
        uint256 totalAmount, 
        uint256 timestamp
    );
    
    event FeesWithdrawn(
        address indexed to,
        uint256 amount,
        uint256 timestamp
    );
    
    event TreasuryAddressUpdated(
        address indexed oldAddress,
        address indexed newAddress
    );
    
    event AAAccountCreated(
        address indexed agent,
        address indexed aaAccount,
        uint256 timestamp
    );
    
    event AccountFactoryUpdated(
        address indexed oldFactory,
        address indexed newFactory
    );
    
    // ============ 修饰符 ============
    
    modifier onlyRegisteredAgent() {
        if (!agents[msg.sender].isActive) revert AgentNotRegistered();
        _;
    }
    
    modifier onlyVerifiedAgent() {
        if (!agents[msg.sender].isVerified) revert AgentNotActive();
        _;
    }
    
    modifier validReputation(uint256 reputation) {
        if (reputation > 10000) revert InvalidReputationScore();
        _;
    }
    
    // ============ 初始化函数 ============
    
    function initialize(address _token) public initializer {
        if (_token == address(0)) revert InvalidTokenAddress();
        
        __Ownable_init();
        __UUPSUpgradeable_init();
        __ReentrancyGuard_init();
        __Pausable_init();
        
        // 设置代币合约
        token = IERC20(_token);
        
        // 设置默认参数
        registrationFee = 10 * 10**18;      // 10 代币
        messageFee = 1 * 10**18;             // 1 代币
        serviceFeeRate = 300;                // 3%
        minStakeAmount = 100 * 10**18;      // 100 代币
        reputationThreshold = 1000;          // 1000 声誉点
        lockPeriod = 30 days;                // 30天锁定期
        rewardRate = 5;                      // 5% 年化
        lastRewardTime = block.timestamp;
        totalStaked = 0;                     // 初始总质押量为0
    }
    
    // ============ 智能体管理 ============
    
    /**
     * @dev 注册智能体（传统 EOA 方式）
     * @param identifier CID 标识符（CID 或 IPNS 名称）
     *                   推荐使用 IPNS 名称映射到可更新的 DID 文档
     *                   示例: k51qzi5uqu5dlvj2baxnqndepeb86cbk3ng7n3i46uzyxzyqj2xjonzllnv0v8
     * @param publicKey 公钥
     * @param stakedAmount 质押代币数量
     */
    function registerAgent(
        string calldata identifier,
        string calldata publicKey,
        uint256 stakedAmount
    ) external nonReentrant whenNotPaused {
        _registerAgent(msg.sender, identifier, publicKey, stakedAmount, address(0), false);
    }
    
    /**
     * @dev 注册智能体（ERC-4337 AA 账户方式）
     * @param identifier DID 标识符（CID 或 IPNS 名称）
     * @param publicKey 公钥
     * @param stakedAmount 质押代币数量
     * @param salt 用于创建 AA 账户的盐值
     */
    function registerAgentWithAA(
        string calldata identifier,
        string calldata publicKey,
        uint256 stakedAmount,
        uint256 salt
    ) external nonReentrant whenNotPaused returns (address aaAccount) {
        require(accountFactory != address(0), "Account factory not set");
        
        // 调用工厂创建 AA 账户
        // solhint-disable-next-line avoid-low-level-calls
        (bool success, bytes memory data) = accountFactory.call(
            abi.encodeWithSignature("createAccount(address,uint256)", msg.sender, salt)
        );
        require(success, "Failed to create AA account");
        // createAccount 返回 DIAPAccount 合约实例，需要转换为地址
        aaAccount = address(abi.decode(data, (address)));
        
        // 注册智能体
        _registerAgent(msg.sender, identifier, publicKey, stakedAmount, aaAccount, true);
        
        emit AAAccountCreated(msg.sender, aaAccount, block.timestamp);
        
        return aaAccount;
    }
    
    /**
     * @dev 内部注册函数
     */
    function _registerAgent(
        address agentAddress,
        string calldata identifier,
        string calldata publicKey,
        uint256 stakedAmount,
        address aaAccount,
        bool isAA
    ) internal {
        if (agents[agentAddress].isActive) revert AgentAlreadyRegistered();
        if (stakedAmount < minStakeAmount) revert InsufficientStakeAmount();
        if (stakedAmount > type(uint128).max) revert StakeAmountTooLarge();
        if (bytes(identifier).length == 0) revert InvalidDIDDocument();
        if (!_isValidIdentifier(identifier)) revert InvalidIdentifierFormat();
        
        // 优先使用IPNS，检查标识符类型
        IdentifierType idType = _getIdentifierType(identifier);
        if (idType != IdentifierType.IPNS_NAME && idType != IdentifierType.IPFS_CID) revert MustUseIPNSOrIPFS();
        
        if (identifierToAgent[identifier] != address(0)) revert DIDAlreadyExists();
        
        // 计算总费用（质押金额 + 注册费用）
        uint256 totalCost = stakedAmount + registrationFee;
        
        // 确定代币转移的目标地址
        address tokenDestination = isAA ? aaAccount : address(this);
        
        // 检查代币余额和授权
        if (token.balanceOf(msg.sender) < totalCost) revert InsufficientTokenBalance();
        if (token.allowance(msg.sender, address(this)) < totalCost) revert InsufficientTokenAllowance();
        
        // 转移注册费到合约
        if (!token.transferFrom(msg.sender, address(this), registrationFee)) revert TokenTransferFailed();
        accumulatedFees += registrationFee;
        
        // 转移质押代币（EOA 到合约，AA 到 AA 账户）
        if (!token.transferFrom(msg.sender, tokenDestination, stakedAmount)) revert TokenTransferFailed();
        
        // 创建智能体
        agents[agentAddress] = Agent({
            identifier: identifier,
            publicKey: publicKey,
            stakedAmount: uint128(stakedAmount),
            totalEarnings: 0,
            reputation: 1000,
            registrationTime: uint64(block.timestamp),
            lastActivity: uint64(block.timestamp),
            totalServices: 0,
            isActive: true,
            isVerified: false,
            aaAccount: aaAccount,
            isAAAccount: isAA
        });
        
        identifierToAgent[identifier] = agentAddress;
        
        // Gas优化：维护可枚举列表
        agentIndex[agentAddress] = agentList.length;
        agentList.push(agentAddress);
        
        totalAgents++;
        totalStaked += stakedAmount;
        
        emit AgentRegistered(agentAddress, identifier, stakedAmount);
    }
    
    /**
     * @dev 取消注册智能体并提取质押代币
     */
    function unstakeAgent() external nonReentrant {
        Agent storage agent = agents[msg.sender];
        if (!agent.isActive) revert AgentNotRegistered();
        if (block.timestamp < agent.registrationTime + lockPeriod) revert LockPeriodNotEnded();
        
        uint256 stakedAmount = agent.stakedAmount;
        
        // 标记智能体为非活跃
        agent.isActive = false;
        
        // Gas优化：从可枚举列表中移除（使用swap-and-pop模式）
        uint256 index = agentIndex[msg.sender];
        uint256 lastIndex = agentList.length - 1;
        if (index != lastIndex) {
            address lastAgent = agentList[lastIndex];
            agentList[index] = lastAgent;
            agentIndex[lastAgent] = index;
        }
        agentList.pop();
        delete agentIndex[msg.sender];
        
        totalAgents--;
        totalStaked -= stakedAmount;
        
        // 返还质押代币
        if (!token.transfer(msg.sender, stakedAmount)) revert TokenTransferFailed();
        
        emit AgentUnstaked(msg.sender, stakedAmount);
    }
    
    /**
     * @dev 验证智能体身份 (通过ZKP)
     * @param agent 智能体地址
     * @param proof ZKP证明
     */
    function verifyAgent(
        address agent,
        uint256[8] calldata proof
    ) external onlyOwner {
        if (!agents[agent].isActive) revert AgentNotRegistered();
        if (agents[agent].isVerified) revert AgentAlreadyVerified();
        
        // 调用DIAPVerification合约进行实际验证
        // 这里需要先检查verification合约是否已设置
        if (address(verification) == address(0)) revert VerificationContractNotSet();
        
        // 通过verification合约验证身份
        // 注意：这里需要verification合约提供相应的验证接口
        bool isValid = _verifyAgentThroughVerification(agent, proof);
        if (!isValid) revert InvalidZKPProof();
        
        agents[agent].isVerified = true;
        agents[agent].reputation += 1000; // 验证奖励
        
        emit AgentVerified(agent, true);
        emit ReputationUpdated(agent, agents[agent].reputation - 1000, agents[agent].reputation);
    }
    
    /**
     * @dev 发送消息
     * @param toAgent 接收方智能体
     * @param messageCID 消息CID
     */
    function sendMessage(
        address toAgent,
        string calldata messageCID
    ) external onlyRegisteredAgent nonReentrant {
        if (!agents[toAgent].isActive) revert TargetAgentNotActive();
        if (bytes(messageCID).length == 0) revert InvalidMessageCID();
        
        // 检查代币余额和授权
        if (token.balanceOf(msg.sender) < messageFee) revert InsufficientTokenBalance();
        if (token.allowance(msg.sender, address(this)) < messageFee) revert InsufficientTokenAllowance();
        
        // 转移消息费用
        if (!token.transferFrom(msg.sender, address(this), messageFee)) revert TokenTransferFailed();
        
        // 累积消息费
        accumulatedFees += messageFee;
        
        bytes32 messageId = keccak256(abi.encodePacked(
            msg.sender,
            toAgent,
            messageCID,
            block.timestamp
        ));
        
        messages[messageId] = Message({
            fromAgent: msg.sender,
            toAgent: toAgent,
            messageCID: messageCID,
            timestamp: block.timestamp,
            isVerified: false,
            fee: messageFee
        });
        
        totalMessages++;
        totalVolume += messageFee;
        
        // 更新活动时间
        agents[msg.sender].lastActivity = uint64(block.timestamp);
        agents[toAgent].lastActivity = uint64(block.timestamp);
        
        emit MessageSent(messageId, msg.sender, toAgent, messageFee);
    }
    
    /**
     * @dev 创建服务
     * @param consumer 消费者地址
     * @param serviceType 服务类型
     * @param price 服务价格
     */
    function createService(
        address consumer,
        string calldata serviceType,
        uint256 price
    ) external onlyVerifiedAgent nonReentrant whenNotPaused {
        if (consumer == address(0)) revert InvalidConsumerAddress();
        if (consumer == msg.sender) revert CannotCreateServiceForSelf();
        if (!agents[consumer].isActive) revert ConsumerNotActive();
        if (!agents[consumer].isVerified) revert ConsumerNotVerified();
        if (price == 0) revert InvalidPrice();
        if (bytes(serviceType).length == 0) revert ServiceTypeRequired();
        
        // 检查服务价格是否在合理范围内
        if (price > 1000000 * 10**18) revert PriceTooHigh(); // 最大100万代币
        
        uint256 serviceId = totalServices;
        services[serviceId] = Service({
            provider: msg.sender,
            consumer: consumer,
            serviceType: serviceType,
            price: price,
            timestamp: block.timestamp,
            isCompleted: false,
            resultCID: ""
        });
        
        totalServices++;
        
        emit ServiceCreated(serviceId, msg.sender, consumer, price);
    }
    
    /**
     * @dev 完成服务
     * @param serviceId 服务ID
     * @param resultCID 结果CID
     */
    function completeService(
        uint256 serviceId,
        string calldata resultCID
    ) external onlyVerifiedAgent nonReentrant whenNotPaused {
        Service storage service = services[serviceId];
        if (service.provider != msg.sender) revert NotServiceProvider();
        if (service.isCompleted) revert ServiceAlreadyCompleted();
        if (bytes(resultCID).length == 0) revert InvalidResultCID();
        if (service.timestamp == 0) revert ServiceNotFound();
        
        // 检查服务是否过期（例如30天）
        if (block.timestamp > service.timestamp + 30 days) revert ServiceExpired();
        
        // 计算奖励和费用
        uint256 reward = service.price * (10000 - serviceFeeRate) / 10000;
        // uint256 fee = service.price - reward; // 费用留在合约中
        
        // 更新智能体信息（在转账之前，防止重入攻击）
        Agent storage agent = agents[msg.sender];
        uint256 oldReputation = agent.reputation;
        
        // 安全检查：确保不会溢出
        if (uint256(agent.totalEarnings) + reward > type(uint128).max) revert TotalEarningsOverflow();
        if (agent.totalServices >= type(uint32).max) revert TotalServicesOverflow();
        if (agent.reputation + 10 > type(uint64).max) revert ReputationOverflow();
        
        // 更新状态（Checks-Effects-Interactions 模式）
        service.isCompleted = true;
        service.resultCID = resultCID;
        agent.totalEarnings += uint128(reward);
        agent.totalServices++;
        agent.reputation += 10; // 完成服务奖励声誉
        totalVolume += service.price;
        
        // 外部调用放在最后（Interactions）
        // 检查合约代币余额是否足够支付奖励
        if (token.balanceOf(address(this)) >= reward) {
            // 从合约余额支付奖励
            if (!token.transfer(msg.sender, reward)) revert TokenTransferFailed();
        } else {
            // 如果余额不足，需要动态铸造（需要owner权限）
            // 这里简化处理，实际应用中需要更复杂的逻辑
            revert InsufficientContractBalance();
        }
        
        emit ServiceCompleted(serviceId, resultCID, reward);
        emit ReputationUpdated(msg.sender, oldReputation, agent.reputation);
    }
    
    // ============ 声誉管理 ============
    
    /**
     * @dev 更新声誉分数
     * @param agent 智能体地址
     * @param reputationChange 声誉变化
     */
    function updateReputation(
        address agent,
        int256 reputationChange
    ) external onlyOwner validReputation(agents[agent].reputation) {
        uint256 oldReputation = agents[agent].reputation;
        uint256 newReputation;
        
        if (reputationChange > 0) {
            newReputation = oldReputation + uint256(reputationChange);
            if (newReputation > 10000) newReputation = 10000;
            if (newReputation > type(uint64).max) newReputation = type(uint64).max;
        } else {
            uint256 decrease = uint256(-reputationChange);
            if (decrease >= oldReputation) {
                newReputation = 0;
            } else {
                newReputation = oldReputation - decrease;
            }
        }
        
        agents[agent].reputation = uint64(newReputation);
        emit ReputationUpdated(agent, oldReputation, newReputation);
    }
    
    // ============ 奖励分配 ============
    
    /**
     * @dev 分配奖励（使用代币而非ETH）
     */
    function distributeRewards() external onlyOwner {
        uint256 timeElapsed = block.timestamp - lastRewardTime;
        uint256 currentTotalStaked = _getTotalStaked();
        
        if (currentTotalStaked > 0 && timeElapsed > 0) {
            uint256 rewards = currentTotalStaked * rewardRate * timeElapsed / (365 days * 10000);
            
            // 检查合约代币余额是否足够
            if (rewards > 0 && token.balanceOf(address(this)) >= rewards + currentTotalStaked) {
                _distributeRewardsToStakers(rewards);
                totalRewards += rewards;
                lastRewardTime = block.timestamp;
                
                emit RewardsDistributed(rewards, block.timestamp);
            }
        }
    }
    
    // ============ 内部函数 ============
    
    /**
     * @dev 验证标识符格式是否有效
     * @param identifier 标识符字符串
     * @return 是否有效
     */
    function _isValidIdentifier(string memory identifier) internal pure returns (bool) {
        bytes memory b = bytes(identifier);
        uint256 len = b.length;
        
        // 长度检查：最小10字符，最大100字符
        if (len < 10 || len > 100) return false;
        
        // 检查是否以有效前缀开头
        if (len >= 2) {
            // CIDv0: Qm (0x51, 0x6d)
            if (b[0] == 0x51 && b[1] == 0x6d) return true;
            
            // CIDv1: bafy, bafk, bafz (0x62, 0x61, 0x66)
            if (len >= 4 && b[0] == 0x62 && b[1] == 0x61 && b[2] == 0x66) return true;
            
            // IPNS: k (base36编码的PeerID) (0x6b)
            if (b[0] == 0x6b && len >= 50) return true;
        }
        
        return false;
    }
    
    /**
     * @dev 获取标识符类型
     * @param identifier 标识符字符串
     * @return 标识符类型
     */
    function _getIdentifierType(string memory identifier) internal pure returns (IdentifierType) {
        bytes memory b = bytes(identifier);
        uint256 len = b.length;
        
        if (len < 2) return IdentifierType.UNKNOWN;
        
        // IPNS 名称检测 (k开头，长度50-65) (0x6b)
        if (b[0] == 0x6b && len >= 50 && len <= 65) {
            return IdentifierType.IPNS_NAME;
        }
        
        // IPFS CID 检测 (Q=0x51, m=0x6d, b=0x62, a=0x61, f=0x66)
        if ((b[0] == 0x51 && b[1] == 0x6d) || 
            (len >= 4 && b[0] == 0x62 && b[1] == 0x61 && b[2] == 0x66)) {
            return IdentifierType.IPFS_CID;
        }
        
        return IdentifierType.UNKNOWN;
    }
    
    function _verifyZKProof(uint256[8] calldata proof) internal pure returns (bool) {
        // 这里应该集成实际的Noir ZKP验证逻辑
        // 简化实现，实际需要调用ZKP验证合约
        return proof.length == 8 && proof[0] != 0;
    }
    
    function _getTotalStaked() internal view returns (uint256) {
        return totalStaked;
    }
    
    function _distributeRewardsToStakers(uint256 rewardsAmount) internal {
        // 根据质押比例分配奖励
        if (totalStaked == 0) return;
        
        // 将奖励加入待分配池
        pendingRewards += rewardsAmount;
        emit RewardsDistributed(rewardsAmount, block.timestamp);
    }
    
    /**
     * @dev 分批处理奖励分配（Gas优化）
     * @return 是否完成所有分配
     */
    function processRewardBatch() external onlyOwner returns (bool) {
        if (pendingRewards == 0 || totalStaked == 0) return true;
        
        uint256 startIndex = lastProcessedIndex;
        uint256 endIndex = startIndex + BATCH_SIZE;
        if (endIndex > agentList.length) {
            endIndex = agentList.length;
        }
        
        uint256 processedRewards = 0;
        
        for (uint256 i = startIndex; i < endIndex;) {
            address agentAddr = agentList[i];
            Agent storage agent = agents[agentAddr];
            
            if (agent.isActive && agent.stakedAmount > 0) {
                // 按质押比例分配奖励
                uint256 agentReward = (pendingRewards * agent.stakedAmount) / totalStaked;
                if (agentReward > 0) {
                    // 这里可以实际转移代币或记录待领取奖励
                    // 为了简化，这里只是记录
                    processedRewards += agentReward;
                }
            }
            unchecked {
                ++i;  // Gas 优化
            }
        }
        
        lastProcessedIndex = endIndex;
        
        // 如果处理完所有智能体，重置状态
        if (endIndex >= agentList.length) {
            pendingRewards = 0;
            lastProcessedIndex = 0;
            return true;
        }
        
        return false;
    }
    
    /**
     * @dev 通过验证合约验证智能体
     * @param proof ZKP证明
     * @return 是否验证成功
     */
    function _verifyAgentThroughVerification(
        address /* agent */,
        uint256[8] calldata proof
    ) internal pure returns (bool) {
        // 这里应该调用DIAPVerification合约的验证函数
        // 由于接口复杂性，这里使用简化实现
        // 实际应用中需要定义相应的接口
        return proof.length == 8 && proof[0] != 0;
    }
    
    // ============ 管理函数 ============
    
    function setRegistrationFee(uint256 _fee) external onlyOwner {
        registrationFee = _fee;
    }
    
    function setMessageFee(uint256 _fee) external onlyOwner {
        messageFee = _fee;
    }
    
    function setServiceFeeRate(uint256 _rate) external onlyOwner {
        if (_rate > 1000) revert FeeRateTooHigh(); // 最大10%
        serviceFeeRate = _rate;
    }
    
    function setMinStakeAmount(uint256 _amount) external onlyOwner {
        minStakeAmount = _amount;
    }
    
    function setRewardRate(uint256 _rate) external onlyOwner {
        rewardRate = _rate;
    }
    
    function setVerificationContract(address _verification) external onlyOwner {
        if (_verification == address(0)) revert InvalidVerificationContractAddress();
        verification = _verification;
    }
    
    /**
     * @dev 设置账户工厂地址
     * @param _factory 工厂地址
     */
    function setAccountFactory(address _factory) external onlyOwner {
        require(_factory != address(0), "Invalid factory address");
        address oldFactory = accountFactory;
        accountFactory = _factory;
        emit AccountFactoryUpdated(oldFactory, _factory);
    }
    
    /**
     * @dev 设置金库地址
     * @param _treasury 金库地址
     */
    function setTreasuryAddress(address _treasury) external onlyOwner {
        if (_treasury == address(0)) revert InvalidTreasuryAddress();
        address oldTreasury = treasuryAddress;
        treasuryAddress = _treasury;
        emit TreasuryAddressUpdated(oldTreasury, _treasury);
    }
    
    /**
     * @dev 提取累积的费用到金库
     */
    function withdrawFees() external onlyOwner nonReentrant {
        if (treasuryAddress == address(0)) revert TreasuryAddressNotSet();
        if (accumulatedFees == 0) revert NoFeesToWithdraw();
        
        uint256 amount = accumulatedFees;
        accumulatedFees = 0;
        
        if (!token.transfer(treasuryAddress, amount)) revert TokenTransferFailed();
        
        emit FeesWithdrawn(treasuryAddress, amount, block.timestamp);
    }
    
    /**
     * @dev 设置智能体元数据CID（混合存储）
     * @param agent 智能体地址
     * @param metadataCID 元数据CID
     */
    function setAgentMetadataCID(address agent, string calldata metadataCID) external onlyOwner {
        if (!agents[agent].isActive) revert AgentNotActive();
        agentMetadataCID[agent] = metadataCID;
    }
    
    /**
     * @dev 设置服务元数据CID（混合存储）
     * @param serviceId 服务ID
     * @param metadataCID 元数据CID
     */
    function setServiceMetadataCID(uint256 serviceId, string calldata metadataCID) external onlyOwner {
        if (services[serviceId].timestamp == 0) revert ServiceNotFound();
        serviceMetadataCID[serviceId] = metadataCID;
    }
    
    function pause() external onlyOwner {
        _pause();
    }
    
    function unpause() external onlyOwner {
        _unpause();
    }
    
    // solhint-disable-next-line no-empty-blocks
    function _authorizeUpgrade(address newImplementation) internal override onlyOwner {}
    
    // ============ 查询函数 ============
    
    function getAgent(address agent) external view returns (Agent memory) {
        return agents[agent];
    }
    
    function getMessage(bytes32 messageId) external view returns (Message memory) {
        return messages[messageId];
    }
    
    function getService(uint256 serviceId) external view returns (Service memory) {
        return services[serviceId];
    }
    
    function getNetworkStats() external view returns (
        uint256 _totalAgents,
        uint256 _totalMessages,
        uint256 _totalServices,
        uint256 _totalVolume,
        uint256 _totalRewards
    ) {
        return (totalAgents, totalMessages, totalServices, totalVolume, totalRewards);
    }
    
    /**
     * @dev 获取智能体标识符类型
     * @param agent 智能体地址
     * @return 标识符类型 (UNKNOWN, IPFS_CID, IPNS_NAME)
     */
    function getAgentIdentifierType(address agent) external view returns (IdentifierType) {
        if (!agents[agent].isActive) revert AgentNotRegistered();
        return _getIdentifierType(agents[agent].identifier);
    }
    
    /**
     * @dev 获取智能体的 AA 账户地址
     * @param agent 智能体地址
     * @return AA 账户地址（如果没有则返回 address(0)）
     */
    function getAgentAAAccount(address agent) external view returns (address) {
        return agents[agent].aaAccount;
    }
    
    /**
     * @dev 检查智能体是否使用 AA 账户
     * @param agent 智能体地址
     * @return 是否使用 AA 账户
     */
    function isAgentUsingAA(address agent) external view returns (bool) {
        return agents[agent].isAAAccount;
    }
    
    /**
     * @dev 获取智能体的代币余额（支持 AA 账户）
     * @param agent 智能体地址
     * @return 代币余额
     */
    function getAgentTokenBalance(address agent) external view returns (uint256) {
        if (!agents[agent].isActive) revert AgentNotRegistered();
        
        if (agents[agent].isAAAccount && agents[agent].aaAccount != address(0)) {
            // 如果是 AA 账户，返回 AA 账户的余额
            return token.balanceOf(agents[agent].aaAccount);
        } else {
            // 如果是 EOA，返回 EOA 的余额
            return token.balanceOf(agent);
        }
    }
    
    /**
     * @dev 验证调用者（支持 AA 账户）
     * @param agent 智能体地址
     * @param caller 调用者地址
     * @return 是否有效
     */
    function isValidCaller(address agent, address caller) external view returns (bool) {
        if (!agents[agent].isActive) return false;
        
        // 如果是 EOA，调用者必须是智能体本身
        if (!agents[agent].isAAAccount) {
            return caller == agent;
        }
        
        // 如果是 AA 账户，调用者可以是 AA 账户或智能体本身
        return caller == agents[agent].aaAccount || caller == agent;
    }
    
    
    // 接收ETH
    receive() external payable {}
}
