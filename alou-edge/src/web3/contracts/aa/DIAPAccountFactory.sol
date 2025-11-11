// SPDX-License-Identifier: MIT
pragma solidity ^0.8.30;

import "./DIAPAccount.sol";
import "./IEntryPoint.sol";
import "@openzeppelin/contracts/utils/Create2.sol";
import "@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol";

/**
 * @title DIAPAccountFactory
 * @dev DIAP 智能体账户工厂合约
 * @notice 使用 CREATE2 创建可预测地址的账户
 */
contract DIAPAccountFactory {
    
    // ============ Custom Errors ============
    
    error AccountAlreadyExists();
    error ZeroAddress();
    
    // ============ 状态变量 ============
    
    DIAPAccount public immutable ACCOUNT_IMPLEMENTATION;
    IEntryPoint public immutable ENTRY_POINT;
    
    // 账户注册表
    mapping(address => address) public ownerToAccount;  // owner => account
    mapping(address => address) public accountToOwner;  // account => owner
    address[] public allAccounts;
    
    // ============ 事件定义 ============
    
    event AccountCreated(
        address indexed account,
        address indexed owner,
        uint256 salt
    );
    
    // ============ 构造函数 ============
    
    constructor(IEntryPoint _entryPoint) {
        if (address(_entryPoint) == address(0)) revert ZeroAddress();
        
        ENTRY_POINT = _entryPoint;
        ACCOUNT_IMPLEMENTATION = new DIAPAccount(_entryPoint);
    }
    
    // ============ 账户创建 ============
    
    /**
     * @dev 创建账户
     * @param owner 账户所有者
     * @param salt 盐值（用于生成唯一地址）
     * @return account 创建的账户地址
     */
    function createAccount(
        address owner,
        uint256 salt
    ) public returns (DIAPAccount account) {
        if (owner == address(0)) revert ZeroAddress();
        
        address addr = getAddress(owner, salt);
        uint256 codeSize = addr.code.length;
        
        if (codeSize > 0) {
            return DIAPAccount(payable(addr));
        }
        
        // 使用 CREATE2 部署代理合约
        account = DIAPAccount(payable(
            new ERC1967Proxy{salt: bytes32(salt)}(
                address(ACCOUNT_IMPLEMENTATION),
                abi.encodeCall(DIAPAccount.initialize, (owner))
            )
        ));
        
        // 记录到注册表
        ownerToAccount[owner] = address(account);
        accountToOwner[address(account)] = owner;
        allAccounts.push(address(account));
        
        emit AccountCreated(address(account), owner, salt);
    }
    
    /**
     * @dev 获取账户地址（不创建）
     * @param owner 账户所有者
     * @param salt 盐值
     * @return 账户地址
     */
    function getAddress(
        address owner,
        uint256 salt
    ) public view returns (address) {
        return Create2.computeAddress(
            bytes32(salt),
            keccak256(abi.encodePacked(
                type(ERC1967Proxy).creationCode,
                abi.encode(
                    address(ACCOUNT_IMPLEMENTATION),
                    abi.encodeCall(DIAPAccount.initialize, (owner))
                )
            ))
        );
    }
    
    /**
     * @dev 批量创建账户
     * @param owners 所有者地址数组
     * @param salts 盐值数组
     * @return accounts 创建的账户地址数组
     */
    function batchCreateAccounts(
        address[] calldata owners,
        uint256[] calldata salts
    ) external returns (address[] memory accounts) {
        require(owners.length == salts.length, "Length mismatch");
        require(owners.length <= 50, "Batch too large");
        
        accounts = new address[](owners.length);
        for (uint256 i = 0; i < owners.length;) {
            DIAPAccount account = createAccount(owners[i], salts[i]);
            accounts[i] = address(account);
            unchecked { ++i; }  // Gas 优化
        }
    }
    
    // ============ 查询函数 ============
    
    /**
     * @dev 根据 owner 获取账户地址
     * @param owner 所有者地址
     * @return 账户地址
     */
    function getAccountByOwner(address owner) external view returns (address) {
        return ownerToAccount[owner];
    }
    
    /**
     * @dev 根据账户地址获取 owner
     * @param account 账户地址
     * @return 所有者地址
     */
    function getOwnerByAccount(address account) external view returns (address) {
        return accountToOwner[account];
    }
    
    /**
     * @dev 获取所有账户
     * @return 所有账户地址数组
     */
    function getAllAccounts() external view returns (address[] memory) {
        return allAccounts;
    }
    
    /**
     * @dev 获取账户总数
     * @return 账户总数
     */
    function getAccountCount() external view returns (uint256) {
        return allAccounts.length;
    }
    
    /**
     * @dev 检查账户是否存在
     * @param account 账户地址
     * @return 是否存在
     */
    function isAccount(address account) external view returns (bool) {
        return accountToOwner[account] != address(0);
    }
}
