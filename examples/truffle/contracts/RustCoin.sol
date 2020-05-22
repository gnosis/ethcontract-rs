pragma solidity ^0.6.0;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";

contract RustCoin is ERC20 {
  constructor() ERC20("Rust Coin", "RUST") public {
    _mint(msg.sender, 1337 * (10 ** uint256(decimals())));
  }

  receive() external payable {
    _mint(msg.sender, msg.value);
  }
}
