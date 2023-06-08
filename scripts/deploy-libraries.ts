/* global ethers */
/* eslint prefer-const: "off" */

import hre, { ethers } from "hardhat";
import { deployContractWithDeployer, getTransactionFees } from './util';

export async function deploy() {
    await hre.run('compile');

    const [deployer] = await ethers.getSigners();
    const balance = await ethers.provider.getBalance(deployer.address);
    console.log("Deploying libraries with the account:", deployer.address, ' balance:', ethers.utils.formatEther(balance));

    const txArgs = await getTransactionFees();

    const { address: accountHelperAddress } = await deployContractWithDeployer(deployer, "AccountHelper", {}, txArgs);
    const { address: checkpointHelperAddress } = await deployContractWithDeployer(deployer, "CheckpointHelper", {}, txArgs);
    const { address: epochVoteSubmissionHelperAddress } = await deployContractWithDeployer(deployer, "EpochVoteSubmissionHelper", {}, txArgs);
    const { address: executableQueueHelperAddress } = await deployContractWithDeployer(deployer, "ExecutableQueueHelper", {}, txArgs);
    const { address: subnetIDHelperAddress } = await deployContractWithDeployer(deployer, "SubnetIDHelper", {}, txArgs);
    // nested libs
    const { address: crossMsgHelperAddress } = await deployContractWithDeployer(deployer, "CrossMsgHelper", { "SubnetIDHelper": subnetIDHelperAddress }, txArgs);
    const { address: storableMsgHelperAddress } = await deployContractWithDeployer(deployer, "StorableMsgHelper", { "SubnetIDHelper": subnetIDHelperAddress }, txArgs);

    return {
        "AccountHelper": accountHelperAddress,
        "CheckpointHelper": checkpointHelperAddress,
        "EpochVoteSubmissionHelper": epochVoteSubmissionHelperAddress,
        "ExecutableQueueHelper": executableQueueHelperAddress,
        "SubnetIDHelper": subnetIDHelperAddress,
        "CrossMsgHelper": crossMsgHelperAddress,
        "StorableMsgHelper": storableMsgHelperAddress,
    };
}

// deploy();
// We recommend this pattern to be able to use async/await everywhere
// and properly handle errors.
if (require.main === module) {
    deploy()
        .then(() => process.exit(0))
        .catch((error: Error) => {
            console.error(error)
            process.exit(1)
        })
}
