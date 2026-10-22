// SPDX-License-Identifier: MIT
pragma solidity 0.8.24;

import "../interfaces/ITokenFactory.sol";
import "../interfaces/IAgentRegistry.sol";
import "../interfaces/IMandateManager.sol";
import "./MemeToken.sol";
import "./AgentWallet.sol";

/// @title TokenFactory
/// @notice Launching a meme token also creates its agent identity and a locked-down wallet.
/// @dev EXPERIMENTAL. Meme trading is high-risk and this factory does not provide liquidity,
///      a tax, or a live trading path. The entire token supply is minted to the human caller.
///      The new wallet starts empty. The default mandate is paper-only and requires approval.
contract TokenFactory is ITokenFactory {
    IAgentRegistry public immutable registry;
    IMandateManager public immutable mandates;

    /// @dev Conservative paper ceilings. The owner can tighten these further. They cannot
    ///      be used to trade live in this version.
    uint256 public constant DEFAULT_MAX_POSITION = 0.05 ether;
    uint256 public constant DEFAULT_DAILY_LIMIT = 0.2 ether;
    uint16 public constant DEFAULT_STOP_BPS = 1_500;

    error ZeroAddress();
    error BadSupply();
    error EmptyString();

    constructor(address registry_, address mandates_) {
        if (registry_ == address(0) || mandates_ == address(0)) revert ZeroAddress();
        registry = IAgentRegistry(registry_);
        mandates = IMandateManager(mandates_);
    }

    function launch(string calldata name, string calldata symbol, uint256 supply, string calldata agentURI)
        external
        returns (uint256 agentId, address token, address wallet)
    {
        if (bytes(name).length == 0 || bytes(symbol).length == 0) revert EmptyString();
        if (supply == 0) revert BadSupply();

        token = address(new MemeToken(name, symbol, supply, msg.sender));
        AgentWallet created = new AgentWallet(msg.sender, address(registry), address(mandates));
        wallet = address(created);

        agentId = registry.register(msg.sender, token, wallet, agentURI);
        created.bind(agentId);

        address[] memory allowed = new address[](1);
        allowed[0] = token;
        mandates.setInitial(
            agentId,
            DEFAULT_MAX_POSITION,
            DEFAULT_DAILY_LIMIT,
            DEFAULT_STOP_BPS,
            true,
            false,
            true,
            allowed
        );

        emit TokenLaunched(agentId, msg.sender, token, wallet);
    }
}
