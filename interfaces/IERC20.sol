// SPDX-License-Identifier: MIT
pragma solidity 0.8.24;

/// @notice Minimal ERC-20 surface used by the owner withdraw path.
/// @dev EXPERIMENTAL. Not a trading router and not a permission for an agent to move funds.
interface IERC20 {
    function transfer(address to, uint256 amount) external returns (bool);
    function balanceOf(address account) external view returns (uint256);
}
