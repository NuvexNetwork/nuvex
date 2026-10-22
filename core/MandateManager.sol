// SPDX-License-Identifier: MIT
pragma solidity 0.8.24;

import "../interfaces/IMandateManager.sol";
import "../interfaces/IAgentRegistry.sol";
import "../interfaces/IAgentWallet.sol";

/// @title MandateManager
/// @notice Stores the only rules an agent is allowed to act inside.
/// @dev EXPERIMENTAL and high-risk, especially for meme trading.
///      Invariants:
///      - The session key / agent cannot call setMandate, pause, or revoke.
///      - autoExecute cannot be enabled in v1. Live trading is a later, separate review.
///      - paperOnly defaults to true at launch.
///      - consume() only accounts a paper notional. It never transfers tokens.
contract MandateManager is IMandateManager {
    uint8 public constant DECISION_ALLOWED = 1;
    uint8 public constant DECISION_NEEDS_APPROVAL = 2;
    uint8 public constant DECISION_REJECTED = 3;

    uint256 public constant MAX_TOKENS = 16;
    uint16 public constant MAX_STOP_BPS = 10_000;
    uint16 public constant MIN_STOP_BPS = 1;

    IAgentRegistry public immutable registry;
    address public immutable admin;
    address public factory;

    mapping(uint256 => Mandate) private _mandates;
    mapping(uint256 => mapping(address => bool)) private _allowed;
    mapping(uint256 => address[]) private _allowedList;

    error NotAdmin();
    error NotFactory();
    error NotOwner();
    error NotWallet();
    error FactoryAlreadySet();
    error ZeroAddress();
    error AlreadySet();
    error Revoked();
    error AutoDisabled();
    error BadStop();
    error BadLimits();
    error TokenList();
    error NotAllowed();
    error PausedMandate();
    error OverPosition();
    error OverDaily();
    error UnknownAgent();

    modifier onlyAdmin() {
        if (msg.sender != admin) revert NotAdmin();
        _;
    }

    constructor(address registry_) {
        if (registry_ == address(0)) revert ZeroAddress();
        registry = IAgentRegistry(registry_);
        admin = msg.sender;
    }

    function setFactory(address factory_) external onlyAdmin {
        if (factory != address(0)) revert FactoryAlreadySet();
        if (factory_ == address(0)) revert ZeroAddress();
        factory = factory_;
    }

    function setInitial(
        uint256 agentId,
        uint256 maxPositionWei,
        uint256 dailyLimitWei,
        uint16 stopLossBps,
        bool approvalRequired,
        bool autoExecute,
        bool paperOnly,
        address[] calldata allowedTokens_
    ) external {
        if (msg.sender != factory) revert NotFactory();
        if (_mandates[agentId].exists) revert AlreadySet();
        _write(agentId, maxPositionWei, dailyLimitWei, stopLossBps, approvalRequired, autoExecute, paperOnly, allowedTokens_);
    }

    function setMandate(
        uint256 agentId,
        uint256 maxPositionWei,
        uint256 dailyLimitWei,
        uint16 stopLossBps,
        bool approvalRequired,
        bool autoExecute,
        bool paperOnly,
        address[] calldata allowedTokens_
    ) external {
        Mandate storage current = _mandates[agentId];
        if (!current.exists) revert UnknownAgent();
        if (msg.sender != registry.ownerOf(agentId)) revert NotOwner();
        if (current.revoked) revert Revoked();
        bool paused_ = current.paused;
        uint256 spent = _spentNow(current);
        uint64 dayStart = current.dayStart;
        _write(agentId, maxPositionWei, dailyLimitWei, stopLossBps, approvalRequired, autoExecute, paperOnly, allowedTokens_);
        _mandates[agentId].paused = paused_;
        _mandates[agentId].spentToday = spent;
        _mandates[agentId].dayStart = dayStart;
    }

    function pause(uint256 agentId, bool paused_) external {
        Mandate storage current = _mandates[agentId];
        if (!current.exists) revert UnknownAgent();
        if (msg.sender != registry.ownerOf(agentId)) revert NotOwner();
        if (current.revoked) revert Revoked();
        current.paused = paused_;
        emit MandatePaused(agentId, paused_);
    }

    /// @notice Permanent. Clears the wallet session so the agent key dies with the mandate.
    function revoke(uint256 agentId) external {
        Mandate storage current = _mandates[agentId];
        if (!current.exists) revert UnknownAgent();
        if (msg.sender != registry.ownerOf(agentId)) revert NotOwner();
        current.revoked = true;
        current.paused = true;
        IAgentWallet(registry.walletOf(agentId)).onMandateRevoked();
        emit MandateRevoked(agentId);
    }

    function consume(uint256 agentId, address tokenIn, address tokenOut, uint256 notionalWei)
        external
        returns (bool needsApproval)
    {
        if (msg.sender != registry.walletOf(agentId)) revert NotWallet();
        Mandate storage current = _mandates[agentId];
        _check(current, agentId, tokenIn, tokenOut, notionalWei);
        _rollDay(current);
        current.spentToday += notionalWei;
        return current.approvalRequired;
    }

    function preview(uint256 agentId, address tokenIn, address tokenOut, uint256 notionalWei)
        external
        view
        returns (uint8 decision, bool needsApproval)
    {
        Mandate storage current = _mandates[agentId];
        if (!_ok(current, agentId, tokenIn, tokenOut, notionalWei)) {
            return (DECISION_REJECTED, false);
        }
        if (current.approvalRequired) return (DECISION_NEEDS_APPROVAL, true);
        return (DECISION_ALLOWED, false);
    }

    function getMandate(uint256 agentId) external view returns (Mandate memory mandate) {
        mandate = _mandates[agentId];
        if (!mandate.exists) revert UnknownAgent();
        if (block.timestamp >= mandate.dayStart + 1 days) {
            mandate.spentToday = 0;
        }
    }

    function isAllowedToken(uint256 agentId, address token) external view returns (bool) {
        return _allowed[agentId][token];
    }

    function allowedTokens(uint256 agentId) external view returns (address[] memory) {
        return _allowedList[agentId];
    }

    function _write(
        uint256 agentId,
        uint256 maxPositionWei,
        uint256 dailyLimitWei,
        uint16 stopLossBps,
        bool approvalRequired,
        bool autoExecute,
        bool paperOnly,
        address[] calldata allowedTokens_
    ) internal {
        // Live auto-execution is intentionally unavailable. A later version would
        // need a separate review, and still could not add a withdraw path for the agent.
        if (autoExecute) revert AutoDisabled();
        if (!paperOnly) revert AutoDisabled();
        if (stopLossBps < MIN_STOP_BPS || stopLossBps > MAX_STOP_BPS) revert BadStop();
        if (maxPositionWei == 0 || dailyLimitWei < maxPositionWei) revert BadLimits();
        if (allowedTokens_.length == 0 || allowedTokens_.length > MAX_TOKENS) revert TokenList();

        _setAllowed(agentId, allowedTokens_);
        Mandate storage mandate = _mandates[agentId];
        mandate.maxPositionWei = maxPositionWei;
        mandate.dailyLimitWei = dailyLimitWei;
        mandate.spentToday = 0;
        mandate.dayStart = uint64(block.timestamp);
        mandate.stopLossBps = stopLossBps;
        mandate.approvalRequired = approvalRequired;
        mandate.autoExecute = false;
        mandate.paperOnly = true;
        mandate.exists = true;
        emit MandateSet(agentId, maxPositionWei, dailyLimitWei, true, approvalRequired);
    }

    function _setAllowed(uint256 agentId, address[] calldata tokens) internal {
        address[] storage prev = _allowedList[agentId];
        for (uint256 i = 0; i < prev.length; i++) {
            _allowed[agentId][prev[i]] = false;
        }
        delete _allowedList[agentId];
        for (uint256 i = 0; i < tokens.length; i++) {
            address token = tokens[i];
            if (token == address(0)) revert ZeroAddress();
            if (_allowed[agentId][token]) revert TokenList();
            _allowed[agentId][token] = true;
            _allowedList[agentId].push(token);
        }
    }

    function _check(Mandate storage mandate, uint256 agentId, address tokenIn, address tokenOut, uint256 notionalWei) internal view {
        if (!_ok(mandate, agentId, tokenIn, tokenOut, notionalWei)) {
            if (!mandate.exists || mandate.revoked) revert Revoked();
            if (mandate.paused) revert PausedMandate();
            if (tokenIn == tokenOut || (!_allowed[agentId][tokenIn] && !_allowed[agentId][tokenOut])) revert NotAllowed();
            if (notionalWei == 0 || notionalWei > mandate.maxPositionWei) revert OverPosition();
            revert OverDaily();
        }
    }

    function _ok(Mandate storage mandate, uint256 agentId, address tokenIn, address tokenOut, uint256 notionalWei) internal view returns (bool) {
        if (!mandate.exists || mandate.revoked || mandate.paused) return false;
        if (!mandate.paperOnly || mandate.autoExecute) return false;
        if (tokenIn == tokenOut) return false;
        if (!_allowed[agentId][tokenIn] && !_allowed[agentId][tokenOut]) return false;
        if (notionalWei == 0 || notionalWei > mandate.maxPositionWei) return false;
        uint256 spent = _spentNow(mandate);
        if (spent + notionalWei > mandate.dailyLimitWei) return false;
        return true;
    }

    function _spentNow(Mandate storage mandate) internal view returns (uint256) {
        if (block.timestamp >= mandate.dayStart + 1 days) return 0;
        return mandate.spentToday;
    }

    function _rollDay(Mandate storage mandate) internal {
        if (block.timestamp >= mandate.dayStart + 1 days) {
            mandate.spentToday = 0;
            mandate.dayStart = uint64(block.timestamp);
        }
    }
}
