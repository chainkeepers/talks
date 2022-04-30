const { except } = require("chai");
const { ethers } = require("hardhat");

const { sendTokens, getERC20, getTxGasUsed, pinBlock } = require("../utils.js");

const parseEther = ethers.utils.parseEther;

const WETH = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2";
const USDC = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";
const DAI = "0x6B175474E89094C44Da98b954EedeAC495271d0F";

const TOKENS = {
  WETH: "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2",
  DAI: "0x6B175474E89094C44Da98b954EedeAC495271d0F",
  USDC: "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
};

const VENUE_UNISWAP_2 = 1;
const VENUE_UNISWAP_3 = 2;


async function reportBalances(address) {
  for(const [symbol, token] of Object.entries(TOKENS)){
    token_contract = await getERC20(token);
    const balance = await token_contract.balanceOf(address);
    console.log(`  ${symbol} = ${balance}`);
  }
}


/*

  Note that once everything works, the prints are unnecessary.  They help a lot
  during construction!

  Try to simulate this arb by setting token2 to FREE and the venues appropriately.
  https://etherscan.io/tx/0xfdacddf7c0e41ab5078f0746746677d94651df2968edadf006ca7fbe5d5ba73e

*/

describe("Test Arbitrage_v1", function() {

  it("works", async function() {
    
    await pinBlock(14684222);

    console.log("deploying contract");

    const factory = await ethers.getContractFactory("Arbitrage_v1");
    const contract = await factory.deploy();
    const weth_token = await getERC20(WETH);
    
    console.log(`contract deployed at ${contract.address}`);

    const amount_in = parseEther("1");
    const token1 = TOKENS["WETH"];
    const token2 = TOKENS["USDC"];
    const token3 = TOKENS["DAI"];
    const token4 = TOKENS["WETH"];

    const venue1 = 3000;
    const venue2 = 0;
    const venue3 = 3000;

    console.log("before funding");

    await reportBalances(contract.address);

    await sendTokens(contract.address, amount_in, token1);

    console.log("after funding");

    await reportBalances(contract.address);

    const balance_before = await weth_token.balanceOf(contract.address);

    console.log("before swap");

    const tx = await contract.eval(
      amount_in,
      token1,
      [token2, token3, token4],
      [venue1, venue2, venue3],
    );

    const gasUsed = await getTxGasUsed(tx.hash);

    console.log(`swap used ${gasUsed} gas`);

    console.log("after swap");

    await reportBalances(contract.address);

    const balance_after = await weth_token.balanceOf(contract.address);

    const profit = balance_after - balance_before;
    console.log(`profit = ${profit / 1e15} mETH`);

  })

});
