// SPDX-License-Identifier: MIT
pragma solidity 0.8.24;

/// @notice User-defined rules for one agent.
/// @dev The agent never writes this struct. `autoExecute` cannot be turned on in v1.
struct Mandate {
    uint256 maxPositionWei;
    uint256 dailyLimitWei;
    uint256 spentToday;
    uint64 dayStart;
    uint16 stopLossBps;
    bool approvalRequired;
    bool autoExecute;
    bool paperOnly;
    bool paused;
    bool revoked;
    bool exists;
}

interface IMandateManager {
    event MandateSet(uint256 indexed agentId, uint256 maxPositionWei, uint256 dailyLimitWei, bool paperOnly, bool approvalRequired);
    event MandatePaused(uint256 indexed agentId, bool paused);
    event MandateRevoked(uint256 indexed agentId);

    function setInitial(
        uint256 agentId,
        uint256 maxPositionWei,
        uint256 dailyLimitWei,
        uint16 stopLossBps,
        bool approvalRequired,
        bool autoExecute,
        bool paperOnly,
        address[] calldata allowedTokens
    ) external;

    function setMandate(
        uint256 agentId,
        uint256 maxPositionWei,
        uint256 dailyLimitWei,
        uint16 stopLossBps,
        bool approvalRequired,
        bool autoExecute,
        bool paperOnly,
        address[] calldata allowedTokens
    ) external;

    function pause(uint256 agentId, bool paused_) external;
    function revoke(uint256 agentId) external;

    /// @notice Records paper usage against the daily cap. Does not move tokens.
    /// @return needsApproval True when the owner must still accept the intent.
    function consume(uint256 agentId, address tokenIn, address tokenOut, uint256 notionalWei) external returns (bool needsApproval);

    function preview(uint256 agentId, address tokenIn, address tokenOut, uint256 notionalWei)
        external
        view
        returns (uint8 decision, bool needsApproval);

    function getMandate(uint256 agentId) external view returns (Mandate memory);
    function isAllowedToken(uint256 agentId, address token) external view returns (bool);
    function allowedTokens(uint256 agentId) external view returns (address[] memory);
}
