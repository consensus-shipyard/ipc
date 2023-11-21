import { SignerWithAddress } from '@nomiclabs/hardhat-ethers/signers'
import { providers, Wallet, ContractFactory, Contract } from 'ethers'
import { Contract, ethers } from 'hardhat'
import ganache from 'ganache-core'
import * as linker from 'solc/linker'
const { getSelectors, FacetCutAction } = require('./js/diamond.js')
const fs = require('fs')

export const ZERO_ADDRESS = '0x0000000000000000000000000000000000000000'

export async function deployContractWithDeployer(
    deployer: SignerWithAddress,
    contractName: string,
    libs: { [key in string]: string },
    ...args: any[]
): Promise<Contract> {
    const contractFactory = await ethers.getContractFactory(contractName, {
        signer: deployer,
        libraries: libs,
    })
    return contractFactory.deploy(...args)
}

export async function getTransactionFees() {
    const feeData = await ethers.provider.getFeeData()

    return {
        maxFeePerGas: feeData.maxFeePerGas,
        maxPriorityFeePerGas: feeData.maxPriorityFeePerGas,
        type: 2,
    }
}

interface Facet {
    facetAddress: string
    functionSelectors: string[]
}
type FacetMap = { [key: string]: string[] }

export async function getFacets(diamondAddress: string): Promise<FacetMap> {
    // Ensure you have the ABI for the diamond loupe functions
    const diamondLoupeABI = [
        {
            inputs: [],
            name: 'facets',
            outputs: [
                {
                    components: [
                        {
                            internaltype: 'address',
                            name: 'facetaddress',
                            type: 'address',
                        },
                        {
                            internaltype: 'bytes4[]',
                            name: 'functionselectors',
                            type: 'bytes4[]',
                        },
                    ],
                    name: 'facets_',
                    type: 'tuple[]',
                },
            ],
            statemutability: 'view',
            constant: true,
            type: 'function',
        },
    ]

    const provider = ethers.provider
    const diamond = new Contract(diamondAddress, diamondLoupeABI, provider)
    const facetsData = await diamond.facets()

    // Convert facetsData to the Facet[] type.
    const facets: Facet[] = facetsData.map((facetData) => ({
        facetAddress: facetData[0],
        functionSelectors: facetData[1],
    }))

    const facetMap = facets.reduce((acc, facet) => {
        acc[facet.facetAddress] = facet.functionSelectors
        return acc
    }, {})

    return facetMap
}

async function startGanache() {
    return new Promise((resolve, reject) => {
        const server = ganache.server({
            gasPrice: '0x0', // Set gas price to 0
        })
        server.listen(8545, (err) => {
            if (err) reject(err)
            else resolve(server)
        })
    })
}

async function stopGanache(server) {
    return new Promise((resolve, reject) => {
        server.close((err) => {
            if (err) reject(err)
            else resolve()
        })
    })
}

export async function getRuntimeBytecode(bytecode) {
    // Check if bytecode is provided
    if (!bytecode) {
        throw new Error('No bytecode provided')
    }
    const ganacheServer = await startGanache()

    const provider = new providers.JsonRpcProvider('http://127.0.0.1:8545')
    const wallet = new Wallet(process.env.PRIVATE_KEY, provider)
    const contractFactory = new ContractFactory([], bytecode, wallet)
    const contract = await contractFactory.deploy()
    await contract.deployed()

    const runtimeBytecode = await provider.getCode(contract.address)

    await stopGanache(ganacheServer)

    return runtimeBytecode
}

export async function getBytecodeFromFacet(facet) {
    const facetName = facet.name
    const libs = facet.libs
    const bytecodeNeedsLink = getBytecodeFromFacetTypeChainFilename(
        `./typechain/factories/${facetName}__factory.ts`,
    )
    let libs2 = {}
    // Loop through each key in the libs
    for (let key in libs) {
        let newKey = `src/lib/${key}.sol:${key}`
        libs2[newKey] = libs[key]
    }

    // Link the bytecode with the libraries
    const bytecode = linker.linkBytecode(bytecodeNeedsLink, libs2)
    return await getRuntimeBytecode(bytecode)
}

function getBytecodeFromFacetTypeChainFilename(fileName) {
    try {
        // Read the file synchronously
        const fileContent = fs.readFileSync(fileName, 'utf8')

        // Split the file content into lines
        const lines = fileContent.split('\n')

        // Initialize a flag to identify when the target line is found
        let found = false

        for (const line of lines) {
            // If the previous line was the target line, return the current line
            if (found) {
                // Trim semicolons and quotes from the beginning and end of the string
                return line.trim().replace(/^[";]+|[";]+$/g, '')
            }

            // Check if the current line is the target line
            if (line.includes('const _bytecode =')) {
                found = true
            }
        }

        // If the loop completes without returning, the target line was not found
        throw new Error('Target line "const _bytecode =" not found in the file')
    } catch (error) {
        console.error('Error reading file:', error.message)
    }
}

// Loop through each contract address in the facets
// query web3 api to get deployed bytecode
export async function getOnChainBytecodeFromFacets(facets) {
    const deployedBytecode = {}
    for (let contractAddress in facets) {
        try {
            // Fetch the bytecode of the contract
            const bytecode = await ethers.provider.getCode(contractAddress)
            deployedBytecode[bytecode] = contractAddress
            // Log the bytecode to the console
        } catch (error) {
            // Print any errors to stderr
            console.error(
                `Error fetching bytecode for ${contractAddress}:`,
                error.message,
            )
        }
    }
    return deployedBytecode
}

/**
 * Filters the input array to only return strings that start with '0x'.
 *
 * @param {Object} input - The object containing the functionSelectors array.
 * @returns {Array} - An array of strings from functionSelectors that start with '0x'.
 */
function filterSelectors(input) {
    return input.filter((item) => {
        return typeof item === 'string' && item.startsWith('0x')
    })
}

// given a facet address and a diamond address,
// upgrade the diamond to use the new facet
export async function upgradeFacet(
    diamondAddress: string,
    replacementFacetName: string,
    facetLibs: { [key in string]: string },
) {
    console.info(`
Diamond Facet Upgrade:
-----------------------------------
Diamond Address: ${diamondAddress}
Replacement Facet Name: ${replacementFacetName}
`)

    if (!diamondAddress) throw new Error(`Gateway is missing`)

    const [deployer] = await ethers.getSigners()
    const txArgs = await getTransactionFees()
    let replacementFacet = await deployContractWithDeployer(
        deployer,
        replacementFacetName,
        facetLibs,
        txArgs,
    )
    await replacementFacet.deployed()

    const facetCuts = [
        {
            facetAddress: replacementFacet.address,
            action: FacetCutAction.Replace,
            functionSelectors: filterSelectors(getSelectors(replacementFacet)),
        },
    ]
    const diamondCutter = await ethers.getContractAt(
        'DiamondCutFacet',
        diamondAddress,
        deployer,
    )
    // 0x0 (contract address) and "" (call data) can be used to send call data to contract
    const tx = await diamondCutter.diamondCut(
        facetCuts,
        ethers.constants.AddressZero,
        ethers.utils.formatBytes32String(''),
        txArgs,
    )
    await tx.wait()
    return { replacementFacetName: replacementFacet.address }
}
