const { expect } = require("chai");
const { ethers } = require("hardhat");
const { loadFixture } = require("@nomicfoundation/hardhat-network-helpers");

describe("AgentProtocol", function () {
  async function deployFixture() {
    const [admin, user, session, stranger] = await ethers.getSigners();
    const registry = await ethers.deployContract("AgentRegistry");
    const mandates = await ethers.deployContract("MandateManager", [await registry.getAddress()]);
    const factory = await ethers.deployContract("TokenFactory", [await registry.getAddress(), await mandates.getAddress()]);
    await registry.setFactory(await factory.getAddress());
    await mandates.setFactory(await factory.getAddress());

    const supply = ethers.parseEther("1000000");
    await factory.connect(user).launch("Sample", "SAMP", supply, "ipfs://agent-sample");
    const info = await registry.getAgent(1);
    const wallet = await ethers.getContractAt("AgentWallet", info.wallet);
    const token = await ethers.getContractAt("MemeToken", info.token);
    const expiry = (await ethers.provider.getBlock("latest")).timestamp + 86_400;
    await wallet.connect(user).setSession(session.address, expiry, ethers.parseEther("1"));

    return { admin, user, session, stranger, registry, mandates, factory, wallet, token, supply };
  }

  function buy(token, notional) {
    return {
      kind: 1,
      tokenIn: ethers.ZeroAddress,
      tokenOut: token,
      notionalWei: notional,
      maxSlippageBps: 100,
      strategyId: ethers.id("paper"),
    };
  }

  it("mints the supply to the human and leaves the agent wallet empty", async function () {
    const { user, wallet, token, supply } = await loadFixture(deployFixture);
    expect(await token.balanceOf(user.address)).to.equal(supply);
    expect(await token.balanceOf(await wallet.getAddress())).to.equal(0n);
    expect(await ethers.provider.getBalance(await wallet.getAddress())).to.equal(0n);
  });

  it("starts in paper mode and requires approval", async function () {
    const { mandates } = await loadFixture(deployFixture);
    const mandate = await mandates.getMandate(1);
    expect(mandate.paperOnly).to.equal(true);
    expect(mandate.approvalRequired).to.equal(true);
    expect(mandate.autoExecute).to.equal(false);
    expect(mandate.maxPositionWei).to.equal(ethers.parseEther("0.05"));
  });

  it("records a paper intent without moving balances", async function () {
    const { user, session, wallet, token } = await loadFixture(deployFixture);
    const walletAddress = await wallet.getAddress();
    await user.sendTransaction({ to: walletAddress, value: ethers.parseEther("0.2") });
    const before = await ethers.provider.getBalance(walletAddress);

    await wallet.connect(session).submitIntent(buy(await token.getAddress(), ethers.parseEther("0.01")));

    expect(await ethers.provider.getBalance(walletAddress)).to.equal(before);
    expect(await token.balanceOf(walletAddress)).to.equal(0n);
    const intent = await wallet.intents(1);
    expect(intent.status).to.equal(1n);
    await wallet.connect(user).approveIntent(1);
    expect((await wallet.intents(1)).status).to.equal(2n);
    expect(await ethers.provider.getBalance(walletAddress)).to.equal(before);
  });

  it("rejects the session key on withdraw, live execution, and arbitrary calls", async function () {
    const { user, session, wallet } = await loadFixture(deployFixture);
    const walletAddress = await wallet.getAddress();
    await user.sendTransaction({ to: walletAddress, value: ethers.parseEther("0.2") });

    await expect(
      wallet.connect(session).ownerWithdraw(ethers.ZeroAddress, ethers.parseEther("0.2"), user.address),
    ).to.be.revertedWithCustomError(wallet, "NotOwner");

    await expect(
      wallet.connect(user).ownerWithdraw(ethers.ZeroAddress, ethers.parseEther("0.1"), session.address),
    ).to.be.revertedWithCustomError(wallet, "SessionDestination");

    await expect(wallet.connect(user).ownerWithdraw(ethers.ZeroAddress, ethers.parseEther("0.1"), user.address))
      .to.changeEtherBalances([wallet, user], [ethers.parseEther("-0.1"), ethers.parseEther("0.1")]);

    await expect(wallet.connect(session).executeLive(1)).to.be.revertedWithCustomError(wallet, "LiveExecutionDisabled");
    await expect(wallet.connect(user).executeLive(1)).to.be.revertedWithCustomError(wallet, "LiveExecutionDisabled");
    await expect(user.sendTransaction({ to: walletAddress, data: "0xdeadbeef" })).to.be.revertedWithCustomError(wallet, "BadAction");
  });

  it("enforces size, allowlist, and daily cap", async function () {
    const { user, session, mandates, wallet, token } = await loadFixture(deployFixture);
    const tokenAddress = await token.getAddress();

    await expect(
      wallet.connect(session).submitIntent(buy(tokenAddress, ethers.parseEther("1"))),
    ).to.be.revertedWithCustomError(mandates, "OverPosition");

    await expect(
      wallet.connect(session).submitIntent(buy(user.address, ethers.parseEther("0.01"))),
    ).to.be.revertedWithCustomError(mandates, "NotAllowed");

    await mandates.connect(user).setMandate(
      1,
      ethers.parseEther("0.05"),
      ethers.parseEther("0.05"),
      1_500,
      true,
      false,
      true,
      [tokenAddress],
    );
    await wallet.connect(session).submitIntent(buy(tokenAddress, ethers.parseEther("0.05")));
    await expect(
      wallet.connect(session).submitIntent(buy(tokenAddress, ethers.parseEther("0.05"))),
    ).to.be.revertedWithCustomError(mandates, "OverDaily");
  });

  it("refuses live or auto mandates, and revoke kills the session", async function () {
    const { user, session, mandates, wallet, token, stranger } = await loadFixture(deployFixture);
    const tokenAddress = await token.getAddress();

    await expect(mandates.connect(session).setMandate(
      1,
      ethers.parseEther("0.05"),
      ethers.parseEther("0.2"),
      1_500,
      true,
      false,
      true,
      [tokenAddress],
    )).to.be.revertedWithCustomError(mandates, "NotOwner");

    await expect(mandates.connect(user).setMandate(
      1,
      ethers.parseEther("0.05"),
      ethers.parseEther("0.2"),
      1_500,
      false,
      true,
      false,
      [tokenAddress],
    )).to.be.revertedWithCustomError(mandates, "AutoDisabled");

    await mandates.connect(user).revoke(1);
    expect(await wallet.sessionActive()).to.equal(false);
    await expect(
      wallet.connect(session).submitIntent(buy(tokenAddress, ethers.parseEther("0.01"))),
    ).to.be.revertedWithCustomError(wallet, "BadSession");
    await expect(
      wallet.connect(stranger).submitIntent(buy(tokenAddress, ethers.parseEther("0.01"))),
    ).to.be.revertedWithCustomError(wallet, "BadSession");
  });
});
