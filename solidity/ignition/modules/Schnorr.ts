import { buildModule } from "@nomicfoundation/hardhat-ignition/modules"

const SchnorrVerifyModule = buildModule("SchnorrVerifyMod", (m) => {
    const schnorr = m.contract("SchnorrVerify", [])

    return { schnorr }
})

module.exports = SchnorrVerifyModule
