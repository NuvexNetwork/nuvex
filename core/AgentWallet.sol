// SPDX-License-Identifier: MIT
pragma solidity 0.8.24;

import "../interfaces/IAgentWallet.sol";
import "../interfaces/IAgentRegistry.sol";
import "../interfaces/IMandateManager.sol";
import "../interfaces/IERC20.sol";

/// @title AgentWallet
/// @notice Permissioned wallet for one agent.
/// @dev EXPERIMENTAL. High-risk if used with real meme positions.
///
///      What the agent (session signer) CAN do:
///        submit a paper buy/sell intent inside the mandate.
///
///      What the agent CANNOT do:
///        withdraw BNB or tokens, approve a router, call an arbitrary contract,
///        change the mandate, or turn on live execution.
///
///      executeLive always reverts. There is no generic execute(address,bytes).
contract AgentWallet is IAgentWallet {
    uint8 public constant KIND_BUY = 1;
    uint8 public constant KIND_SELL = 2;
    uint16 public constant MAX_SLIPPAGE_BPS = 1_000;
    uint8 public constant STATUS_PENDING = 1;
    uint8 public constant STATUS_PAPER = 2;

    struct Intent {
        uint8 kind;
        uint8 status;
        address tokenIn;
        address tokenOut;
        uint256 notionalWei;
        uint16 maxSlippageBps;
        bytes32 strategyId;
        address submitter;
    }

    IAgentRegistry public immutable registry;
    IMandateManager public immutable mandates;
    address public immutable factory;
    address public immutable initialOwner;

    uint256 public agentId;
    address public sessionSigner;
    uint64 public sessionExpiry;
    uint256 public sessionCap;
    uint256 public sessionSpent;
    bool public sessionActive;
    uint256 public nextIntent = 1;
    uint256 private _lock = 1;

    mapping(uint256 => Intent) public intents;

    error NotOwner();
    error NotFactory();
    error NotRegistry();
    error NotMandate();
    error ZeroAddress();
    error AlreadyBound();
    error NotBound();
    error BadSession();
    error SessionExpired();
    error SessionCapExceeded();
    error BadAction();
    error Slippage();
    error UnknownIntent();
    error NotPending();
    error SessionDestination();
    error LiveExecutionDisabled();
    error Reentrancy();
    error TransferFailed();

    modifier onlyOwner() {
        if (msg.sender != owner()) revert NotOwner();
        _;
    }

    modifier nonReentrant() {
        if (_lock != 1) revert Reentrancy();
        _lock = 2;
        _;
        _lock = 1;
    }

    constructor(address initialOwner_, address registry_, address mandates_) {
        if (initialOwner_ == address(0) || registry_ == address(0) || mandates_ == address(0)) revert ZeroAddress();
        initialOwner = initialOwner_;
        registry = IAgentRegistry(registry_);
        mandates = IMandateManager(mandates_);
        factory = msg.sender;
    }

    function owner() public view returns (address) {
        if (agentId == 0) return initialOwner;
        return registry.ownerOf(agentId);
    }

    function bind(uint256 agentId_) external {
        if (msg.sender != factory) revert NotFactory();
        if (agentId != 0) revert AlreadyBound();
        if (registry.walletOf(agentId_) != address(this)) revert NotBound();
        agentId = agentId_;
    }

    /// @notice Limited signer. Cap is an extra ceiling on top of the mandate, not a way around it.
    function setSession(address signer, uint64 expiry, uint256 spendCap) external onlyOwner {
        if (signer == address(0) || signer == address(this)) revert BadSession();
        if (expiry <= block.timestamp || spendCap == 0) revert BadSession();
        sessionSigner = signer;
        sessionExpiry = expiry;
        sessionCap = spendCap;
        sessionSpent = 0;
        sessionActive = true;
        emit SessionSet(agentId, signer, expiry, spendCap);
    }

    function clearSession() external onlyOwner {
        _clearSession();
    }

    function onOwnerChanged() external {
        if (msg.sender != address(registry)) revert NotRegistry();
        _clearSession();
    }

    function onMandateRevoked() external {
        if (msg.sender != address(mandates)) revert NotMandate();
        _clearSession();
    }

    /// @notice Paper-only. Balances of this contract do not change here.
    function submitIntent(AgentAction calldata action) external nonReentrant returns (uint256 intentId) {
        if (agentId == 0) revert NotBound();
        bool fromOwner = msg.sender == owner();
        if (!fromOwner) {
            if (!sessionActive || msg.sender != sessionSigner) revert BadSession();
            if (block.timestamp > sessionExpiry) revert SessionExpired();
        }
        if (action.kind != KIND_BUY && action.kind != KIND_SELL) revert BadAction();
        if (action.maxSlippageBps > MAX_SLIPPAGE_BPS) revert Slippage();

        bool needsApproval = mandates.consume(agentId, action.tokenIn, action.tokenOut, action.notionalWei);

        if (!fromOwner) {
            uint256 nextSpent = sessionSpent + action.notionalWei;
            if (nextSpent > sessionCap) revert SessionCapExceeded();
            sessionSpent = nextSpent;
        }

        intentId = nextIntent++;
        intents[intentId] = Intent({
            kind: action.kind,
            status: needsApproval ? STATUS_PENDING : STATUS_PAPER,
            tokenIn: action.tokenIn,
            tokenOut: action.tokenOut,
            notionalWei: action.notionalWei,
            maxSlippageBps: action.maxSlippageBps,
            strategyId: action.strategyId,
            submitter: msg.sender
        });
        emit PaperIntent(agentId, intentId, action.kind, action.notionalWei, needsApproval);
    }

    /// @notice Owner accepts a pending paper intent. Still does not move funds.
    function approveIntent(uint256 intentId) external onlyOwner nonReentrant {
        Intent storage intent = intents[intentId];
        if (intent.submitter == address(0)) revert UnknownIntent();
        if (intent.status != STATUS_PENDING) revert NotPending();
        intent.status = STATUS_PAPER;
        emit PaperApproved(intentId);
    }

    /// @notice Human owner pulls funds. Cannot target the session signer.
    function ownerWithdraw(address token, uint256 amount, address to) external onlyOwner nonReentrant {
        if (to == address(0)) revert ZeroAddress();
        if (sessionActive && to == sessionSigner) revert SessionDestination();
        if (amount == 0) revert BadAction();
        if (token == address(0)) {
            (bool ok, ) = to.call{value: amount}("");
            if (!ok) revert TransferFailed();
        } else {
            if (!IERC20(token).transfer(to, amount)) revert TransferFailed();
        }
        emit OwnerWithdraw(agentId, token, to, amount);
    }

    /// @notice Fail closed. A future version must be a new contract, not a flag flip.
    function executeLive(uint256) external pure {
        revert LiveExecutionDisabled();
    }

    receive() external payable {}

    /// @notice Rejects arbitrary calls, including router and token approvals from this wallet.
    fallback() external payable {
        revert BadAction();
    }

    function _clearSession() internal {
        sessionActive = false;
        sessionSigner = address(0);
        sessionExpiry = 0;
        sessionCap = 0;
        sessionSpent = 0;
        emit SessionCleared(agentId);
    }
}
