// SPDX-License-Identifier: MIT
pragma solidity 0.8.24;

/// @notice Launches a fixed-supply meme token and an agent identity together.
/// @dev EXPERIMENTAL and high-risk. Meme tokens often go to zero.
///      The launch does not fund the agent and does not enable live trading.
interface ITokenFactory {
    event TokenLaunched(uint256 indexed agentId, address indexed owner, address indexed token, address wallet);

    function launch(string calldata name, string calldata symbol, uint256 supply, string calldata agentURI)
        external
        returns (uint256 agentId, address token, address wallet);
}
