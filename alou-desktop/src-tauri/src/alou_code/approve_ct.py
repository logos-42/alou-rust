#!/usr/bin/env python3
import os
from web3 import Web3
from eth_account import Account

# Load environment variables
private_key = os.getenv('POLYMARKET_PRIVATE_KEY')
rpc_url = os.getenv('RPC_URL', 'https://polygon-rpc.com')

if not private_key:
    print("Error: POLYMARKET_PRIVATE_KEY not set")
    exit(1)

# Connect to Polygon
w3 = Web3(Web3.HTTPProvider(rpc_url))
if not w3.is_connected():
    print("Failed to connect to Polygon")
    exit(1)

print(f"Connected to Polygon. Chain ID: {w3.eth.chain_id}")

# Create account
account = Account.from_key(private_key)
print(f"Account: {account.address}")

# Contract addresses
CT_TOKEN_ADDRESS = '0x4D97DCd97eC945f40cF65F87097ACe5EA0476045'  # Conditional Tokens
EXCHANGE_ADDRESS = '0x4bFb41d5B3570DeFd03C39a9A4D8dE6Bd8B8982E'  # Main exchange

# CT Token ABI (simplified - just approve function)
ct_abi = [
    {
        "constant": False,
        "inputs": [
            {"name": "spender", "type": "address"},
            {"name": "amount", "type": "uint256"}
        ],
        "name": "approve",
        "outputs": [{"name": "", "type": "bool"}],
        "type": "function"
    },
    {
        "constant": True,
        "inputs": [
            {"name": "owner", "type": "address"},
            {"name": "spender", "type": "address"}
        ],
        "name": "allowance",
        "outputs": [{"name": "", "type": "uint256"}],
        "type": "function"
    }
]

# Create contract instance
ct_contract = w3.eth.contract(address=CT_TOKEN_ADDRESS, abi=ct_abi)

# Check current allowance
current_allowance = ct_contract.functions.allowance(account.address, EXCHANGE_ADDRESS).call()
print(f"Current CT allowance for exchange: {current_allowance}")

if current_allowance > 0:
    print("CT already approved")
else:
    # Approve max uint256
    max_uint256 = 2**256 - 1
    nonce = w3.eth.get_transaction_count(account.address)
    
    tx = ct_contract.functions.approve(EXCHANGE_ADDRESS, max_uint256).build_transaction({
        'chainId': w3.eth.chain_id,
        'gas': 100000,
        'gasPrice': w3.eth.gas_price,
        'nonce': nonce,
    })
    
    signed_tx = w3.eth.account.sign_transaction(tx, private_key)
    tx_hash = w3.eth.send_raw_transaction(signed_tx.raw_transaction)
    print(f"Approval transaction sent: {tx_hash.hex()}")
    
    # Wait for confirmation
    receipt = w3.eth.wait_for_transaction_receipt(tx_hash)
    print(f"Transaction confirmed in block {receipt.blockNumber}")
    
    # Check new allowance
    new_allowance = ct_contract.functions.allowance(account.address, EXCHANGE_ADDRESS).call()
    print(f"New CT allowance: {new_allowance}")