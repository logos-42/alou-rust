#!/usr/bin/env python3
import os
from web3 import Web3
from eth_account import Account

# Load environment variables
private_key = os.getenv("POLYMARKET_PRIVATE_KEY")
rpc_url = os.getenv("RPC_URL")

if not private_key:
    print("Error: POLYMARKET_PRIVATE_KEY not set")
    exit(1)

if not rpc_url:
    rpc_url = "https://polygon-mainnet.g.alchemy.com/v2/demo"

# Connect to Polygon
w3 = Web3(Web3.HTTPProvider(rpc_url))
if not w3.is_connected():
    print("Error: Failed to connect to Polygon")
    exit(1)

print(f"Connected to Polygon. Chain ID: {w3.eth.chain_id}")

# Get account from private key
account = Account.from_key(private_key)
address = account.address
print(f"Account: {address}")

# USDC.e (Bridged) contract on Polygon
usdc_address = "0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174"

# Polymarket contracts to approve
contracts = [
    ("Main Exchange", "0x4bFb41d5B3570DeFd03C39a9A4D8dE6Bd8B8982E"),
    ("Neg Risk", "0xC5d563A36AE78145C45a50134d48A1215220f80a"),
    ("Neg Risk Adapter", "0xd91E80cF2E7be2e162c6513ceD06f1dD0dA35296")
]

# USDC ABI (simplified - just approve function)
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
    }
]

# Create contract instance
usdc_contract = w3.eth.contract(address=usdc_address, abi=usdc_abi)

max_uint256 = 2**256 - 1

for contract_name, contract_address in contracts:
    # Check current allowance
    current_allowance = usdc_contract.functions.allowance(address, contract_address).call()
    print(f"\n{contract_name} ({contract_address}):")
    print(f"  Current USDC allowance: {current_allowance / 1e6} USDC")
    
    if current_allowance > 0:
        print("  Allowance already set. Skipping.")
    else:
        # Approve a large amount (max uint256)
        approve_txn = usdc_contract.functions.approve(contract_address, max_uint256).build_transaction({
            'from': address,
            'nonce': w3.eth.get_transaction_count(address),
            'gas': 200000,
            'gasPrice': w3.eth.gas_price
        })
        
        # Sign and send transaction
        signed_txn = w3.eth.account.sign_transaction(approve_txn, private_key)
        tx_hash = w3.eth.send_raw_transaction(signed_txn.raw_transaction)
        print(f"  Approval transaction sent: {tx_hash.hex()}")
        
        # Wait for confirmation
        print("  Waiting for confirmation...")
        receipt = w3.eth.wait_for_transaction_receipt(tx_hash)
        print(f"  Transaction confirmed in block {receipt.blockNumber}")
        
        # Check new allowance
        new_allowance = usdc_contract.functions.allowance(address, contract_address).call()
        print(f"  New USDC allowance: {new_allowance / 1e6} USDC")
        
        # Update nonce for next transaction
        w3.eth.get_transaction_count(address)  # This will refresh

print("\nDone!")