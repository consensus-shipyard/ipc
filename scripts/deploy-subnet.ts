/* global ethers */
/* eslint prefer-const: "off" */

import hre, { ethers } from "hardhat";
import { deployContractWithDeployer, getTransactionFees } from './util';

export async function deploy(gatewayAddress: string, libs: { [key in string]: string }) {
    if (!gatewayAddress) throw new Error(`Gateway is missing`);
    if (!libs || Object.keys(libs).length === 0) throw new Error(`Libraries are missing`);

    await hre.run('compile');

    const [deployer] = await ethers.getSigners();
    const balance = await ethers.provider.getBalance(deployer.address);
    console.log("Deploying subnet with the account:", deployer.address, ' balance:', ethers.utils.formatEther(balance));

    const txArgs = await getTransactionFees();

    const gateway = await ethers.getContractAt("Gateway", gatewayAddress, deployer);
    const parentId = await gateway.getNetworkName();

    console.log(parentId);

    const constructorParams = {
        parentId,
        name: ethers.utils.formatBytes32String('Subnet'),
        ipcGatewayAddr: gatewayAddress,
        consensus: 0,
        minActivationCollateral: ethers.utils.parseEther("1"),
        minValidators: 3,
        bottomUpCheckPeriod: 10,
        topDownCheckPeriod: 10,
        majorityPercentage: 66,
        genesis: 0
    }

    const { address: subnetAddress } = await deployContractWithDeployer(deployer, "SubnetActor", libs, constructorParams, txArgs);

    return {
        "SubnetActor": subnetAddress
    }
}