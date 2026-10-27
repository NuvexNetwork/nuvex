// SPDX-License-Identifier: MIT
pragma solidity 0.8.24;

/// @notice On-chain product record for the Build → Publish → Launch loop.
/// @dev Files live off-chain. The hash lets anyone check the published bundle.
struct Product {
    address owner;
    string name;
    string symbol;
    string brief;
    bytes32 contentHash;
    uint8 stage;
    uint256 agentId;
    address token;
    address wallet;
    address sale;
    uint64 createdAt;
}

interface IStudio {
    event ProductCreated(uint256 indexed productId, address indexed owner, string name, string symbol);
    event ProductRevised(uint256 indexed productId, bytes32 contentHash);
    event ProductPublished(uint256 indexed productId);
    event ProductLaunched(uint256 indexed productId, uint256 indexed agentId, address indexed token, address wallet);
    event SaleOpened(uint256 indexed productId, address indexed sale, uint256 amount, uint256 priceWei);

    function create(string calldata name, string calldata symbol, string calldata brief, bytes32 contentHash)
        external
        returns (uint256 productId);
}
