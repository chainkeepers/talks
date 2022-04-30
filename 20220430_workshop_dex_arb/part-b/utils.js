const { alchemy_endpoint } = require('./config.js');
const parseEther = ethers.utils.parseEther;


async function pinBlock(number) {
  await network.provider.request({
    method: 'hardhat_reset',
    params: [
      {
        forking: {
          jsonRpcUrl: alchemy_endpoint,
          blockNumber: number,
        },
      },
    ],
  });
}


async function impersonate(who) {
  await network.provider.request({
    method: 'hardhat_impersonateAccount',
    params: [who],
  });
  return await ethers.getSigner(who);
}


async function setBalance(address, amount) {
  return await network.provider.send(
    'hardhat_setBalance',
    [address, ethers.utils.hexlify(amount)]
  );
}


async function getERC20(address) {
  return await ethers.getContractAt(
    '@openzeppelin/contracts/token/ERC20/ERC20.sol:ERC20',
    address
  );
}


/* Bware, that keys have to be lower-case. */
const tokenHolders = {
  /* WETH is in WETH */
  '0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2': '0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2',
  /* DAI is in DAI */
  '0x6b175474e89094c44da98b954eedeac495271d0f': '0x6B175474E89094C44Da98b954EedeAC495271d0F',
  /* USDC is in KRAKEN 10 */
  '0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48': '0xAe2D4617c862309A3d75A0fFB358c7a5009c673F',
};
const BINANCE_ADDRESS = '0x3f5CE5FBFe3E9af3971dD833D26bA9b5C936f0bE';
const KRAKEN_ADDRESS = '0x2910543Af39abA0Cd09dBb2D50200b3E800A63D2';


async function sendTokens(sendToAddr, amount, tokenAddr) {

  const tokenAddrLower = tokenAddr.toLowerCase();
  const holder = tokenAddrLower in tokenHolders
        ? tokenHolders[tokenAddrLower]
        : BINANCE_ADDRESS;
  const holderSigner = await impersonate(holder);
  token = await getERC20(tokenAddr);
  const holderBalance = await token.balanceOf(holder);

  if (holderBalance < amount)
    throw `${holder} has too few ${tokenAddr}`;

  await setBalance(holder, parseEther('10')); // we need to have some ETH for basefee
  await token.connect(holderSigner).transfer(sendToAddr, amount);

  await token.balanceOf(holder);
}


module.exports = {
  pinBlock,
  impersonate,
  sendTokens,
  getERC20,
}
