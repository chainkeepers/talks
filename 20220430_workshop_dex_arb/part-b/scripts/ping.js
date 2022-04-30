/* Wait until the hardhat network becomes awailable and exit. */

const { ethers } = require('hardhat');

function timeout(time) {
  return new Promise(resolve => setTimeout(resolve, time))
}


async function main() {

  while(true){
    try{
      await ethers.provider.getBlock('latest');
      break
    }catch{
    }
    await timeout(1000)
  }

  process.exit(0);
}


main();
