// SPDX-License-Identifier: MIT
pragma solidity 0.8.24;

import "../interfaces/IStudio.sol";
import "../interfaces/IAgentRegistry.sol";
import "../interfaces/IERC20.sol";
import "./BoardSale.sol";

/// @title Studio
/// @notice Build → publish → launch a product token on BNB Chain, then optionally open a BNB board.
/// @dev EXPERIMENTAL. Not a CEX listing. The agent still cannot withdraw.
contract Studio is IStudio {
    uint8 public constant STAGE_DRAFT = 1;
    uint8 public constant STAGE_PUBLISHED = 2;
    uint8 public constant STAGE_LAUNCHED = 3;

    IAgentRegistry public immutable registry;
    uint256 public nextId = 1;
    mapping(uint256 => Product) private _products;

    error ZeroAddress();
    error EmptyString();
    error NotOwner();
    error BadStage();
    error UnknownProduct();
    error AgentMismatch();
    error TransferFailed();
    error BadValue();

    constructor(address registry_) {
        if (registry_ == address(0)) revert ZeroAddress();
        registry = IAgentRegistry(registry_);
    }

    function create(string calldata name, string calldata symbol, string calldata brief, bytes32 contentHash)
        external
        returns (uint256 productId)
    {
        if (bytes(name).length == 0 || bytes(symbol).length == 0) revert EmptyString();
        productId = nextId++;
        _products[productId] = Product({
            owner: msg.sender,
            name: name,
            symbol: symbol,
            brief: brief,
            contentHash: contentHash,
            stage: STAGE_DRAFT,
            agentId: 0,
            token: address(0),
            wallet: address(0),
            sale: address(0),
            createdAt: uint64(block.timestamp)
        });
        emit ProductCreated(productId, msg.sender, name, symbol);
    }

    function revise(uint256 productId, string calldata brief, bytes32 contentHash) external {
        Product storage product = _requireOwner(productId);
        if (product.stage != STAGE_DRAFT) revert BadStage();
        product.brief = brief;
        product.contentHash = contentHash;
        emit ProductRevised(productId, contentHash);
    }

    function publish(uint256 productId) external {
        Product storage product = _requireOwner(productId);
        if (product.stage != STAGE_DRAFT) revert BadStage();
        product.stage = STAGE_PUBLISHED;
        emit ProductPublished(productId);
    }

    /// @notice Bind a factory launch you already signed. The registry must name you as owner.
    function bindLaunch(uint256 productId, uint256 agentId) external {
        Product storage product = _requireOwner(productId);
        if (product.stage != STAGE_PUBLISHED) revert BadStage();
        AgentInfo memory info = registry.getAgent(agentId);
        if (info.owner != msg.sender) revert AgentMismatch();
        product.stage = STAGE_LAUNCHED;
        product.agentId = agentId;
        product.token = info.token;
        product.wallet = info.wallet;
        emit ProductLaunched(productId, agentId, info.token, info.wallet);
    }

    function startSale(uint256 productId, uint256 amount, uint256 priceWei) external returns (address sale) {
        Product storage product = _requireOwner(productId);
        if (product.stage != STAGE_LAUNCHED) revert BadStage();
        if (product.sale != address(0) || amount == 0 || priceWei == 0) revert BadValue();
        BoardSale created = new BoardSale(product.token, msg.sender, priceWei);
        sale = address(created);
        product.sale = sale;
        if (!IERC20(product.token).transferFrom(msg.sender, sale, amount)) revert TransferFailed();
        emit SaleOpened(productId, sale, amount, priceWei);
    }

    function getProduct(uint256 productId) external view returns (Product memory) {
        Product memory product = _products[productId];
        if (product.owner == address(0)) revert UnknownProduct();
        return product;
    }

    function _requireOwner(uint256 productId) internal view returns (Product storage product) {
        product = _products[productId];
        if (product.owner == address(0)) revert UnknownProduct();
        if (product.owner != msg.sender) revert NotOwner();
    }
}
