const { except } = require("chai");
const { ethers } = require("hardhat");
const parseEther = ethers.utils.parseEther;

const { getERC20, sendTokens, pinBlock } = require("../utils.js");

const CONTRACT = "Arbitrage_v1";

const WETH = '0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2';
const DAI = "0x6B175474E89094C44Da98b954EedeAC495271d0F";
const USDC = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";


describe(`testing ${CONTRACT}`, function(){
  it("works", async function(){

    await pinBlock(13249151);

    console.log("deploying contract");

    const factory = await ethers.getContractFactory(CONTRACT);
    const contract = await factory.deploy();

    console.log(`deployed ${CONTRACT} at ${contract.address}`);

    const amount = parseEther("1");

    const usdc_token = await getERC20(WETH);
    const dai_token = await getERC20(DAI);

    const balance_before = await usdc_token.balanceOf(contract.address);

    console.log(`balance_before=${balance_before}`);

    await sendTokens(contract.address, amount, WETH);

    const balance_before_swap = await usdc_token.balanceOf(contract.address);
    const dai_before_swap = await dai_token.balanceOf(contract.address);

    console.log(`balance_before_swap=${balance_before_swap}`);
    console.log(`dai_before_swap=${dai_before_swap}`);

    const tokens = [
      WETH,
      DAI,
      WETH,
    ];
    const venues = [
      0,
      3000,
    ]

    await contract.eval(amount, tokens, venues);

    const balance_after_swap = await usdc_token.balanceOf(contract.address);
    const dai_after_swap = await dai_token.balanceOf(contract.address);

    console.log(`balance_after_swap=${balance_after_swap}`);
    console.log(`dai_after_swap=${dai_after_swap}`);

    console.log(`eth_profit=${(balance_after_swap - balance_before_swap) / 1e18}`);

  })
});
