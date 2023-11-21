import hre, { ethers } from 'hardhat'
import { deployContractWithDeployer, getTransactionFees } from './util'

const { getSelectors, FacetCutAction } = require('./js/diamond.js')

async function deploySubnetActorDiamond(
    gatewayDiamondAddress: string,
    libs: { [key in string]: string },
) {
    if (!gatewayDiamondAddress) throw new Error(`Gateway is missing`)
    if (!libs || Object.keys(libs).length === 0)
        throw new Error(`Libraries are missing`)

    console.log('Deploying Subnet Actor diamond with libraries:', libs)

    await hre.run('compile')

    const [deployer] = await ethers.getSigners()
    const txArgs = await getTransactionFees()

    const FacetNames = ['SubnetActorGetterFacet', 'SubnetActorManagerFacet']

    console.log('Target facets: ', FacetNames)
    // The `facetCuts` variable is the FacetCut[] that contains the functions to add during diamond deployment
    const facetCuts = []

    type Libraries = {
        [libraryName: string]: string
    }

    // ----

    const getterFacetLibs: Libraries = {
        CheckpointHelper: libs['CheckpointHelper'],
        SubnetIDHelper: libs['SubnetIDHelper'],
    }

    let getterFacet = await deployContractWithDeployer(
        deployer,
        'SubnetActorGetterFacet',
        getterFacetLibs,
        txArgs,
    )
    await getterFacet.deployed()
    console.log(`${FacetNames[0]} deployed: ${getterFacet.address}`)

    facetCuts.push({
        facetAddress: getterFacet.address,
        action: FacetCutAction.Add,
        functionSelectors: getSelectors(getterFacet),
    })

    console.log(
        'Subnet Actor Getter facet address: ',
        facetCuts[0].facetAddress,
    )

    // ----

    let diamondCutFacet = await deployContractWithDeployer(
        deployer,
        'DiamondCutFacet',
        {},
        txArgs,
    )
    await diamondCutFacet.deployed()

    facetCuts.push({
        facetAddress: diamondCutFacet.address,
        action: FacetCutAction.Add,
        functionSelectors: getSelectors(diamondCutFacet),
    })

    // ----

    const managerFacetLibs: Libraries = {
        //CrossMsgHelper: libs['CrossMsgHelper'],
        LibStaking: libs['LibStaking'],
        //SubnetIDHelper: libs['SubnetIDHelper'],
        //CheckpointHelper: libs['CheckpointHelper'],
        //EpochVoteSubmissionHelper: libs['EpochVoteSubmissionHelper'],
        //ExecutableQueueHelper: libs['ExecutableQueueHelper'],
    }

    const managerFacet = await deployContractWithDeployer(
        deployer,
        'SubnetActorManagerFacet',
        managerFacetLibs,
        txArgs,
    )

    console.log(`${FacetNames[1]} deployed: ${managerFacet.address}`)
    facetCuts.push({
        facetAddress: managerFacet.address,
        action: FacetCutAction.Add,
        functionSelectors: getSelectors(managerFacet),
    })

    console.log('Subnet Actor Manager facet: ', facetCuts[1].facetAddress)

    // ----

    const gatewayGetterFacet = await ethers.getContractAt(
        'GatewayGetterFacet',
        gatewayDiamondAddress,
    )
    const parentId = await gatewayGetterFacet.getNetworkName()
    console.log("parentId", parentId[0])
    console.log("parentId", parentId[1])

    const constructorParams = {
        parentId: {root:  parentId[0],route: [parentId[0]]},
        name: ethers.utils.formatBytes32String('Subnet'),
        ipcGatewayAddr: gatewayDiamondAddress,
        consensus: 0,
        minActivationCollateral: ethers.utils.parseEther('1'),
        minValidators: 3,
        bottomUpCheckPeriod: 10,
        topDownCheckPeriod: 10,
        majorityPercentage: 66,
        genesis: 0,
    }

    console.log("constructorParams", constructorParams)

    const diamondLibs: Libraries = {
        SubnetIDHelper: libs['SubnetIDHelper'],
    }

    // deploy Diamond
    const { address: diamondAddress } = await deployContractWithDeployer(
        deployer,
        'SubnetActorDiamond',
        diamondLibs,
        facetCuts,
        constructorParams,
        txArgs,
    )

    console.log('Subnet Actor Diamond address:', diamondAddress)

    // returning the address of the diamond
    return {
        SubnetActorDiamond: diamondAddress,
    }
}

// We recommend this pattern to be able to use async/await everywhere
// and properly handle errors.
if (require.main === module) {
    deploySubnetActorDiamond()
        .then(() => process.exit(0))
        .catch(error => {
            console.error(error)
            process.exit(1)
        })
}

exports.deployDiamond = deploySubnetActorDiamond