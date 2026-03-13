require("@nomicfoundation/hardhat-toolbox")
require("@parity/hardhat-polkadot")

/** @type import('hardhat/config').HardhatUserConfig */
module.exports = {
    solidity: "0.8.28",
    networks: {
        hardhat: {
            polkadot: {
                target: "evm",
            },
            nodeConfig: {
                nodeBinaryPath: "./bin/polkadot-parachain",
                rpcPort: 50127,
                dev: true,
            },
            adapterConfig: {
                adapterBinaryPath: "./bin/eth-rpc",
                dev: true,
            },
        },
        localNode: {
            polkadot: {
                target: "evm",
            },
            url: `http://127.0.0.1:8545`,
            chainId: 420420421,
            accounts: [vars.get("LOCAL_PRIVATE_KEY")],
        },
    },
}
