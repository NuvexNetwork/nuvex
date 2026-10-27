// SPDX-License-Identifier: MIT
pragma solidity 0.8.24;

import "../interfaces/IERC20.sol";

/// @title BoardSale
/// @notice Fixed BNB price for a launched product token. Not a DEX and not a Binance listing.
/// @dev EXPERIMENTAL. Tokens can go to zero. The agent wallet cannot withdraw proceeds.
contract BoardSale {
    IERC20 public immutable token;
    address public immutable owner;
    uint256 public immutable priceWei;
    bool public open = true;
    uint256 private _lock = 1;

    event Bought(address indexed buyer, uint256 bnbIn, uint256 tokensOut);
    event Closed(uint256 leftover, uint256 proceeds);
    event Taken(uint256 proceeds);

    error NotOwner();
    error ClosedSale();
    error BadValue();
    error Empty();
    error Reentrancy();
    error TransferFailed();

    modifier onlyOwner() {
        if (msg.sender != owner) revert NotOwner();
        _;
    }

    modifier nonReentrant() {
        if (_lock != 1) revert Reentrancy();
        _lock = 2;
        _;
        _lock = 1;
    }

    constructor(address token_, address owner_, uint256 priceWei_) {
        if (token_ == address(0) || owner_ == address(0) || priceWei_ == 0) revert BadValue();
        token = IERC20(token_);
        owner = owner_;
        priceWei = priceWei_;
    }

    function remaining() public view returns (uint256) {
        return token.balanceOf(address(this));
    }

    function quote(uint256 bnbIn) public view returns (uint256 tokensOut) {
        if (bnbIn == 0) return 0;
        return (bnbIn * 1e18) / priceWei;
    }

    function buy() external payable nonReentrant {
        if (!open) revert ClosedSale();
        if (msg.value == 0) revert Empty();
        uint256 tokensOut = quote(msg.value);
        if (tokensOut == 0 || tokensOut > remaining()) revert BadValue();
        if (!token.transfer(msg.sender, tokensOut)) revert TransferFailed();
        emit Bought(msg.sender, msg.value, tokensOut);
    }

    function take() external onlyOwner nonReentrant {
        uint256 proceeds = address(this).balance;
        if (proceeds == 0) revert Empty();
        (bool ok, ) = owner.call{value: proceeds}("");
        if (!ok) revert TransferFailed();
        emit Taken(proceeds);
    }

    function close() external onlyOwner nonReentrant {
        open = false;
        uint256 leftover = remaining();
        if (leftover > 0 && !token.transfer(owner, leftover)) revert TransferFailed();
        uint256 proceeds = address(this).balance;
        if (proceeds > 0) {
            (bool ok, ) = owner.call{value: proceeds}("");
            if (!ok) revert TransferFailed();
        }
        emit Closed(leftover, proceeds);
    }

    receive() external payable {
        revert BadValue();
    }
}
