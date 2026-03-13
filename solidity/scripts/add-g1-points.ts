import hre from "hardhat";

const G1_POINT =
  "0x000000000000000000000000000000000f0b0417d1bb6d1457ed46251e612172c3897b145fff1e581641d300c0678eba456840cc9ef971743d6a6925eb9f224a000000000000000000000000000000000173d224a3963a6b786181218a81174da2143ec85e2f0acf7f32376f79239bd5b4afe737f037796d080d33dc410c6c92";
const G1_POINT_2 =
  "0x0000000000000000000000000000000006932e08272bb01497eb879bf75092d25536df1f05625403e04be68314652134551dfdec504b65951e36a040db846aee000000000000000000000000000000000304ed17f031b4f731c5e44201faea35dcf5b711d93bdcc91a893f81e7b855b69db7f83d4da2e608de119298d137286d";
const SUM_POINT =
  "0x000000000000000000000000000000000a25813623353195dde936add52600543b9a37a70806a7bda210c6c17c09cc90e8638aa501d51004bddac60b9ec369140000000000000000000000000000000015609e2beb512940f01eedc2962fea7c3f09877f766e6faa0d4288121e8f19b7f416c5e196a3a4439751d21d7dcb8a97";

async function main() {
  const contractAddress = "0xFa285e7D910a22e5589146F7c8e430a46647e6a3";
  const bls = await hre.ethers.getContractFactory("BlsContract");
  const blsContract = bls.attach(contractAddress);

  const tx1 = await blsContract.addBlsG1Points(G1_POINT, G1_POINT_2, SUM_POINT);
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
