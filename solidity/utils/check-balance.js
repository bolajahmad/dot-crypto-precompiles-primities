const { ethers, network } = require("hardhat");

async function main() {
  // Get the signer from the private key in your config
  const [signer] = await ethers.getSigners();
  const address = await signer.getAddress();

  console.log(`\nNetwork: ${network.name}`);
  console.log(`Checking balance for Alice (ECDSA): ${address}`);

  // Fetch balance from the eth-rpc bridge
  const balance = await ethers.provider.getBalance(address);

  console.log(`Balance: ${ethers.formatEther(balance)} WND`);

  if (balance === 0n) {
    console.log("WARNING: Balance is 0! Deployment will fail.");
    console.log(`2. Send WND from 'Alice' to the address above: ${address}`);
  } else {
    console.log("\n✅ Success! You have enough funds to deploy.");
  }
}

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });
