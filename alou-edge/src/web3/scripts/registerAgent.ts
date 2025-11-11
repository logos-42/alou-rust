import { ethers } from "hardhat";
import * as dotenv from "dotenv";
import { formatUnits, parseUnits } from "ethers";

import agentNetworkArtifact from "../../../abi/diap/DIAPAgentNetwork.json";
import tokenArtifact from "../../../abi/diap/DIAPToken.json";
import accountArtifact from "../../../abi/aa/DIAPAccount.json";

dotenv.config();

type AgentInfo = {
  agentAddress: string;
  didDocument: string;
  publicKey: string;
  stakedAmount: bigint;
  registrationTime: bigint;
  lastHeartbeat: bigint;
  reputationScore: bigint;
  totalTasksCompleted: bigint;
  isActive: boolean;
  isAAAccount: boolean;
  aaAccount: string;
};

async function main() {
  const agentNetworkAddress =
    process.env.DIAP_AGENT_NETWORK_ADDRESS ??
    process.env.DIAP_NETWORK_ADDRESS ??
    "";
  const tokenAddress =
    process.env.DIAP_TOKEN_ADDRESS ??
    process.env.DIAP_TOKEN ??
    process.env.TOKEN_ADDRESS ??
    "";
  const accountFactoryAddress =
    process.env.DIAP_ACCOUNT_FACTORY_ADDRESS ??
    process.env.ACCOUNT_FACTORY_ADDRESS ??
    "";

  if (!agentNetworkAddress || !tokenAddress) {
    throw new Error(
      "缺少 DIAP_AGENT_NETWORK_ADDRESS 或 DIAP_TOKEN_ADDRESS，请在 .env 中配置。"
    );
  }

  if (!accountFactoryAddress) {
    console.warn(
      "[警告] 未配置 DIAP_ACCOUNT_FACTORY_ADDRESS，虽然注册流程不直接使用，但建议同步记录。"
    );
  }

  const didDocumentInput =
    process.env.AGENT_DID ??
    process.env.DIAP_AGENT_DID ??
    "ipns/k51qzi5uqu5dik127hx8dsbqdosj4gehgtdahx5ynmi5wjdjqqb6uzj14o2127";
  const publicKeyInput =
    process.env.AGENT_PUBLIC_KEY ??
  process.env.DIAP_AGENT_PUBLIC_KEY ??
  didDocumentInput;

  const didDocument = didDocumentInput.startsWith("ipns/")
    ? didDocumentInput.slice(5)
    : didDocumentInput;
  const publicKey = publicKeyInput.startsWith("ipns/")
    ? publicKeyInput.slice(5)
    : publicKeyInput;
  const saltRaw =
    process.env.AGENT_AA_SALT ?? process.env.DIAP_AGENT_SALT ?? "42";

  const salt = BigInt(saltRaw);

  const [signer] = await ethers.getSigners();
  if (!signer) {
    throw new Error("无法获取 Hardhat signer，请检查网络配置。");
  }

  const signerAddress = await signer.getAddress();

  console.log(`使用账户: ${signerAddress}`);
  console.log(`DIAPAgentNetwork: ${agentNetworkAddress}`);
  console.log(`DIAPToken: ${tokenAddress}`);
  console.log(`DID Document (输入值): ${didDocumentInput}`);
  console.log(`DID Document (合约使用): ${didDocument}`);
  console.log(`Public Key (输入值): ${publicKeyInput}`);
  console.log(`Public Key (合约使用): ${publicKey}`);
  console.log(`Salt: ${salt}`);

  const agentNetwork = new ethers.Contract(
    agentNetworkAddress,
    agentNetworkArtifact.abi,
    signer
  );

  const token = new ethers.Contract(tokenAddress, tokenArtifact.abi, signer);

  const tokenDecimals = Number(await token.decimals());

  const [minStakeAmount, registrationFee, existingAgent, didOwner] =
    await Promise.all([
      agentNetwork.minStakeAmount(),
      agentNetwork.registrationFee(),
      agentNetwork.getAgent(signerAddress),
      agentNetwork.didToAgent(didDocument),
    ]);

  console.log("当前账户注册信息:", existingAgent);
  console.log("DID 对应地址:", didOwner);
  console.log(
    "AccountFactory 地址:",
    await agentNetwork.accountFactory().catch(() => "未知")
  );

  if ((existingAgent as AgentInfo).isActive) {
    console.log("当前账户已注册智能体，跳过注册流程。");
    console.log(`  DID: ${(existingAgent as AgentInfo).didDocument}`);
    console.log(`  AA Account: ${(existingAgent as AgentInfo).aaAccount}`);
    return;
  }

  if (
    didOwner &&
    didOwner !== ethers.ZeroAddress &&
    didOwner.toLowerCase() !== signerAddress.toLowerCase()
  ) {
    throw new Error(
      `该 DID 已被地址 ${didOwner} 使用，请更换 didDocument/IPNS 名称。`
    );
  }

  const totalRequired = minStakeAmount + registrationFee;

  console.log(
    `最小质押: ${formatUnits(minStakeAmount, tokenDecimals)} (${minStakeAmount} wei)`
  );
  console.log(
    `注册费用: ${formatUnits(registrationFee, tokenDecimals)} (${registrationFee} wei)`
  );
  console.log(
    `授权总额: ${formatUnits(totalRequired, tokenDecimals)} (${totalRequired} wei)`
  );

  const currentAllowance = await token.allowance(
    await signer.getAddress(),
    agentNetworkAddress
  );

  if (currentAllowance < totalRequired) {
    console.log(
      `当前授权不足 (${formatUnits(
        currentAllowance,
        tokenDecimals
      )})，发送 approve...`
    );
    const approveTx = await token.approve(agentNetworkAddress, totalRequired);
    console.log(`approve 交易已发送: ${approveTx.hash}`);
    await approveTx.wait();
    console.log("approve 已确认。");
  } else {
    console.log("现有授权充足，跳过 approve。");
  }

  const stakeAmountRaw =
    process.env.AGENT_STAKE_AMOUNT ?? process.env.DIAP_AGENT_STAKE_AMOUNT;
  const stakeAmount =
    stakeAmountRaw !== undefined
      ? parseUnits(stakeAmountRaw, tokenDecimals)
      : minStakeAmount;

  if (stakeAmount < minStakeAmount) {
    throw new Error(
      `质押金额 ${formatUnits(
        stakeAmount,
        tokenDecimals
      )} 小于最小质押要求 ${formatUnits(minStakeAmount, tokenDecimals)}`
    );
  }

  console.log(
    `调用 registerAgentWithAA，质押金额: ${formatUnits(
      stakeAmount,
      tokenDecimals
    )}`
  );

  const registerFn = agentNetwork.getFunction("registerAgentWithAA");

  try {
    await registerFn.staticCall(didDocument, publicKey, stakeAmount, salt);
  } catch (staticError: any) {
    console.error("registerAgentWithAA 预执行失败：", staticError);
    if (staticError?.data) {
      try {
        const parsed = agentNetwork.interface.parseError(staticError.data);
        console.error("合约错误:", parsed?.name, parsed?.args);
      } catch {
        console.error("无法解析错误数据:", staticError.data);
      }
    } else if (staticError?.errorName) {
      console.error("合约错误:", staticError.errorName, staticError.args);
    }
    throw staticError;
  }

  const registerTx = await registerFn(didDocument, publicKey, stakeAmount, salt);

  console.log(`注册交易已发送: ${registerTx.hash}`);
  const receipt = await registerTx.wait();
  console.log(`注册交易已确认，区块: ${receipt.blockNumber}`);

  const agentInfoRaw = (await agentNetwork.getAgent(
    await signer.getAddress()
  )) as AgentInfo;

  console.log("Agent 信息：");
  console.log(`  isActive: ${agentInfoRaw.isActive}`);
  console.log(`  isAAAccount: ${agentInfoRaw.isAAAccount}`);
  console.log(`  AA Account: ${agentInfoRaw.aaAccount}`);
  console.log(
    `  Staked Amount: ${formatUnits(agentInfoRaw.stakedAmount, tokenDecimals)}`
  );

  if (agentInfoRaw.aaAccount !== ethers.ZeroAddress) {
    const account = new ethers.Contract(
      agentInfoRaw.aaAccount,
      accountArtifact.abi,
      signer
    );
    const owner = await account.owner();
    const tokenBalance = await account.getTokenBalance(tokenAddress);

    console.log(`AA Account Owner: ${owner}`);
    console.log(
      `AA Account Token Balance: ${formatUnits(
        tokenBalance,
        tokenDecimals
      )} (${tokenBalance} wei)`
    );
  } else {
    console.warn("未检测到 AA Account 地址。");
  }

  console.log("注册流程完成。");
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});

