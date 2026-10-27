# Qurion Contracts

Solidity **0.8.24** protocol for Qurion on **BNB Smart Chain**. Hardhat.

Launching a meme token mints a **fixed supply to the human caller**, registers an agent, and opens an **empty** agent wallet. Studio records Build → Publish → Launch. An optional board sells owner inventory at a **fixed BNB price**. The default mandate is **paper-only**. `executeLive` **always reverts**. The session key **cannot withdraw**.

This is **not** a Binance CEX listing, **not** Flap Portal, and **not** audited.

The production console at [app.qurion.xyz](https://app.qurion.xyz) launches originals through **Flap** (`Portal.newTokenV6` on BNB). These contracts are the **optional Qurion factory** path: they only apply after you deploy them and set `NEXT_PUBLIC_FACTORY_ADDRESS` (and related addresses) in the app.

## Status

| | |
| --- | --- |
| Compiler | Solidity 0.8.24, optimizer 200 runs |
| Networks in config | Hardhat local, BSC testnet (97), BSC (56) |
| Audit | None |
| Mainnet factory | Not assumed. Do not treat empty `.env.example` addresses as deployed. |
| License | MIT (`SPDX-License-Identifier: MIT` on sources) |

Experimental. Meme tokens are high-risk. Do not point `deploy:testnet` at a funded mainnet key until mandates have been reviewed.

## Architecture

```
TokenFactory.launch(name, symbol, supply, agentURI)
        │
        ├─► MemeToken          fixed supply → msg.sender
        ├─► AgentWallet        empty; session cannot withdraw
        ├─► AgentRegistry      (owner, token, wallet, uri)
        └─► MandateManager     paper, approval on, autoExecute forced off

Studio.create → publish → bindLaunch(productId, agentId)
        └─► BoardSale          owner parks tokens; buyers pay BNB to the owner
```

### Contracts

| Contract | Responsibility |
| --- | --- |
| `core/MemeToken.sol` | Minimal ERC-20-like token. Supply minted once to the recipient. No later mint, tax, or blacklist. |
| `core/TokenFactory.sol` | Deploys token + wallet, registers the agent, writes the initial mandate. |
| `core/AgentRegistry.sol` | Agent identity. Factory is one-shot. |
| `core/AgentWallet.sol` | Paper buy/sell intents only. No `execute(address,bytes)`. No ERC-20 `approve` for a router. `executeLive` reverts. |
| `core/MandateManager.sol` | Size cap, daily cap, stop, allowlist, pause, revoke. Owner-only `setMandate`. `consume()` accounts paper notional; it never transfers. |
| `core/Studio.sol` | Product stages: draft → published → launched. Optional `startSale`. |
| `core/BoardSale.sol` | Fixed BNB price. Proceeds to the **owner**, not the agent. |
| `interfaces/` | Stable ABI surface for the app and this repo’s tests. |

### Mandate defaults (factory)

| Field | Default |
| --- | --- |
| `maxPosition` | 0.05 BNB |
| `dailyLimit` | 0.2 BNB |
| `stopLoss` | 15% (1500 bps) |
| `approvalRequired` | true |
| `autoExecute` | false (cannot be enabled in this version) |
| `paperOnly` | true |

`MAX_TOKENS` on a mandate is 16. Stop must be in `(0, 10000]` bps.

## Invariants (do not weaken)

1. **Mint to the human.** `MemeToken` balance of `msg.sender` equals `supply`. Agent wallet token balance is 0 at launch.
2. **No withdraw from the agent path.** Wallet has no owner-drain called by the session key.
3. **No live.** `executeLive` reverts. There is no generic call target.
4. **Paper consume is accounting.** `MandateManager.consume` does not move ERC-20 or BNB.
5. **Pause / revoke are owner acts.** They clear the right to submit. They are not an agent privilege.
6. **Board proceeds are owner proceeds.** The agent cannot start or skim a sale.
7. **Not a CEX listing.** Nothing here lists a token on Binance the exchange.

If a future version adds live execution, it must be a **new contract** and a second owner approval — not a flag flip on this code.

## Layout

```
core/           Implementations
interfaces/     ITokenFactory, IAgentRegistry, IAgentWallet, IMandateManager, IStudio, IERC20
scripts/        deploy.js — deploys registry, mandates, factory, studio; links factory
test/           AgentProtocol.js, Studio.js
hardhat.config.js
.env.example    Empty DEPLOYER_PRIVATE_KEY. Copy locally. Never commit a filled copy.
```

`paths.sources` is the repo root; Hardhat is told to ignore `node_modules`, `artifacts`, and `cache`.

## Develop

Node 20+.

```bash
npm install
npm test
npm run build
```

Tests cover:

- Supply mints to the caller; agent wallet stays empty.
- Studio walk: create → publish → `TokenFactory.launch` → `bindLaunch` → BNB board buy.

## Deploy

Local:

```bash
npx hardhat node          # other terminal
npm run deploy:local
```

Testnet:

```bash
cp .env.example .env      # fill DEPLOYER_PRIVATE_KEY locally only
# .env is gitignored
npm run deploy:testnet
```

`hardhat.config.js` reads `DEPLOYER_PRIVATE_KEY`, `BSC_TESTNET_RPC`, and `BSC_RPC` from the **environment**. Empty accounts array if no key — compile and tests still work.

The script prints:

```json
{
  "chainId": 97,
  "registry": "0x…",
  "mandates": "0x…",
  "factory": "0x…",
  "studio": "0x…",
  "note": "Live execution is disabled. Default mandates are paper-only. Studio is Build → Publish → Launch on BNB."
}
```

Wire those into the console as public addresses (not keys):

```
NEXT_PUBLIC_FACTORY_ADDRESS=
NEXT_PUBLIC_MANDATE_ADDRESS=
NEXT_PUBLIC_WALLET_ADDRESS=
NEXT_PUBLIC_STUDIO_ADDRESS=
```

Until those are set, Launch in the app still uses **Flap**, and the factory button stays disabled.

**Do not** run this against BSC mainnet from a key you cannot afford to lose. There is no audit and no bug bounty in this repo.

## Public GitHub

Safe to publish **if** `.env` stays out of git (it is gitignored) and you never paste a deployer key into `hardhat.config.js`.

Check before push:

```bash
git status
git check-ignore -v .env
git grep -E 'DEPLOYER_PRIVATE_KEY=0x' || true
```

Also gitignored: `node_modules/`, `cache/`, `artifacts/`. Recompile after clone. Do not commit a keystore JSON.

On-chain bytecode is public once deployed anyway. The risk of a public repo is **keys**, **false “audited / listed” claims**, and people forking the factory as if it were Flap.

Flap Portal on BNB (`0xe2cE6ab80874Fa9Fa2aAE65D277Dd6B8e65C9De0`) is **not** in this repository.

## Related

| Surface | Role |
| --- | --- |
| Qurion agent | Local TS layer: paper intents, architect draft, factory **args** only |
| `main-app` | Next console; Flap launch; optional factory if addresses are set |
| `landing-page` | qurion.xyz |

---

Experimental software. High-risk if used with real meme inventory. The agent cannot withdraw. Live stays off.
