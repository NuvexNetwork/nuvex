// SPDX-License-Identifier: MIT
pragma solidity 0.8.24;

/// @title MemeToken
/// @notice Fixed-supply token used by the launcher.
/// @dev EXPERIMENTAL. High-risk meme asset. No tax, no blacklist, no mint after deployment.
///      Nothing in this contract gives an AI agent special rights.
contract MemeToken {
    string public name;
    string public symbol;
    uint8 public constant decimals = 18;
    uint256 public immutable totalSupply;

    mapping(address => uint256) public balanceOf;
    mapping(address => mapping(address => uint256)) public allowance;

    event Transfer(address indexed from, address indexed to, uint256 amount);
    event Approval(address indexed owner, address indexed spender, uint256 amount);

    error ZeroAddress();
    error BadSupply();
    error InsufficientBalance();
    error InsufficientAllowance();

    constructor(string memory name_, string memory symbol_, uint256 supply, address recipient) {
        if (recipient == address(0)) revert ZeroAddress();
        if (supply == 0) revert BadSupply();
        name = name_;
        symbol = symbol_;
        totalSupply = supply;
        balanceOf[recipient] = supply;
        emit Transfer(address(0), recipient, supply);
    }

    function transfer(address to, uint256 amount) external returns (bool) {
        _transfer(msg.sender, to, amount);
        return true;
    }

    function approve(address spender, uint256 amount) external returns (bool) {
        if (spender == address(0)) revert ZeroAddress();
        allowance[msg.sender][spender] = amount;
        emit Approval(msg.sender, spender, amount);
        return true;
    }

    function transferFrom(address from, address to, uint256 amount) external returns (bool) {
        uint256 allowed = allowance[from][msg.sender];
        if (allowed < amount) revert InsufficientAllowance();
        if (allowed != type(uint256).max) {
            allowance[from][msg.sender] = allowed - amount;
        }
        _transfer(from, to, amount);
        return true;
    }

    function _transfer(address from, address to, uint256 amount) internal {
        if (to == address(0)) revert ZeroAddress();
        uint256 bal = balanceOf[from];
        if (bal < amount) revert InsufficientBalance();
        unchecked {
            balanceOf[from] = bal - amount;
            balanceOf[to] += amount;
        }
        emit Transfer(from, to, amount);
    }
}
