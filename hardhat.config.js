require("@nomicfoundation/hardhat-toolbox");
const { subtask } = require("hardhat/config");
const { TASK_COMPILE_SOLIDITY_GET_SOURCE_PATHS } = require("hardhat/builtin-tasks/task-names");

// The Solidity sources sit next to this config. Ignore dependencies and build output.
subtask(TASK_COMPILE_SOLIDITY_GET_SOURCE_PATHS).setAction(async (_, __, runSuper) => {
  const paths = await runSuper();
  return paths.filter((file) => !file.includes(`${require("path").sep}node_modules${require("path").sep}`)
    && !file.includes(`${require("path").sep}artifacts${require("path").sep}`)
    && !file.includes(`${require("path").sep}cache${require("path").sep}`));
});

// Never commit a key. The deploy script reads DEPLOYER_PRIVATE_KEY from the environment
// only when you intentionally broadcast. Local tests do not need one.
const deployerKey = process.env.DEPLOYER_PRIVATE_KEY;

/** @type import('hardhat/config').HardhatUserConfig */
module.exports = {
  solidity: {
    version: "0.8.24",
    settings: { optimizer: { enabled: true, runs: 200 } },
  },
  paths: {
    sources: ".",
    tests: "./test",
    cache: "./cache",
    artifacts: "./artifacts",
  },
  networks: {
    bscTestnet: {
      url: process.env.BSC_TESTNET_RPC || "https://data-seed-prebsc-1-s1.binance.org:8545",
      chainId: 97,
      accounts: deployerKey ? [deployerKey] : [],
    },
    bsc: {
      url: process.env.BSC_RPC || "https://bsc-dataseed.binance.org",
      chainId: 56,
      accounts: deployerKey ? [deployerKey] : [],
    },
  },
};
