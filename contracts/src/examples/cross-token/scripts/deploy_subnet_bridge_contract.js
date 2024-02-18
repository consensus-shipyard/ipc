const hre = require('hardhat')

async function main() {
    const gateway = process.env.GATEWAY
    const USDCaddress = process.env.USDC
    const accountAddress = await getAccountAddress()

    // Validate environment variables
    if (!gateway || !USDCaddress) {
        throw new Error('All required environment variables must be provided')
    }

    parentSubnetChainId = 314159

    // Parent SubnetID value
    const parentSubnet = [parentSubnetChainId, []]
    const subnetTokenBridge = await createSubnetTokenBridge(
        gateway,
        USDCaddress,
        parentSubnet,
    )

    console.log(
        'Subnet Token Bridge Token deployed to:',
        subnetTokenBridge.address,
    )
}

async function createSubnetTokenBridge(
    gateway,
    parentSubnetUSDC,
    parentSubnet,
) {
    const SubnetTokenBridge = await ethers.getContractFactory(
        'SubnetTokenBridge',
    )
    const subnetTokenBridge = await SubnetTokenBridge.deploy(
        gateway,
        parentSubnetUSDC,
        parentSubnet,
    )
    return subnetTokenBridge
}

async function getAccountAddress() {
    // Getting a list of accounts
    const accounts = await hre.ethers.getSigners()

    // Assuming the first account is the one you want to use
    const currentAccount = accounts[0]

    // Getting the public address of the current account
    const publicAddress = currentAccount.address
    return publicAddress
}

main()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error(error)
        process.exit(1)
    })
