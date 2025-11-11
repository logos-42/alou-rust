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
 * @title DIAPPaymentChannel
 * @dev DIAP支付通道合约 - 状态通道和链下支付
 */
contract DIAPPaymentChannel is
    Initializable,
    UUPSUpgradeable,
    OwnableUpgradeable,
    ReentrancyGuardUpgradeable,
    PausableUpgradeable
{
    // ============ Custom Errors ============
    
    error DepositMustBeGreaterThanZero();
    error ChannelIDRequired();
    error ChannelAlreadyExists();
    error CannotOpenChannelWithSelf();
    error InvalidParticipantAddress();
    error SenderNotRegistered();
    error Participant2NotRegistered();
    error InsufficientBalance();
    error InsufficientAllowance();
    error TransferFailed();
    error ChannelNotFound();
    error ChannelNotActive();
    error NotChannelParticipant();
    error NonceMustBeGreater();
    error InvalidBalanceDistribution();
    error InvalidSignatures();
    error NoActiveChallengePerio();
    error ChallengePeriodExpired();
    error NewNonceMustBeGreater();
    error ChallengePeriodNotEnded();
    error InsufficientFunds();
    error TransferToParticipant1Failed();
    error TransferToParticipant2Failed();
    error RateTooHigh();
    
    // ============ 结构体定义 ============

    struct PaymentChannel {
        address participant1;
        address participant2;
        uint256 balance1;
        uint256 balance2;
        uint256 totalDeposited;
        uint256 nonce;
        bool isActive;
        uint256 lastUpdate;
        uint256 challengeDeadline;
        string channelId;
    }

    // ============ 状态变量 ============

    DIAPToken public token;
    DIAPAgentNetwork public agentNetwork;

    mapping(string => PaymentChannel) public paymentChannels;

    uint256 public channelFeeRate;
    uint256 public constant CHALLENGE_PERIOD = 24 hours;

    // solhint-disable-next-line var-name-mixedcase
    bytes32 public DOMAIN_SEPARATOR;
    bytes32 public constant CHANNEL_STATE_TYPEHASH =
        keccak256(
            "ChannelState(string channelId,uint256 balance1,uint256 balance2,uint256 nonce)"
        );

    // ============ 事件定义 ============

    event PaymentChannelOpened(
        string indexed channelId,
        address indexed participant1,
        address indexed participant2,
        uint256 totalDeposit
    );

    event PaymentChannelClosed(
        string indexed channelId,
        uint256 finalBalance1,
        uint256 finalBalance2
    );

    event ChannelChallenged(
        string indexed channelId,
        uint256 newNonce,
        uint256 newBalance1,
        uint256 newBalance2
    );

    // ============ 初始化函数 ============

    function initialize(address _token, address _agentNetwork) public initializer {
        __Ownable_init();
        __UUPSUpgradeable_init();
        __ReentrancyGuard_init();
        __Pausable_init();

        token = DIAPToken(_token);
        agentNetwork = DIAPAgentNetwork(payable(_agentNetwork));

        channelFeeRate = 5; // 0.05%

        DOMAIN_SEPARATOR = keccak256(
            abi.encode(
                keccak256(
                    "EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)"
                ),
                keccak256(bytes("DIAP Payment Channel")),
                keccak256(bytes("1")),
                block.chainid,
                address(this)
            )
        );
    }

    // ============ 支付通道功能 ============

    function openPaymentChannel(
        address participant2,
        uint256 deposit,
        string calldata channelId
    ) external nonReentrant whenNotPaused {
        if (deposit == 0) revert DepositMustBeGreaterThanZero();
        if (bytes(channelId).length == 0) revert ChannelIDRequired();
        if (paymentChannels[channelId].lastUpdate != 0) revert ChannelAlreadyExists();
        if (participant2 == msg.sender) revert CannotOpenChannelWithSelf();
        if (participant2 == address(0)) revert InvalidParticipantAddress();

        if (!agentNetwork.getAgent(msg.sender).isActive) revert SenderNotRegistered();
        if (!agentNetwork.getAgent(participant2).isActive) revert Participant2NotRegistered();

        if (token.balanceOf(msg.sender) < deposit) revert InsufficientBalance();
        if (token.allowance(msg.sender, address(this)) < deposit) revert InsufficientAllowance();

        paymentChannels[channelId] = PaymentChannel({
            participant1: msg.sender,
            participant2: participant2,
            balance1: deposit,
            balance2: 0,
            totalDeposited: deposit,
            nonce: 0,
            isActive: true,
            lastUpdate: block.timestamp,
            challengeDeadline: 0,
            channelId: channelId
        });

        if (!token.transferFrom(msg.sender, address(this), deposit)) revert TransferFailed();

        emit PaymentChannelOpened(channelId, msg.sender, participant2, deposit);
    }

    function initiateChannelClose(
        string calldata channelId,
        uint256 finalBalance1,
        uint256 finalBalance2,
        uint256 nonce,
        bytes calldata signature1,
        bytes calldata signature2
    ) external nonReentrant whenNotPaused {
        PaymentChannel storage channel = paymentChannels[channelId];
        if (channel.lastUpdate == 0) revert ChannelNotFound();
        if (!channel.isActive) revert ChannelNotActive();
        if (msg.sender != channel.participant1 && msg.sender != channel.participant2) revert NotChannelParticipant();

        if (nonce <= channel.nonce) revert NonceMustBeGreater();
        if (finalBalance1 + finalBalance2 > channel.totalDeposited) revert InvalidBalanceDistribution();

        if (!_verifyChannelSignatures(
                channelId,
                finalBalance1,
                finalBalance2,
                nonce,
                channel.participant1,
                channel.participant2,
                signature1,
                signature2
            )) revert InvalidSignatures();

        channel.balance1 = finalBalance1;
        channel.balance2 = finalBalance2;
        channel.nonce = nonce;
        channel.challengeDeadline = block.timestamp + CHALLENGE_PERIOD;

        emit PaymentChannelClosed(channelId, finalBalance1, finalBalance2);
    }

    function challengeChannelClose(
        string calldata channelId,
        uint256 newBalance1,
        uint256 newBalance2,
        uint256 newNonce,
        bytes calldata signature1,
        bytes calldata signature2
    ) external nonReentrant whenNotPaused {
        PaymentChannel storage channel = paymentChannels[channelId];
        if (channel.lastUpdate == 0) revert ChannelNotFound();
        if (!channel.isActive) revert ChannelNotActive();
        if (channel.challengeDeadline == 0) revert NoActiveChallengePerio();
        if (block.timestamp >= channel.challengeDeadline) revert ChallengePeriodExpired();
        if (msg.sender != channel.participant1 && msg.sender != channel.participant2) revert NotChannelParticipant();

        if (newNonce <= channel.nonce) revert NewNonceMustBeGreater();
        if (newBalance1 + newBalance2 > channel.totalDeposited) revert InvalidBalanceDistribution();

        if (!_verifyChannelSignatures(
                channelId,
                newBalance1,
                newBalance2,
                newNonce,
                channel.participant1,
                channel.participant2,
                signature1,
                signature2
            )) revert InvalidSignatures();

        channel.balance1 = newBalance1;
        channel.balance2 = newBalance2;
        channel.nonce = newNonce;

        emit ChannelChallenged(channelId, newNonce, newBalance1, newBalance2);
    }

    function finalizeChannelClose(string calldata channelId) external nonReentrant {
        PaymentChannel storage channel = paymentChannels[channelId];
        if (channel.lastUpdate == 0) revert ChannelNotFound();
        if (!channel.isActive) revert ChannelNotActive();
        if (channel.challengeDeadline == 0) revert NoActiveChallengePerio();
        if (block.timestamp < channel.challengeDeadline) revert ChallengePeriodNotEnded();

        uint256 fee = (channel.totalDeposited * channelFeeRate) / 10000;
        uint256 totalToDistribute = channel.balance1 + channel.balance2;
        if (totalToDistribute + fee > channel.totalDeposited) revert InsufficientFunds();

        if (channel.balance1 > 0) {
            if (!token.transfer(channel.participant1, channel.balance1)) revert TransferToParticipant1Failed();
        }
        if (channel.balance2 > 0) {
            if (!token.transfer(channel.participant2, channel.balance2)) revert TransferToParticipant2Failed();
        }

        channel.isActive = false;
    }

    // ============ 内部函数 ============

    function _verifyChannelSignatures(
        string calldata channelId,
        uint256 balance1,
        uint256 balance2,
        uint256 nonce,
        address participant1,
        address participant2,
        bytes calldata signature1,
        bytes calldata signature2
    ) internal view returns (bool) {
        bytes32 structHash = keccak256(
            abi.encode(
                CHANNEL_STATE_TYPEHASH,
                keccak256(bytes(channelId)),
                balance1,
                balance2,
                nonce
            )
        );

        bytes32 digest = keccak256(abi.encodePacked("\x19\x01", DOMAIN_SEPARATOR, structHash));

        address signer1 = _recoverSigner(digest, signature1);
        address signer2 = _recoverSigner(digest, signature2);

        return
            (signer1 == participant1 && signer2 == participant2) ||
            (signer1 == participant2 && signer2 == participant1);
    }

    function _recoverSigner(
        bytes32 digest,
        bytes calldata signature
    ) internal pure returns (address) {
        require(signature.length == 65, "Invalid signature length");

        bytes32 r;
        bytes32 s;
        uint8 v;

        assembly {
            r := calldataload(signature.offset)
            s := calldataload(add(signature.offset, 32))
            v := byte(0, calldataload(add(signature.offset, 64)))
        }

        if (
            uint256(s) > 0x7FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF5D576E7357A4501DDFE92F46681B20A0
        ) {
            return address(0);
        }

        if (v != 27 && v != 28) {
            return address(0);
        }

        return ecrecover(digest, v, r, s);
    }

    // ============ 管理函数 ============

    function setChannelFeeRate(uint256 _rate) external onlyOwner {
        if (_rate > 100) revert RateTooHigh();
        channelFeeRate = _rate;
    }

    function pause() external onlyOwner {
        _pause();
    }

    function unpause() external onlyOwner {
        _unpause();
    }

    function _authorizeUpgrade(address newImplementation) internal override onlyOwner {}

    // ============ 查询函数 ============

    function getPaymentChannel(
        string calldata channelId
    ) external view returns (PaymentChannel memory) {
        return paymentChannels[channelId];
    }

    receive() external payable {}
}
