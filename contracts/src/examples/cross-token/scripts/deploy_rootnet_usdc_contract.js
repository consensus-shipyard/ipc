const hre = require('hardhat')

async function main() {
    // Deploy ERC20 token
    const ERC20 = await hre.ethers.getContractFactory('USDCTest')
    const erc20Token = await ERC20.deploy();
    await erc20Token.deployed()
    console.log(
        'USDCTest Token Contract deployed to:',
        erc20Token.address,
    )

    // Mint tokens
    const mintAmount = hre.ethers.utils.parseUnits('1000', 18) // 1000 tokens
    await erc20Token.mint(mintAmount)

    console.log(
        mintAmount.toString(),
        'USDCTest Tokens created for user',
    )

}

main()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error(error)
        process.exit(1)
    })
