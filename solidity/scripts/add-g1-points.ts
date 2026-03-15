import hre from "hardhat";

const G1_POINT_A = {
  x: "0x000000000000000000000000000000001676d2c7cc18a826b1a8bc3f07d11f64dba5971112857db053a6a58c8449d1f12128b2f52cfe18a53850e66410f08279",
  y: "0x000000000000000000000000000000001722769c0cb2bfdf62ef5f31fad4c56b35d4c90dcf2e4c9357dcbbe948221ed28af69537b4f67d728a8e3c2f50ebcc4a",
};
const G1_POINT_B = {
  x: "0x000000000000000000000000000000000f6020c303297cdb04cd881468ffc8e35903b9f42bcaaed8a3301819c4fd87793e81f15b60994eda48be6b0c8386920e",
  y: "0x00000000000000000000000000000000047600106790a6cb17c439472502dad27e7192d577e7223b80fed04338227b95cf3de9ebf36580a7e604d2065e905e57"
};

async function main() {
  const contractAddress = "0x9BE1b0B3c00A19e6f250B43bb908cec7611Df9a0";
  const bls = await hre.ethers.getContractFactory("BLSContract");
  const blsContract = bls.attach(contractAddress);

  const tx1 = await blsContract.addG1Points(G1_POINT_A, G1_POINT_B);
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
