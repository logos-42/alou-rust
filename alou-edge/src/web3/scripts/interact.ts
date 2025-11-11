import { ethers } from "hardhat";
import type { HardhatEthersSigner } from "@nomicfoundation/hardhat-ethers/signers";
import { Wallet } from "ethers";
import * as dotenv from "dotenv";

dotenv.config();

const ERC20_ABI = [
  "function balanceOf(address account) view returns (uint256)",
  "function decimals() view returns (uint8)",
  "function symbol() view returns (string)",
  "function transfer(address to, uint256 amount) returns (bool)",
];

async function main() {
  const tokenAddress =
    process.env.DIAP_TOKEN_ADDRESS ?? process.env.TOKEN_ADDRESS;

  if (!tokenAddress) {
    throw new Error(
      "DIAP_TOKEN_ADDRESS 未设置，请在 .env 中提供合约地址（或保留 TOKEN_ADDRESS 回退值）。"
    );
  }

  type SignerLike = HardhatEthersSigner | Wallet;
  let signer: SignerLike | undefined = (await ethers.getSigners())[0];

  if (!signer) {
    const rpcUrl =
      process.env.ETH_RPC_URL ??
      process.env.SEPOLIA_RPC_URL ??
      process.env.TESTNET_RPC_URL ??
      undefined;
    const privateKey =
      process.env.PRIVATE_KEY ??
      process.env.SEPOLIA_PRIVATE_KEY ??
      process.env.TESTNET_PRIVATE_KEY ??
      undefined;

    if (!rpcUrl) {
      throw new Error(
        "未检测到 Hardhat provider，且未设置 ETH_RPC_URL/SEPOLIA_RPC_URL。"
      );
    }

    if (!privateKey) {
      throw new Error(
        "未检测到 Hardhat signer，且 PRIVATE_KEY 未设置（或缺少 SEPOLIA_PRIVATE_KEY/TESTNET_PRIVATE_KEY 回退值）。"
      );
    }

    const fallbackProvider = new ethers.JsonRpcProvider(rpcUrl);
    signer = new Wallet(privateKey, fallbackProvider);
  }

  const provider = signer.provider ?? ethers.provider;

  if (!provider) {
    throw new Error("未能初始化 provider。");
  }

  const network = await provider.getNetwork();
  console.log(`使用网络: ${network.name} (${network.chainId})`);
  console.log(`签名者地址: ${await signer.getAddress()}`);

  const token = new ethers.Contract(tokenAddress, ERC20_ABI, signer);
  const symbol = await token.symbol();
  const decimals = Number(await token.decimals());

  const signerAddress = await signer.getAddress();
  const balance = await token.balanceOf(signerAddress);
  console.log(
    `当前余额: ${ethers.formatUnits(balance, decimals)} ${symbol}`.trim()
  );

  const recipient = process.env.TRANSFER_RECIPIENT;
  const transferAmount = process.env.TRANSFER_AMOUNT;

  if (recipient && transferAmount) {
    const amount = ethers.parseUnits(transferAmount, decimals);
    console.log(
      `准备向 ${recipient} 转账 ${transferAmount} ${symbol} (raw: ${amount})`
    );
    const tx = await token.transfer(recipient, amount);
    console.log(`交易已发送，hash: ${tx.hash}`);
    const receipt = await tx.wait();
    console.log(`交易已确认，区块: ${receipt.blockNumber}`);
  } else {
    console.log(
      "未设置 TRANSFER_RECIPIENT/TRANSFER_AMOUNT，跳过转账，仅进行读操作。"
    );
  }
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});

