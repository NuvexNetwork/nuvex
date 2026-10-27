const hre = require("hardhat");

/**
 * Deploys the four core contracts and links the factory.
 * This script does not launch a token and does not print a private key.
 *
 * EXPERIMENTAL. Do not point this at mainnet until the mandates have been reviewed.
 */
async function main() {
  const [deployer] = await hre.ethers.getSigners();
  if (!deployer) {
    throw new Error("No deployer account. Set DEPLOYER_PRIVATE_KEY for a public network, or use the local Hardhat node.");
  }

  console.log("Deployer", deployer.address);

  const registry = await hre.ethers.deployContract("AgentRegistry");
  await registry.waitForDeployment();
  const mandates = await hre.ethers.deployContract("MandateManager", [await registry.getAddress()]);
  await mandates.waitForDeployment();
  const factory = await hre.ethers.deployContract("TokenFactory", [await registry.getAddress(), await mandates.getAddress()]);
  await factory.waitForDeployment();

  await (await registry.setFactory(await factory.getAddress())).wait();
  await (await mandates.setFactory(await factory.getAddress())).wait();
  const studio = await hre.ethers.deployContract("Studio", [await registry.getAddress()]);
  await studio.waitForDeployment();

  console.log(JSON.stringify({
    chainId: Number((await hre.ethers.provider.getNetwork()).chainId),
    registry: await registry.getAddress(),
    mandates: await mandates.getAddress(),
    factory: await factory.getAddress(),
    studio: await studio.getAddress(),
    note: "Live execution is disabled. Default mandates are paper-only. Studio is Build → Publish → Launch on BNB.",
  }, null, 2));
}

main().catch((error) => {
  console.error(error);
  process.exit(1);
});
