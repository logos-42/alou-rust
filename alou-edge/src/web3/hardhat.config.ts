import { HardhatUserConfig } from "hardhat/config";
import "@nomicfoundation/hardhat-toolbox";
import * as dotenv from "dotenv";

dotenv.config();

const rpcUrl =
  process.env.ETH_RPC_URL ??
  process.env.SEPOLIA_RPC_URL ??
  process.env.TESTNET_RPC_URL ??
  "";
const privateKey =
  process.env.PRIVATE_KEY ??
  process.env.SEPOLIA_PRIVATE_KEY ??
  process.env.TESTNET_PRIVATE_KEY ??
  "";

const accounts = privateKey ? [privateKey] : [];

const config: HardhatUserConfig = {
  solidity: "0.8.30",
  defaultNetwork: "hardhat",
  paths: {
    sources: "./contracts",
    artifacts: "./artifacts",
    cache: "./cache",
    tests: "./test",
  },
  networks: {
    hardhat: {},
    sepolia: {
      url: rpcUrl,
      accounts,
    },
  },
};

export default config;

