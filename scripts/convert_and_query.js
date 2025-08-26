// Script to convert f410 address to Ethereum address and query subnet permission mode
// Usage: node convert_and_query.js

const ethers = require('ethers');

// Your subnet information
const subnetId = '/r31337/t410f5kakfhdd3amp56oqwpxbupfalxsbjjrqilas5my';
const f410Address = 't410f5kakfhdd3amp56oqwpxbupfalxsbjjrqilas5my';

// Function to convert f410 address to Ethereum hex address
function f410ToEthAddress(f410Addr) {
    // Remove the 't410' prefix (or 'f410' for mainnet)
    const withoutPrefix = f410Addr.replace(/^[tf]410/, '');

    // The remaining part is base32-encoded, we need to decode it
    // For simplicity, we'll use a more direct approach since f410 addresses
    // are essentially Ethereum addresses with a specific encoding

    // Base32 decode the address (simplified version)
    // In a real implementation, you'd use a proper base32 decoder
    try {
        // For f410 addresses, the payload is typically 20 bytes (Ethereum address)
        // This is a simplified conversion - in practice you'd need proper base32 decoding
        console.log('F410 address (without prefix):', withoutPrefix);

        // For demonstration, let's show the manual steps:
        console.log('\nTo convert this properly, you need to:');
        console.log('1. Decode the base32 part:', withoutPrefix);
        console.log('2. Extract the 20-byte Ethereum address');
        console.log('3. Format as 0x-prefixed hex');

        // For your specific address, you'll need to use a proper base32 decoder
        // or the Filecoin address utilities
        return null; // Placeholder
    } catch (error) {
        console.error('Conversion error:', error);
        return null;
    }
}

// ABI for the permissionMode function
const subnetActorABI = [
    "function permissionMode() external view returns (uint8)"
];

// Function to query permission mode
async function queryPermissionMode(contractAddress, rpcUrl) {
    try {
        const provider = new ethers.JsonRpcProvider(rpcUrl);
        const contract = new ethers.Contract(contractAddress, subnetActorABI, provider);

        const permissionMode = await contract.permissionMode();

        const modes = {
            0: 'Collateral',
            1: 'Federated',
            2: 'Static'
        };

        console.log(`\nPermission Mode: ${modes[permissionMode]} (${permissionMode})`);
        return permissionMode;
    } catch (error) {
        console.error('Query error:', error);
    }
}

// Manual conversion helper using Filecoin libraries (pseudo-code)
function showManualConversionSteps() {
    console.log('\n=== MANUAL CONVERSION STEPS ===');
    console.log('Your f410 address:', f410Address);
    console.log('\nTo convert to Ethereum address, you can:');

    console.log('\n1. Using Filecoin address libraries:');
    console.log('   - Parse the address using Filecoin address utilities');
    console.log('   - Extract the delegated payload (20 bytes)');
    console.log('   - Format as 0x-prefixed hex string');

    console.log('\n2. Using online tools:');
    console.log('   - Use Filecoin explorer tools that support address conversion');
    console.log('   - Input:', f410Address);

    console.log('\n3. Using the IPC codebase conversion:');
    console.log('   - The address should decode to a 20-byte Ethereum address');
    console.log('   - Format: 0x followed by 40 hex characters');
}

// Example of how to use once you have the Ethereum address
function showQueryExample() {
    console.log('\n=== ONCE YOU HAVE THE ETHEREUM ADDRESS ===');
    console.log('Example usage with the converted address:');
    console.log('');
    console.log('const contractAddress = "0x..."; // Your converted address');
    console.log('const rpcUrl = "http://localhost:8545"; // Your RPC endpoint for chain 31337');
    console.log('');
    console.log('queryPermissionMode(contractAddress, rpcUrl);');
}

// For your specific case, let's try a different approach
function tryDirectConversion() {
    console.log('\n=== ATTEMPTING DIRECT CONVERSION ===');

    // f410 addresses are base32-encoded delegated addresses
    const addressWithoutPrefix = f410Address.replace(/^[tf]410/, '');

    console.log('Address without prefix:', addressWithoutPrefix);
    console.log('This needs base32 decoding to get the 20-byte Ethereum address');

    // You would need a proper base32 decoder here
    // The result should be a 20-byte array that represents the Ethereum address
}

// Main execution
async function main() {
    console.log('=== F410 TO ETHEREUM ADDRESS CONVERTER ===');
    console.log('Subnet ID:', subnetId);
    console.log('F410 Address:', f410Address);

    showManualConversionSteps();
    tryDirectConversion();
    showQueryExample();

    // If you already know the Ethereum address, uncomment and use:
    // const ethAddress = "0x..."; // Replace with actual converted address
    // const rpcUrl = "http://localhost:8545"; // Your RPC URL for chain 31337
    // await queryPermissionMode(ethAddress, rpcUrl);
}

if (require.main === module) {
    main().catch(console.error);
}

module.exports = {
    f410ToEthAddress,
    queryPermissionMode
};