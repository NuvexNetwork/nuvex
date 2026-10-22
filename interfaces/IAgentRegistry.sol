// SPDX-License-Identifier: MIT
pragma solidity 0.8.24;

/// @notice Identity record for one agent.
/// @dev Inspired by the ERC-8004 idea of an on-chain agent id, owner, and URI.
///      This is NOT a full ERC-8004 deployment. There is no reputation registry here.
struct AgentInfo {
    address owner;
    address token;
    address wallet;
    string agentURI;
    bool active;
    uint64 createdAt;
}

/// @notice On-chain identity directory. The factory is the only registrar in v1.
interface IAgentRegistry {
    event AgentRegistered(
        uint256 indexed agentId,
        address indexed owner,
        address indexed token,
        address wallet,
        string agentURI
    );
    event AgentUpdated(uint256 indexed agentId, string agentURI, bool active);
    event OwnershipTransferred(uint256 indexed agentId, address indexed previousOwner, address indexed nextOwner);

    function register(address owner, address token, address wallet, string calldata agentURI) external returns (uint256 agentId);
    function setAgentURI(uint256 agentId, string calldata agentURI) external;
    function setActive(uint256 agentId, bool active) external;
    function transferOwnership(uint256 agentId, address nextOwner) external;

    function ownerOf(uint256 agentId) external view returns (address);
    function walletOf(uint256 agentId) external view returns (address);
    function tokenOf(uint256 agentId) external view returns (address);
    function getAgent(uint256 agentId) external view returns (AgentInfo memory);
}
