require('@nomiclabs/hardhat-waffle');

// You need to export an object to set up your config
// Go to https://hardhat.org/config/ to learn more
/**
 * @type import('hardhat/config').HardhatUserConfig
 */
module.exports = {
  solidity: '0.8.13',
  networks: {
    local: {
      /// Use `hhrun` if you want to utilize the local hardhat node with forked mainnet.
      url: 'http://127.0.0.1:8888',
      timeout: 60000,
    },
    goerli: {
      url: 'https://rpc.goerli.mudit.blog',
    },
  },
};
