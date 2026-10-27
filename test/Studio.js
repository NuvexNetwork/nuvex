const { expect } = require("chai");
const { ethers } = require("hardhat");
const { loadFixture } = require("@nomicfoundation/hardhat-network-helpers");

describe("Studio", function () {
  async function deployFixture() {
    const [admin, user, buyer] = await ethers.getSigners();
    const registry = await ethers.deployContract("AgentRegistry");
    const mandates = await ethers.deployContract("MandateManager", [await registry.getAddress()]);
    const factory = await ethers.deployContract("TokenFactory", [await registry.getAddress(), await mandates.getAddress()]);
    await registry.setFactory(await factory.getAddress());
    await mandates.setFactory(await factory.getAddress());
    const studio = await ethers.deployContract("Studio", [await registry.getAddress()]);
    return { admin, user, buyer, registry, factory, studio };
  }

  it("walks draft → publish → factory launch → bind → BNB board", async function () {
    const { user, buyer, studio, factory, registry } = await loadFixture(deployFixture);
    const hash = ethers.id("bundle-v1");
    await studio.connect(user).create("Harbor Note", "HARB", "A desk that watches a dump.", hash);
    await studio.connect(user).publish(1);

    const supply = ethers.parseEther("1000000");
    await factory.connect(user).launch("Harbor Note", "HARB", supply, "qurion://product/1");
    await studio.connect(user).bindLaunch(1, 1);

    const product = await studio.getProduct(1);
    expect(product.stage).to.equal(3);
    expect(product.agentId).to.equal(1n);
    const info = await registry.getAgent(1);
    expect(product.token).to.equal(info.token);

    const token = await ethers.getContractAt("MemeToken", product.token);
    const amount = ethers.parseEther("1000");
    const price = ethers.parseEther("0.001");
    await token.connect(user).approve(await studio.getAddress(), amount);
    await studio.connect(user).startSale(1, amount, price);

    const saleAddr = (await studio.getProduct(1)).sale;
    const sale = await ethers.getContractAt("BoardSale", saleAddr);
    await sale.connect(buyer).buy({ value: price });
    expect(await token.balanceOf(buyer.address)).to.equal(ethers.parseEther("1"));

    const before = await ethers.provider.getBalance(user.address);
    await sale.connect(user).take();
    expect(await ethers.provider.getBalance(user.address)).to.be.greaterThan(before - ethers.parseEther("0.01"));
  });

  it("keeps revise closed after publish and bind closed for a stranger", async function () {
    const { user, buyer, studio, factory } = await loadFixture(deployFixture);
    await studio.connect(user).create("Quiet", "QUIET", "brief", ethers.id("a"));
    await expect(studio.connect(buyer).publish(1)).to.be.revertedWithCustomError(studio, "NotOwner");
    await studio.connect(user).publish(1);
    await expect(studio.connect(user).revise(1, "no", ethers.id("b"))).to.be.revertedWithCustomError(studio, "BadStage");
    await factory.connect(user).launch("Quiet", "QUIET", ethers.parseEther("1"), "qurion://product/1");
    await expect(studio.connect(buyer).bindLaunch(1, 1)).to.be.revertedWithCustomError(studio, "NotOwner");
  });
});
