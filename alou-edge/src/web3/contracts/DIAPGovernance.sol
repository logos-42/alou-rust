// SPDX-License-Identifier: MIT
pragma solidity ^0.8.30;


import "@openzeppelin/contracts/governance/Governor.sol";
import "@openzeppelin/contracts/governance/extensions/GovernorVotes.sol";
import "@openzeppelin/contracts/governance/extensions/GovernorVotesQuorumFraction.sol";
import "@openzeppelin/contracts/governance/extensions/GovernorTimelockControl.sol";
import "@openzeppelin/contracts/governance/extensions/GovernorSettings.sol";
import "@openzeppelin/contracts/governance/extensions/GovernorCountingSimple.sol";
import "@openzeppelin/contracts/access/Ownable.sol";
import "./DIAPToken.sol";
import "./DIAPAgentNetwork.sol";

/**
 * @title DIAPGovernance
 * @dev DIAP智能体网络治理合约
 * @notice 基于OpenZeppelin Governor实现去中心化治理
 */
contract DIAPGovernance is 
    Governor,
    GovernorVotes,
    GovernorVotesQuorumFraction,
    GovernorTimelockControl,
    GovernorSettings,
    GovernorCountingSimple,
    Ownable
{
    
    // ============ Custom Errors ============
    
    error NotAuthorizedToCreateProposals();
    error NotAuthorizedForEmergencyActions();
    error EmergencyActionFailed();
    error InvalidMultisigWalletAddress();
    error NotAuthorized();
    error NotAuthorizedForUpgrade();
    error TotalVotesOverflow();
    error ReputationVotesOverflow();
    
    // ============ 提案类型 ============
    
    enum ProposalType {
        NETWORK_UPGRADE,     // 网络升级
        PARAMETER_CHANGE,    // 参数调整
        TREASURY_MANAGEMENT, // 资金管理
        AGENT_POLICY,       // 智能体政策
        TOKEN_ECONOMICS,    // 代币经济
        EMERGENCY_ACTION    // 紧急行动
    }
    
    // ============ 状态变量 ============
    
    DIAPAgentNetwork public agentNetwork;
    
    // 提案类型映射
    mapping(uint256 => ProposalType) public proposalTypes;
    
    // 特殊权限
    mapping(address => bool) public emergencyExecutors;
    mapping(address => bool) public proposalCreators;
    
    // 多签钱包控制升级权限
    address public multisigWallet;
    mapping(address => bool) public upgradeAuthorizers;
    
    // 治理参数
    uint256 public constant PROPOSAL_THRESHOLD = 1000 * 10**18; // 1000代币门槛
    uint256 public constant VOTING_DELAY = 1 days;    // 1天延迟
    uint256 public constant VOTING_PERIOD = 3 days;   // 3天投票期
    uint256 public constant EXECUTION_DELAY = 1 days; // 1天执行延迟
    uint256 public constant QUORUM_FRACTION = 4;      // 4% 法定人数
    
    // 紧急提案参数
    uint256 public constant EMERGENCY_VOTING_PERIOD = 6 hours; // 6小时紧急投票期
    uint256 public constant EMERGENCY_QUORUM_FRACTION = 10;    // 10% 紧急法定人数
    
    // ============ 事件定义 ============
    
    event DIAPProposalCreated(
        uint256 indexed proposalId,
        ProposalType indexed proposalType,
        address indexed proposer,
        string description
    );
    
    event DIAPProposalExecuted(
        uint256 indexed proposalId,
        ProposalType indexed proposalType
    );
    
    event EmergencyActionExecuted(
        address indexed executor,
        string action,
        uint256 timestamp
    );
    
    event GovernanceParameterUpdated(
        string parameter,
        uint256 oldValue,
        uint256 newValue
    );
    
    // ============ 构造函数 ============
    
    constructor(
        IVotes _token,
        TimelockController _timelock,
        DIAPAgentNetwork _agentNetwork
    )
        Governor("DIAP Governance")
        GovernorVotes(_token)
        GovernorVotesQuorumFraction(QUORUM_FRACTION)
        GovernorTimelockControl(_timelock)
        GovernorSettings(
            VOTING_DELAY,
            VOTING_PERIOD,
            PROPOSAL_THRESHOLD
        )
    {
        agentNetwork = _agentNetwork;
        
        // 设置初始权限
        emergencyExecutors[msg.sender] = true;
        proposalCreators[msg.sender] = true;
    }
    
    // ============ 提案创建 ============
    
    /**
     * @dev 创建网络升级提案
     * @param targets 目标合约
     * @param values ETH数量
     * @param calldatas 调用数据
     * @param description 提案描述
     */
    function proposeNetworkUpgrade(
        address[] calldata targets,
        uint256[] calldata values,
        bytes[] calldata calldatas,
        string calldata description
    ) external returns (uint256) {
        if (!proposalCreators[msg.sender] && !_isVerifiedAgent(msg.sender)) revert NotAuthorizedToCreateProposals();
        
        uint256 proposalId = propose(targets, values, calldatas, description);
        proposalTypes[proposalId] = ProposalType.NETWORK_UPGRADE;
        
        emit DIAPProposalCreated(proposalId, ProposalType.NETWORK_UPGRADE, msg.sender, description);
        return proposalId;
    }
    
    /**
     * @dev 创建参数调整提案
     * @param targets 目标合约
     * @param values ETH数量
     * @param calldatas 调用数据
     * @param description 提案描述
     */
    function proposeParameterChange(
        address[] calldata targets,
        uint256[] calldata values,
        bytes[] calldata calldatas,
        string calldata description
    ) external returns (uint256) {
        if (!proposalCreators[msg.sender] && !_isVerifiedAgent(msg.sender)) revert NotAuthorizedToCreateProposals();
        
        uint256 proposalId = propose(targets, values, calldatas, description);
        proposalTypes[proposalId] = ProposalType.PARAMETER_CHANGE;
        
        emit DIAPProposalCreated(proposalId, ProposalType.PARAMETER_CHANGE, msg.sender, description);
        return proposalId;
    }
    
    /**
     * @dev 创建资金管理提案
     * @param targets 目标合约
     * @param values ETH数量
     * @param calldatas 调用数据
     * @param description 提案描述
     */
    function proposeTreasuryManagement(
        address[] calldata targets,
        uint256[] calldata values,
        bytes[] calldata calldatas,
        string calldata description
    ) external returns (uint256) {
        if (!proposalCreators[msg.sender] && !_isVerifiedAgent(msg.sender)) revert NotAuthorizedToCreateProposals();
        
        uint256 proposalId = propose(targets, values, calldatas, description);
        proposalTypes[proposalId] = ProposalType.TREASURY_MANAGEMENT;
        
        emit DIAPProposalCreated(proposalId, ProposalType.TREASURY_MANAGEMENT, msg.sender, description);
        return proposalId;
    }
    
    /**
     * @dev 创建智能体政策提案
     * @param targets 目标合约
     * @param values ETH数量
     * @param calldatas 调用数据
     * @param description 提案描述
     */
    function proposeAgentPolicy(
        address[] calldata targets,
        uint256[] calldata values,
        bytes[] calldata calldatas,
        string calldata description
    ) external returns (uint256) {
        if (!proposalCreators[msg.sender] && !_isVerifiedAgent(msg.sender)) revert NotAuthorizedToCreateProposals();
        
        uint256 proposalId = propose(targets, values, calldatas, description);
        proposalTypes[proposalId] = ProposalType.AGENT_POLICY;
        
        emit DIAPProposalCreated(proposalId, ProposalType.AGENT_POLICY, msg.sender, description);
        return proposalId;
    }
    
    /**
     * @dev 创建代币经济提案
     * @param targets 目标合约
     * @param values ETH数量
     * @param calldatas 调用数据
     * @param description 提案描述
     */
    function proposeTokenEconomics(
        address[] calldata targets,
        uint256[] calldata values,
        bytes[] calldata calldatas,
        string calldata description
    ) external returns (uint256) {
        if (!proposalCreators[msg.sender] && !_isVerifiedAgent(msg.sender)) revert NotAuthorizedToCreateProposals();
        
        uint256 proposalId = propose(targets, values, calldatas, description);
        proposalTypes[proposalId] = ProposalType.TOKEN_ECONOMICS;
        
        emit DIAPProposalCreated(proposalId, ProposalType.TOKEN_ECONOMICS, msg.sender, description);
        return proposalId;
    }
    
    /**
     * @dev 创建紧急行动提案
     * @param targets 目标合约
     * @param values ETH数量
     * @param calldatas 调用数据
     * @param description 提案描述
     */
    function proposeEmergencyAction(
        address[] calldata targets,
        uint256[] calldata values,
        bytes[] calldata calldatas,
        string calldata description
    ) external returns (uint256) {
        if (!emergencyExecutors[msg.sender]) revert NotAuthorizedForEmergencyActions();
        
        uint256 proposalId = propose(targets, values, calldatas, description);
        proposalTypes[proposalId] = ProposalType.EMERGENCY_ACTION;
        
        emit DIAPProposalCreated(proposalId, ProposalType.EMERGENCY_ACTION, msg.sender, description);
        return proposalId;
    }
    
    // ============ 投票功能 ============
    
    /**
     * @dev 获取投票权重 (代币 + 声誉)
     * @param account 账户地址
     * @param blockNumber 区块号
     * @return 投票权重
     */
    function getVotes(address account, uint256 blockNumber) 
        public view override(IGovernor, Governor) returns (uint256) {
        
        // 基础代币投票权重
        uint256 tokenVotes = token.getPastVotes(account, blockNumber);
        
        // 声誉投票权重 (如果智能体网络可用)
        uint256 reputationVotes = 0;
        try agentNetwork.getAgent(account) returns (DIAPAgentNetwork.Agent memory agent) {
            if (agent.isActive && agent.isVerified) {
                // 使用平方根转换防止巨鲸控制，并添加溢出保护
                uint256 reputation = agent.reputation;
                if (reputation > 0) {
                    // reputation最大10000，sqrt(10000) = 100
                    // 乘以10**15确保有足够的精度，但不会溢出
                    uint256 sqrtReputation = _sqrt(reputation);
                    // 限制最大声誉投票权重，防止溢出
                    reputationVotes = sqrtReputation * 10**15;
                    // 确保不会溢出
                    if (reputationVotes / 10**15 != sqrtReputation) revert ReputationVotesOverflow();
                }
            }
        } catch {
            // 如果智能体网络不可用，只使用代币投票
            reputationVotes = 0;
        }
        
        // 检查总和不会溢出
        if (tokenVotes > type(uint256).max - reputationVotes) revert TotalVotesOverflow();
        
        return tokenVotes + reputationVotes;
    }
    
    /**
     * @dev 计算平方根（Babylonian方法）
     * @param x 输入值
     * @return y 平方根
     */
    function _sqrt(uint256 x) internal pure returns (uint256 y) {
        if (x == 0) return 0;
        uint256 z = (x + 1) / 2;
        y = x;
        while (z < y) {
            y = z;
            z = (x / z + z) / 2;
        }
    }
    
    /**
     * @dev 获取当前投票权重
     * @param account 账户地址
     * @return 投票权重
     */
    function getCurrentVotes(address account) external view returns (uint256) {
        return getVotes(account, block.number - 1);
    }
    
    // ============ 提案执行 ============
    
    /**
     * @dev 执行提案
     * @param targets 目标合约
     * @param values ETH数量
     * @param calldatas 调用数据
     * @param descriptionHash 描述哈希
     */
    function execute(
        address[] memory targets,
        uint256[] memory values,
        bytes[] memory calldatas,
        bytes32 descriptionHash
    ) public payable override(IGovernor, Governor) returns (uint256) {
        uint256 proposalId = super.execute(targets, values, calldatas, descriptionHash);
        
        // 发出提案执行事件
        ProposalType proposalType = proposalTypes[proposalId];
        emit DIAPProposalExecuted(proposalId, proposalType);
        
        return proposalId;
    }
    
    // ============ 紧急功能 ============
    
    /**
     * @dev 执行紧急行动
     * @param action 行动描述
     * @param targets 目标合约
     * @param values ETH数量
     * @param calldatas 调用数据
     */
    function executeEmergencyAction(
        string calldata action,
        address[] calldata targets,
        uint256[] calldata values,
        bytes[] calldata calldatas
    ) external {
        if (!emergencyExecutors[msg.sender]) revert NotAuthorizedForEmergencyActions();
        
        // 执行紧急行动
        for (uint256 i = 0; i < targets.length;) {
            (bool success, ) = targets[i].call{value: values[i]}(calldatas[i]);
            if (!success) revert EmergencyActionFailed();
            unchecked {
                ++i;  // Gas 优化
            }
        }
        
        emit EmergencyActionExecuted(msg.sender, action, block.timestamp);
    }
    
    // ============ 管理函数 ============
    
    /**
     * @dev 设置紧急执行者
     * @param executor 执行者地址
     * @param authorized 是否授权
     */
    function setEmergencyExecutor(address executor, bool authorized) external onlyOwner {
        emergencyExecutors[executor] = authorized;
    }
    
    /**
     * @dev 设置提案创建者
     * @param creator 创建者地址
     * @param authorized 是否授权
     */
    function setProposalCreator(address creator, bool authorized) external onlyOwner {
        proposalCreators[creator] = authorized;
    }
    
    /**
     * @dev 设置多签钱包地址
     * @param _multisigWallet 多签钱包地址
     */
    function setMultisigWallet(address _multisigWallet) external onlyOwner {
        if (_multisigWallet == address(0)) revert InvalidMultisigWalletAddress();
        multisigWallet = _multisigWallet;
    }
    
    /**
     * @dev 设置升级授权者
     * @param authorizer 授权者地址
     * @param authorized 是否授权
     */
    function setUpgradeAuthorizer(address authorizer, bool authorized) external {
        if (msg.sender != multisigWallet && msg.sender != owner()) revert NotAuthorized();
        upgradeAuthorizers[authorizer] = authorized;
    }
    
    /**
     * @dev 检查升级权限（多签控制）
     */
    function _authorizeUpgrade(address /* newImplementation */) internal view {
        if (
            !upgradeAuthorizers[msg.sender]
            && msg.sender != multisigWallet
            && msg.sender != owner()
        ) revert NotAuthorizedForUpgrade();
        // 注意：DIAPGovernance不是UUPS合约，这里只是权限检查
    }
    
    /**
     * @dev 更新治理参数
     * @param parameter 参数名称
     * @param newValue 新值
     */
    function updateGovernanceParameter(string calldata parameter, uint256 newValue) external onlyOwner {
        // 这里可以根据需要添加更多参数更新逻辑
        emit GovernanceParameterUpdated(parameter, 0, newValue);
    }
    
    // ============ 查询函数 ============
    
    /**
     * @dev 获取提案信息
     * @param proposalId 提案ID
     * @return proposalType 提案类型
     * @return proposalState 提案状态
     * @return forVotes 赞成票数
     * @return againstVotes 反对票数
     * @return abstainVotes 弃权票数
     */
    function getProposalInfo(uint256 proposalId) external view returns (
        ProposalType proposalType,
        ProposalState proposalState,
        uint256 forVotes,
        uint256 againstVotes,
        uint256 abstainVotes
    ) {
        proposalType = proposalTypes[proposalId];
        proposalState = state(proposalId);
        
        // 获取投票信息 - 使用GovernorCountingSimple的默认实现
        (uint256 forVotes_, uint256 againstVotes_, uint256 abstainVotes_) = proposalVotes(proposalId);
        forVotes = forVotes_;
        againstVotes = againstVotes_;
        abstainVotes = abstainVotes_;
    }
    
    /**
     * @dev 获取治理统计信息
     * @return totalProposals 总提案数
     * @return activeProposals 活跃提案数
     * @return executedProposals 已执行提案数
     */
    function getGovernanceStats() external pure returns (
        uint256 totalProposals,
        uint256 activeProposals,
        uint256 executedProposals
    ) {
        // 这里需要遍历所有提案来计算统计信息
        // 为了gas效率，可以考虑使用链下计算
        return (0, 0, 0);
    }
    
    // ============ 内部函数 ============
    
    /**
     * @dev 检查是否为已验证智能体
     * @param agent 智能体地址
     * @return 是否为已验证智能体
     */
    function _isVerifiedAgent(address agent) internal view returns (bool) {
        try agentNetwork.getAgent(agent) returns (DIAPAgentNetwork.Agent memory agentInfo) {
            return agentInfo.isActive && agentInfo.isVerified;
        } catch {
            return false;
        }
    }
    
    // ============ 重写函数 ============
    
    /**
     * @dev 获取投票延迟
     * @return 投票延迟时间
     */
    function votingDelay() public view override(IGovernor, GovernorSettings) returns (uint256) {
        return super.votingDelay();
    }
    
    /**
     * @dev 获取投票期
     * @return 投票期时间
     */
    function votingPeriod() public view override(IGovernor, GovernorSettings) returns (uint256) {
        return super.votingPeriod();
    }
    
    /**
     * @dev 获取提案门槛
     * @return 提案门槛
     */
    function proposalThreshold() public view override(Governor, GovernorSettings) returns (uint256) {
        return super.proposalThreshold();
    }
    
    /**
     * @dev 获取法定人数
     * @param blockNumber 区块号
     * @return 法定人数
     */
    function quorum(uint256 blockNumber)
        public
        view
        override(IGovernor, GovernorVotesQuorumFraction)
        returns (uint256)
    {
        return super.quorum(blockNumber);
    }
    
    /**
     * @dev 重写_cancel函数以解决多重继承冲突
     */
    function _cancel(
        address[] memory targets,
        uint256[] memory values,
        bytes[] memory calldatas,
        bytes32 descriptionHash
    ) internal override(Governor, GovernorTimelockControl) returns (uint256) {
        return super._cancel(targets, values, calldatas, descriptionHash);
    }
    
    /**
     * @dev 重写_execute函数以解决多重继承冲突
     */
    function _execute(
        uint256 proposalId,
        address[] memory targets,
        uint256[] memory values,
        bytes[] memory calldatas,
        bytes32 descriptionHash
    ) internal override(Governor, GovernorTimelockControl) {
        super._execute(proposalId, targets, values, calldatas, descriptionHash);
    }
    
    /**
     * @dev 重写_executor函数以解决多重继承冲突
     */
    function _executor() internal view override(Governor, GovernorTimelockControl) returns (address) {
        return super._executor();
    }
    
    /**
     * @dev 重写state函数以解决多重继承冲突
     */
    function state(uint256 proposalId) public view override(Governor, GovernorTimelockControl) returns (ProposalState) {
        return super.state(proposalId);
    }
    
    /**
     * @dev 重写supportsInterface函数以解决多重继承冲突
     */
    function supportsInterface(bytes4 interfaceId)
        public
        view
        override(Governor, GovernorTimelockControl)
        returns (bool)
    {
        return super.supportsInterface(interfaceId);
    }
}
