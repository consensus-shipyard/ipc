import { SignerWithAddress } from '@nomiclabs/hardhat-ethers/signers';
import { Contract } from 'ethers';
import { ethers } from 'hardhat';

export const ZERO_ADDRESS = '0x0000000000000000000000000000000000000000';

export async function deployContractWithDeployer(
  deployer: SignerWithAddress,
  contractName: string,
  libs: { [key in string]: string },
  ...args: any[]
): Promise<Contract> {
  const contractFactory = await ethers.getContractFactory(contractName, { signer: deployer, libraries: libs, });
  return contractFactory.deploy(...args);
}

export async function getTransactionFees() {
  const feeData = await ethers.provider.getFeeData();

  return {
      maxFeePerGas: feeData.maxFeePerGas,
      maxPriorityFeePerGas: feeData.maxPriorityFeePerGas,
      type: 2
  };
}