// SPDX-License-Identifier: MIT
pragma solidity ^0.8.30;

import "@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol";
import "@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol";
import "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";
import "@openzeppelin/contracts-upgradeable/security/ReentrancyGuardUpgradeable.sol";
import "@openzeppelin/contracts-upgradeable/security/PausableUpgradeable.sol";
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";

/**
 * @title DIAPSubscription
 * @dev DIAP订阅管理合约 - 支持多代币支付的会员订阅系统
 */
contract DIAPSubscription is
    Initializable,
    UUPSUpgradeable,
    OwnableUpgradeable,
    ReentrancyGuardUpgradeable,
    PausableUpgradeable
{
    using SafeERC20 for IERC20;

    // ============ Custom Errors ============
    
    error InvalidPlan();
    error PlanNotActive();
    error InvalidAmount();
    error InsufficientBalance();
    error InsufficientAllowance();
    error SubscriptionNotFound();
    error SubscriptionNotActive();
    error SubscriptionNotExpired();
    error TokenTransferFailed();
    error InvalidTokenAddress();
    error TokenNotSupported();
    error InvalidDuration();
    
    // ============ 结构体定义 ============

    struct SubscriptionPlan {
        uint256 planId;
        string name;                    // "monthly" or "yearly"
        string displayName;             // "Monthly Plan" or "Yearly Plan"
        uint256 priceUSD;                // Price in USD (scaled by 1e6, e.g., 20 USD = 20000000)
        uint256 durationDays;            // 30 for monthly, 365 for yearly
        bool isActive;
        address[] supportedTokens;       // Supported token addresses
    }

    struct Subscription {
        uint256 subscriptionId;
        address user;
        uint256 planId;
        address tokenAddress;            // Token used for payment
        uint256 amountPaid;             // Amount paid in token units
        uint256 startedAt;
        uint256 expiresAt;
        SubscriptionStatus status;
    }

    enum SubscriptionStatus {
        Active,
        Expired,
        Cancelled
    }

    // ============ 状态变量 ============

    // Platform wallet address for receiving payments
    address public platformWallet;

    // Plan ID counter
    uint256 public nextPlanId;

    // Subscription ID counter
    uint256 public nextSubscriptionId;

    // Plans mapping
    mapping(uint256 => SubscriptionPlan) public plans;

    // User subscriptions: user => subscriptionId => Subscription
    mapping(address => mapping(uint256 => Subscription)) public userSubscriptions;

    // Active subscription: user => subscriptionId
    mapping(address => uint256) public activeSubscriptions;

    // Subscription count per user
    mapping(address => uint256) public userSubscriptionCount;

    // Token price oracle (simplified - in production, use Chainlink or similar)
    mapping(address => uint256) public tokenPricesUSD; // Price in USD scaled by 1e6

    // ============ 事件定义 ============

    event SubscriptionCreated(
        uint256 indexed subscriptionId,
        address indexed user,
        uint256 indexed planId,
        address tokenAddress,
        uint256 amountPaid,
        uint256 expiresAt
    );

    event SubscriptionRenewed(
        uint256 indexed subscriptionId,
        address indexed user,
        uint256 expiresAt
    );

    event SubscriptionCancelled(
        uint256 indexed subscriptionId,
        address indexed user
    );

    event SubscriptionExpired(
        uint256 indexed subscriptionId,
        address indexed user
    );

    event PlanCreated(
        uint256 indexed planId,
        string name,
        uint256 priceUSD,
        uint256 durationDays
    );

    event PlanUpdated(
        uint256 indexed planId,
        bool isActive
    );

    // ============ 初始化函数 ============

    function initialize(address _platformWallet) public initializer {
        __Ownable_init();
        __UUPSUpgradeable_init();
        __ReentrancyGuard_init();
        __Pausable_init();

        platformWallet = _platformWallet;
        nextPlanId = 1;
        nextSubscriptionId = 1;
    }

    // ============ 管理函数 ============

    /**
     * @dev Create a new subscription plan
     */
    function createPlan(
        string memory name,
        string memory displayName,
        uint256 priceUSD,
        uint256 durationDays,
        address[] memory supportedTokens
    ) external onlyOwner {
        if (durationDays == 0) revert InvalidDuration();
        if (priceUSD == 0) revert InvalidAmount();

        uint256 planId = nextPlanId++;
        plans[planId] = SubscriptionPlan({
            planId: planId,
            name: name,
            displayName: displayName,
            priceUSD: priceUSD,
            durationDays: durationDays,
            isActive: true,
            supportedTokens: supportedTokens
        });

        emit PlanCreated(planId, name, priceUSD, durationDays);
    }

    /**
     * @dev Update plan status
     */
    function updatePlanStatus(uint256 planId, bool isActive) external onlyOwner {
        if (plans[planId].planId == 0) revert InvalidPlan();
        plans[planId].isActive = isActive;
        emit PlanUpdated(planId, isActive);
    }

    /**
     * @dev Set token price in USD (scaled by 1e6)
     */
    function setTokenPrice(address token, uint256 priceUSD) external onlyOwner {
        tokenPricesUSD[token] = priceUSD;
    }

    /**
     * @dev Set platform wallet address
     */
    function setPlatformWallet(address _platformWallet) external onlyOwner {
        platformWallet = _platformWallet;
    }

    // ============ 订阅功能 ============

    /**
     * @dev Create a new subscription
     * @param planId The subscription plan ID
     * @param tokenAddress The token address to pay with
     * @param amount The amount to pay (in token units)
     */
    function createSubscription(
        uint256 planId,
        address tokenAddress,
        uint256 amount
    ) external nonReentrant whenNotPaused {
        if (plans[planId].planId == 0) revert InvalidPlan();
        if (!plans[planId].isActive) revert PlanNotActive();
        if (amount == 0) revert InvalidAmount();

        // Check if token is supported
        bool isSupported = false;
        for (uint256 i = 0; i < plans[planId].supportedTokens.length; i++) {
            if (plans[planId].supportedTokens[i] == tokenAddress) {
                isSupported = true;
                break;
            }
        }
        if (!isSupported) revert TokenNotSupported();

        // Calculate required amount based on plan price and token price
        uint256 requiredAmount = calculateRequiredAmount(planId, tokenAddress);
        if (amount < requiredAmount) revert InvalidAmount();

        // Check balance and allowance
        IERC20 token = IERC20(tokenAddress);
        if (token.balanceOf(msg.sender) < amount) revert InsufficientBalance();
        if (token.allowance(msg.sender, address(this)) < amount) revert InsufficientAllowance();

        // Transfer tokens from user to platform wallet
        token.safeTransferFrom(msg.sender, platformWallet, amount);

        // Create subscription
        uint256 subscriptionId = nextSubscriptionId++;
        uint256 startedAt = block.timestamp;
        uint256 expiresAt = startedAt + (plans[planId].durationDays * 1 days);

        // Cancel previous active subscription if exists
        uint256 previousActiveId = activeSubscriptions[msg.sender];
        if (previousActiveId != 0) {
            userSubscriptions[msg.sender][previousActiveId].status = SubscriptionStatus.Cancelled;
        }

        Subscription memory subscription = Subscription({
            subscriptionId: subscriptionId,
            user: msg.sender,
            planId: planId,
            tokenAddress: tokenAddress,
            amountPaid: amount,
            startedAt: startedAt,
            expiresAt: expiresAt,
            status: SubscriptionStatus.Active
        });

        userSubscriptions[msg.sender][subscriptionId] = subscription;
        activeSubscriptions[msg.sender] = subscriptionId;
        userSubscriptionCount[msg.sender]++;

        emit SubscriptionCreated(
            subscriptionId,
            msg.sender,
            planId,
            tokenAddress,
            amount,
            expiresAt
        );
    }

    /**
     * @dev Renew an existing subscription
     */
    function renewSubscription(uint256 subscriptionId) external nonReentrant whenNotPaused {
        Subscription storage subscription = userSubscriptions[msg.sender][subscriptionId];
        if (subscription.subscriptionId == 0) revert SubscriptionNotFound();
        if (subscription.user != msg.sender) revert SubscriptionNotFound();
        if (subscription.status != SubscriptionStatus.Active) revert SubscriptionNotActive();

        uint256 planId = subscription.planId;
        address tokenAddress = subscription.tokenAddress;
        uint256 requiredAmount = calculateRequiredAmount(planId, tokenAddress);

        // Check balance and allowance
        IERC20 token = IERC20(tokenAddress);
        if (token.balanceOf(msg.sender) < requiredAmount) revert InsufficientBalance();
        if (token.allowance(msg.sender, address(this)) < requiredAmount) revert InsufficientAllowance();

        // Transfer tokens
        token.safeTransferFrom(msg.sender, platformWallet, requiredAmount);

        // Extend subscription
        uint256 currentExpiresAt = subscription.expiresAt;
        if (currentExpiresAt < block.timestamp) {
            // If expired, start from now
            subscription.startedAt = block.timestamp;
            subscription.expiresAt = block.timestamp + (plans[planId].durationDays * 1 days);
        } else {
            // If not expired, extend from current expiration
            subscription.expiresAt = currentExpiresAt + (plans[planId].durationDays * 1 days);
        }

        subscription.amountPaid += requiredAmount;

        emit SubscriptionRenewed(subscriptionId, msg.sender, subscription.expiresAt);
    }

    /**
     * @dev Cancel an active subscription
     */
    function cancelSubscription(uint256 subscriptionId) external {
        Subscription storage subscription = userSubscriptions[msg.sender][subscriptionId];
        if (subscription.subscriptionId == 0) revert SubscriptionNotFound();
        if (subscription.user != msg.sender) revert SubscriptionNotFound();
        if (subscription.status != SubscriptionStatus.Active) revert SubscriptionNotActive();

        subscription.status = SubscriptionStatus.Cancelled;
        
        if (activeSubscriptions[msg.sender] == subscriptionId) {
            activeSubscriptions[msg.sender] = 0;
        }

        emit SubscriptionCancelled(subscriptionId, msg.sender);
    }

    /**
     * @dev Mark subscription as expired (can be called by anyone)
     */
    function expireSubscription(address user, uint256 subscriptionId) external {
        Subscription storage subscription = userSubscriptions[user][subscriptionId];
        if (subscription.subscriptionId == 0) revert SubscriptionNotFound();
        if (subscription.status != SubscriptionStatus.Active) revert SubscriptionNotActive();
        if (subscription.expiresAt > block.timestamp) revert SubscriptionNotExpired();

        subscription.status = SubscriptionStatus.Expired;
        
        if (activeSubscriptions[user] == subscriptionId) {
            activeSubscriptions[user] = 0;
        }

        emit SubscriptionExpired(subscriptionId, user);
    }

    // ============ 查询函数 ============

    /**
     * @dev Get active subscription for a user
     */
    function getActiveSubscription(address user) external view returns (Subscription memory) {
        uint256 subscriptionId = activeSubscriptions[user];
        if (subscriptionId == 0) {
            revert SubscriptionNotFound();
        }
        Subscription memory subscription = userSubscriptions[user][subscriptionId];
        if (subscription.status != SubscriptionStatus.Active || subscription.expiresAt <= block.timestamp) {
            revert SubscriptionNotFound();
        }
        return subscription;
    }

    /**
     * @dev Get subscription by ID
     */
    function getSubscription(address user, uint256 subscriptionId) external view returns (Subscription memory) {
        Subscription memory subscription = userSubscriptions[user][subscriptionId];
        if (subscription.subscriptionId == 0) revert SubscriptionNotFound();
        return subscription;
    }

    /**
     * @dev Get plan details
     */
    function getPlan(uint256 planId) external view returns (SubscriptionPlan memory) {
        if (plans[planId].planId == 0) revert InvalidPlan();
        return plans[planId];
    }

    /**
     * @dev Calculate required token amount for a plan
     */
    function calculateRequiredAmount(uint256 planId, address tokenAddress) public view returns (uint256) {
        SubscriptionPlan memory plan = plans[planId];
        if (plan.planId == 0) revert InvalidPlan();

        uint256 tokenPrice = tokenPricesUSD[tokenAddress];
        if (tokenPrice == 0) revert TokenNotSupported();

        // Calculate: (plan.priceUSD * 1e18) / (tokenPrice * 1e12)
        // Assuming token has 18 decimals
        uint256 requiredAmount = (plan.priceUSD * 1e18) / (tokenPrice * 1e12);
        return requiredAmount;
    }

    /**
     * @dev Check if user has active subscription
     */
    function hasActiveSubscription(address user) external view returns (bool) {
        uint256 subscriptionId = activeSubscriptions[user];
        if (subscriptionId == 0) return false;
        
        Subscription memory subscription = userSubscriptions[user][subscriptionId];
        return subscription.status == SubscriptionStatus.Active && 
               subscription.expiresAt > block.timestamp;
    }

    // ============ 内部函数 ============

    // solhint-disable-next-line no-empty-blocks
    function _authorizeUpgrade(address newImplementation) internal override onlyOwner {}

    // ============ 接收ETH ============

    receive() external payable {
        // Contract can receive ETH but doesn't handle it
        // This is for compatibility
    }
}

