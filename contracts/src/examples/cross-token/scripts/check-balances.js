const hre = require('hardhat')

async function main() {
    const gateway = process.env.GATEWAY
    const USDCaddress = process.env.USDC
    const subnetTokenBridgeAddress = process.env.SubnetTokenBridge
    const subnetAddress = process.env.SUBNETADDRESS
    const rootTokenBridge = process.env.RootTokenBridge
    const accountAddress = await getAccountAddress()

    const usdcToken = await ethers.getContractAt(
        'USDCMock',
        USDCaddress,
    )

    const subnetTokenBridge = await ethers.getContractAt(
        'SubnetTokenBridge',
        subnetTokenBridgeAddress,
    )

    const subnetUDSCTokenAddress = await subnetTokenBridge.getProxyTokenAddress()

    const subnetUSDCToken = await ethers.getContractAt(
        'SubnetUSDCProxy',
        subnetUDSCTokenAddress,
    )

    const balance = await subnetUSDCToken.balanceOf(accountAddress)
    console.log('balance is ', balance)

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
