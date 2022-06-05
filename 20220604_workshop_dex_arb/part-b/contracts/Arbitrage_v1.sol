// From https://docs.uniswap.org/protocol/guides/swaps/single-swaps
import '@uniswap/v3-periphery/contracts/interfaces/ISwapRouter.sol';

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


contract Arbitrage_v1 {

  /*
    amount token[0] -> (venue[0]) token[1]
    -> token[0] => amount_out

    if amount >= amount_out => fail
  */

  address constant UNISWAP_V2_ROUTER = 0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D;
  address constant UNISWAP_V3_ROUTER = 0xE592427A0AEce92De3Edee1F18E0157C05861564;

  function eval(uint256 amount, address[] calldata tokens, uint256[] calldata venues) public {

    uint256[] memory amounts_out;
    
    for(uint256 i = 0; i < tokens.length - 1; i++){

      if(venues[i] == 0x0){

        address[] memory path = new address[](2);
        path[0] = tokens[i];
        path[1] = tokens[i+1];

        IERC20(path[0]).approve(UNISWAP_V2_ROUTER, amount);

        // Swap on Uniswap v2
        amounts_out = UniswapV2Router(UNISWAP_V2_ROUTER)
          .swapExactTokensForTokens(
                                    amount,
                                    0,
                                    path,
                                    address(this),
                                    block.timestamp
                                    );
        amount = amounts_out[1];

      }else{
        // Swap on Uniswap v3
        
        IERC20(tokens[i]).approve(UNISWAP_V3_ROUTER, amount);

        amount = ISwapRouter(UNISWAP_V3_ROUTER)
          .exactInputSingle(
                            ISwapRouter.ExactInputSingleParams(
                                                               tokens[i],
                                                               tokens[i+1],
                                                               uint24(venues[i]),
                                                               address(this),
                                                               block.timestamp,
                                                               amount,
                                                               0,
                                                               0
                                                               )
                            );
      }

    }

  }

}
