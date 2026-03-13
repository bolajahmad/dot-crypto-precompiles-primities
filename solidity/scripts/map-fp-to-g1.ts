import hre from "hardhat";

const FP_SCALAR =
  "0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";

const G1_POINT =
  "0x0000000000000000000000000000000011a9a0372b8f332d5c30de9ad14e50372a73fa4c45d5f2fa5097f2d6fb93bcac592f2e1711ac43db0519870c7d0ea41500000000000000000000000000000000092c0f994164a0719f51c24ba3788de240ff926b55f58c445116e8bc6a47cd63392fd4e8e22bdf9feaa96ee773222133";

async function main() {
  const contractAddress = "0xdFfE65928E3B6DDB16bc98016bFDdE986C7e7409";
  const bls = await hre.ethers.getContractFactory("BlsContract");
  const blsContract = bls.attach(contractAddress);

  const tx1 = await blsContract.mapFpToG1Point(FP_SCALAR, G1_POINT);
  const receipt = await tx1.wait();

  console.log({ receipt });

  for (const log of receipt.logs) {
    const parsed = blsContract.interface.parseLog(log);
    console.log(parsed);
  }
}

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });
