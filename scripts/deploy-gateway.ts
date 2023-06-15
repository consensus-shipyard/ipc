/* global ethers */
/* eslint prefer-const: "off" */

import hre, { ethers } from "hardhat";
import { deployContractWithDeployer, getTransactionFees } from './util';

export async function deploy(libs: { [key in string]: string }) {
    if (!libs || Object.keys(libs).length === 0) throw new Error(`Libraries are missing`);

    await hre.run('compile');

    const [deployer] = await ethers.getSigners();
    const balance = await ethers.provider.getBalance(deployer.address);
    console.log("Deploying gateway with the account:", deployer.address, ' balance:', ethers.utils.formatEther(balance));

    const txArgs = await getTransactionFees();

    const gatewayConstructorParams = {
        networkName: {
            root: 314159,
            route: [ethers.constants.AddressZero]
        },
        bottomUpCheckPeriod: 10,
        topDownCheckPeriod: 10,
        msgFee: ethers.utils.parseUnits("10", "gwei"),
        majorityPercentage: 66
    }

    const { address: gatewayAddress } = await deployContractWithDeployer(deployer, "Gateway", libs, gatewayConstructorParams, txArgs);

    return {
        "Gateway": gatewayAddress
    }

}