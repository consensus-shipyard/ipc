import { ethers } from 'hardhat';
import { getTransactionFees } from './util';

async function main() {
  try {
    const [deployer] = await ethers.getSigners();
    const balance = await deployer.getBalance();

    console.log(`Deploying contracts with account: ${deployer.address} and balance: ${balance.toString()}`);

    const libMap: {[key in string]: string} = {
        AccountHelper: '0x11217A7f62F7902B07E7681b316910708c864fbf',
        CheckpointHelper: '0x5e1d761eE2540Eb5b4506716812F56fa8c7F670b',
        EpochVoteSubmissionHelper: '0x93c9f8e808EbFE9dC166c9F0452A17B391767107',
        ExecutableQueueHelper: '0x7aaB791Fd004266C65AB1df6Ec2133e9d69DAf68',
        SubnetIDHelper: '0xEd05Cde2E66c9fd4A3FC9DEdaf5e2F1bB6ca29f4',
        CrossMsgHelper: '0x0C255aBa7d22A39f4832D7b4e1f3971060486116',
    };

    const gatewayAddress = "0xa60A839C28772347664a07bA6aBa0E76254Fba83";
    const txArgs = await getTransactionFees();

    // deploy
    const registry = await ethers.getContractFactory('SubnetRegistry', { signer: deployer, libraries: libMap });
    const contract = await registry.deploy(gatewayAddress, txArgs);

    // FEVM: 
    console.log(`contract deployed to: ${contract.address}`);

    process.exit(0);
  } catch (error) {
    console.error(error);
    process.exit(1);
  }
}

main();
