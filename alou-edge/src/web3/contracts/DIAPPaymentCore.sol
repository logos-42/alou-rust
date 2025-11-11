// SPDX-License-Identifier: MIT
pragma solidity ^0.8.30;

import "@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol";
import "@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol";
import "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";
import "@openzeppelin/contracts-upgradeable/security/ReentrancyGuardUpgradeable.sol";
import "@openzeppelin/contracts-upgradeable/security/PausableUpgradeable.sol";
import "./DIAPToken.sol";
import "./DIAPAgentNetwork.sol";

/**
 * @title DIAPPaymentCore
 * @dev DIAP核心支付合约 - 基础支付和服务托管
 */
contract DIAPPaymentCore is
    Initializable,
    UUPSUpgradeable,
    OwnableUpgradeable,
    ReentrancyGuardUpgradeable,
    PausableUpgradeable
{
    // ============ Custom Errors ============
    
    error AmountMustBeGreaterThanZero();
    error PaymentIDRequired();
    error PaymentIDAlreadyExists();
    error SenderNotRegistered();
    error RecipientNotRegistered();
    error InsufficientBalance();
    error PaymentNotFound();
    error PaymentNotPending();
    error NotPaymentRecipient();
    error PaymentExpired();
    error InsufficientAllowance();
    error TransferToRecipientFailed();
    error TransferFeeFailed();
    error NotPaymentSender();
    error InvalidProviderAddress();
    error CannotCreateServiceForSelf();
    error InvalidPrice();
    error ServiceTypeCIDRequired();
    error ConsumerNotRegistered();
    error ProviderNotRegistered();
    error TokenTransferFailed();
    error NotServiceProvider();
    error ServiceNotEscrowed();
    error InvalidResultCID();
    error ServiceExpired();
    error TransferToProviderFailed();
    error NotServiceConsumer();
    error CancellationPeriodExpired();
    error RefundTransferFailed();
    error RateTooHigh();
    
    // ============ 结构体定义 ============

    struct Payment {
        address from;
        address to;
        uint256 amount;
        string paymentId;
        string description;
        uint256 timestamp;
        PaymentStatus status;
        string metadata;
    }

    struct Service {
        address provider;
        address consumer;
        uint256 price;
        uint256 escrowedAmount;
        uint256 timestamp;
        uint256 completionTime;
        ServiceStatus status;
        string serviceTypeCID;
        string resultCID;
    }

    enum PaymentStatus {
        PENDING,
        CONFIRMED,
        FAILED,
        CANCELLED
    }

    enum ServiceStatus {
        Created,
        Escrowed,
        Active,
        Completed,
        Cancelled
    }

    // ============ 状态变量 ============

    DIAPToken public token;
    DIAPAgentNetwork public agentNetwork;

    mapping(string => Payment) public payments;
    mapping(uint256 => Service) public services;

    uint256 public totalPayments;
    uint256 public totalServices;
    uint256 public totalVolume;

    uint256 public paymentFeeRate; // 支付费用率 (基点)

    // ============ 事件定义 ============

    event PaymentCreated(
        string indexed paymentId,
        address indexed from,
        address indexed to,
        uint256 amount
    );

    event PaymentConfirmed(string indexed paymentId, uint256 timestamp);

    event PaymentFailed(string indexed paymentId, string reason);

    event ServiceCreated(
        uint256 indexed serviceId,
        address indexed provider,
        address indexed consumer,
        uint256 price
    );

    event ServiceCompleted(
        uint256 indexed serviceId,
        address indexed provider,
        uint256 amount
    );

    event ServiceCancelled(uint256 indexed serviceId, uint256 refundAmount);

    // ============ 初始化函数 ============

    function initialize(address _token, address _agentNetwork) public initializer {
        __Ownable_init();
        __UUPSUpgradeable_init();
        __ReentrancyGuard_init();
        __Pausable_init();

        token = DIAPToken(_token);
        agentNetwork = DIAPAgentNetwork(payable(_agentNetwork));

        paymentFeeRate = 10; // 0.1%
    }

    // ============ 基础支付功能 ============

    function createPayment(
        address to,
        uint256 amount,
        string calldata paymentId,
        string calldata description,
        string calldata metadata
    ) external nonReentrant whenNotPaused {
        if (amount == 0) revert AmountMustBeGreaterThanZero();
        if (bytes(paymentId).length == 0) revert PaymentIDRequired();
        if (payments[paymentId].timestamp != 0) revert PaymentIDAlreadyExists();
        
        // 获取实际的智能体地址（支持 AA 账户调用）
        address actualSender = _getActualAgent(msg.sender);
        address actualRecipient = _getActualAgent(to);
        
        if (!agentNetwork.getAgent(actualSender).isActive) revert SenderNotRegistered();
        if (!agentNetwork.getAgent(actualRecipient).isActive) revert RecipientNotRegistered();

        uint256 fee = (amount * paymentFeeRate) / 10000;
        uint256 totalAmount = amount + fee;

        if (token.balanceOf(msg.sender) < totalAmount) revert InsufficientBalance();

        payments[paymentId] = Payment({
            from: actualSender,
            to: actualRecipient,
            amount: amount,
            paymentId: paymentId,
            description: description,
            timestamp: block.timestamp,
            status: PaymentStatus.PENDING,
            metadata: metadata
        });

        totalPayments++;

        emit PaymentCreated(paymentId, actualSender, actualRecipient, amount);
    }

    function confirmPayment(string calldata paymentId) external nonReentrant whenNotPaused {
        Payment storage payment = payments[paymentId];
        if (payment.timestamp == 0) revert PaymentNotFound();
        if (payment.status != PaymentStatus.PENDING) revert PaymentNotPending();
        if (payment.to != msg.sender) revert NotPaymentRecipient();
        if (block.timestamp > payment.timestamp + 24 hours) revert PaymentExpired();

        uint256 fee = (payment.amount * paymentFeeRate) / 10000;
        uint256 totalAmount = payment.amount + fee;

        if (token.balanceOf(payment.from) < totalAmount) revert InsufficientBalance();
        if (token.allowance(payment.from, address(this)) < totalAmount) revert InsufficientAllowance();

        if (!token.transferFrom(payment.from, payment.to, payment.amount)) revert TransferToRecipientFailed();
        if (fee > 0) {
            if (!token.transferFrom(payment.from, address(this), fee)) revert TransferFeeFailed();
        }

        payment.status = PaymentStatus.CONFIRMED;
        totalVolume += payment.amount;

        emit PaymentConfirmed(paymentId, block.timestamp);
    }

    function cancelPayment(string calldata paymentId) external {
        Payment storage payment = payments[paymentId];
        if (payment.timestamp == 0) revert PaymentNotFound();
        if (payment.status != PaymentStatus.PENDING) revert PaymentNotPending();
        if (payment.from != msg.sender) revert NotPaymentSender();

        payment.status = PaymentStatus.CANCELLED;

        emit PaymentFailed(paymentId, "Cancelled by sender");
    }

    // ============ 服务托管支付功能 ============

    function createServiceOrder(
        address provider,
        string calldata serviceTypeCID,
        uint256 price
    ) external nonReentrant whenNotPaused returns (uint256) {
        if (provider == address(0)) revert InvalidProviderAddress();
        if (provider == msg.sender) revert CannotCreateServiceForSelf();
        if (price == 0) revert InvalidPrice();
        if (bytes(serviceTypeCID).length == 0) revert ServiceTypeCIDRequired();

        if (!agentNetwork.getAgent(msg.sender).isActive) revert ConsumerNotRegistered();
        if (!agentNetwork.getAgent(provider).isActive) revert ProviderNotRegistered();

        uint256 fee = (price * paymentFeeRate) / 10000;
        uint256 totalAmount = price + fee;

        if (token.balanceOf(msg.sender) < totalAmount) revert InsufficientBalance();
        if (token.allowance(msg.sender, address(this)) < totalAmount) revert InsufficientAllowance();

        if (!token.transferFrom(msg.sender, address(this), totalAmount)) revert TokenTransferFailed();

        uint256 serviceId = totalServices;
        services[serviceId] = Service({
            provider: provider,
            consumer: msg.sender,
            price: price,
            escrowedAmount: totalAmount,
            timestamp: block.timestamp,
            completionTime: 0,
            status: ServiceStatus.Escrowed,
            serviceTypeCID: serviceTypeCID,
            resultCID: ""
        });

        totalServices++;

        emit ServiceCreated(serviceId, provider, msg.sender, price);

        return serviceId;
    }

    function completeServiceOrder(
        uint256 serviceId,
        string calldata resultCID
    ) external nonReentrant whenNotPaused {
        Service storage service = services[serviceId];
        if (service.timestamp == 0) revert PaymentNotFound();
        if (service.provider != msg.sender) revert NotServiceProvider();
        if (service.status != ServiceStatus.Escrowed) revert ServiceNotEscrowed();
        if (bytes(resultCID).length == 0) revert InvalidResultCID();
        if (block.timestamp > service.timestamp + 30 days) revert ServiceExpired();

        uint256 providerAmount = service.price;
        
        // 更新状态（Checks-Effects-Interactions 模式）
        service.status = ServiceStatus.Completed;
        service.completionTime = block.timestamp;
        service.resultCID = resultCID;
        totalVolume += service.price;

        // 外部调用放在最后（Interactions）
        require(
            token.transfer(service.provider, providerAmount),
            "Transfer to provider failed"
        );

        emit ServiceCompleted(serviceId, service.provider, providerAmount);
    }

    function cancelServiceOrder(uint256 serviceId) external nonReentrant {
        Service storage service = services[serviceId];
        if (service.timestamp == 0) revert PaymentNotFound();
        if (service.consumer != msg.sender) revert NotServiceConsumer();
        if (service.status != ServiceStatus.Escrowed) revert ServiceNotEscrowed();
        if (block.timestamp > service.timestamp + 24 hours) revert CancellationPeriodExpired();

        service.status = ServiceStatus.Cancelled;

        uint256 refundAmount = service.escrowedAmount;
        if (!token.transfer(service.consumer, refundAmount)) revert RefundTransferFailed();

        emit ServiceCancelled(serviceId, refundAmount);
    }

    // ============ 管理函数 ============

    function setPaymentFeeRate(uint256 _rate) external onlyOwner {
        if (_rate > 100) revert RateTooHigh();
        paymentFeeRate = _rate;
    }

    function pause() external onlyOwner {
        _pause();
    }

    function unpause() external onlyOwner {
        _unpause();
    }

    function _authorizeUpgrade(address newImplementation) internal override onlyOwner {}

    // ============ 查询函数 ============

    function getPayment(string calldata paymentId) external view returns (Payment memory) {
        return payments[paymentId];
    }

    function getServiceOrder(uint256 serviceId) external view returns (Service memory) {
        return services[serviceId];
    }

    function getPaymentStats()
        external
        view
        returns (uint256 _totalPayments, uint256 _totalServices, uint256 _totalVolume)
    {
        return (totalPayments, totalServices, totalVolume);
    }
    
    // ============ 内部辅助函数 ============
    
    /**
     * @dev 获取实际的智能体地址（支持 AA 账户）
     * @param caller 调用者地址
     * @return 实际的智能体地址
     */
    function _getActualAgent(address caller) internal view returns (address) {
        // 首先检查调用者本身是否是注册的智能体
        if (agentNetwork.getAgent(caller).isActive) {
            return caller;
        }
        
        // 如果不是，检查是否是某个智能体的 AA 账户
        // 这需要遍历或使用反向映射（为了简化，这里返回调用者）
        // 实际实现中，可以在 AgentNetwork 中添加 aaAccountToAgent 映射
        return caller;
    }

    receive() external payable {}
}
