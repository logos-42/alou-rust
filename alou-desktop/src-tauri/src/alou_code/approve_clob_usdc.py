#!/usr/bin/env python3
"""
Approve USDC for Polymarket CLOB contract.
Run this BEFORE placing orders.
"""
import os
import sys
from web3 import Web3
from eth_account import Account

# Load environment variables
private_key = os.getenv("POLYMARKET_PRIVATE_KEY")
rpc_url = os.getenv("RPC_URL", "https://polygon-mainnet.g.alchemy.com/v2/demo")

if not private_key:
    print("Error: POLYMARKET_PRIVATE_KEY not set")
    print("  Run: export POLYMARKET_PRIVATE_KEY='0x...'", file=sys.stderr)
    sys.exit(1)

# Connect to Polygon
w3 = Web3(Web3.HTTPProvider(rpc_url))
if not w3.is_connected():
    print("Error: Failed to connect to Polygon RPC")
    sys.exit(1)

print(f"Connected to Polygon. Chain ID: {w3.eth.chain_id}")

# Get account
account = Account.from_key(private_key)
address = account.address
print(f"Account: {address}")

# Native USDC on Polygon (NEW - used by the Rust code)
usdc_address = "0x3c499c542cEF5E3811e1192ce70d8cC03d5c3359"

# Polymarket CLOB exchange contract
clob_address = "0x4bFb41d5B3570DeFd03C39a9A4D8dE6Bd8B8982E"

# USDC ABI
usdc_abi = [
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
    },
    {
        "constant": True,
        "inputs": [{"name": "account", "type": "address"}],
        "name": "balanceOf",
        "outputs": [{"name": "", "type": "uint256"}],
        "type": "function"
    }
]

# Connect to USDC contract
usdc = w3.eth.contract(address=usdc_address, abi=usdc_abi)

# Check current allowance
current_allowance = usdc.functions.allowance(address, clob_address).call()
current_allowance_usdc = current_allowance / 1e6
print(f"\nCurrent allowance for CLOB: {current_allowance_usdc:.2f} USDC")

# Check balance
balance = usdc.functions.balanceOf(address).call()
balance_usdc = balance / 1e6
print(f"Current USDC balance: {balance_usdc:.2f} USDC")

if current_allowance > 0:
    print("\nAllowance already set. Skipping approval.")
    print(f"  If you need more, run with APPROVE_AMOUNT environment variable")
    sys.exit(0)

# Approve amount (default: large number for convenience)
approve_amount = int(os.getenv("APPROVE_AMOUNT", "1000000"))  # 1M USDC default
approve_amount_usdc = approve_amount / 1e6

print(f"\nApproving {approve_amount_usdc:.2f} USDC for CLOB contract...")
print(f"  Spender: {clob_address}")
print(f"  Amount: {approve_amount_usdc:.2f} USDC")

# Build and send transaction
nonce = w3.eth.get_transaction_count(address)
chain_id = w3.eth.chain_id

tx = usdc.functions.approve(clob_address, approve_amount).build_transaction({
    'chainId': chain_id,
    'from': address,
    'nonce': nonce,
    'gas': 100000,
    'gasPrice': w3.eth.gas_price,
})

# Sign and send
signed_tx = w3.eth.account.sign_transaction(tx, private_key)
tx_hash = w3.eth.send_raw_transaction(signed_tx.raw_transaction)
print(f"\nTransaction sent: {tx_hash.hex()}")
print("Waiting for confirmation...")

# Wait for receipt
receipt = w3.eth.wait_for_transaction_receipt(tx_hash)
if receipt.status == 1:
    print(f"✓ Approval confirmed! Gas used: {receipt.gas_used}")
else:
    print("✗ Transaction failed!")
    sys.exit(1)

# Verify new allowance
new_allowance = usdc.functions.allowance(address, clob_address).call()
print(f"New allowance: {new_allowance / 1e6:.2f} USDC")
