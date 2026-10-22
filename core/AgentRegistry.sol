// SPDX-License-Identifier: MIT
pragma solidity 0.8.24;

import "../interfaces/IAgentRegistry.sol";
import "../interfaces/IAgentWallet.sol";

/// @title AgentRegistry
/// @notice ERC-8004-style identity directory: one id, one owner, one URI, one wallet.
/// @dev EXPERIMENTAL. Not a complete ERC-8004 stack (no reputation registry, no validation registry).
///      Only the token factory can register agents in this version, so a caller cannot spoof an identity.
contract AgentRegistry is IAgentRegistry {
    address public immutable admin;
    address public factory;
    uint256 public nextId = 1;

    mapping(uint256 => AgentInfo) private _agents;
    mapping(address => uint256) public walletToAgent;

    error NotAdmin();
    error NotFactory();
    error NotOwner();
    error FactoryAlreadySet();
    error ZeroAddress();
    error UnknownAgent();
    error WalletTaken();

    modifier onlyAdmin() {
        if (msg.sender != admin) revert NotAdmin();
        _;
    }

    modifier onlyAgentOwner(uint256 agentId) {
        if (msg.sender != _require(agentId).owner) revert NotOwner();
        _;
    }

    constructor() {
        admin = msg.sender;
    }

    /// @notice One-time link set by the deployer after the factory exists.
    function setFactory(address factory_) external onlyAdmin {
        if (factory != address(0)) revert FactoryAlreadySet();
        if (factory_ == address(0)) revert ZeroAddress();
        factory = factory_;
    }

    function register(address owner, address token, address wallet, string calldata agentURI)
        external
        returns (uint256 agentId)
    {
        if (msg.sender != factory) revert NotFactory();
        if (owner == address(0) || wallet == address(0)) revert ZeroAddress();
        if (walletToAgent[wallet] != 0) revert WalletTaken();

        agentId = nextId++;
        _agents[agentId] = AgentInfo({
            owner: owner,
            token: token,
            wallet: wallet,
            agentURI: agentURI,
            active: true,
            createdAt: uint64(block.timestamp)
        });
        walletToAgent[wallet] = agentId;
        emit AgentRegistered(agentId, owner, token, wallet, agentURI);
    }

    function setAgentURI(uint256 agentId, string calldata agentURI) external onlyAgentOwner(agentId) {
        _agents[agentId].agentURI = agentURI;
        emit AgentUpdated(agentId, agentURI, _agents[agentId].active);
    }

    function setActive(uint256 agentId, bool active) external onlyAgentOwner(agentId) {
        _agents[agentId].active = active;
        emit AgentUpdated(agentId, _agents[agentId].agentURI, active);
    }

    /// @notice Moves the identity. The wallet drops its session key in the same transaction.
    function transferOwnership(uint256 agentId, address nextOwner) external onlyAgentOwner(agentId) {
        if (nextOwner == address(0)) revert ZeroAddress();
        AgentInfo storage info = _agents[agentId];
        address previous = info.owner;
        info.owner = nextOwner;
        IAgentWallet(info.wallet).onOwnerChanged();
        emit OwnershipTransferred(agentId, previous, nextOwner);
    }

    function ownerOf(uint256 agentId) external view returns (address) {
        return _require(agentId).owner;
    }

    function walletOf(uint256 agentId) external view returns (address) {
        return _require(agentId).wallet;
    }

    function tokenOf(uint256 agentId) external view returns (address) {
        return _require(agentId).token;
    }

    function getAgent(uint256 agentId) external view returns (AgentInfo memory) {
        return _require(agentId);
    }

    function _require(uint256 agentId) internal view returns (AgentInfo storage info) {
        info = _agents[agentId];
        if (info.owner == address(0)) revert UnknownAgent();
    }
}
