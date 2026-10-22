# Contracts

BNB Chain contracts for AgentProtocol. Solidity 0.8.24, Hardhat.

Launching a meme token mints a fixed supply to the caller and registers an agent with an empty wallet. The default mandate is paper-only. `executeLive` always reverts, and the session key cannot withdraw.

```bash
npm install
npm test
```

Copy `.env.example` before a testnet deploy. Do not commit a key.

This repository is experimental. Meme tokens are high-risk.
