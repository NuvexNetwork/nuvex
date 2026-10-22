// SPDX-License-Identifier: MIT
pragma solidity 0.8.24;

/// @notice A proposed buy or sell. This is not a withdrawal.
/// @dev kind: 1 = buy, 2 = sell. No other kind is accepted.
struct AgentAction {
    uint8 kind;
    address tokenIn;
    address tokenOut;
    uint256 notionalWei;
    uint16 maxSlippageBps;
    bytes32 strategyId;
}

interface IAgentWallet {
    event SessionSet(uint256 indexed agentId, address indexed signer, uint64 expiry, uint256 spendCap);
    event SessionCleared(uint256 indexed agentId);
    event PaperIntent(uint256 indexed agentId, uint256 indexed intentId, uint8 kind, uint256 notionalWei, bool needsApproval);
    event PaperApproved(uint256 indexed intentId);
    event OwnerWithdraw(uint256 indexed agentId, address indexed token, address indexed to, uint256 amount);

    function setSession(address signer, uint64 expiry, uint256 spendCap) external;
    function clearSession() external;
    function submitIntent(AgentAction calldata action) external returns (uint256 intentId);
    function approveIntent(uint256 intentId) external;

    /// @notice User-only. The session signer cannot call this.
    function ownerWithdraw(address token, uint256 amount, address to) external;

    /// @notice Permanently disabled in this version. Kept so callers fail closed.
    function executeLive(uint256 intentId) external pure;

    function onOwnerChanged() external;
    function onMandateRevoked() external;
}
