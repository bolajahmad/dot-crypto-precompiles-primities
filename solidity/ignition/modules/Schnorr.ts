import { buildModule } from "@nomicfoundation/hardhat-ignition/modules"

const SchnorrContractModule = buildModule("SchnorrContractMod", (m) => {
    const schnorr = m.contract("SchnorrContract", [])

    return { schnorr }
})

module.exports = SchnorrContractModule
