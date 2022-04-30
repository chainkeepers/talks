/* SPDX-License-Identifier: MIT */

pragma solidity ^0.8.13;

// We need the artifacts hardhat compile generates.
import { IERC20 } from '@openzeppelin/contracts/token/ERC20/ERC20.sol';

interface UniswapV2Router {
  function swapExactTokensForTokens(
    uint amountIn,
    uint amountOutMin,
    address[] calldata path,
    address to,
    uint deadline
  ) external returns (uint[] memory amounts);
}


// From https://docs.uniswap.org/protocol/guides/swaps/single-swaps
import '@uniswap/v3-periphery/contracts/interfaces/ISwapRouter.sol';

address constant UNISWAP_V2_ROUTER_ADDRESS = 0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D;
address constant UNISWAP_V3_ROUTER_ADDRESS = 0xE592427A0AEce92De3Edee1F18E0157C05861564;


contract Arbitrage_v1 {

  /*

    swap X tokenu A za co nejvic tokenu B na prvni venue -> dostanu Y
    swap Y tokenu B za co nejvic tokenu C na druhe venue -> dostanu Z
    ...

    Note that to save gas, you might want to do all the approvals upon construction.

    Note that calldata will save gas as it does no copy the data from the
    trasnaction into the EVM memory.

    Note on uint256: Acknowledge that the slots in EVM are 256 long If you were
    to save gas by packing, you'd have to think hard.  Search "Solidity data
    packing" and cf.
    https://dev.to/javier123454321/solidity-gas-optimizations-pt-3-packing-structs-23f4

    In case you fail to pull it off - DO REACH OUT and I'll try to help!

    Note that you could save a HUGE TUN of gas by not using the router contract and
    instead implement parts of it in the contract directly.

    The router does two main things:
    - 1 : puts token_in and token_out together to get the pool address
      => you can get this off-chain in advance and save the on-chain computation

    - 2 : it calculates how much you can get and requests this amount from the 
          "swap" on the pool contract

    - 3 : moreover - it does transferFrom so you need to approve first
      => if you use the pool contract directly, you dont need either of those
  */

  function eval(uint256 amount_in, address token_in, address[] calldata tokens, uint256[] calldata venues) public {
    address[] memory path = new address[](2);

    // saves gas to only create the variable once - do check this to see it and behold
    address token_out;
    uint256 venue;

    for(uint256 i = 0; i < tokens.length; i++) {

      // saves gas because indexing is computation - feel free to check this
      venue = venues[i]; 
      token_out = tokens[i];

      if(venue == 0){

        // Uniswap V2 swap

        path[0] = token_in;
        path[1] = token_out;

        IERC20(token_in).approve(UNISWAP_V2_ROUTER_ADDRESS, amount_in);

        uint[] memory output = UniswapV2Router(UNISWAP_V2_ROUTER_ADDRESS)
                               .swapExactTokensForTokens(
                                 amount_in,
                                 0,
                                 path,
                                 address(this),
                                 block.timestamp
                               );

        amount_in = output[1];

      }else if(venue <= 3000){

        // Uniswap V3 swap

        IERC20(token_in).approve(UNISWAP_V3_ROUTER_ADDRESS, amount_in);

         amount_in = ISwapRouter(UNISWAP_V3_ROUTER_ADDRESS)
                     .exactInputSingle(
                       ISwapRouter.ExactInputSingleParams(
                         token_in,
                         token_out,
                         uint24(venue),
                         address(this),
                         block.timestamp,
                         amount_in,
                         0,
                         0
                        )
                      );

      }else{
        require(false, "501");
      }

      token_in = token_out;

    }
  }
}
